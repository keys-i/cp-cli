//! LeetCode platform crate

mod client;
mod error;

pub use client::Client;
pub use error::Error;

#[derive(Debug)]
pub struct Problem {
    pub id: Box<str>,
    pub title: Box<str>,
    /// Problem statement in the HTML format returned by LeetCode
    pub statement: Box<str>,
}

#[cfg(test)]
mod tests;
