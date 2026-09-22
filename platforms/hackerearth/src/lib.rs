//! Bounded public HackerEarth practice access

mod client;
mod error;

pub use client::Client;
pub use error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProblemSummary {
    pub slug: Box<str>,
    pub title: Box<str>,
    pub difficulty: Difficulty,
    pub attempted_by: Option<u32>,
    pub success_rate: Option<u8>,
    pub canonical_url: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PracticePage {
    pub topic: Box<str>,
    pub page: u8,
    pub problems: Vec<ProblemSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Problem {
    pub summary: ProblemSummary,
    /// Public HackerEarth HTML source, which callers must sanitize before rendering
    pub statement_html: Box<str>,
    pub tags: Vec<Box<str>>,
    pub points: Option<u16>,
    pub time_limit_seconds: Option<u16>,
    pub memory_limit_mb: Option<u16>,
}

#[cfg(test)]
mod tests;
