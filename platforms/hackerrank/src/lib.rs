//! Public HackerRank Community API client

mod client;
mod error;

pub use client::Client;
pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Other,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Track {
    pub slug: Box<str>,
    pub name: Box<str>,
    pub domain_slug: Box<str>,
    pub domain_name: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProblemSummary {
    pub id: u64,
    pub slug: Box<str>,
    pub title: Box<str>,
    pub difficulty: Difficulty,
    pub preview: Option<Box<str>>,
    pub track: Option<Track>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProblemList {
    pub total: u32,
    pub problems: Vec<ProblemSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Problem {
    pub summary: ProblemSummary,
    /// Public HackerRank source text, which callers must render as untrusted Markdown-like content
    pub statement: Box<str>,
    /// Public HackerRank source text, which callers must render as untrusted Markdown-like content
    pub input_format: Box<str>,
    /// Public HackerRank source text, which callers must render as untrusted Markdown-like content
    pub output_format: Box<str>,
    /// Supported HackerRank language keys
    pub languages: Vec<Box<str>>,
    /// Public starter source supplied by HackerRank for a language
    pub starters: Vec<Starter>,
    /// Public sample input/output pairs when HackerRank includes them
    pub samples: Vec<Sample>,
    /// Whether HackerRank marks the challenge test cases as public
    pub has_public_test_cases: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Starter {
    pub language: Box<str>,
    pub template: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Sample {
    pub input: Box<str>,
    pub output: Box<str>,
}

/// Public profile fields HackerRank exposes without an authenticated session
#[derive(Debug, PartialEq, Eq)]
pub struct Profile {
    pub id: u64,
    pub username: Box<str>,
    pub name: Option<Box<str>>,
    pub country: Option<Box<str>>,
    pub level: Option<u32>,
    pub event_count: Option<u32>,
    pub created_at: Option<Box<str>>,
}

#[cfg(test)]
mod tests;
