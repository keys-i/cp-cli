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

#[derive(Debug, serde::Serialize)]
pub(crate) struct Problem {
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) statement: Box<str>,
}
