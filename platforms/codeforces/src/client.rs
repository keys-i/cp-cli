use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::Deserialize;
use sha2::{Digest, Sha512};

use crate::{
    ApiCredentials, Contest, Discussion, DiscussionComment, DiscussionPreview, Error, Problem,
    RatingChange, Submission, UserProfile,
};

pub(crate) const MAX_RESPONSE_BYTES: usize = 3 * 1024 * 1024;
const MAX_PROBLEMS: usize = 50_000;
const MAX_CONTESTS: usize = 100;
const MAX_DISCUSSIONS: usize = 20;
const MAX_COMMENTS: usize = 1_000;

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://codeforces.com/api".into(),
        })
    }

    /// Fetch bounded public problem metadata from Codeforces
    pub async fn problems(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<Problem>, Error> {
        let response = self
            .http
            .get(format!("{}/problemset.problems?lang=en", self.endpoint))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }

        let response: ApiResponse<ProblemSet> = read_response(response, progress).await?;
        let result = match (response.status.as_ref(), response.result) {
            ("OK", Some(result)) => result,
            _ => return Err(Error::ApiFailure),
        };
        if result.problems.len() > MAX_PROBLEMS {
            return Err(Error::ApiFailure);
        }

        let statistics = result
            .problem_statistics
            .into_iter()
            .filter_map(|statistic| {
                Some((
                    (statistic.contest_id?, statistic.index),
                    statistic.solved_count,
                ))
            })
            .collect::<std::collections::HashMap<_, _>>();

        Ok(result
            .problems
            .into_iter()
            .filter_map(|problem| problem.into_problem(&statistics))
            .collect())
    }

    /// Resolve one public problem from its canonical contest ID and index
    pub async fn problem(
        &self,
        contest_id: u32,
        index: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Problem, Error> {
        if contest_id == 0 {
            return Err(Error::InvalidContestId);
        }
        if !valid_index(index) {
            return Err(Error::InvalidIndex);
        }

        self.problems(progress)
            .await?
            .into_iter()
            .find(|problem| {
                problem.contest_id == contest_id && problem.index.eq_ignore_ascii_case(index)
            })
            .ok_or_else(|| Error::NotFound {
                contest_id,
                index: index.into(),
            })
    }

    /// Fetch the next and most recent public non-gym contests
    pub async fn contests(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<Contest>, Error> {
        let result: Vec<ContestData> = self.get("contest.list?gym=false", progress).await?;
        if result.len() > MAX_PROBLEMS {
            return Err(Error::ApiFailure);
        }
        Ok(result
            .into_iter()
            .take(MAX_CONTESTS)
            .map(ContestData::into_contest)
            .collect())
    }

    /// Fetch one public Codeforces account profile
    pub async fn user_profile(
        &self,
        handle: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<UserProfile, Error> {
        validate_handle(handle)?;
        let result: Vec<UserData> = self
            .get(&format!("user.info?handles={handle}"), progress)
            .await?;
        result
            .into_iter()
            .next()
            .map(UserData::into_profile)
            .ok_or(Error::ApiFailure)
    }

    /// Fetch a public Codeforces rating history
    pub async fn rating_history(
        &self,
        handle: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<RatingChange>, Error> {
        validate_handle(handle)?;
        let result: Vec<RatingChangeData> = self
            .get(&format!("user.rating?handle={handle}"), progress)
            .await?;
        if result.len() > MAX_PROBLEMS {
            return Err(Error::ApiFailure);
        }
        Ok(result
            .into_iter()
            .map(RatingChangeData::into_rating_change)
            .collect())
    }

    /// Fetch up to 100 public submissions, newest first
    pub async fn submissions(
        &self,
        handle: &str,
        count: u8,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<Submission>, Error> {
        validate_handle(handle)?;
        if count == 0 || count > 100 {
            return Err(Error::InvalidSubmissionCount);
        }
        let result: Vec<SubmissionData> = self
            .get(
                &format!("user.status?handle={handle}&from=1&count={count}"),
                progress,
            )
            .await?;
        if result.len() > usize::from(count) {
            return Err(Error::ApiFailure);
        }
        Ok(result
            .into_iter()
            .map(SubmissionData::into_submission)
            .collect())
    }

    /// Verify Codeforces API credentials with the authorized `user.friends` endpoint
    pub async fn authenticated_friends(
        &self,
        credentials: &ApiCredentials,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<Box<str>>, Error> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Clock)?
            .as_secs();
        let query = signed_friends_query(credentials, time, &signature_prefix()?);
        self.get(&format!("user.friends?{query}"), progress).await
    }

    /// Fetch up to 20 public blog entries from Codeforces' recent activity feed
    pub async fn recent_discussions(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Vec<DiscussionPreview>, Error> {
        let result: Vec<RecentActionData> = self.get("recentActions?maxCount=20", progress).await?;
        if result.len() > MAX_DISCUSSIONS {
            return Err(Error::ApiFailure);
        }

        let mut seen = std::collections::HashSet::new();
        Ok(result
            .into_iter()
            .filter_map(|action| {
                let entry = action.blog_entry?;
                seen.insert(entry.id).then(|| DiscussionPreview {
                    id: entry.id,
                    author: entry.author_handle,
                    title: entry.title,
                    created_time_seconds: entry.creation_time_seconds,
                    activity_time_seconds: action.time_seconds,
                    rating: entry.rating,
                    tags: entry.tags,
                    latest_comment: action.comment.map(CommentData::into_comment),
                    url: format!("https://codeforces.com/blog/entry/{}", entry.id).into(),
                })
            })
            .collect())
    }

    /// Fetch a public blog entry and its comments
    pub async fn discussion(
        &self,
        id: u32,
        mut progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Discussion, Error> {
        if id == 0 {
            return Err(Error::InvalidBlogEntryId);
        }
        let entry: BlogEntryData = self
            .get(&format!("blogEntry.view?blogEntryId={id}"), &mut progress)
            .await?;
        // Codeforces documents an anonymous API ceiling of one request every two seconds
        tokio::time::sleep(Duration::from_secs(2)).await;
        let comments: Vec<CommentData> = self
            .get(
                &format!("blogEntry.comments?blogEntryId={id}"),
                &mut progress,
            )
            .await?;
        if comments.len() > MAX_COMMENTS {
            return Err(Error::ApiFailure);
        }
        let content_html = entry.content.ok_or(Error::ApiFailure)?;
        Ok(Discussion {
            id: entry.id,
            author: entry.author_handle,
            title: entry.title,
            content_html,
            created_time_seconds: entry.creation_time_seconds,
            modified_time_seconds: entry.modification_time_seconds,
            rating: entry.rating,
            tags: entry.tags,
            comments: comments
                .into_iter()
                .map(CommentData::into_comment)
                .collect(),
            url: format!("https://codeforces.com/blog/entry/{}", entry.id).into(),
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path_and_query: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<T, Error> {
        let response = self
            .http
            .get(format!("{}/{}", self.endpoint, path_and_query))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        let response: ApiResponse<T> = read_response(response, progress).await?;
        match (response.status.as_ref(), response.result) {
            ("OK", Some(result)) => Ok(result),
            _ => Err(Error::ApiFailure),
        }
    }
}

fn signature_prefix() -> Result<String, Error> {
    let mut bytes = [0_u8; 3];
    getrandom::fill(&mut bytes).map_err(|_| Error::SignatureNonce)?;
    Ok(format!("{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2]))
}

fn signed_friends_query(credentials: &ApiCredentials, time: u64, prefix: &str) -> String {
    let query = format!("apiKey={}&onlyOnline=true&time={time}", credentials.key);
    let signature = format!("{prefix}/user.friends?{query}#{}", credentials.secret);
    let hash = Sha512::digest(signature.as_bytes());
    format!("{query}&apiSig={prefix}{hash:x}")
}

pub(crate) fn http_builder() -> ClientBuilder {
    HttpClient::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(30))
        .redirect(Policy::none())
        .retry(reqwest::retry::never())
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

async fn read_response<T: for<'de> Deserialize<'de>>(
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

fn valid_index(index: &str) -> bool {
    !index.is_empty() && index.len() <= 16 && index.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn validate_handle(handle: &str) -> Result<(), Error> {
    if handle.is_empty()
        || handle.len() > 24
        || !handle
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(Error::InvalidHandle);
    }
    Ok(())
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    status: Box<str>,
    result: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemSet {
    problems: Vec<ProblemData>,
    #[serde(default)]
    problem_statistics: Vec<ProblemStatistics>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemData {
    contest_id: Option<u32>,
    index: Box<str>,
    name: Box<str>,
    rating: Option<u16>,
    #[serde(default)]
    tags: Vec<Box<str>>,
}

impl ProblemData {
    fn into_problem(
        self,
        statistics: &std::collections::HashMap<(u32, Box<str>), u32>,
    ) -> Option<Problem> {
        let contest_id = self.contest_id?;
        if !valid_index(&self.index) {
            return None;
        }
        let solved_count = statistics.get(&(contest_id, self.index.clone())).copied();
        let url = format!(
            "https://codeforces.com/problemset/problem/{contest_id}/{}",
            self.index
        );
        Some(Problem {
            contest_id,
            index: self.index,
            title: self.name,
            rating: self.rating,
            tags: self.tags,
            solved_count,
            url: url.into(),
            statement: None,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProblemStatistics {
    contest_id: Option<u32>,
    index: Box<str>,
    solved_count: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContestData {
    id: u32,
    name: Box<str>,
    #[serde(rename = "type")]
    kind: Box<str>,
    phase: Box<str>,
    start_time_seconds: Option<i64>,
    duration_seconds: Option<u32>,
}

impl ContestData {
    fn into_contest(self) -> Contest {
        Contest {
            id: self.id,
            name: self.name,
            kind: self.kind,
            phase: self.phase,
            start_time_seconds: self.start_time_seconds,
            duration_seconds: self.duration_seconds,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserData {
    handle: Box<str>,
    first_name: Option<Box<str>>,
    last_name: Option<Box<str>>,
    country: Option<Box<str>>,
    city: Option<Box<str>>,
    organization: Option<Box<str>>,
    rank: Option<Box<str>>,
    rating: Option<i32>,
    max_rank: Option<Box<str>>,
    max_rating: Option<i32>,
    contribution: i32,
    friend_of_count: u32,
    registration_time_seconds: i64,
    last_online_time_seconds: i64,
}

impl UserData {
    fn into_profile(self) -> UserProfile {
        UserProfile {
            handle: self.handle,
            first_name: self.first_name,
            last_name: self.last_name,
            country: self.country,
            city: self.city,
            organization: self.organization,
            rank: self.rank,
            rating: self.rating,
            max_rank: self.max_rank,
            max_rating: self.max_rating,
            contribution: self.contribution,
            friend_of_count: self.friend_of_count,
            registration_time_seconds: self.registration_time_seconds,
            last_online_time_seconds: self.last_online_time_seconds,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RatingChangeData {
    contest_id: u32,
    contest_name: Box<str>,
    rank: u32,
    old_rating: i32,
    new_rating: i32,
    rating_update_time_seconds: i64,
}

impl RatingChangeData {
    fn into_rating_change(self) -> RatingChange {
        RatingChange {
            contest_id: self.contest_id,
            contest_name: self.contest_name,
            rank: self.rank,
            old_rating: self.old_rating,
            new_rating: self.new_rating,
            rating_update_time_seconds: self.rating_update_time_seconds,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SubmissionData {
    id: u64,
    contest_id: Option<u32>,
    creation_time_seconds: i64,
    relative_time_seconds: i64,
    problem: SubmissionProblemData,
    programming_language: Box<str>,
    verdict: Option<Box<str>>,
    testset: Box<str>,
    passed_test_count: u32,
    time_consumed_millis: u32,
    memory_consumed_bytes: u64,
}

#[derive(Deserialize)]
struct SubmissionProblemData {
    index: Box<str>,
    name: Box<str>,
}

impl SubmissionData {
    fn into_submission(self) -> Submission {
        Submission {
            id: self.id,
            contest_id: self.contest_id,
            creation_time_seconds: self.creation_time_seconds,
            relative_time_seconds: self.relative_time_seconds,
            problem_index: self.problem.index,
            problem_name: self.problem.name,
            programming_language: self.programming_language,
            verdict: self.verdict,
            testset: self.testset,
            passed_test_count: self.passed_test_count,
            time_consumed_millis: self.time_consumed_millis,
            memory_consumed_bytes: self.memory_consumed_bytes,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentActionData {
    time_seconds: i64,
    blog_entry: Option<BlogEntryData>,
    comment: Option<CommentData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlogEntryData {
    id: u32,
    creation_time_seconds: i64,
    author_handle: Box<str>,
    title: Box<str>,
    content: Option<Box<str>>,
    modification_time_seconds: Option<i64>,
    #[serde(default)]
    tags: Vec<Box<str>>,
    rating: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommentData {
    id: u32,
    creation_time_seconds: i64,
    commentator_handle: Box<str>,
    text: Box<str>,
    parent_comment_id: Option<u32>,
    rating: i32,
}

impl CommentData {
    fn into_comment(self) -> DiscussionComment {
        DiscussionComment {
            id: self.id,
            author: self.commentator_handle,
            content_html: self.text,
            created_time_seconds: self.creation_time_seconds,
            parent_comment_id: self.parent_comment_id,
            rating: self.rating,
        }
    }
}
