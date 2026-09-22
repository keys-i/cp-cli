use std::time::Duration;

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};

use crate::{
    ArchivePage, Difficulty, Error, PROBLEM_CONTENT_ATTRIBUTION, PROBLEM_CONTENT_LICENSE,
    PROBLEM_CONTENT_LICENSE_URL, Problem, ProblemCatalog, ProblemSummary, RecentProblems,
};

pub(crate) const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_STATEMENT_BYTES: usize = 512 * 1024;
const MAX_ARCHIVE_PAGE: u8 = 20;
const ARCHIVE_PAGE_SIZE: usize = 50;
const RECENT_PAGE_SIZE: usize = 10;
const MAX_CATALOG_PROBLEMS: usize = 4_096;
const CATALOG_HEADER: &str = "ID##Title##Published##Solved By";
const MAX_TITLE_BYTES: usize = 256;
const MAX_METADATA_BYTES: usize = 128;
const NOT_FOUND_SENTINEL: &str = "Data for that problem cannot be found";

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://projecteuler.net/".into(),
        })
    }

    /// List one bounded page of fifty public archive problems
    pub async fn archive(
        &self,
        page: u8,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<ArchivePage, Error> {
        if !(1..=MAX_ARCHIVE_PAGE).contains(&page) {
            return Err(Error::InvalidArchivePage);
        }
        let body = self
            .get_html(&format!("archives;page={page}"), progress)
            .await?;
        let problems = parse_problem_rows(&body, ARCHIVE_PAGE_SIZE)?;
        Ok(ArchivePage { page, problems })
    }

    /// List the ten most recently published public problems
    pub async fn recent(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<RecentProblems, Error> {
        let body = self.get_html("recent", progress).await?;
        let problems = parse_problem_rows(&body, RECENT_PAGE_SIZE)?;
        Ok(RecentProblems { problems })
    }

    /// List every public problem from Project Euler's compact catalogue
    pub async fn catalog(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<ProblemCatalog, Error> {
        let body = self.get_html("minimal=problems", progress).await?;
        Ok(ProblemCatalog {
            problems: parse_catalog(&body)?,
        })
    }

    /// Fetch the public statement and metadata for a Project Euler problem
    pub async fn problem(
        &self,
        number: u16,
        mut progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Problem, Error> {
        if number == 0 {
            return Err(Error::InvalidProblemNumber);
        }
        let page = self
            .get_html(&format!("problem={number}"), |received, total| {
                progress(received, total)
            })
            .await?;
        let (summary, difficulty) = parse_problem_page(&page, number)?;
        let statement_html = self
            .get_html(&format!("minimal={number}"), |received, total| {
                progress(received, total)
            })
            .await?;
        if statement_html.len() > MAX_STATEMENT_BYTES || !valid_html(&statement_html) {
            return Err(Error::InvalidResponse);
        }
        Ok(Problem {
            summary,
            difficulty,
            statement_html,
            attribution: PROBLEM_CONTENT_ATTRIBUTION,
            license: PROBLEM_CONTENT_LICENSE,
            license_url: PROBLEM_CONTENT_LICENSE_URL,
        })
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
        let body = read_response(response, progress).await?;
        if body.contains(NOT_FOUND_SENTINEL) {
            return Err(Error::NotFound);
        }
        Ok(body)
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
    let body = String::from_utf8(body).map_err(|_| Error::InvalidResponse)?;
    Ok(body.into())
}

fn parse_problem_rows(body: &str, expected_count: usize) -> Result<Vec<ProblemSummary>, Error> {
    let table =
        between(body, "<table id=\"problems_table\"", "</table>").ok_or(Error::InvalidResponse)?;
    let problems = table
        .split("<tr>")
        .skip(1)
        .filter_map(|row| row.split_once("</tr>").map(|(row, _)| row))
        .filter(|row| row.contains("<td class=\"id_column\">"))
        .map(parse_archive_row)
        .collect::<Result<Vec<_>, _>>()?;
    if problems.len() != expected_count {
        return Err(Error::InvalidResponse);
    }
    Ok(problems)
}

fn parse_catalog(body: &str) -> Result<Vec<ProblemSummary>, Error> {
    let mut lines = body.lines();
    if lines.next() != Some(CATALOG_HEADER) {
        return Err(Error::InvalidResponse);
    }
    let mut previous = 0;
    let mut problems = Vec::with_capacity(1_024);
    for line in lines {
        if problems.len() == MAX_CATALOG_PROBLEMS {
            return Err(Error::InvalidResponse);
        }
        let mut fields = line.split("##");
        let number = fields
            .next()
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|number| *number > previous)
            .ok_or(Error::InvalidResponse)?;
        let title = fields
            .next()
            .and_then(decode_text)
            .filter(|title| valid_text(title, MAX_TITLE_BYTES))
            .ok_or(Error::InvalidResponse)?;
        let published_at = fields
            .next()
            .and_then(decode_text)
            .filter(|published| valid_text(published, MAX_METADATA_BYTES))
            .ok_or(Error::InvalidResponse)?;
        let solved_count = fields
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or(Error::InvalidResponse)?;
        if fields.next().is_some() {
            return Err(Error::InvalidResponse);
        }
        previous = number;
        problems.push(ProblemSummary {
            number,
            title: title.into(),
            solved_count: Some(solved_count),
            published_at: Some(published_at.into()),
            canonical_url: canonical_url(number),
        });
    }
    (!problems.is_empty())
        .then_some(problems)
        .ok_or(Error::InvalidResponse)
}

fn parse_archive_row(row: &str) -> Result<ProblemSummary, Error> {
    let number = between(row, "<td class=\"id_column\">", "</td>")
        .and_then(|number| number.parse::<u16>().ok())
        .filter(|number| *number > 0)
        .ok_or(Error::InvalidResponse)?;
    let anchor_start = row
        .find("<a href=\"problem=")
        .ok_or(Error::InvalidResponse)?;
    let anchor = &row[anchor_start..];
    let href_start = "<a href=\"".len();
    let href_end = anchor
        .get(href_start..)
        .and_then(|value| value.find('\"').map(|end| href_start + end))
        .ok_or(Error::InvalidResponse)?;
    let href = anchor
        .get(href_start..href_end)
        .ok_or(Error::InvalidResponse)?;
    if href != format!("problem={number}") {
        return Err(Error::InvalidResponse);
    }
    let tag_end = anchor.find('>').ok_or(Error::InvalidResponse)?;
    let tag = &anchor[..tag_end];
    let title = anchor
        .get(tag_end + 1..)
        .and_then(|text| text.split_once("</a>").map(|(title, _)| title))
        .and_then(decode_text)
        .filter(|title| valid_text(title, MAX_TITLE_BYTES))
        .ok_or(Error::InvalidResponse)?;
    let published_at = attribute(tag, "title")
        .and_then(|published| published.strip_prefix("Published on "))
        .and_then(decode_text)
        .filter(|published| valid_text(published, MAX_METADATA_BYTES));
    let solved_count = between(anchor, "<div class=\"center\">", "</div>")
        .and_then(|count| count.parse::<u32>().ok());
    Ok(ProblemSummary {
        number,
        title: title.into(),
        solved_count,
        published_at: published_at.map(Into::into),
        canonical_url: canonical_url(number),
    })
}

fn parse_problem_page(
    body: &str,
    number: u16,
) -> Result<(ProblemSummary, Option<Difficulty>), Error> {
    let title = between(body, "<div id=\"content\">", "</h2>")
        .and_then(|content| content.rsplit_once("<h2>").map(|(_, title)| title))
        .and_then(decode_text)
        .filter(|title| valid_text(title, MAX_TITLE_BYTES))
        .ok_or(Error::InvalidResponse)?;
    if !body.contains(&format!("<h3>Problem {number}</h3>")) {
        return Err(Error::InvalidResponse);
    }
    let tooltip = between(body, "tooltiptext_right\">", "</span>");
    let status = tooltip.and_then(|value| value.split_once("<br>").map(|(status, _)| status));
    let published_at = status
        .and_then(|value| value.strip_prefix("Published on "))
        .map(|value| {
            value
                .split_once(" and solved by ")
                .map_or(value, |(date, _)| date)
        })
        .and_then(decode_text)
        .filter(|date| valid_text(date, MAX_METADATA_BYTES));
    let solved_count = status
        .and_then(|value| value.split_once("solved by ").map(|(_, count)| count))
        .and_then(|count| {
            count
                .split_once('<')
                .map(|(count, _)| count)
                .or(Some(count))
        })
        .and_then(|count| count.trim().parse::<u32>().ok());
    let difficulty = tooltip.and_then(parse_difficulty);
    Ok((
        ProblemSummary {
            number,
            title: title.into(),
            solved_count,
            published_at: published_at.map(Into::into),
            canonical_url: canonical_url(number),
        },
        difficulty,
    ))
}

fn parse_difficulty(tooltip: &str) -> Option<Difficulty> {
    let value = tooltip.split_once("Difficulty: Level ")?.1;
    let (level, percentage) = value.split_once(" [")?;
    let percentage = percentage.strip_suffix("%]")?;
    Some(Difficulty {
        level: level.parse().ok()?,
        percentage: percentage.parse().ok()?,
    })
}

fn between<'a>(value: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let value = value.split_once(start)?.1;
    value.split_once(end).map(|(value, _)| value)
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}=\"");
    let value = tag.split_once(&prefix)?.1;
    value.split_once('\"').map(|(value, _)| value)
}

fn decode_text(value: &str) -> Option<String> {
    let value = value
        .replace("<sup>", "")
        .replace("</sup>", "")
        .replace("<sub>", "")
        .replace("</sub>", "");
    if value.contains('<') || value.len() > MAX_TITLE_BYTES {
        return None;
    }
    let value = value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    (!value.is_empty() && !value.as_bytes().contains(&0)).then_some(value)
}

fn canonical_url(number: u16) -> Box<str> {
    format!("https://projecteuler.net/problem={number}").into()
}

fn valid_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.as_bytes().contains(&0)
}

fn valid_html(value: &str) -> bool {
    !value.trim().is_empty() && !value.as_bytes().contains(&0)
}
