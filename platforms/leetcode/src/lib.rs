//! LeetCode platform crate

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

#[derive(Debug)]
pub struct Problem {
    pub id: Box<str>,
    pub title: Box<str>,
    /// Problem statement in the HTML format returned by LeetCode
    pub statement: Box<str>,
}

#[cfg(test)]
mod tests;
