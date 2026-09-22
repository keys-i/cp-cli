use std::{collections::HashMap, time::Duration};

use reqwest::{Client as HttpClient, ClientBuilder, redirect::Policy};
use serde::Deserialize;

use crate::{Discussion, DiscussionList, DiscussionSummary, Error, UserStats};

pub(crate) const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_TOPICS: usize = 100;
const MAX_RENDERED_TOPICS: usize = 20;
const MAX_POSTS: usize = 200;
const MAX_TITLE_BYTES: usize = 256;
const MAX_SLUG_BYTES: usize = 256;
const MAX_NAME_BYTES: usize = 128;
const MAX_TIMESTAMP_BYTES: usize = 64;
const MAX_BODY_BYTES: usize = 768 * 1024;

pub struct Client {
    pub(crate) http: HttpClient,
    pub(crate) endpoint: Box<str>,
    pub(crate) profile_endpoint: Box<str>,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            http: http_builder().build()?,
            endpoint: "https://discuss.codechef.com/".into(),
            profile_endpoint: "https://www.codechef.com/".into(),
        })
    }

    /// List at most twenty public discussions from CodeChef Discuss
    pub async fn latest(
        &self,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<DiscussionList, Error> {
        let response: LatestResponse = self.get_json("latest.json", progress).await?;
        if response.topic_list.topics.len() > MAX_TOPICS || response.users.len() > MAX_TOPICS {
            return Err(Error::InvalidResponse);
        }
        let users = response
            .users
            .into_iter()
            .filter_map(valid_user)
            .collect::<HashMap<_, _>>();
        let discussions = response
            .topic_list
            .topics
            .into_iter()
            .take(MAX_RENDERED_TOPICS)
            .map(|topic| topic.into_summary(&users))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DiscussionList { discussions })
    }

    /// Fetch one public discussion and its original post from CodeChef Discuss
    pub async fn discussion(
        &self,
        id: u32,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<Discussion, Error> {
        if id == 0 {
            return Err(Error::InvalidTopicId);
        }
        let response: TopicResponse = self.get_json(&format!("t/{id}.json"), progress).await?;
        if response.id != id
            || response.post_stream.posts.is_empty()
            || response.post_stream.posts.len() > MAX_POSTS
        {
            return Err(Error::InvalidResponse);
        }
        let post = response
            .post_stream
            .posts
            .into_iter()
            .find(|post| post.post_number == 1)
            .ok_or(Error::InvalidResponse)?;
        let body_html = valid_body(post.cooked).ok_or(Error::InvalidResponse)?;
        let author = display_name(post.name, Some(post.username));
        let summary = TopicData {
            id: response.id,
            title: response.title,
            slug: response.slug,
            created_at: response.created_at,
            last_posted_at: response.last_posted_at,
            views: response.views,
            posts_count: response.posts_count,
            like_count: response.like_count,
            posters: Vec::new(),
        }
        .into_summary(&HashMap::new())?;
        Ok(Discussion {
            summary: DiscussionSummary { author, ..summary },
            body_html,
        })
    }

    /// Fetch the bounded, public statistics rendered on a CodeChef profile
    pub async fn user_stats(
        &self,
        handle: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<UserStats, Error> {
        if !valid_handle(handle) {
            return Err(Error::InvalidHandle);
        }
        let url = self.profile_url(handle)?;
        let response = self.http.get(url).send().await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        profile_from_html(handle, read_body(response, progress).await?)
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        progress: impl FnMut(usize, Option<u64>),
    ) -> Result<T, Error> {
        let url = self.endpoint_path(path)?;
        let response = self.http.get(url).send().await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::NotFound);
        }
        if !response.status().is_success() {
            return Err(Error::Status(response.status()));
        }
        Ok(serde_json::from_slice(
            &read_body(response, progress).await?,
        )?)
    }

    fn endpoint_path(&self, path: &str) -> Result<reqwest::Url, Error> {
        endpoint_path(&self.endpoint, path)
    }

    pub(crate) fn profile_url(&self, handle: &str) -> Result<reqwest::Url, Error> {
        if !valid_handle(handle) {
            return Err(Error::InvalidHandle);
        }
        endpoint_path(&self.profile_endpoint, &format!("users/{handle}"))
    }
}

