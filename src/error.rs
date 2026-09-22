pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid account name: use 1–128 ASCII letters, digits, dots, hyphens or underscores")]
    InvalidAccountName,
    #[error("invalid HackerEarth topic: use lowercase URL path segments separated by slashes")]
    InvalidPracticeTopic,
    #[error("invalid problem slug: use 1–128 ASCII letters, digits, hyphens or underscores")]
    InvalidProblemId,
    #[error("invalid problem identifier: use a positive problem number or title slug")]
    InvalidProblemSelector,
    #[error("problem number was not found; search for it and use the returned slug")]
    ProblemNumberNotFound,
    #[error("invalid language: use a LeetCode language slug such as rust, cpp or python3")]
    InvalidLanguage,
    #[error("this problem does not provide a {0} starter")]
    LanguageUnavailable(Box<str>),
    #[error("invalid search query: use 1–100 printable ASCII characters")]
    InvalidSearchQuery,
    #[error("invalid problem tag: use 1–64 lowercase letters, digits or hyphens")]
    InvalidProblemTag,
    #[error("invalid contest ID: use 1–128 ASCII letters, digits or internal hyphens")]
    InvalidContestSlug,
    #[error("invalid discussion identifier: use a positive numeric topic ID")]
    InvalidDiscussionId,
    #[error("invalid {platform} problem ID; use {expected}")]
    InvalidCatalogProblemId {
        platform: &'static str,
        expected: &'static str,
    },
    #[error("{platform} problem pages start at 1")]
    InvalidProblemPage { platform: &'static str },
    #[error("{platform} catalogue page {page} does not exist; the last page is {last}")]
    ProblemPageNotFound {
        platform: &'static str,
        page: u8,
        last: usize,
    },
    #[error("Exercism requires --track <slug>, such as --track rust")]
    ExercismTrackRequired,
    #[error("{platform} requires an account name for {capability}")]
    AccountNameRequired {
        platform: &'static str,
        capability: &'static str,
    },
    #[error("{platform} does not provide {capability}")]
    CapabilityUnavailable {
        platform: &'static str,
        capability: &'static str,
    },
    #[error("contest {id} was not found on {platform}")]
    ContestNotFound {
        platform: &'static str,
        id: Box<str>,
    },
    #[error("{0}")]
    ExercismCli(Box<str>),
    #[error("{platform} does not expose a callable terminal API for `cp-cli {command}`")]
    UnsupportedPlatformCommand {
        platform: &'static str,
        command: &'static str,
    },
    #[cfg(not(feature = "codechef"))]
    #[error("CodeChef support is disabled; rebuild with --features codechef")]
    CodeChefDisabled,
    #[cfg(feature = "codechef")]
    #[error(transparent)]
    CodeChef(#[from] platform_codechef::Error),
    #[cfg(not(feature = "codeforces"))]
    #[error("Codeforces support is disabled; rebuild with --features codeforces")]
    CodeforcesDisabled,
    #[cfg(feature = "codeforces")]
    #[error(transparent)]
    Codeforces(#[from] platform_codeforces::Error),
    #[cfg(not(feature = "hackerrank"))]
    #[error("HackerRank support is disabled; rebuild with --features hackerrank")]
    HackerRankDisabled,
    #[cfg(feature = "hackerrank")]
    #[error(transparent)]
    HackerRank(#[from] platform_hackerrank::Error),
    #[cfg(not(feature = "hackerearth"))]
    #[error("HackerEarth support is disabled; rebuild with --features hackerearth")]
    HackerEarthDisabled,
    #[cfg(feature = "hackerearth")]
    #[error(transparent)]
    HackerEarth(#[from] platform_hackerearth::Error),
    #[cfg(not(feature = "project-euler"))]
    #[error("Project Euler support is disabled; rebuild with --features project-euler")]
    ProjectEulerDisabled,
    #[cfg(feature = "project-euler")]
    #[error(transparent)]
    ProjectEuler(#[from] platform_project_euler::Error),
    #[cfg(not(feature = "leetcode"))]
    #[error("LeetCode support is disabled; rebuild with --features leetcode")]
    LeetCodeDisabled,
    #[cfg(feature = "leetcode")]
    #[error(transparent)]
    LeetCode(#[from] platform_leetcode::Error),
    #[error(
        "LeetCode authentication is required; set LEETCODE_SESSION and LEETCODE_CSRFTOKEN together"
    )]
    AuthenticationRequired,
    #[error("LEETCODE_SESSION and LEETCODE_CSRFTOKEN must be set together as valid Unicode")]
    InvalidAuthenticationEnvironment,
    #[error("CODEFORCES_API_KEY and CODEFORCES_API_SECRET must be set together as valid Unicode")]
    InvalidCodeforcesAuthenticationEnvironment,
    #[error("auth login requires an interactive terminal")]
    InteractiveLoginRequired,
    #[cfg(any(feature = "codeforces", feature = "leetcode"))]
    #[error("could not access the operating system's secure credential store")]
    CredentialStore(#[source] keyring::Error),
    #[error(
        "saved LeetCode credentials are unreadable; run `cp-cli auth logout`, then configure a valid environment session"
    )]
    InvalidStoredCredentials,
    #[error(
        "saved Codeforces API credentials are unreadable; run `cp-cli --platform codeforces auth logout`, then log in again"
    )]
    InvalidCodeforcesStoredCredentials,
    #[error("a solution file or its metadata already exists at {0}")]
    SolutionExists(std::path::PathBuf),
    #[error("could not access workspace path {path}: {source}")]
    WorkspaceIo {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid solution workspace at {path}: {reason}")]
    InvalidWorkspace {
        path: std::path::PathBuf,
        reason: &'static str,
    },
    #[error("choose --lang and --dir when problem pick is not attached to a terminal")]
    InteractivePickRequired,
    #[error("problem pick was cancelled")]
    PickCancelled,
    #[error("the LeetCode test run {0} did not finish within 60 seconds")]
    TestJudgeTimeout(Box<str>),
    #[error("the LeetCode judge did not finish submission {0} within 60 seconds")]
    JudgeTimeout(u64),
    #[error("could not write output: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not write JSON output: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<crate::exercism_cli::ExercismCliError> for Error {
    fn from(error: crate::exercism_cli::ExercismCliError) -> Self {
        Self::ExercismCli(error.to_string().into())
    }
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
