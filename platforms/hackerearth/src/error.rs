#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HackerEarth practice topics must use 1–128 lowercase URL-safe path characters")]
    InvalidTopic,
    #[error("HackerEarth problem slugs must use 1–128 lowercase URL-safe characters")]
    InvalidSlug,
    #[error("HackerEarth practice pages must be between 1 and 100")]
    InvalidPage,
    #[error("could not complete the HackerEarth request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("HackerEarth returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("HackerEarth response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("HackerEarth could not find that public practice problem")]
    NotFound,
    #[error("HackerEarth returned incomplete or inconsistent public practice data")]
    InvalidResponse,
}
