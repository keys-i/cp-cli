//! LeetCode platform crate

mod client;
mod error;

pub use client::Client;
pub use error::Error;

pub struct Credentials {
    session: Box<str>,
    csrf: Box<str>,
}

impl Credentials {
    pub fn new(session: &str, csrf: &str) -> Result<Self, Error> {
        if !valid_credential(session) || !valid_credential(csrf) {
            return Err(Error::InvalidCredentials);
        }
        Ok(Self {
            session: session.into(),
            csrf: csrf.into(),
        })
    }

    pub(crate) fn session(&self) -> &str {
        &self.session
    }

    pub(crate) fn csrf(&self) -> &str {
        &self.csrf
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Credentials")
            .field("session", &"<redacted>")
            .field("csrf", &"<redacted>")
            .finish()
    }
}

fn valid_credential(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b';')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProblemSummary {
    pub number: u32,
    pub id: Box<str>,
    pub title: Box<str>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SearchResults {
    pub total: u32,
    pub problems: Vec<ProblemSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DifficultyCounts {
    pub all: u32,
    pub easy: u32,
    pub medium: u32,
    pub hard: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecentSubmission {
    pub id: u64,
    pub status: Box<str>,
    pub title: Box<str>,
    pub slug: Box<str>,
    pub timestamp: u64,
    pub language: Box<str>,
    pub runtime: Option<Box<str>>,
    pub memory: Option<Box<str>>,
    pub url: Option<Box<str>>,
    pub pending: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AccountStats {
    pub username: Box<str>,
    pub solved: DifficultyCounts,
    pub accepted_submissions: DifficultyCounts,
    pub submissions: DifficultyCounts,
    pub recent_submissions: Vec<RecentSubmission>,
    pub has_more_submissions: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ContestSummary {
    pub slug: Box<str>,
    pub title: Box<str>,
    pub start_time: u64,
    pub duration_seconds: u32,
    pub virtual_contest: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ContestRegistration {
    pub slug: Box<str>,
    pub registered: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionList {
    pub total: Option<u32>,
    pub discussions: Vec<DiscussionSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiscussionSummary {
    pub id: u32,
    pub title: Box<str>,
    pub author: Option<Box<str>>,
    pub created_at: u64,
    pub views: u32,
    pub comments: u32,
    pub votes: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Discussion {
    pub id: u32,
    pub title: Box<str>,
    pub author: Option<Box<str>>,
    /// Public Markdown supplied by LeetCode
    pub content: Box<str>,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub views: u32,
    pub comments: u32,
    pub votes: i32,
    pub tags: Vec<Box<str>>,
    pub pinned: bool,
}

#[derive(Debug)]
pub struct Problem {
    pub number: u32,
    pub id: Box<str>,
    pub title: Box<str>,
    /// Problem statement in the HTML format returned by LeetCode
    pub statement: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StarterCode {
    pub question_id: Box<str>,
    pub id: Box<str>,
    pub title: Box<str>,
    pub snippets: Vec<CodeSnippet>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CodeSnippet {
    pub language: Box<str>,
    pub language_slug: Box<str>,
    pub source: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubmissionState {
    Pending,
    Complete(SubmissionResult),
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmissionResult {
    pub id: u64,
    pub status: Box<str>,
    pub accepted: bool,
    pub runtime: Option<Box<str>>,
    pub memory: Option<Box<str>>,
    pub passed: Option<u32>,
    pub total: Option<u32>,
    pub message: Option<Box<str>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TestCases {
    pub question_id: Box<str>,
    pub input: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RunState {
    Pending,
    Complete(RunResult),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RunResult {
    pub id: Box<str>,
    pub status: Box<str>,
    pub passed: bool,
    pub runtime: Option<Box<str>>,
    pub memory: Option<Box<str>>,
    pub passed_cases: Option<u32>,
    pub total_cases: Option<u32>,
    pub input: Option<Box<str>>,
    pub output: Option<Box<str>>,
    pub expected: Option<Box<str>>,
    pub message: Option<Box<str>>,
}

#[cfg(test)]
mod tests;
