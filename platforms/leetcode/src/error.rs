#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid LeetCode problem slug")]
    InvalidSlug,
    #[error("could not complete the LeetCode request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("LeetCode returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("LeetCode response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("LeetCode returned invalid JSON")]
    Decode(#[from] serde_json::Error),
    #[error("LeetCode rejected the problem query")]
    Graphql,
    #[error("problem not found; use the slug from its LeetCode URL, such as two-sum")]
    NotFound,
    #[error(
        "this problem requires a LeetCode Premium session; authentication is not supported yet"
    )]
    PremiumRequired,
    #[error("LeetCode returned incomplete or inconsistent problem data")]
    InvalidResponse,
}
