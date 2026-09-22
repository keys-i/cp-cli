//! Native Codeforces reads used by root command dispatch

use serde::Serialize;

use crate::domain::{CommunityDiscussion, CommunityDiscussionList, DiscussionId};
#[cfg(feature = "codeforces")]
use crate::domain::{CommunityDiscussionSummary, StatementFormat};
use crate::error::Result;

#[cfg(feature = "codeforces")]
const SUBMISSION_LIMIT: u8 = 100;
#[cfg(feature = "codeforces")]
const MAX_CODEFORCES_HTML_BYTES: usize = 768 * 1024;

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesStats {
    pub(crate) profile: CodeforcesProfile,
    pub(crate) rating_history: Vec<CodeforcesRating>,
    pub(crate) recent_submissions: Vec<CodeforcesSubmission>,
    pub(crate) recent_accepted_count: u32,
    pub(crate) recent_solved_count: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesProfile {
    pub(crate) handle: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rank: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_rank: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) max_rating: Option<i32>,
    pub(crate) contribution: i32,
    pub(crate) friend_of_count: u32,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesRating {
    pub(crate) contest_id: u32,
    pub(crate) contest_name: Box<str>,
    pub(crate) rank: u32,
    pub(crate) old_rating: i32,
    pub(crate) new_rating: i32,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesSubmission {
    pub(crate) id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) contest_id: Option<u32>,
    pub(crate) timestamp: i64,
    pub(crate) problem_index: Box<str>,
    pub(crate) problem_name: Box<str>,
    pub(crate) language: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) verdict: Option<Box<str>>,
    pub(crate) passed_test_count: u32,
    pub(crate) runtime_millis: u32,
    pub(crate) memory_bytes: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesContestList {
    pub(crate) contests: Vec<CodeforcesContest>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CodeforcesContest {
    pub(crate) id: u32,
    pub(crate) name: Box<str>,
    pub(crate) kind: Box<str>,
    pub(crate) phase: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) start_time_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_seconds: Option<u32>,
    pub(crate) url: Box<str>,
}

/// Load public profile, rating, and latest-submission data for one handle
#[cfg(feature = "codeforces")]
pub(crate) async fn stats(
    handle: &str,
    mut progress: impl FnMut(usize, Option<u64>),
) -> Result<CodeforcesStats> {
    let client = platform_codeforces::Client::new()?;
    let profile = client.user_profile(handle, &mut progress).await?;
    // Codeforces limits unauthenticated public API calls to one request per two seconds
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let rating_history = client.rating_history(handle, &mut progress).await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let submissions = client
        .submissions(handle, SUBMISSION_LIMIT, |current, total| {
            progress(current, total)
        })
        .await?;

    let recent_accepted_count = submissions
        .iter()
        .filter(|submission| submission.verdict.as_deref() == Some("OK"))
        .count() as u32;
    let recent_solved_count = submissions
        .iter()
        .filter(|submission| submission.verdict.as_deref() == Some("OK"))
        .map(|submission| (submission.contest_id, submission.problem_index.as_ref()))
        .collect::<std::collections::HashSet<_>>()
        .len() as u32;

    Ok(CodeforcesStats {
        profile: CodeforcesProfile {
            handle: profile.handle,
            rank: profile.rank,
            rating: profile.rating,
            max_rank: profile.max_rank,
            max_rating: profile.max_rating,
            contribution: profile.contribution,
            friend_of_count: profile.friend_of_count,
        },
        rating_history: rating_history.into_iter().map(rating).collect(),
        recent_submissions: submissions.into_iter().map(submission).collect(),
        recent_accepted_count,
        recent_solved_count,
    })
}

#[cfg(not(feature = "codeforces"))]
pub(crate) async fn stats(
    _handle: &str,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CodeforcesStats> {
    Err(crate::Error::CodeforcesDisabled)
}

/// Load upcoming and recent public Codeforces contests
#[cfg(feature = "codeforces")]
pub(crate) async fn contests(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CodeforcesContestList> {
    Ok(CodeforcesContestList {
        contests: platform_codeforces::Client::new()?
            .contests(progress)
            .await?
            .into_iter()
            .map(map_contest)
            .collect(),
    })
}

#[cfg(not(feature = "codeforces"))]
pub(crate) async fn contests(
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CodeforcesContestList> {
    Err(crate::Error::CodeforcesDisabled)
}

/// Resolve a contest from the bounded public contest list
pub(crate) async fn contest(
    id: u32,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<Option<CodeforcesContest>> {
    if id == 0 {
        return Ok(None);
    }
    Ok(contests(progress)
        .await?
        .contests
        .into_iter()
        .find(|contest| contest.id == id))
}

/// Load Codeforces' bounded official recent-discussion feed
#[cfg(feature = "codeforces")]
pub(crate) async fn discussions(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussionList> {
    let discussions = platform_codeforces::Client::new()?
        .recent_discussions(progress)
        .await?
        .into_iter()
        .map(discussion_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CommunityDiscussionList {
        platform: "Codeforces".into(),
        discussions,
    })
}

#[cfg(not(feature = "codeforces"))]
pub(crate) async fn discussions(
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussionList> {
    Err(crate::Error::CodeforcesDisabled)
}

/// Load one Codeforces blog entry and its public reply metadata
#[cfg(feature = "codeforces")]
pub(crate) async fn discussion(
    id: DiscussionId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussion> {
    map_discussion(
        platform_codeforces::Client::new()?
            .discussion(id.get(), progress)
            .await?,
    )
}

#[cfg(not(feature = "codeforces"))]
pub(crate) async fn discussion(
    _id: DiscussionId,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussion> {
    Err(crate::Error::CodeforcesDisabled)
}

#[cfg(feature = "codeforces")]
fn rating(change: platform_codeforces::RatingChange) -> CodeforcesRating {
    CodeforcesRating {
        contest_id: change.contest_id,
        contest_name: change.contest_name,
        rank: change.rank,
        old_rating: change.old_rating,
        new_rating: change.new_rating,
        timestamp: change.rating_update_time_seconds,
    }
}

#[cfg(feature = "codeforces")]
fn submission(value: platform_codeforces::Submission) -> CodeforcesSubmission {
    CodeforcesSubmission {
        id: value.id,
        contest_id: value.contest_id,
        timestamp: value.creation_time_seconds,
        problem_index: value.problem_index,
        problem_name: value.problem_name,
        language: value.programming_language,
        verdict: value.verdict,
        passed_test_count: value.passed_test_count,
        runtime_millis: value.time_consumed_millis,
        memory_bytes: value.memory_consumed_bytes,
    }
}

#[cfg(feature = "codeforces")]
fn map_contest(value: platform_codeforces::Contest) -> CodeforcesContest {
    let id = value.id;
    CodeforcesContest {
        id,
        name: value.name,
        kind: value.kind,
        phase: value.phase,
        start_time_seconds: value.start_time_seconds,
        duration_seconds: value.duration_seconds,
        url: format!("https://codeforces.com/contest/{id}").into(),
    }
}

#[cfg(feature = "codeforces")]
fn discussion_summary(
    discussion: platform_codeforces::DiscussionPreview,
) -> Result<CommunityDiscussionSummary> {
    Ok(CommunityDiscussionSummary {
        id: DiscussionId::from_u32(discussion.id)?,
        title: plain_html_title(&discussion.title)?,
        author: Some(discussion.author),
        created_at: timestamp(discussion.created_time_seconds),
        activity_at: timestamp(discussion.activity_time_seconds),
        views: 0,
        // recentActions gives only the latest comment, never a total
        replies: u32::from(discussion.latest_comment.is_some()),
        votes: discussion.rating,
        url: discussion.url,
    })
}

#[cfg(feature = "codeforces")]
fn map_discussion(discussion: platform_codeforces::Discussion) -> Result<CommunityDiscussion> {
    if discussion.content_html.len() > MAX_CODEFORCES_HTML_BYTES
        || discussion.content_html.contains('\0')
    {
        return Err(crate::Error::Codeforces(
            platform_codeforces::Error::ApiFailure,
        ));
    }
    Ok(CommunityDiscussion {
        summary: CommunityDiscussionSummary {
            id: DiscussionId::from_u32(discussion.id)?,
            title: plain_html_title(&discussion.title)?,
            author: Some(discussion.author),
            created_at: timestamp(discussion.created_time_seconds),
            activity_at: timestamp(
                discussion
                    .modified_time_seconds
                    .unwrap_or(discussion.created_time_seconds),
            ),
            views: 0,
            replies: u32::try_from(discussion.comments.len()).unwrap_or(u32::MAX),
            votes: discussion.rating,
            url: discussion.url,
        },
        content: discussion.content_html,
        content_format: StatementFormat::Html,
    })
}

#[cfg(feature = "codeforces")]
fn plain_html_title(value: &str) -> Result<Box<str>> {
    let markdown = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "iframe", "object"])
        .build()
        .convert(value)?;
    let title = markdown.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() || title.len() > 512 {
        return Err(crate::Error::Codeforces(
            platform_codeforces::Error::ApiFailure,
        ));
    }
    Ok(title.into())
}

#[cfg(feature = "codeforces")]
fn timestamp(value: i64) -> Box<str> {
    u64::try_from(value)
        .map(crate::output::timestamp_utc)
        .map(|value| format!("{value} UTC").into())
        .unwrap_or_else(|_| "Unknown time".into())
}

#[cfg(all(test, feature = "codeforces"))]
mod tests {
    use super::*;

    #[test]
    fn serializes_native_contest_view() {
        let value = map_contest(platform_codeforces::Contest {
            id: 1,
            name: "Round".into(),
            kind: "CF".into(),
            phase: "BEFORE".into(),
            start_time_seconds: Some(2),
            duration_seconds: Some(7200),
        });
        let json = serde_json::to_string(&value).unwrap_or_default();
        assert!(json.contains("https://codeforces.com/contest/1"));
    }

    #[test]
    fn maps_discussion_html_and_reply_metadata() -> Result<()> {
        let discussion = map_discussion(platform_codeforces::Discussion {
            id: 42,
            author: "possum".into(),
            title: "<p>A post</p>".into(),
            content_html: "<script>untrusted()</script><p>Useful text</p>".into(),
            created_time_seconds: 1_700_000_000,
            modified_time_seconds: Some(1_700_000_001),
            rating: 4,
            tags: vec![],
            comments: vec![platform_codeforces::DiscussionComment {
                id: 3,
                author: "reader".into(),
                content_html: "<p>Useful reply</p>".into(),
                created_time_seconds: 1_700_000_002,
                parent_comment_id: None,
                rating: 0,
            }],
            url: "https://codeforces.com/blog/entry/42".into(),
        })?;
        assert_eq!(discussion.summary.id.get(), 42);
        assert_eq!(discussion.summary.title.as_ref(), "A post");
        assert_eq!(discussion.summary.replies, 1);
        assert_eq!(discussion.summary.votes, 4);
        assert!(matches!(discussion.content_format, StatementFormat::Html));
        assert!(discussion.content.contains("<script>"));
        Ok(())
    }
}
