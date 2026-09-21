use std::time::Duration;

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Difficulty, Error, Problem, ProblemSummary, SearchResults};

pub(crate) const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const QUERY: &str = "query questionData($titleSlug: String!) {
    question(titleSlug: $titleSlug) { titleSlug title content isPaidOnly }
}";
const SEARCH_QUERY: &str = "query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) {
    problemsetQuestionList: questionList(categorySlug: $categorySlug, limit: $limit, skip: $skip, filters: $filters) {
        total: totalNum
        questions: data { frontendQuestionId: questionFrontendId title titleSlug difficulty isPaidOnly }
    }
}";
const SEARCH_LIMIT: u32 = 20;

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://leetcode.com/graphql/".into(),
        })
    }

    /// Fetch a public problem by its title slug, without authentication
    pub async fn problem(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Problem, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }

        let response: ProblemResponse = self
            .graphql(
                &ProblemRequest {
                    query: QUERY,
                    operation_name: "questionData",
                    variables: Variables { title_slug: slug },
                },
                progress,
            )
            .await?;
        if !response.errors.is_empty() {
            return Err(Error::Graphql);
        }
        let question = response
            .data
            .ok_or(Error::InvalidResponse)?
            .question
            .ok_or(Error::NotFound)?;
        if question.title_slug.as_ref() != slug || question.title.trim().is_empty() {
            return Err(Error::InvalidResponse);
        }
        let statement = question
            .content
            .filter(|content| !content.trim().is_empty())
            .ok_or(if question.is_paid_only {
                Error::PremiumRequired
            } else {
                Error::InvalidResponse
            })?;

        Ok(Problem {
            id: question.title_slug,
            title: question.title,
            statement,
        })
    }

    /// Search public problems by title keywords, without authentication
    pub async fn search(
        &self,
        query: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<SearchResults, Error> {
        let query = query.trim();
        if query.is_empty()
            || query.len() > 100
            || !query
                .bytes()
                .all(|byte| byte.is_ascii_graphic() || byte == b' ')
        {
            return Err(Error::InvalidQuery);
        }

        let response: SearchResponse = self
            .graphql(
                &SearchRequest {
                    query: SEARCH_QUERY,
                    operation_name: "problemsetQuestionList",
                    variables: SearchVariables {
                        category_slug: "",
                        limit: SEARCH_LIMIT,
                        skip: 0,
                        filters: SearchFilters {
                            search_keywords: query,
                        },
                    },
                },
                progress,
            )
            .await?;
        if !response.errors.is_empty() {
            return Err(Error::Graphql);
        }
        let list = response
            .data
            .ok_or(Error::InvalidResponse)?
            .problemset_question_list;
        if list.questions.len() > SEARCH_LIMIT as usize || list.total < list.questions.len() as u32
        {
            return Err(Error::InvalidResponse);
        }

        let mut problems = Vec::with_capacity(list.questions.len());
        for question in list.questions {
            let number = question
                .frontend_question_id
                .parse()
                .ok()
                .filter(|number: &u32| *number > 0)
                .ok_or(Error::InvalidResponse)?;
            if !valid_slug(&question.title_slug) || !valid_label(&question.title, 256) {
                return Err(Error::InvalidResponse);
            }
            let difficulty = match question.difficulty {
                SearchDifficulty::Easy => Difficulty::Easy,
                SearchDifficulty::Medium => Difficulty::Medium,
                SearchDifficulty::Hard => Difficulty::Hard,
                SearchDifficulty::Unknown => return Err(Error::InvalidResponse),
            };
            problems.push(ProblemSummary {
                number,
                id: question.title_slug,
                title: question.title,
                difficulty,
                paid_only: question.is_paid_only,
            });
        }
        Ok(SearchResults {
            total: list.total,
            problems,
        })
    }

    async fn graphql<T: DeserializeOwned>(
        &self,
        request: &impl Serialize,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<T, Error> {
        let response = self
            .http
            .post(self.endpoint.as_ref())
            .header(reqwest::header::REFERER, "https://leetcode.com/")
            .header(reqwest::header::ORIGIN, "https://leetcode.com")
            .json(request)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        read_response(response, progress).await
    }
}

pub(crate) fn http_builder() -> ClientBuilder {
    HttpClient::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .no_zstd()
        .pool_max_idle_per_host(2)
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
}

async fn read_response<T: DeserializeOwned>(
    mut response: reqwest::Response,
    mut progress: impl FnMut(usize, Option<u64>),
) -> Result<T, Error> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(Error::ResponseTooLarge {
            limit: MAX_RESPONSE_BYTES,
        });
    }

    let mut body = Vec::with_capacity(8 * 1024);
    let total = response.content_length();
    progress(0, total);
    while let Some(chunk) = response.chunk().await? {
        if chunk.len() > MAX_RESPONSE_BYTES - body.len() {
            return Err(Error::ResponseTooLarge {
                limit: MAX_RESPONSE_BYTES,
            });
        }
        body.extend_from_slice(&chunk);
        progress(body.len(), total);
    }
    Ok(serde_json::from_slice(&body)?)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Variables<'a> {
    title_slug: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: SearchVariables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchVariables<'a> {
    category_slug: &'static str,
    limit: u32,
    skip: u32,
    filters: SearchFilters<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchFilters<'a> {
    search_keywords: &'a str,
}

#[derive(Deserialize)]
struct ProblemResponse {
    data: Option<ProblemData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct ProblemData {
    #[serde(deserialize_with = "Option::deserialize")]
    question: Option<Question>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Question {
    title_slug: Box<str>,
    title: Box<str>,
    content: Option<Box<str>>,
    is_paid_only: bool,
}

#[derive(Deserialize)]
struct SearchResponse {
    data: Option<SearchData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchData {
    problemset_question_list: SearchList,
}

#[derive(Deserialize)]
struct SearchList {
    total: u32,
    questions: Vec<SearchQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchQuestion {
    frontend_question_id: Box<str>,
    title: Box<str>,
    title_slug: Box<str>,
    difficulty: SearchDifficulty,
    is_paid_only: bool,
}

#[derive(Deserialize)]
enum SearchDifficulty {
    Easy,
    Medium,
    Hard,
    #[serde(other)]
    Unknown,
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 128
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_label(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value == value.trim()
        && !value.chars().any(char::is_control)
}
