use std::{path::PathBuf, str::FromStr};

use crate::Error;

pub(crate) fn official_path(value: &str) -> String {
    let Ok(url) = url::Url::parse(value) else {
        return value.to_owned();
    };
    let mut path = url.path().to_owned();
    if let Some(query) = url.query() {
        path.push('?');
        path.push_str(query);
    }
    path
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct AccountName(Box<str>);

impl FromStr for AccountName {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(Error::InvalidAccountName);
        }
        Ok(Self(value.into()))
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct PracticeTopic(Box<str>);

impl FromStr for PracticeTopic {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let valid = !value.is_empty()
            && value.len() <= 128
            && !value.starts_with('/')
            && !value.ends_with('/')
            && value.split('/').all(|segment| {
                !segment.is_empty()
                    && !segment.starts_with('-')
                    && !segment.ends_with('-')
                    && segment.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
                    })
            });
        valid
            .then(|| Self(value.into()))
            .ok_or(Error::InvalidPracticeTopic)
    }
}

impl AsRef<str> for PracticeTopic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AccountName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "leetcode"), allow(dead_code))]
pub(crate) enum ProblemSelector {
    Number(u32),
    Slug(ProblemId),
}

impl FromStr for ProblemSelector {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
            return value
                .parse::<u32>()
                .ok()
                .filter(|number| *number > 0)
                .map(Self::Number)
                .ok_or(Error::InvalidProblemSelector);
        }
        value
            .parse::<ProblemId>()
            .map(Self::Slug)
            .map_err(|_| Error::InvalidProblemSelector)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LanguageSlug(Box<str>);

