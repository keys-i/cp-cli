use std::{collections::BTreeMap, time::Duration};

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    Difficulty, Error, Problem, ProblemList, ProblemSummary, Profile, Sample, Starter, Track,
};

pub(crate) const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_STATEMENT_BYTES: usize = 512 * 1024;
const MAX_TEXT_BYTES: usize = 128 * 1024;
const MAX_LANGUAGES: usize = 64;
const MAX_STARTERS: usize = 64;
const MAX_SAMPLES: usize = 16;
const MAX_TEMPLATE_BYTES: usize = 128 * 1024;
const MAX_SAMPLE_BYTES: usize = 64 * 1024;
const MAX_PROFILE_TEXT_BYTES: usize = 256;
const MAX_PROFILE_LEVEL: u32 = 100_000;
const MAX_PROFILE_EVENTS: u32 = 10_000_000;
const MAX_LIST_LIMIT: u8 = 20;
const MAX_OFFSET: u32 = 1_000_000;
const MAX_TOTAL: u64 = 10_000_000;
const MAX_PROBLEMS: usize = MAX_LIST_LIMIT as usize;

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://www.hackerrank.com/".into(),
        })
    }

    /// List a public HackerRank Community track
    pub async fn list_track(
        &self,
        track: &str,
        offset: u32,
        limit: u8,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<ProblemList, Error> {
        if !valid_slug(track) {
            return Err(Error::InvalidSlug);
        }
        if offset > MAX_OFFSET || limit == 0 || limit > MAX_LIST_LIMIT {
            return Err(Error::InvalidPage);
        }

        let path = format!("rest/contests/master/tracks/{track}/challenges");
        let response: ListResponse = self
            .get_json(
                &path,
                &[
                    ("offset", offset.to_string()),
                    ("limit", limit.to_string()),
                    ("track_login", "true".to_owned()),
                ],
                progress,
            )
            .await?;
        list_from_response(response)
    }

    /// Fetch one public HackerRank Community problem from the master contest
    pub async fn problem(
        &self,
        slug: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Problem, Error> {
        if !valid_slug(slug) {
            return Err(Error::InvalidSlug);
        }

        let path = format!("rest/contests/master/challenges/{slug}");
        let response: ProblemResponse = self.get_json(&path, &[], progress).await?;
        if !response.status {
            return Err(Error::NotFound);
        }
        let model = response.model.ok_or(Error::NotFound)?;
        problem_from_model(model, Some(slug))
    }

    /// Fetch a public HackerRank profile without using browser credentials
    pub async fn profile(
        &self,
        username: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Profile, Error> {
        if !valid_username(username) {
            return Err(Error::InvalidUsername);
        }
        let path = format!("rest/contests/master/hackers/{username}/profile");
        let response: ProfileResponse = self.get_json(&path, &[], progress).await?;
        profile_from_model(response.model.ok_or(Error::NotFound)?)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&'static str, String)],
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<T, Error> {
        let mut url = self.endpoint_path(path)?;
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        let response = self.http.get(url).send().await?;
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
    Ok(serde_json::from_slice(&body)?)
}

#[derive(Deserialize)]
struct ListResponse {
    models: Vec<ChallengeModel>,
    total: u64,
}

#[derive(Deserialize)]
struct ProblemResponse {
    status: bool,
    model: Option<ChallengeModel>,
}

#[derive(Deserialize)]
struct ProfileResponse {
    model: Option<ProfileModel>,
}

#[derive(Deserialize)]
struct ProfileModel {
    id: u64,
    username: Box<str>,
    name: Option<Box<str>>,
    country: Option<Box<str>>,
    level: Option<u32>,
    event_count: Option<u32>,
    created_at: Option<Box<str>>,
}

#[derive(Deserialize)]
struct ChallengeModel {
    id: u64,
    slug: Box<str>,
    name: Box<str>,
    difficulty_name: Option<Box<str>>,
    preview: Option<Box<str>>,
    track: Option<TrackModel>,
    problem_statement: Option<Box<str>>,
    input_format: Option<Box<str>>,
    output_format: Option<Box<str>>,
    #[serde(default)]
    languages: Vec<Box<str>>,
    #[serde(default)]
    onboarding: Option<BTreeMap<Box<str>, OnboardingEntry>>,
    #[serde(default)]
    sample_test_cases: Option<Vec<SampleModel>>,
    #[serde(default)]
    public_test_cases: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OnboardingEntry {
    Template { template: Option<Box<str>> },
    Text(Box<str>),
}

#[derive(Deserialize)]
struct SampleModel {
    input: Box<str>,
    output: Box<str>,
}

#[derive(Deserialize)]
struct TrackModel {
    slug: Box<str>,
    name: Box<str>,
    track_slug: Box<str>,
    track_name: Box<str>,
}

fn list_from_response(response: ListResponse) -> Result<ProblemList, Error> {
    if response.total > MAX_TOTAL || response.models.len() > MAX_PROBLEMS {
        return Err(Error::InvalidResponse);
    }
    let problems = response
        .models
        .into_iter()
        .map(|model| summary_from_model(model, None))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ProblemList {
        total: response.total as u32,
        problems,
    })
}

fn problem_from_model(
    model: ChallengeModel,
    expected_slug: Option<&str>,
) -> Result<Problem, Error> {
    let ChallengeModel {
        id,
        slug,
        name,
        difficulty_name,
        preview,
        track,
        problem_statement,
        input_format,
        output_format,
        languages,
        onboarding,
        sample_test_cases,
        public_test_cases,
    } = model;
    let statement = checked_text(problem_statement, MAX_STATEMENT_BYTES)?;
    let input_format = checked_text(input_format, MAX_TEXT_BYTES)?;
    let output_format = checked_text(output_format, MAX_TEXT_BYTES)?;
    if languages.len() > MAX_LANGUAGES || languages.iter().any(|language| !valid_language(language))
    {
        return Err(Error::InvalidResponse);
    }
    let starters = starters_from_onboarding(onboarding)?;
    let samples = samples_from_model(sample_test_cases)?;
    let summary = summary_from_parts(
        id,
        slug,
        name,
        difficulty_name,
        preview,
        track,
        expected_slug,
    )?;
    Ok(Problem {
        summary,
        statement,
        input_format,
        output_format,
        languages,
        starters,
        samples,
        has_public_test_cases: public_test_cases,
    })
}

fn starters_from_onboarding(
    onboarding: Option<BTreeMap<Box<str>, OnboardingEntry>>,
) -> Result<Vec<Starter>, Error> {
    let mut starters = Vec::new();
    for (language, entry) in onboarding.unwrap_or_default() {
        let template = match entry {
            OnboardingEntry::Template {
                template: Some(template),
            } => template,
            OnboardingEntry::Template { template: None } => continue,
            OnboardingEntry::Text(text) => {
                let _ = text.len();
                continue;
            }
        };
        if !valid_language(&language) || !valid_text(&template, MAX_TEMPLATE_BYTES) {
            return Err(Error::InvalidResponse);
        }
        starters.push(Starter { language, template });
        if starters.len() > MAX_STARTERS {
            return Err(Error::InvalidResponse);
        }
    }
    Ok(starters)
}

fn samples_from_model(samples: Option<Vec<SampleModel>>) -> Result<Vec<Sample>, Error> {
    let samples = samples.unwrap_or_default();
    if samples.len() > MAX_SAMPLES {
        return Err(Error::InvalidResponse);
    }
    samples
        .into_iter()
        .map(|sample| {
            if !valid_text(&sample.input, MAX_SAMPLE_BYTES)
                || !valid_text(&sample.output, MAX_SAMPLE_BYTES)
            {
                return Err(Error::InvalidResponse);
            }
            Ok(Sample {
                input: sample.input,
                output: sample.output,
            })
        })
        .collect()
}

fn profile_from_model(model: ProfileModel) -> Result<Profile, Error> {
    if model.id == 0
        || !valid_username(&model.username)
        || model.level.is_some_and(|level| level > MAX_PROFILE_LEVEL)
        || model
            .event_count
            .is_some_and(|events| events > MAX_PROFILE_EVENTS)
    {
        return Err(Error::InvalidResponse);
    }
    Ok(Profile {
        id: model.id,
        username: model.username,
        name: profile_text(model.name)?,
        country: profile_text(model.country)?,
        level: model.level,
        event_count: model.event_count,
        created_at: profile_text(model.created_at)?,
    })
}

fn profile_text(value: Option<Box<str>>) -> Result<Option<Box<str>>, Error> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    valid_text(value, MAX_PROFILE_TEXT_BYTES)
        .then(|| Some(value.into()))
        .ok_or(Error::InvalidResponse)
}

fn summary_from_model(
    model: ChallengeModel,
    expected_slug: Option<&str>,
) -> Result<ProblemSummary, Error> {
    summary_from_parts(
        model.id,
        model.slug,
        model.name,
        model.difficulty_name,
        model.preview,
        model.track,
        expected_slug,
    )
}

fn summary_from_parts(
    id: u64,
    slug: Box<str>,
    title: Box<str>,
    difficulty_name: Option<Box<str>>,
    preview: Option<Box<str>>,
    track: Option<TrackModel>,
    expected_slug: Option<&str>,
) -> Result<ProblemSummary, Error> {
    if id == 0
        || !valid_slug(&slug)
        || expected_slug.is_some_and(|expected| expected != slug.as_ref())
        || !valid_text(&title, 256)
        || preview
            .as_deref()
            .is_some_and(|text| !valid_text(text, MAX_TEXT_BYTES))
    {
        return Err(Error::InvalidResponse);
    }
    let track = track.map(track_from_model).transpose()?;
    Ok(ProblemSummary {
        id,
        slug,
        title,
        difficulty: difficulty(difficulty_name.as_deref()),
        preview,
        track,
    })
}

fn track_from_model(track: TrackModel) -> Result<Track, Error> {
    if !valid_slug(&track.slug)
        || !valid_slug(&track.track_slug)
        || !valid_text(&track.name, 128)
        || !valid_text(&track.track_name, 128)
    {
        return Err(Error::InvalidResponse);
    }
    Ok(Track {
        slug: track.slug,
        name: track.name,
        domain_slug: track.track_slug,
        domain_name: track.track_name,
    })
}

fn checked_text(text: Option<Box<str>>, limit: usize) -> Result<Box<str>, Error> {
    let text = text.ok_or(Error::InvalidResponse)?;
    valid_text(&text, limit)
        .then_some(text)
        .ok_or(Error::InvalidResponse)
}

fn difficulty(value: Option<&str>) -> Difficulty {
    match value {
        Some("Easy") => Difficulty::Easy,
        Some("Medium") => Difficulty::Medium,
        Some("Hard") => Difficulty::Hard,
        _ => Difficulty::Other,
    }
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_language(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn valid_username(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_text(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit && !value.as_bytes().contains(&0)
}
