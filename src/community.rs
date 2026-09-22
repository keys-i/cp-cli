use crate::{
    domain::{CommunityDiscussion, CommunityDiscussionList, DiscussionId},
    error::{Error, Result},
};

#[cfg(feature = "codechef")]
use crate::domain::{CommunityDiscussionSummary, StatementFormat};

#[cfg(feature = "codechef")]
const MAX_CODECHEF_HTML_BYTES: usize = 768 * 1024;

pub(crate) async fn codechef_list(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussionList> {
    #[cfg(feature = "codechef")]
    {
        let discussions = platform_codechef::Client::new()?
            .latest(progress)
            .await?
            .discussions
            .into_iter()
            .map(codechef_summary)
            .collect::<Result<Vec<_>>>()?;
        Ok(CommunityDiscussionList {
            platform: "CodeChef".into(),
            discussions,
        })
    }
    #[cfg(not(feature = "codechef"))]
    {
        let _ = progress;
        Err(Error::CodeChefDisabled)
    }
}

pub(crate) async fn codechef_show(
    id: DiscussionId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CommunityDiscussion> {
    #[cfg(feature = "codechef")]
    {
        let discussion = platform_codechef::Client::new()?
            .discussion(id.get(), progress)
            .await?;
        codechef_discussion(discussion)
    }
    #[cfg(not(feature = "codechef"))]
    {
        let _ = (id, progress);
        Err(Error::CodeChefDisabled)
    }
}

#[cfg(feature = "codechef")]
fn codechef_summary(
    discussion: platform_codechef::DiscussionSummary,
) -> Result<CommunityDiscussionSummary> {
    Ok(CommunityDiscussionSummary {
        id: DiscussionId::from_u32(discussion.id)?,
        title: discussion.title,
        author: discussion.author,
        created_at: discussion.created_at,
        activity_at: discussion.activity_at,
        views: discussion.views.unwrap_or(0),
        replies: discussion.posts.saturating_sub(1),
        votes: i32::try_from(discussion.like_count).unwrap_or(i32::MAX),
        url: discussion.url,
    })
}

#[cfg(feature = "codechef")]
fn codechef_discussion(discussion: platform_codechef::Discussion) -> Result<CommunityDiscussion> {
    if discussion.body_html.len() > MAX_CODECHEF_HTML_BYTES || discussion.body_html.contains('\0') {
        return Err(Error::CodeChef(platform_codechef::Error::InvalidResponse));
    }
    Ok(CommunityDiscussion {
        summary: codechef_summary(discussion.summary)?,
        content: discussion.body_html,
        content_format: StatementFormat::Html,
    })
}

#[cfg(all(test, feature = "codechef"))]
mod tests {
    use super::*;

    fn summary(id: u32) -> platform_codechef::DiscussionSummary {
        platform_codechef::DiscussionSummary {
            id,
            title: "A discussion".into(),
            slug: "a-discussion".into(),
            author: Some("chef".into()),
            created_at: "2026-09-22T00:00:00Z".into(),
            activity_at: "2026-09-22T01:00:00Z".into(),
            views: Some(12),
            posts: 3,
            like_count: 4,
            url: "https://discuss.codechef.com/t/a-discussion/42".into(),
        }
    }

    #[test]
    fn maps_discourse_metadata_and_preserves_html_for_the_renderer() -> Result<()> {
        let discussion = codechef_discussion(platform_codechef::Discussion {
            summary: summary(42),
            body_html: "<script>untrusted()</script><p>Useful text</p>".into(),
        })?;
        assert_eq!(discussion.summary.id.get(), 42);
        assert_eq!(discussion.summary.replies, 2);
        assert_eq!(discussion.summary.votes, 4);
        assert_eq!(discussion.summary.views, 12);
        assert!(matches!(discussion.content_format, StatementFormat::Html));
        assert!(discussion.content.contains("<script>"));

        let oversized = platform_codechef::Discussion {
            summary: summary(43),
            body_html: "x".repeat(MAX_CODECHEF_HTML_BYTES + 1).into(),
        };
        assert!(matches!(
            codechef_discussion(oversized),
            Err(Error::CodeChef(platform_codechef::Error::InvalidResponse))
        ));
        Ok(())
    }
}