impl FromStr for LanguageSlug {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 32
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"+#-".contains(&byte)
            })
        {
            return Err(Error::InvalidLanguage);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for LanguageSlug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ProblemQuery(Box<str>);

impl FromStr for ProblemQuery {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty()
            || value.len() > 100
            || !value
                .bytes()
                .all(|byte| byte == b' ' || byte.is_ascii_graphic())
        {
            return Err(Error::InvalidSearchQuery);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for ProblemQuery {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ProblemTag(Box<str>);

impl FromStr for ProblemTag {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 64
            || value.starts_with('-')
            || value.ends_with('-')
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(Error::InvalidProblemTag);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for ProblemTag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ContestSlug(Box<str>);

impl FromStr for ContestSlug {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.len() > 128
            || value.starts_with('-')
            || value.ends_with('-')
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(Error::InvalidContestSlug);
        }
        Ok(Self(value.into()))
    }
}

impl AsRef<str> for ContestSlug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct DiscussionId(u32);

impl FromStr for DiscussionId {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u32>()
            .ok()
            .ok_or(Error::InvalidDiscussionId)
            .and_then(Self::from_u32)
    }
}

impl DiscussionId {
    pub(crate) fn from_u32(value: u32) -> Result<Self, Error> {
        (value > 0 && value <= i32::MAX as u32)
            .then_some(Self(value))
            .ok_or(Error::InvalidDiscussionId)
    }

    pub(crate) fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Problem {
    pub(crate) number: u32,
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) statement: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProblemSearch {
    pub(crate) query: ProblemQuery,
    pub(crate) total: u32,
    pub(crate) results: Vec<ProblemSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProblemList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tag: Option<ProblemTag>,
    pub(crate) total: u32,
    pub(crate) results: Vec<ProblemSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ProblemSummary {
    pub(crate) number: u32,
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) difficulty: Difficulty,
    pub(crate) paid_only: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CatalogPlatform {
    Codeforces,
    HackerEarth,
    HackerRank,
    #[serde(rename = "project-euler")]
    ProjectEuler,
}

impl CatalogPlatform {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Codeforces => "Codeforces",
            Self::HackerEarth => "HackerEarth",
            Self::HackerRank => "HackerRank",
            Self::ProjectEuler => "Project Euler",
        }
    }

    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Codeforces => "codeforces",
            Self::HackerEarth => "hackerearth",
            Self::HackerRank => "hackerrank",
            Self::ProjectEuler => "project-euler",
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum StatementFormat {
    #[cfg_attr(not(feature = "project-euler"), allow(dead_code))]
    Html,
    #[cfg_attr(not(feature = "hackerrank"), allow(dead_code))]
    Markdown,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CatalogProblemList {
    pub(crate) platform: CatalogPlatform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) page: Option<u8>,
    pub(crate) total: u32,
    pub(crate) results: Vec<CatalogProblemSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CatalogProblemSummary {
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rating: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) level: Option<Box<str>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) solved_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) published_at: Option<Box<str>>,
    pub(crate) url: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CatalogProblem {
    pub(crate) platform: CatalogPlatform,
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rating: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) level: Option<Box<str>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) solved_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) published_at: Option<Box<str>>,
    pub(crate) url: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) statement: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) statement_format: Option<StatementFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input_format: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_format: Option<Box<str>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) languages: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) attribution: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) license: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) license_url: Option<Box<str>>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ContestList {
    pub(crate) contests: Vec<Contest>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Contest {
    pub(crate) id: ContestSlug,
    pub(crate) title: Box<str>,
    pub(crate) start_time: u64,
    pub(crate) duration_seconds: u32,
    pub(crate) virtual_contest: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ContestRegistration {
    pub(crate) id: ContestSlug,
    pub(crate) registered: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct ToolAction {
    pub(crate) platform: Box<str>,
    pub(crate) action: Box<str>,
    pub(crate) detail: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PublicAccountStats {
    pub(crate) platform: Box<str>,
    pub(crate) username: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) country: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) joined_at: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) solved: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) highest_rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contests: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) events: Option<u32>,
    pub(crate) url: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscussionList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) problem: Option<ProblemId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<u32>,
    pub(crate) discussions: Vec<DiscussionSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct DiscussionSummary {
    pub(crate) id: DiscussionId,
    pub(crate) title: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) author: Option<Box<str>>,
    pub(crate) created_at: u64,
    pub(crate) views: u32,
    pub(crate) comments: u32,
    pub(crate) votes: i32,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Discussion {
    pub(crate) id: DiscussionId,
    pub(crate) title: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) author: Option<Box<str>>,
    pub(crate) content: Box<str>,
    pub(crate) created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) updated_at: Option<u64>,
    pub(crate) views: u32,
    pub(crate) comments: u32,
    pub(crate) votes: i32,
    pub(crate) tags: Vec<Box<str>>,
    pub(crate) pinned: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CommunityDiscussionList {
    pub(crate) platform: Box<str>,
    pub(crate) discussions: Vec<CommunityDiscussionSummary>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CommunityDiscussionSummary {
    pub(crate) id: DiscussionId,
    pub(crate) title: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) author: Option<Box<str>>,
    pub(crate) created_at: Box<str>,
    pub(crate) activity_at: Box<str>,
    pub(crate) views: u32,
    pub(crate) replies: u32,
    pub(crate) votes: i32,
    pub(crate) url: Box<str>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct CommunityDiscussion {
    #[serde(flatten)]
    pub(crate) summary: CommunityDiscussionSummary,
    pub(crate) content: Box<str>,
    pub(crate) content_format: StatementFormat,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
// Search stays in the CLI when its platform feature is disabled
#[cfg_attr(not(feature = "leetcode"), allow(dead_code))]
pub(crate) enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Medium => "MEDIUM",
            Self::Hard => "HARD",
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct AuthStatus {
    pub(crate) configured: bool,
    pub(crate) authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<AuthSource>,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(feature = "leetcode"), allow(dead_code))]
pub(crate) enum AuthSource {
    Environment,
    SecureStore,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct AuthLogout {
    pub(crate) removed: bool,
    pub(crate) environment_present: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct AccountStats {
    pub(crate) username: Box<str>,
    pub(crate) solved: DifficultyCounts,
    pub(crate) accepted_submissions: DifficultyCounts,
    pub(crate) submissions: DifficultyCounts,
    pub(crate) recent_submissions: Vec<RecentSubmission>,
    pub(crate) has_more_submissions: bool,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub(crate) struct DifficultyCounts {
    pub(crate) all: u32,
    pub(crate) easy: u32,
    pub(crate) medium: u32,
    pub(crate) hard: u32,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct RecentSubmission {
    pub(crate) id: u64,
    pub(crate) status: Box<str>,
    pub(crate) title: Box<str>,
    pub(crate) problem: ProblemId,
    pub(crate) timestamp: u64,
    pub(crate) language: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) memory: Option<Box<str>>,
    pub(crate) pending: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct SolutionFile {
    pub(crate) platform: Box<str>,
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) language: Box<str>,
    pub(crate) path: PathBuf,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct TestResult {
    pub(crate) run_id: Box<str>,
    pub(crate) id: ProblemId,
    pub(crate) path: PathBuf,
    pub(crate) passed: bool,
    pub(crate) status: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) memory: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) passed_cases: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total_cases: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) input: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) expected: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<Box<str>>,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct SubmissionResult {
    pub(crate) id: u64,
    pub(crate) status: Box<str>,
    pub(crate) accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) runtime: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) memory: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) passed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) total: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<Box<str>>,
}
