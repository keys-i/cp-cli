use std::str::FromStr;

use crate::Error;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ProblemId(Box<str>);

impl FromStr for ProblemId {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(Error::InvalidProblemId);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for ProblemId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ProblemQuery(Box<str>);

impl FromStr for ProblemQuery {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty()
            || value.len() > 100
            || !value
                .bytes()
                .all(|byte| byte == b' ' || byte.is_ascii_graphic())
        {
            return Err(Error::InvalidSearchQuery);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for ProblemQuery {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Problem {
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) statement: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProblemSearch {
    pub(crate) query: ProblemQuery,
    pub(crate) total: u32,
    pub(crate) results: Vec<ProblemSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProblemSummary {
    pub(crate) number: u32,
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) difficulty: Difficulty,
    pub(crate) paid_only: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
// Search stays in the CLI when its platform feature is disabled
#[cfg_attr(not(feature = "leetcode"), allow(dead_code))]
pub(crate) enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Medium => "MEDIUM",
            Self::Hard => "HARD",
        }
    }
}
