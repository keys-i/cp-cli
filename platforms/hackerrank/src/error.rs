#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "HackerRank usernames must contain 1 to 128 ASCII letters, digits, dots, hyphens or underscores"
    )]
    InvalidUsername,
    #[error(
        "HackerRank slugs must contain 1 to 128 lowercase ASCII letters, digits, or internal hyphens"
    )]
    InvalidSlug,
    #[error("HackerRank list offsets must be at most 1000000 and limits must be between 1 and 20")]
    InvalidPage,
    #[error("could not complete the HackerRank request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("HackerRank returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("HackerRank response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("HackerRank returned JSON that cp-cli could not read: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("HackerRank could not find that public problem")]
    NotFound,
    #[error("HackerRank returned incomplete or inconsistent problem data")]
    InvalidResponse,
}
