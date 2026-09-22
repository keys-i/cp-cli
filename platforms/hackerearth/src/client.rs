use std::time::Duration;

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::Deserialize;

use crate::{Difficulty, Error, PracticePage, Problem, ProblemSummary};

pub(crate) const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_STATEMENT_BYTES: usize = 512 * 1024;
const MAX_PAGE: u8 = 100;
const MAX_PROBLEMS: usize = 50;
const MAX_TAGS: usize = 16;
const MAX_TITLE_BYTES: usize = 256;

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://www.hackerearth.com/".into(),
        })
    }

    /// List one public practice-topic page
    pub async fn practice(
        &self,
        topic: &str,
        page: u8,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<PracticePage, Error> {
        if !valid_topic(topic) {
            return Err(Error::InvalidTopic);
        }
        if page == 0 || page > MAX_PAGE {
            return Err(Error::InvalidPage);
        }
        let suffix = if page == 1 {
            String::new()
        } else {
            format!("{page}/")
        };
        let path = format!("practice/{topic}/practice-problems/{suffix}");
        let body = self.get_html(&path, progress).await?;
        let problems = parse_practice_page(&body)?;
        Ok(PracticePage {
            topic: topic.into(),
            page,
            problems,
        })
    }

    /// Fetch a public practice problem from its canonical slug
    pub async fn problem(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Problem, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }
        let body = self
            .get_html(&format!("problem/algorithm/{slug}/"), progress)
            .await?;
        let data: ProblemData = serde_json::from_str(json_object(&body, "problemData: ")?)
            .map_err(|_| Error::InvalidResponse)?;
        problem_from_data(slug, data)
    }

    async fn get_html(
        &self,
        path: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Box<str>, Error> {
        let response = self.http.get(self.endpoint_path(path)?).send().await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        read_response(response, progress).await
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

async fn read_response(
    mut response: reqwest::Response,
    mut progress: impl FnMut(usize, Option<u64>),
) -> Result<Box<str>, Error> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(Error::ResponseTooLarge {
            limit: MAX_RESPONSE_BYTES,
        });
    }
    let total = response.content_length();
    let mut body = Vec::with_capacity(8 * 1024);
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
    String::from_utf8(body)
        .map(Into::into)
        .map_err(|_| Error::InvalidResponse)
}

#[derive(Deserialize)]
struct ProblemData {
    title: Box<str>,
    description: Box<str>,
    tags: Box<str>,
    level: Box<str>,
    success_rate: Option<u8>,
    points: Option<u16>,
    attempted_by: Option<u32>,
    time_limit: Option<f64>,
    memory_limit: Option<u16>,
}

fn parse_practice_page(body: &str) -> Result<Vec<ProblemSummary>, Error> {
    let list = between(body, "<ul class=\"prob-list\" id=\"prob-list\">", "</ul>")
        .ok_or(Error::InvalidResponse)?;
    let problems = list
        .split("<li class=\"prob ")
        .skip(1)
        .filter_map(|item| item.split_once("</li>").map(|(item, _)| item))
        .filter(|item| item.contains("/problem/algorithm/"))
        .map(parse_summary)
        .collect::<Result<Vec<_>, _>>()?;
    if problems.is_empty() || problems.len() > MAX_PROBLEMS {
        return Err(Error::InvalidResponse);
    }
    Ok(problems)
}

fn parse_summary(item: &str) -> Result<ProblemSummary, Error> {
    const HREF: &str = "href=\"/problem/algorithm/";
    let anchor = item.split_once(HREF).ok_or(Error::InvalidResponse)?.1;
    let (slug, anchor) = anchor.split_once("/\"").ok_or(Error::InvalidResponse)?;
    if !valid_slug(slug) {
        return Err(Error::InvalidResponse);
    }
    let title = anchor
        .split_once('>')
        .and_then(|(_, text)| text.split_once("</a>").map(|(title, _)| title))
        .and_then(decode_text)
        .filter(|title| valid_text(title, MAX_TITLE_BYTES))
        .ok_or(Error::InvalidResponse)?;
    let difficulty = difficulty_from_text(labeled_text(item, "LEVEL:")?)?;
    let attempted_by = labeled_number(item, "ATTEMPTED BY:");
    let success_rate = labeled_number(item, "SUCCESS RATE:")
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value <= 100);
    Ok(ProblemSummary {
        slug: slug.into(),
        title: title.into(),
        difficulty,
        attempted_by,
        success_rate,
        canonical_url: canonical_url(slug),
    })
}

