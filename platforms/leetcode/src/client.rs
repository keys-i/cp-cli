use std::time::Duration;

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::{Deserialize, Serialize};

use crate::{Error, Problem};

pub(crate) const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const QUERY: &str = "query questionData($titleSlug: String!) {
    question(titleSlug: $titleSlug) { titleSlug title content isPaidOnly }
}";

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
        if slug.is_empty()
            || slug.len() > 128
            || !slug
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(Error::InvalidSlug);
        }

        let response = self
            .http
            .post(self.endpoint.as_ref())
            .header(reqwest::header::REFERER, "https://leetcode.com/")
            .header(reqwest::header::ORIGIN, "https://leetcode.com")
            .json(&ProblemRequest {
                query: QUERY,
                operation_name: "questionData",
                variables: Variables { title_slug: slug },
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }

        let response = read_response(response, progress).await?;
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

async fn read_response(
    mut response: reqwest::Response,
    mut progress: impl FnMut(usize, Option<u64>),
) -> Result<ProblemResponse, Error> {
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
