use crate::{domain::PublicAccountStats, error::Result};

pub(crate) async fn codechef(
    username: &str,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<PublicAccountStats> {
    #[cfg(feature = "codechef")]
    {
        let stats = platform_codechef::Client::new()?
            .user_stats(username, progress)
            .await?;
        Ok(PublicAccountStats {
            platform: "CodeChef".into(),
            username: stats.handle,
            name: Some(stats.name),
            country: None,
            joined_at: None,
            solved: Some(stats.solved),
            rating: stats.rating,
            highest_rating: stats.highest_rating,
            level: None,
            contests: stats.contests,
            events: None,
            url: stats.url,
        })
    }
    #[cfg(not(feature = "codechef"))]
    {
        let _ = (username, progress);
        Err(crate::Error::CodeChefDisabled)
    }
}

pub(crate) async fn hackerrank(
    username: &str,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<PublicAccountStats> {
    #[cfg(feature = "hackerrank")]
    {
        let profile = platform_hackerrank::Client::new()?
            .profile(username, progress)
            .await?;
        Ok(PublicAccountStats {
            platform: "HackerRank".into(),
            url: format!("https://www.hackerrank.com/profile/{}", profile.username).into(),
            username: profile.username,
            name: profile.name,
            country: profile.country,
            joined_at: profile.created_at,
            solved: None,
            rating: None,
            highest_rating: None,
            level: profile.level,
            contests: None,
            events: profile.event_count,
        })
    }
    #[cfg(not(feature = "hackerrank"))]
    {
        let _ = (username, progress);
        Err(crate::Error::HackerRankDisabled)
    }
}
