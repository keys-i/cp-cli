use crate::{
    domain::{Discussion, DiscussionId, DiscussionList, ProblemSelector},
    error::Result,
};

#[cfg(feature = "leetcode")]
use crate::domain::{DiscussionSummary, ProblemId};

pub(crate) async fn list(
    problem: Option<ProblemSelector>,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<DiscussionList> {
    #[cfg(feature = "leetcode")]
    {
        let mut progress = progress;
        let client = platform_leetcode::Client::new()?;
        let (problem, discussions) = match problem {
            Some(problem) => {
                let id = crate::problems::resolve_selector(&client, problem, &mut progress).await?;
                let discussions = client
                    .problem_discussions(id.as_ref(), &mut progress)
                    .await?;
                (Some(id), discussions)
            }
            None => (None, client.trending_discussions(&mut progress).await?),
        };
        from_leetcode_list(problem, discussions)
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (problem, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

pub(crate) async fn show(
    discussion: DiscussionId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<Discussion> {
    #[cfg(feature = "leetcode")]
    {
        from_leetcode(
            platform_leetcode::Client::new()?
                .discussion(discussion.get(), progress)
                .await?,
        )
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (discussion, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
fn from_leetcode_list(
    problem: Option<ProblemId>,
    list: platform_leetcode::DiscussionList,
) -> Result<DiscussionList> {
    Ok(DiscussionList {
        problem,
        total: list.total,
        discussions: list
            .discussions
            .into_iter()
            .map(from_leetcode_summary)
            .collect::<Result<_>>()?,
    })
}

#[cfg(feature = "leetcode")]
fn from_leetcode_summary(
    discussion: platform_leetcode::DiscussionSummary,
) -> Result<DiscussionSummary> {
    Ok(DiscussionSummary {
        id: DiscussionId::from_u32(discussion.id)?,
        title: discussion.title,
        author: discussion.author,
        created_at: discussion.created_at,
        views: discussion.views,
        comments: discussion.comments,
        votes: discussion.votes,
    })
}

#[cfg(feature = "leetcode")]
fn from_leetcode(discussion: platform_leetcode::Discussion) -> Result<Discussion> {
    Ok(Discussion {
        id: DiscussionId::from_u32(discussion.id)?,
        title: discussion.title,
        author: discussion.author,
        content: discussion.content,
        created_at: discussion.created_at,
        updated_at: discussion.updated_at,
        views: discussion.views,
        comments: discussion.comments,
        votes: discussion.votes,
        tags: discussion.tags,
        pinned: discussion.pinned,
    })
}