fn problem_from_data(slug: &str, data: ProblemData) -> Result<Problem, Error> {
    let title = decode_text(&data.title)
        .filter(|title| valid_text(title, MAX_TITLE_BYTES))
        .ok_or(Error::InvalidResponse)?;
    if data.description.is_empty()
        || data.description.len() > MAX_STATEMENT_BYTES
        || data.description.as_bytes().contains(&0)
    {
        return Err(Error::InvalidResponse);
    }
    let tags = data
        .tags
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(|tag| {
            decode_text(tag)
                .filter(|tag| valid_text(tag, 64))
                .map(Into::into)
                .ok_or(Error::InvalidResponse)
        })
        .collect::<Result<Vec<Box<str>>, _>>()?;
    if tags.len() > MAX_TAGS {
        return Err(Error::InvalidResponse);
    }
    let time_limit_seconds = data
        .time_limit
        .filter(|value| value.is_finite() && *value >= 1.0 && *value <= 600.0)
        .and_then(|value| (value.fract() == 0.0).then_some(value as u16));
    Ok(Problem {
        summary: ProblemSummary {
            slug: slug.into(),
            title: title.into(),
            difficulty: difficulty_from_text(&data.level)?,
            attempted_by: data.attempted_by,
            success_rate: data.success_rate.filter(|value| *value <= 100),
            canonical_url: canonical_url(slug),
        },
        statement_html: data.description,
        tags,
        points: data.points,
        time_limit_seconds,
        memory_limit_mb: data.memory_limit,
    })
}

fn json_object<'a>(body: &'a str, marker: &str) -> Result<&'a str, Error> {
    let value = body.split_once(marker).ok_or(Error::NotFound)?.1;
    let start = value.find('{').ok_or(Error::InvalidResponse)?;
    let mut depth = 0_u16;
    let mut escaped = false;
    let mut quoted = false;
    for (index, byte) in value[start..].bytes().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'\"' => quoted = true,
            b'{' => depth = depth.checked_add(1).ok_or(Error::InvalidResponse)?,
            b'}' => {
                depth = depth.checked_sub(1).ok_or(Error::InvalidResponse)?;
                if depth == 0 {
                    return value
                        .get(start..=start + index)
                        .ok_or(Error::InvalidResponse);
                }
            }
            _ => {}
        }
    }
    Err(Error::InvalidResponse)
}

fn labeled_text<'a>(item: &'a str, label: &str) -> Result<&'a str, Error> {
    let value = item.split_once(label).ok_or(Error::InvalidResponse)?.1;
    let value = value.split_once('>').ok_or(Error::InvalidResponse)?.1;
    value
        .split_once('<')
        .map(|(value, _)| value.trim())
        .ok_or(Error::InvalidResponse)
}

fn labeled_number(item: &str, label: &str) -> Option<u32> {
    labeled_text(item, label)
        .ok()?
        .trim_end_matches('%')
        .parse()
        .ok()
}

fn difficulty_from_text(value: &str) -> Result<Difficulty, Error> {
    match value.trim() {
        "Easy" | "E" => Ok(Difficulty::Easy),
        "Medium" | "M" => Ok(Difficulty::Medium),
        "Hard" | "H" => Ok(Difficulty::Hard),
        _ => Err(Error::InvalidResponse),
    }
}

fn between<'a>(value: &'a str, start: &str, end: &str) -> Option<&'a str> {
    value
        .split_once(start)?
        .1
        .split_once(end)
        .map(|(value, _)| value)
}

fn decode_text(value: &str) -> Option<String> {
    if value.contains('<') || value.as_bytes().contains(&0) {
        return None;
    }
    Some(
        value
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .trim()
            .to_owned(),
    )
}

fn canonical_url(slug: &str) -> Box<str> {
    format!("https://www.hackerearth.com/problem/algorithm/{slug}/").into()
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_topic(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('/')
        && !value.ends_with('/')
        && value.split('/').all(valid_slug)
}

fn valid_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.as_bytes().contains(&0)
}
