#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("contest ID must be greater than zero")]
    InvalidContestId,
    #[error("problem index must contain 1 to 16 ASCII letters or digits")]
    InvalidIndex,
    #[error(
        "Codeforces handle must contain 1 to 24 ASCII letters, digits, underscores, or hyphens"
    )]
    InvalidHandle,
    #[error("submission count must be between 1 and 100")]
    InvalidSubmissionCount,
    #[error("blog entry ID must be greater than zero")]
    InvalidBlogEntryId,
    #[error(
        "Codeforces API key must contain 1 to 128 ASCII letters, digits, underscores, or hyphens"
    )]
    InvalidApiKey,
    #[error(
        "Codeforces API secret must contain 1 to 128 ASCII letters, digits, underscores, or hyphens"
    )]
    InvalidApiSecret,
    #[error("the system clock is before the Unix epoch")]
    Clock,
    #[error("could not create a Codeforces API signature nonce")]
    SignatureNonce,
    #[error("could not complete the Codeforces request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("Codeforces returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("Codeforces response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("Codeforces returned JSON that cp-cli could not read: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("Codeforces rejected the request")]
    ApiFailure,
    #[error("problem {contest_id}/{index} was not found on Codeforces")]
    NotFound { contest_id: u32, index: Box<str> },
}
