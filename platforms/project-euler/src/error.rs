#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Project Euler problem numbers must be between 1 and 65535")]
    InvalidProblemNumber,
    #[error("Project Euler archive pages must be between 1 and 20")]
    InvalidArchivePage,
    #[error("could not complete the Project Euler request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("Project Euler returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("Project Euler response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("Project Euler could not find that public problem")]
    NotFound,
    #[error("Project Euler returned incomplete or inconsistent problem data")]
    InvalidResponse,
}
