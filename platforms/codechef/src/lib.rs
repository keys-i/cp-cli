//! Bounded public CodeChef access through public profile HTML and Discuss JSON

mod client;
mod error;

pub use client::Client;
pub use error::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionList {
    pub discussions: Vec<DiscussionSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionSummary {
    pub id: u32,
    pub title: Box<str>,
    pub slug: Box<str>,
    pub author: Option<Box<str>>,
    pub created_at: Box<str>,
    pub activity_at: Box<str>,
    pub views: Option<u32>,
    pub posts: u32,
    pub like_count: u32,
    pub url: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Discussion {
    pub summary: DiscussionSummary,
    /// Public Discourse cooked HTML, which callers must sanitize before rendering
    pub body_html: Box<str>,
}

/// Public fields rendered on a CodeChef user profile
#[derive(Debug, PartialEq, Eq)]
pub struct UserStats {
    pub handle: Box<str>,
    pub name: Box<str>,
    pub solved: u32,
    pub rating: Option<u32>,
    pub highest_rating: Option<u32>,
    pub contests: Option<u32>,
    pub url: Box<str>,
}

#[cfg(test)]
mod tests;
