//! Public Codeforces problem metadata

mod client;
mod error;

pub use client::Client;
pub use error::Error;

/// Credentials generated at Codeforces' API settings page
#[derive(Clone)]
pub struct ApiCredentials {
    key: Box<str>,
    secret: Box<str>,
}

impl ApiCredentials {
    pub fn new(key: impl Into<Box<str>>, secret: impl Into<Box<str>>) -> Result<Self, Error> {
        let key = key.into();
        let secret = secret.into();
        if !valid_api_credential(&key) {
            return Err(Error::InvalidApiKey);
        }
        if !valid_api_credential(&secret) {
            return Err(Error::InvalidApiSecret);
        }
        Ok(Self { key, secret })
    }
}

fn valid_api_credential(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[derive(Debug, PartialEq, Eq)]
pub struct Problem {
    pub contest_id: u32,
    pub index: Box<str>,
    pub title: Box<str>,
    pub rating: Option<u16>,
    pub tags: Vec<Box<str>>,
    pub solved_count: Option<u32>,
    pub url: Box<str>,
    /// Codeforces does not expose statements through its public API
    pub statement: Option<Box<str>>,
}

/// A public Codeforces contest, suitable for calendars and status views
#[derive(Debug, PartialEq, Eq)]
pub struct Contest {
    pub id: u32,
    pub name: Box<str>,
    pub kind: Box<str>,
    pub phase: Box<str>,
    pub start_time_seconds: Option<i64>,
    pub duration_seconds: Option<u32>,
}

/// Public account information returned by `user.info`
#[derive(Debug, PartialEq, Eq)]
pub struct UserProfile {
    pub handle: Box<str>,
    pub first_name: Option<Box<str>>,
    pub last_name: Option<Box<str>>,
    pub country: Option<Box<str>>,
    pub city: Option<Box<str>>,
    pub organization: Option<Box<str>>,
    pub rank: Option<Box<str>>,
    pub rating: Option<i32>,
    pub max_rank: Option<Box<str>>,
    pub max_rating: Option<i32>,
    pub contribution: i32,
    pub friend_of_count: u32,
    pub registration_time_seconds: i64,
    pub last_online_time_seconds: i64,
}

/// One change in a user's Codeforces rating history
#[derive(Debug, PartialEq, Eq)]
pub struct RatingChange {
    pub contest_id: u32,
    pub contest_name: Box<str>,
    pub rank: u32,
    pub old_rating: i32,
    pub new_rating: i32,
    pub rating_update_time_seconds: i64,
}

/// A public submission returned by `user.status`
#[derive(Debug, PartialEq, Eq)]
pub struct Submission {
    pub id: u64,
    pub contest_id: Option<u32>,
    pub creation_time_seconds: i64,
    pub relative_time_seconds: i64,
    pub problem_index: Box<str>,
    pub problem_name: Box<str>,
    pub programming_language: Box<str>,
    pub verdict: Option<Box<str>>,
    pub testset: Box<str>,
    pub passed_test_count: u32,
    pub time_consumed_millis: u32,
    pub memory_consumed_bytes: u64,
}

/// A public Codeforces blog entry surfaced by the recent activity feed
#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionPreview {
    pub id: u32,
    pub author: Box<str>,
    pub title: Box<str>,
    pub created_time_seconds: i64,
    pub activity_time_seconds: i64,
    pub rating: i32,
    pub tags: Vec<Box<str>>,
    pub latest_comment: Option<DiscussionComment>,
    pub url: Box<str>,
}

/// A public Codeforces blog entry and its public comments
#[derive(Debug, PartialEq, Eq)]
pub struct Discussion {
    pub id: u32,
    pub author: Box<str>,
    pub title: Box<str>,
    pub content_html: Box<str>,
    pub created_time_seconds: i64,
    pub modified_time_seconds: Option<i64>,
    pub rating: i32,
    pub tags: Vec<Box<str>>,
    pub comments: Vec<DiscussionComment>,
    pub url: Box<str>,
}

/// Metadata and text for one public Codeforces blog comment
#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionComment {
    pub id: u32,
    pub author: Box<str>,
    pub content_html: Box<str>,
    pub created_time_seconds: i64,
    pub parent_comment_id: Option<u32>,
    pub rating: i32,
}

#[cfg(test)]
mod tests;
