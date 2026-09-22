use crate::{domain::AccountStats, error::Result};

#[cfg(feature = "leetcode")]
use crate::domain::{DifficultyCounts, RecentSubmission};

pub(crate) async fn load() -> Result<AccountStats> {
    #[cfg(feature = "leetcode")]
    {
        let credentials = crate::auth::required_credentials()?;
        let stats = platform_leetcode::Client::new()?
            .account_stats(&credentials)
            .await?;
        Ok(AccountStats {
            username: stats.username,
            solved: counts(stats.solved),
            accepted_submissions: counts(stats.accepted_submissions),
            submissions: counts(stats.submissions),
            recent_submissions: stats
                .recent_submissions
                .into_iter()
                .map(|submission| {
                    Ok(RecentSubmission {
                        id: submission.id,
                        status: submission.status,
                        title: submission.title,
                        problem: submission.slug.parse()?,
                        timestamp: submission.timestamp,
                        language: submission.language,
                        runtime: submission.runtime,
                        memory: submission.memory,
                        pending: submission.pending,
                    })
                })
                .collect::<Result<_>>()?,
            has_more_submissions: stats.has_more_submissions,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        Err(crate::Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
fn counts(counts: platform_leetcode::DifficultyCounts) -> DifficultyCounts {
    DifficultyCounts {
        all: counts.all,
        easy: counts.easy,
        medium: counts.medium,
        hard: counts.hard,
    }
}
