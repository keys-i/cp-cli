use std::time::Duration;

use reqwest::{
    Client as HttpClient, ClientBuilder,
    header::{COOKIE, HeaderValue},
    redirect::Policy,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    AccountStats, CodeSnippet, ContestRegistration, ContestSummary, Credentials, Difficulty,
    DifficultyCounts, Discussion, DiscussionList, DiscussionSummary, Error, Problem,
    ProblemSummary, RecentSubmission, RunResult, RunState, SearchResults, StarterCode,
    SubmissionResult, SubmissionState, TestCases,
};

pub(crate) const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const QUERY: &str = "query questionData($titleSlug: String!) {
    question(titleSlug: $titleSlug) { frontendQuestionId: questionFrontendId titleSlug title content isPaidOnly }
}";
const DAILY_QUERY: &str = "query questionOfToday {
    activeDailyCodingChallengeQuestion { question { frontendQuestionId: questionFrontendId titleSlug title content isPaidOnly } }
}";
const USER_STATUS_QUERY: &str = "query userStatus { userStatus { isSignedIn username } }";
const ACCOUNT_STATS_QUERY: &str = "query accountStats($username: String!) {
    matchedUser(username: $username) {
        username
        submitStats {
            acSubmissionNum { difficulty count submissions }
            totalSubmissionNum { difficulty count submissions }
        }
    }
    submissionList(offset: 0, limit: 10) {
        hasNext
        submissions { id statusDisplay title titleSlug timestamp lang runtime memory url }
    }
}";
const UPCOMING_CONTESTS_QUERY: &str = "query upcomingContests {
    upcomingContests { title titleSlug startTime duration isVirtual }
}";
const CONTEST_QUERY: &str = "query contest($titleSlug: String!) {
    contest(titleSlug: $titleSlug) { title titleSlug startTime duration isVirtual }
}";
const CONTEST_REGISTRATION_QUERY: &str = "query contestRegistration($titleSlug: String!) {
    contest(titleSlug: $titleSlug) { titleSlug userRegistered }
}";
const TRENDING_DISCUSSIONS_QUERY: &str = "query trendingDiscussions {
    cachedTrendingCategoryTopics(first: 10) {
        id title viewCount topLevelCommentCount
        post { voteCount creationDate author { username } }
    }
}";
const DISCUSSION_QUESTION_QUERY: &str = "query discussionQuestion($titleSlug: String!) {
    question(titleSlug: $titleSlug) { questionId titleSlug }
}";
const PROBLEM_DISCUSSIONS_QUERY: &str = "query problemDiscussions($questionId: Int!, $orderBy: TopicSortingOption!, $pageNo: Int!, $numPerPage: Int!) {
    questionTopics(orderBy: $orderBy, questionId: $questionId, pageNo: $pageNo, numPerPage: $numPerPage) {
        totalNum data {
            id title viewCount topLevelCommentCount
            post { voteCount creationDate author { username } }
        }
    }
}";
const DISCUSSION_QUERY: &str = "query discussion($topicId: Int!) {
    topic(id: $topicId) {
        id title viewCount topLevelCommentCount tags pinned
        post { voteCount creationDate updationDate content author { username } }
    }
}";
const DISCUSSION_ARTICLE_QUERY: &str = "query discussionArticle($topicId: ID) {
    ugcArticleDiscussionArticle(topicId: $topicId) { uuid content }
}";
const STARTER_QUERY: &str = "query questionEditorData($titleSlug: String!) {
    question(titleSlug: $titleSlug) { questionId title titleSlug codeDefinition isPaidOnly }
}";
const TEST_CASES_QUERY: &str = "query consolePanelConfig($titleSlug: String!) {
    question(titleSlug: $titleSlug) { questionId titleSlug isPaidOnly enableRunCode exampleTestcaseList sampleTestCase }
}";
const QUESTION_LIST_QUERY: &str = "query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) {
    problemsetQuestionList: questionList(categorySlug: $categorySlug, limit: $limit, skip: $skip, filters: $filters) {
        total: totalNum
        questions: data { frontendQuestionId: questionFrontendId title titleSlug difficulty isPaidOnly }
    }
}";
const QUESTION_LIST_LIMIT: u32 = 20;
const MAX_SOURCE_BYTES: usize = 1024 * 1024;
const MAX_TEST_CASES: usize = 128;
const MAX_TEST_INPUT_BYTES: usize = 1024 * 1024;
const MAX_CODE_DEFINITION_BYTES: usize = 1024 * 1024;
const MAX_STARTER_SNIPPETS: usize = 64;
const MAX_RUN_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_RECENT_SUBMISSIONS: usize = 10;
const MAX_ACCOUNT_COUNT: u32 = 100_000_000;
const MAX_CONTESTS: usize = 64;
const MAX_CONTEST_DURATION: u64 = 7 * 86_400;
const MAX_CONTEST_TIMESTAMP: u64 = 253_402_300_799;
const MAX_DISCUSSIONS: usize = 20;
const MAX_DISCUSSION_TITLE: usize = 256;
const MAX_DISCUSSION_CONTENT: usize = 512 * 1024;
const MAX_DISCUSSION_ARTICLE_UUID: usize = 128;
const MAX_DISCUSSION_TAGS: usize = 20;
const MAX_DISCUSSION_TAG: usize = 64;
const MAX_DISCUSSION_COUNT: u64 = 1_000_000_000;

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
        problem_from_question(question, Some(slug))
    }

    /// Fetch today's public daily challenge, without authentication
    pub async fn daily(&self, progress: impl FnMut(usize, Option<u64>)) -> Result<Problem, Error> {
        let response: DailyResponse = self
            .graphql(
                &DailyRequest {
                    query: DAILY_QUERY,
                    operation_name: "questionOfToday",
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
            .active_daily_coding_challenge_question
            .ok_or(Error::InvalidResponse)?
            .question
            .ok_or(Error::InvalidResponse)?;
        problem_from_question(question, None)
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

        let response: QuestionListResponse = self
            .graphql(
                &QuestionListRequest {
                    query: QUESTION_LIST_QUERY,
                    operation_name: "problemsetQuestionList",
                    variables: QuestionListVariables {
                        category_slug: "",
                        limit: QUESTION_LIST_LIMIT,
                        skip: 0,
                        filters: QuestionListFilters {
                            search_keywords: Some(query),
                            difficulty: None,
                            tags: None,
                        },
                    },
                },
                progress,
            )
            .await?;
        question_list_from_response(response)
    }

    /// List public problems with optional difficulty and tag filters, without authentication
    pub async fn list(
        &self,
        difficulty: Option<Difficulty>,
        tag: Option<&str>,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<SearchResults, Error> {
        if tag.is_some_and(|tag| !valid_tag(tag)) {
            return Err(Error::InvalidTag);
        }

        let response: QuestionListResponse = self
            .graphql(
                &QuestionListRequest {
                    query: QUESTION_LIST_QUERY,
                    operation_name: "problemsetQuestionList",
                    variables: QuestionListVariables {
                        category_slug: "",
                        limit: QUESTION_LIST_LIMIT,
                        skip: 0,
                        filters: QuestionListFilters {
                            search_keywords: None,
                            difficulty: difficulty.map(difficulty_name),
                            tags: tag.map(|tag| [tag]),
                        },
                    },
                },
                progress,
            )
            .await?;
        question_list_from_response(response)
    }

    /// Confirm whether credentials describe a signed-in LeetCode session
    pub async fn auth_status(&self, credentials: &Credentials) -> Result<bool, Error> {
        let response: UserStatusResponse = self
            .graphql_authenticated(
                &UserStatusRequest {
                    query: USER_STATUS_QUERY,
                    operation_name: "userStatus",
                },
                credentials,
                |_, _| {},
            )
            .await?;
        if !response.errors.is_empty() {
            return Err(Error::Graphql);
        }
        Ok(response
            .data
            .ok_or(Error::InvalidResponse)?
            .user_status
            .ok_or(Error::InvalidResponse)?
            .is_signed_in)
    }

    /// Fetch bounded account progress and recent submissions for a signed-in session
    pub async fn account_stats(&self, credentials: &Credentials) -> Result<AccountStats, Error> {
        let status: UserStatusResponse = self
            .graphql_authenticated(
                &UserStatusRequest {
                    query: USER_STATUS_QUERY,
                    operation_name: "userStatus",
                },
                credentials,
                |_, _| {},
            )
            .await?;
        if !status.errors.is_empty() {
            return Err(Error::Graphql);
        }
        let status = status
            .data
            .ok_or(Error::InvalidResponse)?
            .user_status
            .ok_or(Error::InvalidResponse)?;
        if !status.is_signed_in {
            return Err(Error::Authentication);
        }
        let username = status.username.ok_or(Error::InvalidResponse)?;
        if !valid_label(&username, 128) {
            return Err(Error::InvalidResponse);
        }

        let response: AccountStatsResponse = self
            .graphql_authenticated(
                &AccountStatsRequest {
                    query: ACCOUNT_STATS_QUERY,
                    operation_name: "accountStats",
                    variables: AccountStatsVariables {
                        username: username.as_ref(),
                    },
                },
                credentials,
                |_, _| {},
            )
            .await?;
        account_stats_from_response(response, username)
    }

    /// List the upcoming public contests
    pub async fn contests(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<ContestSummary>, Error> {
        let response: UpcomingContestsResponse = self
            .graphql(
                &UpcomingContestsRequest {
                    query: UPCOMING_CONTESTS_QUERY,
                    operation_name: "upcomingContests",
                },
                progress,
            )
            .await?;
        contests_from_response(response)
    }

    /// Fetch one public contest by its title slug
    pub async fn contest(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<ContestSummary, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let response: ContestResponse = self
            .graphql(
                &ContestRequest {
                    query: CONTEST_QUERY,
                    operation_name: "contest",
                    variables: Variables { title_slug: slug },
                },
                progress,
            )
            .await?;
        contest_from_response(response, slug)
    }

    /// Fetch the signed-in participant's registration state for one contest
    pub async fn contest_registration(
        &self,
        slug: &str,
        credentials: &Credentials,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<ContestRegistration, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let response: ContestRegistrationResponse = self
            .graphql_authenticated(
                &ContestRegistrationRequest {
                    query: CONTEST_REGISTRATION_QUERY,
                    operation_name: "contestRegistration",
                    variables: Variables { title_slug: slug },
                },
                credentials,
                progress,
            )
            .await?;
        contest_registration_from_response(response, slug)
    }

    /// List ten public discussions currently trending on LeetCode
    pub async fn trending_discussions(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<DiscussionList, Error> {
        let response: TrendingDiscussionsResponse = self
            .graphql(
                &TrendingDiscussionsRequest {
                    query: TRENDING_DISCUSSIONS_QUERY,
                    operation_name: "trendingDiscussions",
                },
                progress,
            )
            .await?;
        discussion_list_from_trending(response)
    }

    /// List the twenty newest public discussions for a problem
    pub async fn problem_discussions(
        &self,
        slug: &str,
        mut progress: impl FnMut(usize, Option<u64>),
    ) -> Result<DiscussionList, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let question: DiscussionQuestionResponse = self
            .graphql(
                &DiscussionQuestionRequest {
                    query: DISCUSSION_QUESTION_QUERY,
                    operation_name: "discussionQuestion",
                    variables: Variables { title_slug: slug },
                },
                &mut progress,
            )
            .await?;
        let question_id = discussion_question_id(question, slug)?;
        let response: ProblemDiscussionsResponse = self
            .graphql(
                &ProblemDiscussionsRequest {
                    query: PROBLEM_DISCUSSIONS_QUERY,
                    operation_name: "problemDiscussions",
                    variables: ProblemDiscussionsVariables {
                        question_id,
                        order_by: "newest_to_oldest",
                        page_no: 1,
                        num_per_page: MAX_DISCUSSIONS as u32,
                    },
                },
                progress,
            )
            .await?;
        discussion_list_from_problem(response)
    }

    /// Fetch one public discussion and its Markdown post
    pub async fn discussion(
        &self,
        topic_id: u32,
        mut progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Discussion, Error> {
        if topic_id == 0 {
            return Err(Error::InvalidResponse);
        }
        let response: DiscussionResponse = self
            .graphql(
                &DiscussionRequest {
                    query: DISCUSSION_QUERY,
                    operation_name: "discussion",
                    variables: DiscussionVariables { topic_id },
                },
                &mut progress,
            )
            .await?;
        let mut discussion = discussion_from_response(response, topic_id)?;
        if discussion.content.trim() == "article-topic" {
            let article: DiscussionArticleResponse = self
                .graphql(
                    &DiscussionArticleRequest {
                        query: DISCUSSION_ARTICLE_QUERY,
                        operation_name: "discussionArticle",
                        variables: DiscussionArticleVariables { topic_id },
                    },
                    progress,
                )
                .await?;
            discussion.content = discussion_article_content(article)?;
        }
        Ok(discussion)
    }

    /// Fetch every public starter source for a problem
    pub async fn starter(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<StarterCode, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let response: StarterResponse = self
            .graphql(
                &StarterRequest {
                    query: STARTER_QUERY,
                    operation_name: "questionEditorData",
                    variables: Variables { title_slug: slug },
                },
                progress,
            )
            .await?;
        if !response.errors.is_empty() {
            return Err(Error::Graphql);
        }
        starter_from_response(response, slug)
    }

    /// Fetch public example input for LeetCode's remote run-code endpoint
    pub async fn test_cases(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<TestCases, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let response: TestCasesResponse = self
            .graphql(
                &TestCasesRequest {
                    query: TEST_CASES_QUERY,
                    operation_name: "consolePanelConfig",
                    variables: Variables { title_slug: slug },
                },
                progress,
            )
            .await?;
        match test_cases_from_response(response, slug) {
            Err(Error::InvalidResponse) => Err(Error::InvalidTestCases),
            result => result,
        }
    }

    /// Run source once against public example input; this never creates a submission
    pub async fn run(
        &self,
        credentials: &Credentials,
        slug: &str,
        question_id: &str,
        language: &str,
        source: &str,
        input: &str,
    ) -> Result<Box<str>, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        if !valid_question_id(question_id) {
            return Err(Error::InvalidResponse);
        }
        if !valid_language_slug(language) {
            return Err(Error::InvalidLanguage);
        }
        if !valid_source(source) {
            return Err(Error::InvalidSource);
        }
        if !valid_test_input(input) {
            return Err(Error::InvalidTestInput);
        }
        let url = self.endpoint_path(&format!("/problems/{slug}/interpret_solution/"))?;
        let referer = format!("https://leetcode.com/problems/{slug}/");
        let response: RunResponse = self
            .authenticated_json(
                self.authenticated(
                    self.http.post(url).json(&RunRequest {
                        data_input: input,
                        lang: language,
                        question_id,
                        typed_code: source,
                    }),
                    credentials,
                )?
                .header(reqwest::header::REFERER, referer),
            )
            .await?;
        match response.interpret_id {
            Some(id) if valid_interpret_id(&id) => Ok(id),
            Some(_) => Err(Error::RunRejected(
                "LeetCode returned an invalid test-run identifier".into(),
            )),
            None => Err(Error::RunRejected(
                response
                    .detail
                    .or(response.message)
                    .or(response.error)
                    .filter(|message| valid_label(message, 512))
                    .unwrap_or_else(|| "no run identifier was returned".into()),
            )),
        }
    }

    /// Read one remote run status without polling
    pub async fn run_result(&self, credentials: &Credentials, id: &str) -> Result<RunState, Error> {
        if !valid_interpret_id(id) {
            return Err(Error::InvalidResponse);
        }
        let url = self.endpoint_path(&format!("/submissions/detail/{id}/check/"))?;
        let response: RunStatusResponse = self
            .authenticated_json(self.authenticated(self.http.get(url), credentials)?)
            .await?;
        match run_from_response(response, id) {
            Err(Error::InvalidResponse) => Err(Error::InvalidTestResult),
            result => result,
        }
    }

    /// Submit source once; callers own subsequent status polling
    pub async fn submit(
        &self,
        credentials: &Credentials,
        slug: &str,
        question_id: &str,
        language: &str,
        source: &str,
    ) -> Result<u64, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        if !valid_question_id(question_id) {
            return Err(Error::InvalidResponse);
        }
        if !valid_language_slug(language) {
            return Err(Error::InvalidLanguage);
        }
        if !valid_source(source) {
            return Err(Error::InvalidSource);
        }
        let url = self.endpoint_path(&format!("/problems/{slug}/submit/"))?;
        let response: SubmitResponse = self
            .authenticated_json(self.authenticated(
                self.http.post(url).json(&SubmitRequest {
                    question_id,
                    lang: language,
                    typed_code: source,
                }),
                credentials,
            )?)
            .await?;
        response
            .submission_id
            .filter(|id| *id > 0)
            .ok_or(Error::InvalidResponse)
    }

    /// Read one submission status without polling
    pub async fn submission(
        &self,
        credentials: &Credentials,
        id: u64,
    ) -> Result<SubmissionState, Error> {
        if id == 0 {
            return Err(Error::InvalidResponse);
        }
        let url = self.endpoint_path(&format!("/submissions/detail/{id}/check/"))?;
        let response: SubmissionResponse = self
            .authenticated_json(self.authenticated(self.http.get(url), credentials)?)
            .await?;
        match submission_from_response(response, id) {
            Err(Error::InvalidResponse) => Err(Error::InvalidSubmissionResult),
            result => result,
        }
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

    async fn graphql_authenticated<T: DeserializeOwned>(
        &self,
        request: &impl Serialize,
        credentials: &Credentials,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<T, Error> {
        let response = self
            .authenticated(
                self.http.post(self.endpoint.as_ref()).json(request),
                credentials,
            )?
            .send()
            .await?;
        if response.status().is_client_error() && matches!(response.status().as_u16(), 401 | 403) {
            return Err(Error::Authentication);
        }
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        read_response(response, progress).await
    }

    fn authenticated(
        &self,
        request: reqwest::RequestBuilder,
        credentials: &Credentials,
    ) -> Result<reqwest::RequestBuilder, Error> {
        let mut cookie = HeaderValue::from_str(&format!(
            "LEETCODE_SESSION={}; csrftoken={}",
            credentials.session(),
            credentials.csrf()
        ))
        .map_err(|_| Error::InvalidCredentials)?;
        cookie.set_sensitive(true);
        let mut csrf =
            HeaderValue::from_str(credentials.csrf()).map_err(|_| Error::InvalidCredentials)?;
        csrf.set_sensitive(true);
        Ok(request
            .header(COOKIE, cookie)
            .header("x-csrftoken", csrf)
            .header(reqwest::header::REFERER, "https://leetcode.com/")
            .header(reqwest::header::ORIGIN, "https://leetcode.com"))
    }

    async fn authenticated_json<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, Error> {
        let response = request.send().await?;
        if matches!(response.status().as_u16(), 401 | 403) {
            return Err(Error::Authentication);
        }
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        read_response(response, |_, _| {}).await
    }

    fn endpoint_path(&self, path: &str) -> Result<reqwest::Url, Error> {
        let mut url = reqwest::Url::parse(&self.endpoint).map_err(|_| Error::InvalidResponse)?;
        url.set_path(path);
        url.set_query(None);
        Ok(url)
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
struct DailyRequest {
    query: &'static str,
    operation_name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpcomingContestsRequest {
    query: &'static str,
    operation_name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContestRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContestRegistrationRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrendingDiscussionsRequest {
    query: &'static str,
    operation_name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionQuestionRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDiscussionsRequest {
    query: &'static str,
    operation_name: &'static str,
    variables: ProblemDiscussionsVariables,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDiscussionsVariables {
    question_id: u32,
    order_by: &'static str,
    page_no: u32,
    num_per_page: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionRequest {
    query: &'static str,
    operation_name: &'static str,
    variables: DiscussionVariables,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionVariables {
    topic_id: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionArticleRequest {
    query: &'static str,
    operation_name: &'static str,
    variables: DiscussionArticleVariables,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionArticleVariables {
    topic_id: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestionListRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: QuestionListVariables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestionListVariables<'a> {
    category_slug: &'static str,
    limit: u32,
    skip: u32,
    filters: QuestionListFilters<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuestionListFilters<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    search_keywords: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<[&'a str; 1]>,
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
    frontend_question_id: Option<Box<str>>,
    title_slug: Box<str>,
    title: Box<str>,
    content: Option<Box<str>>,
    is_paid_only: bool,
}

#[derive(Deserialize)]
struct DailyResponse {
    data: Option<DailyData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DailyData {
    active_daily_coding_challenge_question: Option<DailyChallenge>,
}

#[derive(Deserialize)]
struct DailyChallenge {
    question: Option<Question>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserStatusRequest {
    query: &'static str,
    operation_name: &'static str,
}

#[derive(Deserialize)]
struct UserStatusResponse {
    data: Option<UserStatusData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserStatusData {
    user_status: Option<UserStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserStatus {
    is_signed_in: bool,
    username: Option<Box<str>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountStatsRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: AccountStatsVariables<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountStatsVariables<'a> {
    username: &'a str,
}

#[derive(Deserialize)]
struct AccountStatsResponse {
    data: Option<AccountStatsData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct UpcomingContestsResponse {
    data: Option<UpcomingContestsData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpcomingContestsData {
    upcoming_contests: Vec<Contest>,
}

#[derive(Deserialize)]
struct ContestResponse {
    data: Option<ContestData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct ContestData {
    #[serde(deserialize_with = "Option::deserialize")]
    contest: Option<Contest>,
}

#[derive(Deserialize)]
struct ContestRegistrationResponse {
    data: Option<ContestRegistrationData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct ContestRegistrationData {
    #[serde(deserialize_with = "Option::deserialize")]
    contest: Option<ContestRegistrationDataContest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContestRegistrationDataContest {
    title_slug: Box<str>,
    user_registered: bool,
}

#[derive(Deserialize)]
struct TrendingDiscussionsResponse {
    data: Option<TrendingDiscussionsData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrendingDiscussionsData {
    cached_trending_category_topics: Vec<DiscussionTopic>,
}

#[derive(Deserialize)]
struct DiscussionQuestionResponse {
    data: Option<DiscussionQuestionData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct DiscussionQuestionData {
    #[serde(deserialize_with = "Option::deserialize")]
    question: Option<DiscussionQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionQuestion {
    question_id: Box<str>,
    title_slug: Box<str>,
}

#[derive(Deserialize)]
struct ProblemDiscussionsResponse {
    data: Option<ProblemDiscussionsData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemDiscussionsData {
    question_topics: Option<DiscussionConnection>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionConnection {
    total_num: NumberResponse,
    data: Vec<DiscussionTopic>,
}

#[derive(Deserialize)]
struct DiscussionResponse {
    data: Option<DiscussionData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct DiscussionData {
    #[serde(deserialize_with = "Option::deserialize")]
    topic: Option<DiscussionTopic>,
}

#[derive(Deserialize)]
struct DiscussionArticleResponse {
    data: Option<DiscussionArticleData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionArticleData {
    ugc_article_discussion_article: Option<DiscussionArticle>,
}

#[derive(Deserialize)]
struct DiscussionArticle {
    uuid: Box<str>,
    content: Box<str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionTopic {
    id: NumberResponse,
    title: Box<str>,
    view_count: NumberResponse,
    top_level_comment_count: NumberResponse,
    post: Option<DiscussionPost>,
    #[serde(default)]
    tags: Vec<Box<str>>,
    #[serde(default)]
    pinned: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiscussionPost {
    vote_count: SignedNumberResponse,
    creation_date: NumberResponse,
    #[serde(default)]
    updation_date: Option<NumberResponse>,
    #[serde(default)]
    content: Option<Box<str>>,
    author: Option<DiscussionAuthor>,
}

#[derive(Deserialize)]
struct DiscussionAuthor {
    username: Box<str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contest {
    title: Box<str>,
    title_slug: Box<str>,
    start_time: NumberResponse,
    duration: NumberResponse,
    is_virtual: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountStatsData {
    matched_user: Option<AccountUser>,
    submission_list: Option<SubmissionList>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountUser {
    username: Box<str>,
    submit_stats: Option<SubmitStats>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmitStats {
    ac_submission_num: Vec<SubmissionCount>,
    total_submission_num: Vec<SubmissionCount>,
}

#[derive(Deserialize)]
struct SubmissionCount {
    difficulty: Box<str>,
    count: u32,
    submissions: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmissionList {
    has_next: bool,
    submissions: Vec<RecentSubmissionResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentSubmissionResponse {
    id: NumberResponse,
    status_display: Box<str>,
    title: Box<str>,
    title_slug: Box<str>,
    timestamp: NumberResponse,
    lang: Box<str>,
    runtime: Option<Box<str>>,
    memory: Option<Box<str>>,
    url: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberResponse {
    Number(u64),
    Text(Box<str>),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SignedNumberResponse {
    Number(i64),
    Text(Box<str>),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StarterRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Deserialize)]
struct StarterResponse {
    data: Option<StarterData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct StarterData {
    #[serde(deserialize_with = "Option::deserialize")]
    question: Option<StarterQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StarterQuestion {
    question_id: Box<str>,
    title: Box<str>,
    title_slug: Box<str>,
    code_definition: Option<Box<str>>,
    is_paid_only: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TestCasesRequest<'a> {
    query: &'static str,
    operation_name: &'static str,
    variables: Variables<'a>,
}

#[derive(Deserialize)]
struct TestCasesResponse {
    data: Option<TestCasesData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
struct TestCasesData {
    #[serde(deserialize_with = "Option::deserialize")]
    question: Option<TestCasesQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCasesQuestion {
    question_id: Box<str>,
    title_slug: Box<str>,
    is_paid_only: bool,
    enable_run_code: bool,
    example_testcase_list: Option<Vec<Box<str>>>,
    sample_test_case: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodeDefinition {
    value: Box<str>,
    text: Box<str>,
    default_code: Box<str>,
}

#[derive(Serialize)]
struct SubmitRequest<'a> {
    question_id: &'a str,
    lang: &'a str,
    typed_code: &'a str,
}

#[derive(Serialize)]
struct RunRequest<'a> {
    data_input: &'a str,
    lang: &'a str,
    question_id: &'a str,
    typed_code: &'a str,
}

#[derive(Deserialize)]
struct RunResponse {
    interpret_id: Option<Box<str>>,
    detail: Option<Box<str>>,
    message: Option<Box<str>>,
    error: Option<Box<str>>,
}

#[derive(Deserialize)]
struct SubmitResponse {
    submission_id: Option<u64>,
}

#[derive(Deserialize)]
struct SubmissionResponse {
    state: Box<str>,
    status_msg: Option<Box<str>>,
    status_code: Option<i32>,
    status_runtime: Option<Box<str>>,
    status_memory: Option<Box<str>>,
    total_correct: Option<u32>,
    total_testcases: Option<u32>,
    compile_error: Option<Box<str>>,
    full_compile_error: Option<Box<str>>,
    runtime_error: Option<Box<str>>,
    full_runtime_error: Option<Box<str>>,
}

#[derive(Deserialize)]
struct RunStatusResponse {
    state: Box<str>,
    status_msg: Option<Box<str>>,
    status_code: Option<i32>,
    correct_answer: Option<bool>,
    status_runtime: Option<Box<str>>,
    status_memory: Option<Box<str>>,
    total_correct: Option<u32>,
    total_testcases: Option<u32>,
    code_answer: Option<Vec<Box<str>>>,
    expected_answer: Option<Vec<Box<str>>>,
    expected_code_answer: Option<Vec<Box<str>>>,
    code_output: Option<OutputValue>,
    std_output_list: Option<OutputValue>,
    input_formatted: Option<Box<str>>,
    last_testcase: Option<Box<str>>,
    expected_output: Option<Box<str>>,
    compile_error: Option<Box<str>>,
    full_compile_error: Option<Box<str>>,
    runtime_error: Option<Box<str>>,
    full_runtime_error: Option<Box<str>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OutputValue {
    Text(Box<str>),
    List(Vec<Box<str>>),
}

#[derive(Deserialize)]
struct QuestionListResponse {
    data: Option<QuestionListData>,
    #[serde(default)]
    errors: Vec<serde::de::IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuestionListData {
    problemset_question_list: QuestionList,
}

#[derive(Deserialize)]
struct QuestionList {
    total: u32,
    questions: Vec<QuestionListQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuestionListQuestion {
    frontend_question_id: Box<str>,
    title: Box<str>,
    title_slug: Box<str>,
    difficulty: QuestionDifficulty,
    is_paid_only: bool,
}

#[derive(Deserialize)]
enum QuestionDifficulty {
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

fn valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 64
        && !tag.starts_with('-')
        && !tag.ends_with('-')
        && tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn valid_question_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.bytes().all(|byte| byte.is_ascii_digit())
        && id.parse::<u64>().is_ok_and(|id| id > 0)
}

fn valid_source(source: &str) -> bool {
    !source.is_empty()
        && source.len() <= MAX_SOURCE_BYTES
        && !source.bytes().any(|byte| byte == b'\0')
}

fn valid_test_input(input: &str) -> bool {
    !input.is_empty()
        && input.len() <= MAX_TEST_INPUT_BYTES
        && !input.bytes().any(|byte| byte == b'\0')
}

fn valid_interpret_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_language_slug(language: &str) -> bool {
    !language.is_empty()
        && language.len() <= 32
        && language
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn difficulty_name(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => "EASY",
        Difficulty::Medium => "MEDIUM",
        Difficulty::Hard => "HARD",
    }
}

fn question_list_from_response(response: QuestionListResponse) -> Result<SearchResults, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let list = response
        .data
        .ok_or(Error::InvalidResponse)?
        .problemset_question_list;
    if list.questions.len() > QUESTION_LIST_LIMIT as usize
        || list.total < list.questions.len() as u32
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
            QuestionDifficulty::Easy => Difficulty::Easy,
            QuestionDifficulty::Medium => Difficulty::Medium,
            QuestionDifficulty::Hard => Difficulty::Hard,
            QuestionDifficulty::Unknown => return Err(Error::InvalidResponse),
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

fn account_stats_from_response(
    response: AccountStatsResponse,
    expected_username: Box<str>,
) -> Result<AccountStats, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let data = response.data.ok_or(Error::InvalidResponse)?;
    let user = data.matched_user.ok_or(Error::Authentication)?;
    if user.username != expected_username || !valid_label(&user.username, 128) {
        return Err(Error::InvalidResponse);
    }
    let stats = user.submit_stats.ok_or(Error::InvalidResponse)?;
    let solved = difficulty_counts(&stats.ac_submission_num, |row| row.count)?;
    let accepted_submissions = difficulty_counts(&stats.ac_submission_num, |row| row.submissions)?;
    let submissions = difficulty_counts(&stats.total_submission_num, |row| row.submissions)?;
    if !counts_within(&solved, &accepted_submissions)
        || !counts_within(&accepted_submissions, &submissions)
    {
        return Err(Error::InvalidResponse);
    }

    let list = data.submission_list.ok_or(Error::InvalidResponse)?;
    if list.submissions.len() > MAX_RECENT_SUBMISSIONS {
        return Err(Error::InvalidResponse);
    }
    let mut recent_submissions = Vec::with_capacity(list.submissions.len());
    for submission in list.submissions {
        let id = positive_number(submission.id).ok_or(Error::InvalidResponse)?;
        let timestamp = positive_number(submission.timestamp).ok_or(Error::InvalidResponse)?;
        if !valid_label(&submission.status_display, 128)
            || !valid_label(&submission.title, 256)
            || !valid_slug(&submission.title_slug)
            || !valid_label(&submission.lang, 64)
        {
            return Err(Error::InvalidResponse);
        }
        let pending = matches!(
            submission.status_display.as_ref(),
            "Pending" | "Judging" | "Started"
        );
        recent_submissions.push(RecentSubmission {
            id,
            status: submission.status_display,
            title: submission.title,
            slug: submission.title_slug,
            timestamp,
            language: submission.lang,
            runtime: optional_label(submission.runtime, 64)?,
            memory: optional_label(submission.memory, 64)?,
            url: optional_label(submission.url, 1024)?,
            pending,
        });
    }
    Ok(AccountStats {
        username: user.username,
        solved,
        accepted_submissions,
        submissions,
        recent_submissions,
        has_more_submissions: list.has_next,
    })
}

fn contests_from_response(
    response: UpcomingContestsResponse,
) -> Result<Vec<ContestSummary>, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let contests = response
        .data
        .ok_or(Error::InvalidResponse)?
        .upcoming_contests;
    if contests.len() > MAX_CONTESTS {
        return Err(Error::InvalidResponse);
    }
    let mut summaries = Vec::with_capacity(contests.len());
    for contest in contests {
        let summary = contest_summary(contest)?;
        if summaries
            .iter()
            .any(|existing: &ContestSummary| existing.slug == summary.slug)
        {
            return Err(Error::InvalidResponse);
        }
        summaries.push(summary);
    }
    summaries.sort_unstable_by(|left, right| {
        (left.start_time, left.slug.as_ref()).cmp(&(right.start_time, right.slug.as_ref()))
    });
    Ok(summaries)
}

fn contest_from_response(
    response: ContestResponse,
    expected_slug: &str,
) -> Result<ContestSummary, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let contest = response
        .data
        .ok_or(Error::InvalidResponse)?
        .contest
        .ok_or(Error::NotFound)?;
    let contest = contest_summary(contest)?;
    if contest.slug.as_ref() != expected_slug {
        return Err(Error::InvalidResponse);
    }
    Ok(contest)
}

fn contest_registration_from_response(
    response: ContestRegistrationResponse,
    expected_slug: &str,
) -> Result<ContestRegistration, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let contest = response
        .data
        .ok_or(Error::InvalidResponse)?
        .contest
        .ok_or(Error::NotFound)?;
    if !valid_slug(&contest.title_slug) || contest.title_slug.as_ref() != expected_slug {
        return Err(Error::InvalidResponse);
    }
    Ok(ContestRegistration {
        slug: contest.title_slug,
        registered: contest.user_registered,
    })
}

fn contest_summary(contest: Contest) -> Result<ContestSummary, Error> {
    let start_time = positive_number(contest.start_time)
        .filter(|start_time| *start_time <= MAX_CONTEST_TIMESTAMP)
        .ok_or(Error::InvalidResponse)?;
    let duration_seconds = positive_number(contest.duration)
        .filter(|duration| *duration <= MAX_CONTEST_DURATION)
        .and_then(|duration| u32::try_from(duration).ok())
        .ok_or(Error::InvalidResponse)?;
    if !valid_slug(&contest.title_slug) || !valid_label(&contest.title, 256) {
        return Err(Error::InvalidResponse);
    }
    Ok(ContestSummary {
        slug: contest.title_slug,
        title: contest.title,
        start_time,
        duration_seconds,
        virtual_contest: contest.is_virtual,
    })
}

fn discussion_list_from_trending(
    response: TrendingDiscussionsResponse,
) -> Result<DiscussionList, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let topics = response
        .data
        .ok_or(Error::InvalidResponse)?
        .cached_trending_category_topics;
    if topics.len() > 10 {
        return Err(Error::InvalidResponse);
    }
    let discussions = discussion_summaries(topics)?;
    Ok(DiscussionList {
        total: None,
        discussions,
    })
}

fn discussion_question_id(
    response: DiscussionQuestionResponse,
    expected_slug: &str,
) -> Result<u32, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let question = response
        .data
        .ok_or(Error::InvalidResponse)?
        .question
        .ok_or(Error::NotFound)?;
    if question.title_slug.as_ref() != expected_slug || !valid_question_id(&question.question_id) {
        return Err(Error::InvalidResponse);
    }
    question
        .question_id
        .parse()
        .ok()
        .filter(|id: &u32| *id > 0)
        .ok_or(Error::InvalidResponse)
}

fn discussion_list_from_problem(
    response: ProblemDiscussionsResponse,
) -> Result<DiscussionList, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let topics = response
        .data
        .ok_or(Error::InvalidResponse)?
        .question_topics
        .ok_or(Error::InvalidResponse)?;
    let total = nonnegative_count(topics.total_num)?;
    if topics.data.len() > MAX_DISCUSSIONS || total < topics.data.len() as u32 {
        return Err(Error::InvalidResponse);
    }
    let discussions = discussion_summaries(topics.data)?;
    Ok(DiscussionList {
        total: Some(total),
        discussions,
    })
}

fn discussion_summaries(
    topics: impl IntoIterator<Item = DiscussionTopic>,
) -> Result<Vec<DiscussionSummary>, Error> {
    let mut discussions = Vec::new();
    for topic in topics {
        let summary = discussion_summary(topic)?;
        if discussions
            .iter()
            .any(|existing: &DiscussionSummary| existing.id == summary.id)
        {
            return Err(Error::InvalidResponse);
        }
        discussions.push(summary);
    }
    Ok(discussions)
}

fn discussion_summary(topic: DiscussionTopic) -> Result<DiscussionSummary, Error> {
    let post = topic.post.ok_or(Error::InvalidResponse)?;
    let id = bounded_count(topic.id)?;
    let views = nonnegative_count(topic.view_count)?;
    let comments = nonnegative_count(topic.top_level_comment_count)?;
    let votes = signed_count(post.vote_count)?;
    let created_at = positive_number(post.creation_date).ok_or(Error::InvalidResponse)?;
    if !valid_label(&topic.title, MAX_DISCUSSION_TITLE) {
        return Err(Error::InvalidResponse);
    }
    Ok(DiscussionSummary {
        id,
        title: topic.title,
        author: discussion_author(post.author)?,
        created_at,
        views,
        comments,
        votes,
    })
}

fn discussion_from_response(
    response: DiscussionResponse,
    expected_id: u32,
) -> Result<Discussion, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let topic = response
        .data
        .ok_or(Error::InvalidResponse)?
        .topic
        .ok_or(Error::NotFound)?;
    let post = topic.post.ok_or(Error::InvalidResponse)?;
    let id = bounded_count(topic.id)?;
    let views = nonnegative_count(topic.view_count)?;
    let comments = nonnegative_count(topic.top_level_comment_count)?;
    let votes = signed_count(post.vote_count)?;
    let created_at = positive_number(post.creation_date).ok_or(Error::InvalidResponse)?;
    let updated_at = match post.updation_date {
        Some(value) => Some(positive_number(value).ok_or(Error::InvalidResponse)?),
        None => None,
    };
    let content = post.content.ok_or(Error::InvalidResponse)?;
    if id != expected_id
        || !valid_label(&topic.title, MAX_DISCUSSION_TITLE)
        || !valid_discussion_content(&content)
        || topic.tags.len() > MAX_DISCUSSION_TAGS
        || topic
            .tags
            .iter()
            .any(|tag| !valid_label(tag, MAX_DISCUSSION_TAG))
    {
        return Err(Error::InvalidResponse);
    }
    Ok(Discussion {
        id,
        title: topic.title,
        author: discussion_author(post.author)?,
        content,
        created_at,
        updated_at,
        views,
        comments,
        votes,
        tags: topic.tags,
        pinned: topic.pinned,
    })
}

fn discussion_article_content(response: DiscussionArticleResponse) -> Result<Box<str>, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let article = response
        .data
        .ok_or(Error::InvalidResponse)?
        .ugc_article_discussion_article
        .ok_or(Error::NotFound)?;
    if !valid_label(&article.uuid, MAX_DISCUSSION_ARTICLE_UUID)
        || !valid_discussion_content(&article.content)
    {
        return Err(Error::InvalidResponse);
    }
    Ok(article.content)
}

fn valid_discussion_content(content: &str) -> bool {
    content.len() <= MAX_DISCUSSION_CONTENT && !content.bytes().any(|byte| byte == b'\0')
}

fn discussion_author(author: Option<DiscussionAuthor>) -> Result<Option<Box<str>>, Error> {
    match author {
        None => Ok(None),
        Some(author) if author.username.as_ref() == "deleted_user" => Ok(None),
        Some(author) if valid_label(&author.username, 64) => Ok(Some(author.username)),
        Some(_) => Err(Error::InvalidResponse),
    }
}

fn bounded_count(value: NumberResponse) -> Result<u32, Error> {
    positive_number(value)
        .filter(|value| *value <= MAX_DISCUSSION_COUNT)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(Error::InvalidResponse)
}

fn nonnegative_count(value: NumberResponse) -> Result<u32, Error> {
    let value = match value {
        NumberResponse::Number(value) => value,
        NumberResponse::Text(value) => value.parse().map_err(|_| Error::InvalidResponse)?,
    };
    u32::try_from(value)
        .ok()
        .filter(|value| *value as u64 <= MAX_DISCUSSION_COUNT)
        .ok_or(Error::InvalidResponse)
}

fn signed_count(value: SignedNumberResponse) -> Result<i32, Error> {
    let value = match value {
        SignedNumberResponse::Number(value) => value,
        SignedNumberResponse::Text(value) => value.parse().map_err(|_| Error::InvalidResponse)?,
    };
    i32::try_from(value)
        .ok()
        .filter(|value| value.unsigned_abs() <= MAX_DISCUSSION_COUNT as u32)
        .ok_or(Error::InvalidResponse)
}

fn positive_number(value: NumberResponse) -> Option<u64> {
    match value {
        NumberResponse::Number(value) => Some(value),
        NumberResponse::Text(value) => value.parse().ok(),
    }
    .filter(|value| *value > 0)
}

fn difficulty_counts(
    rows: &[SubmissionCount],
    value: impl Fn(&SubmissionCount) -> u32,
) -> Result<DifficultyCounts, Error> {
    if rows.len() != 4 {
        return Err(Error::InvalidResponse);
    }
    let mut counts = [None; 4];
    for row in rows {
        if row.count > row.submissions
            || row.count > MAX_ACCOUNT_COUNT
            || row.submissions > MAX_ACCOUNT_COUNT
        {
            return Err(Error::InvalidResponse);
        }
        let index = match row.difficulty.as_ref() {
            "All" => 0,
            "Easy" => 1,
            "Medium" => 2,
            "Hard" => 3,
            _ => return Err(Error::InvalidResponse),
        };
        if counts[index].replace(value(row)).is_some() {
            return Err(Error::InvalidResponse);
        }
    }
    let [Some(all), Some(easy), Some(medium), Some(hard)] = counts else {
        return Err(Error::InvalidResponse);
    };
    Ok(DifficultyCounts {
        all,
        easy,
        medium,
        hard,
    })
}

fn counts_within(left: &DifficultyCounts, right: &DifficultyCounts) -> bool {
    left.all <= right.all
        && left.easy <= right.easy
        && left.medium <= right.medium
        && left.hard <= right.hard
}

fn starter_from_response(
    response: StarterResponse,
    expected_slug: &str,
) -> Result<StarterCode, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let question = response
        .data
        .ok_or(Error::InvalidResponse)?
        .question
        .ok_or(Error::NotFound)?;
    if question.is_paid_only {
        return Err(Error::PremiumRequired);
    }
    if !valid_question_id(&question.question_id)
        || question.title_slug.as_ref() != expected_slug
        || !valid_label(&question.title_slug, 128)
        || !valid_label(&question.title, 256)
    {
        return Err(Error::InvalidResponse);
    }
    let code_definition = question.code_definition.ok_or(Error::InvalidResponse)?;
    if code_definition.len() > MAX_CODE_DEFINITION_BYTES {
        return Err(Error::InvalidResponse);
    }
    let definitions: Vec<CodeDefinition> =
        serde_json::from_str(&code_definition).map_err(|_| Error::InvalidResponse)?;
    if definitions.is_empty() || definitions.len() > MAX_STARTER_SNIPPETS {
        return Err(Error::InvalidResponse);
    }
    let mut snippets = Vec::with_capacity(definitions.len());
    for definition in definitions {
        if !valid_label(&definition.text, 64)
            || !valid_language_slug(&definition.value)
            || !valid_source(&definition.default_code)
            || snippets.iter().any(|snippet: &CodeSnippet| {
                snippet.language == definition.text || snippet.language_slug == definition.value
            })
        {
            return Err(Error::InvalidResponse);
        }
        snippets.push(CodeSnippet {
            language: definition.text,
            language_slug: definition.value,
            source: definition.default_code,
        });
    }
    Ok(StarterCode {
        question_id: question.question_id,
        id: question.title_slug,
        title: question.title,
        snippets,
    })
}

fn test_cases_from_response(
    response: TestCasesResponse,
    expected_slug: &str,
) -> Result<TestCases, Error> {
    if !response.errors.is_empty() {
        return Err(Error::Graphql);
    }
    let question = response
        .data
        .ok_or(Error::InvalidResponse)?
        .question
        .ok_or(Error::NotFound)?;
    if question.is_paid_only {
        return Err(Error::PremiumRequired);
    }
    if !question.enable_run_code {
        return Err(Error::RunUnavailable);
    }
    if !valid_question_id(&question.question_id) || question.title_slug.as_ref() != expected_slug {
        return Err(Error::InvalidResponse);
    }

    let examples = question.example_testcase_list.unwrap_or_default();
    if examples.len() > MAX_TEST_CASES {
        return Err(Error::InvalidResponse);
    }
    let input = if examples.is_empty() {
        question
            .sample_test_case
            .filter(|sample| !sample.trim().is_empty())
            .ok_or(Error::InvalidResponse)?
            .into()
    } else {
        let mut input = String::new();
        for example in examples.iter().filter(|example| !example.trim().is_empty()) {
            if !input.is_empty() {
                input.push('\n');
            }
            input.push_str(example);
            if input.len() > MAX_TEST_INPUT_BYTES {
                return Err(Error::InvalidResponse);
            }
        }
        input
    };
    if !valid_test_input(&input) {
        return Err(Error::InvalidResponse);
    }
    Ok(TestCases {
        question_id: question.question_id,
        input: input.into(),
    })
}

fn submission_from_response(
    response: SubmissionResponse,
    id: u64,
) -> Result<SubmissionState, Error> {
    match response.state.as_ref() {
        "PENDING" | "STARTED" => Ok(SubmissionState::Pending),
        "SUCCESS" => {
            let status =
                optional_label(response.status_msg, 256)?.unwrap_or_else(|| "Finished".into());
            let accepted = response.status_code == Some(10) || status.as_ref() == "Accepted";
            let runtime = optional_label(response.status_runtime, 64)?;
            let memory = optional_label(response.status_memory, 64)?;
            if matches!(
                (response.total_correct, response.total_testcases),
                (Some(passed), Some(total)) if passed > total
            ) {
                return Err(Error::InvalidResponse);
            }
            let message = optional_message(response.full_compile_error)?
                .or(optional_message(response.full_runtime_error)?)
                .or(optional_message(response.compile_error)?)
                .or(optional_message(response.runtime_error)?);
            Ok(SubmissionState::Complete(SubmissionResult {
                id,
                status,
                accepted,
                runtime,
                memory,
                passed: response.total_correct,
                total: response.total_testcases,
                message,
            }))
        }
        _ => Err(Error::InvalidResponse),
    }
}

fn run_from_response(response: RunStatusResponse, id: &str) -> Result<RunState, Error> {
    match response.state.as_ref() {
        "PENDING" | "STARTED" => Ok(RunState::Pending),
        "SUCCESS" => {
            let status = optional_label(response.status_msg, 256)?;
            let status_code = response.status_code;
            let runtime = optional_label(response.status_runtime, 64)?;
            let memory = optional_label(response.status_memory, 64)?;
            if matches!(
                (response.total_correct, response.total_testcases),
                (Some(passed), Some(total)) if passed > total
            ) {
                return Err(Error::InvalidResponse);
            }
            let expected_answers = response
                .expected_code_answer
                .filter(|answers| !answers.is_empty())
                .or(response.expected_answer);
            let observed = response
                .code_answer
                .as_ref()
                .zip(expected_answers.as_ref())
                .filter(|(actual, expected)| !actual.is_empty() && actual.len() == expected.len())
                .and_then(|(actual, expected)| {
                    let total = u32::try_from(actual.len()).ok()?;
                    let passed = u32::try_from(
                        actual
                            .iter()
                            .zip(expected)
                            .filter(|(actual, expected)| actual == expected)
                            .count(),
                    )
                    .ok()?;
                    Some((passed, total))
                });
            let passed_cases = response
                .total_correct
                .or_else(|| observed.map(|(passed, _)| passed));
            let total_cases = response
                .total_testcases
                .or_else(|| observed.map(|(_, total)| total));
            let passed = response
                .correct_answer
                .or_else(|| {
                    passed_cases
                        .zip(total_cases)
                        .map(|(passed, total)| passed == total)
                })
                .unwrap_or(status_code == Some(10) || status.as_deref() == Some("Accepted"));
            let has_compile_error = response
                .full_compile_error
                .as_deref()
                .or(response.compile_error.as_deref())
                .is_some_and(|error| !error.is_empty());
            let has_runtime_error = response
                .full_runtime_error
                .as_deref()
                .or(response.runtime_error.as_deref())
                .is_some_and(|error| !error.is_empty());
            let mut status = status.unwrap_or_else(|| {
                if has_compile_error {
                    "Compile Error".into()
                } else if has_runtime_error {
                    "Runtime Error".into()
                } else if passed {
                    "Accepted".into()
                } else {
                    "Wrong Answer".into()
                }
            });
            if !passed && status_code == Some(10) && status.as_ref() == "Accepted" {
                status = "Wrong Answer".into();
            }
            let message = optional_output(response.full_compile_error)?
                .or(optional_output(response.full_runtime_error)?)
                .or(optional_output(response.compile_error)?)
                .or(optional_output(response.runtime_error)?);
            let output = optional_output_list(response.code_answer)?
                .or(optional_output_value(response.code_output)?)
                .or(optional_output_value(response.std_output_list)?);
            Ok(RunState::Complete(RunResult {
                id: id.into(),
                status,
                passed,
                runtime,
                memory,
                passed_cases,
                total_cases,
                input: optional_output(response.input_formatted)?
                    .or(optional_output(response.last_testcase)?),
                output,
                expected: optional_output_list(expected_answers)?
                    .or(optional_output(response.expected_output)?),
                message,
            }))
        }
        _ => Err(Error::InvalidResponse),
    }
}

fn optional_label(value: Option<Box<str>>, limit: usize) -> Result<Option<Box<str>>, Error> {
    match value {
        Some(value) if value.is_empty() => Ok(None),
        Some(value) if valid_label(&value, limit) => Ok(Some(value)),
        Some(_) => Err(Error::InvalidResponse),
        None => Ok(None),
    }
}

fn optional_message(value: Option<Box<str>>) -> Result<Option<Box<str>>, Error> {
    match value {
        Some(value) if value.is_empty() => Ok(None),
        Some(value) if value.len() <= 64 * 1024 && !value.bytes().any(|byte| byte == b'\0') => {
            Ok(Some(value))
        }
        Some(_) => Err(Error::InvalidResponse),
        None => Ok(None),
    }
}

fn optional_output(value: Option<Box<str>>) -> Result<Option<Box<str>>, Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty() {
        return Ok(None);
    }
    if value.len() > MAX_RUN_OUTPUT_BYTES || value.bytes().any(|byte| byte == b'\0') {
        return Err(Error::InvalidResponse);
    }
    let sanitized: String = value
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        .collect();
    Ok((!sanitized.is_empty()).then(|| sanitized.into()))
}

fn optional_output_value(value: Option<OutputValue>) -> Result<Option<Box<str>>, Error> {
    match value {
        Some(OutputValue::Text(value)) => optional_output(Some(value)),
        Some(OutputValue::List(values)) => optional_output_list(Some(values)),
        None => Ok(None),
    }
}

fn optional_output_list(values: Option<Vec<Box<str>>>) -> Result<Option<Box<str>>, Error> {
    let Some(values) = values else {
        return Ok(None);
    };
    if values.len() > MAX_TEST_CASES {
        return Err(Error::InvalidResponse);
    }
    let mut joined = String::new();
    for (index, value) in values.into_iter().enumerate() {
        let separator = usize::from(index > 0);
        if value.len() + separator > MAX_RUN_OUTPUT_BYTES.saturating_sub(joined.len()) {
            return Err(Error::InvalidResponse);
        }
        if index > 0 {
            joined.push('\n');
        }
        joined.push_str(&value);
    }
    optional_output(Some(joined.into()))
}

fn problem_from_question(
    question: Question,
    expected_slug: Option<&str>,
) -> Result<Problem, Error> {
    let number = question
        .frontend_question_id
        .as_deref()
        .and_then(|number| number.parse::<u32>().ok())
        .filter(|number| *number > 0)
        .ok_or(Error::InvalidResponse)?;
    if !valid_slug(&question.title_slug)
        || expected_slug.is_some_and(|slug| question.title_slug.as_ref() != slug)
        || !valid_label(&question.title, 256)
    {
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
        number,
        id: question.title_slug,
        title: question.title,
        statement,
    })
}

fn valid_label(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.len() <= limit
        && value == value.trim()
        && !value.chars().any(char::is_control)
}
