#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid LeetCode problem slug")]
    InvalidSlug,
    #[error("search query must contain 1 to 100 printable ASCII characters")]
    InvalidQuery,
    #[error("tag must contain 1 to 64 ASCII letters, digits, or internal hyphens")]
    InvalidTag,
    #[error(
        "LeetCode session and CSRF credentials must contain 1 to 4096 printable ASCII characters"
    )]
    InvalidCredentials,
    #[error("source must contain 1 to 1048576 bytes")]
    InvalidSource,
    #[error("test input must contain 1 to 1048576 bytes and no NUL characters")]
    InvalidTestInput,
    #[error("language must contain 1 to 32 lowercase ASCII letters or digits")]
    InvalidLanguage,
    #[error("could not complete the LeetCode request; check your connection and try again")]
    Transport(#[from] reqwest::Error),
    #[error("LeetCode returned HTTP {0}")]
    Status(reqwest::StatusCode),
    #[error("LeetCode authentication failed; refresh your session and CSRF credentials")]
    Authentication,
    #[error("LeetCode response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },
    #[error("LeetCode returned JSON that cp-cli could not read: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("LeetCode rejected the problem query")]
    Graphql,
    #[error("problem not found; use the slug from its LeetCode URL, such as two-sum")]
    NotFound,
    #[error("this problem requires a LeetCode Premium session")]
    PremiumRequired,
    #[error("LeetCode does not permit remote runs for this problem")]
    RunUnavailable,
    #[error("LeetCode rejected the test run: {0}")]
    RunRejected(Box<str>),
    #[error("LeetCode returned incompatible example-test data")]
    InvalidTestCases,
    #[error("LeetCode returned an incompatible test-run result")]
    InvalidTestResult,
    #[error("LeetCode returned an incompatible submission result")]
    InvalidSubmissionResult,
    #[error("LeetCode returned incomplete or inconsistent problem data")]
    InvalidResponse,
}
