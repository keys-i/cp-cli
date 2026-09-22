//! Bounded public Project Euler problem access

mod client;
mod error;

pub use client::Client;
pub use error::Error;

/// The licence applied by Project Euler to archive and recent problem content
pub const PROBLEM_CONTENT_LICENSE: &str = "CC BY-NC-SA 4.0";
/// The official Project Euler licence page
pub const PROBLEM_CONTENT_LICENSE_URL: &str = "https://creativecommons.org/licenses/by-nc-sa/4.0/";
/// Required source credit for displayed Project Euler problem content
pub const PROBLEM_CONTENT_ATTRIBUTION: &str = "Problem content from Project Euler";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Difficulty {
    pub level: u8,
    pub percentage: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProblemSummary {
    pub number: u16,
    pub title: Box<str>,
    pub solved_count: Option<u32>,
    pub published_at: Option<Box<str>>,
    pub canonical_url: Box<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArchivePage {
    pub page: u8,
    pub problems: Vec<ProblemSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RecentProblems {
    pub problems: Vec<ProblemSummary>,
}

/// Every public Project Euler problem listed by its compact catalogue endpoint
#[derive(Debug, PartialEq, Eq)]
pub struct ProblemCatalog {
    pub problems: Vec<ProblemSummary>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Problem {
    pub summary: ProblemSummary,
    pub difficulty: Option<Difficulty>,
    /// Public HTML fragment from Project Euler's minimal problem endpoint
    ///
    /// Callers must treat this as untrusted HTML and sanitize it before rendering
    pub statement_html: Box<str>,
    pub attribution: &'static str,
    pub license: &'static str,
    pub license_url: &'static str,
}

#[cfg(test)]
mod tests;
