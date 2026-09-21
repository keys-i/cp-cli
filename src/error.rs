pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid problem slug: use 1–128 ASCII letters, digits, hyphens or underscores")]
    InvalidProblemId,
    #[error("invalid search query: use 1–100 printable ASCII characters")]
    InvalidSearchQuery,
    #[cfg(not(feature = "leetcode"))]
    #[error("LeetCode support is disabled; rebuild with --features leetcode")]
    LeetCodeDisabled,
    #[cfg(feature = "leetcode")]
    #[error(transparent)]
    LeetCode(#[from] platform_leetcode::Error),
    #[error("could not write output: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not write JSON output: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn is_broken_pipe(&self) -> bool {
        match self {
            Self::Io(error) => error.kind() == std::io::ErrorKind::BrokenPipe,
            Self::Json(error) => error.io_error_kind() == Some(std::io::ErrorKind::BrokenPipe),
            _ => false,
        }
    }
}