fn endpoint_path(endpoint: &str, path: &str) -> Result<reqwest::Url, Error> {
    let mut url = reqwest::Url::parse(endpoint).map_err(|_| Error::InvalidResponse)?;
    if url.scheme() != "https" && url.scheme() != "http" {
        return Err(Error::InvalidResponse);
    }
    url.set_path(path);
    url.set_query(None);
    Ok(url)
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

async fn read_body(
    mut response: reqwest::Response,
    mut progress: impl FnMut(usize, Option<u64>),
) -> Result<Vec<u8>, Error> {
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
    Ok(body)
}

fn valid_handle(handle: &str) -> bool {
    !handle.is_empty()
        && handle.len() <= 64
        && handle
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn profile_from_html(handle: &str, body: Vec<u8>) -> Result<UserStats, Error> {
    let html = std::str::from_utf8(&body).map_err(|_| Error::InvalidProfile)?;
    let name =
        text_between(html, "<h1 class=\"h2-style\">", "</h1>").ok_or(Error::InvalidProfile)?;
    let solved = number_after(html, "Total Problems Solved:").ok_or(Error::InvalidProfile)?;
    let rating = text_between(html, "<div class=\"rating-number\">", "</div>")
        .and_then(|value| number_in(&value));
    let highest_rating =
        text_between(html, "(Highest Rating", ")").and_then(|value| number_in(&value));
    let contests = text_between(html, "No. of Contests Participated: <b>", "</b>")
        .and_then(|value| number_in(&value));
    Ok(UserStats {
        handle: handle.into(),
        name,
        solved,
        rating,
        highest_rating,
        contests,
        url: format!("https://www.codechef.com/users/{handle}").into(),
    })
}

fn text_between(html: &str, start: &str, end: &str) -> Option<Box<str>> {
    let value = html.split_once(start)?.1.split_once(end)?.0;
    let text = value.split('<').next()?.trim();
    (!text.is_empty() && text.len() <= MAX_NAME_BYTES).then(|| text.into())
}

fn number_after(html: &str, marker: &str) -> Option<u32> {
    number_in(html.split_once(marker)?.1)
}

fn number_in(value: &str) -> Option<u32> {
    let digits: String = value
        .trim_start()
        .bytes()
        .take_while(u8::is_ascii_digit)
        .map(char::from)
        .collect();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

#[derive(Deserialize)]
struct LatestResponse {
    topic_list: TopicList,
    #[serde(default)]
    users: Vec<UserData>,
}

#[derive(Deserialize)]
struct TopicList {
    topics: Vec<TopicData>,
}

#[derive(Deserialize)]
struct TopicResponse {
    id: u32,
    title: Box<str>,
    slug: Box<str>,
    created_at: Box<str>,
    last_posted_at: Box<str>,
    #[serde(default)]
    views: Option<u32>,
    posts_count: u32,
    #[serde(default)]
    like_count: u32,
    post_stream: PostStream,
}

#[derive(Deserialize)]
struct TopicData {
    id: u32,
    title: Box<str>,
    slug: Box<str>,
    created_at: Box<str>,
    last_posted_at: Box<str>,
    #[serde(default)]
    views: Option<u32>,
    posts_count: u32,
    #[serde(default)]
    like_count: u32,
    #[serde(default)]
    posters: Vec<Poster>,
}

#[derive(Deserialize)]
struct Poster {
    user_id: u32,
}

#[derive(Deserialize)]
struct UserData {
    id: u32,
    #[serde(default)]
    name: Option<Box<str>>,
    #[serde(default)]
    username: Option<Box<str>>,
}

#[derive(Deserialize)]
struct PostStream {
    posts: Vec<PostData>,
}

#[derive(Deserialize)]
struct PostData {
    post_number: u32,
    cooked: Box<str>,
    #[serde(default)]
    name: Option<Box<str>>,
    username: Box<str>,
}

impl TopicData {
    fn into_summary(self, users: &HashMap<u32, Box<str>>) -> Result<DiscussionSummary, Error> {
        if self.id == 0 || self.posts_count == 0 || self.posters.len() > MAX_POSTS {
            return Err(Error::InvalidResponse);
        }
        let title = valid_text(self.title, MAX_TITLE_BYTES).ok_or(Error::InvalidResponse)?;
        let slug = valid_slug(self.slug).ok_or(Error::InvalidResponse)?;
        let created_at =
            valid_text(self.created_at, MAX_TIMESTAMP_BYTES).ok_or(Error::InvalidResponse)?;
        let activity_at =
            valid_text(self.last_posted_at, MAX_TIMESTAMP_BYTES).ok_or(Error::InvalidResponse)?;
        let author = self
            .posters
            .first()
            .and_then(|poster| users.get(&poster.user_id))
            .cloned();
        let url = format!("https://discuss.codechef.com/t/{slug}/{}", self.id).into();
        Ok(DiscussionSummary {
            id: self.id,
            title,
            slug,
            author,
            created_at,
            activity_at,
            views: self.views,
            posts: self.posts_count,
            like_count: self.like_count,
            url,
        })
    }
}

fn valid_user(user: UserData) -> Option<(u32, Box<str>)> {
    if user.id == 0 {
        return None;
    }
    display_name(user.name, user.username).map(|name| (user.id, name))
}

fn display_name(name: Option<Box<str>>, username: Option<Box<str>>) -> Option<Box<str>> {
    name.and_then(|name| valid_text(name, MAX_NAME_BYTES))
        .or_else(|| username.and_then(|username| valid_text(username, MAX_NAME_BYTES)))
}

fn valid_text(value: Box<str>, limit: usize) -> Option<Box<str>> {
    (!value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control))
        .then_some(value)
}

fn valid_slug(value: Box<str>) -> Option<Box<str>> {
    (value.len() <= MAX_SLUG_BYTES
        && !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
    .then_some(value)
}

fn valid_body(value: Box<str>) -> Option<Box<str>> {
    (!value.is_empty() && value.len() <= MAX_BODY_BYTES && !value.contains('\0')).then_some(value)
}
