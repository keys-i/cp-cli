#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CodeChef account names must use letters, numbers, hyphens, or underscores")]
    InvalidHandle,
    #[error("CodeChef Discuss topic IDs must be greater than zero")]
    InvalidTopicId,
    #[error("could not complete the CodeChef Discuss request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("CodeChef Discuss returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("CodeChef Discuss response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("CodeChef Discuss returned JSON that cp-cli could not read: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("CodeChef Discuss could not find that public discussion")]
    NotFound,
    #[error("CodeChef Discuss returned incomplete or inconsistent discussion data")]
    InvalidResponse,
    #[error("CodeChef returned an incomplete or inconsistent public profile")]
    InvalidProfile,
}
