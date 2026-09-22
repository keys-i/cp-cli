use crate::{
    domain::{Contest, ContestList, ContestRegistration, ContestSlug},
    error::Result,
};

pub(crate) async fn list(progress: impl FnMut(usize, Option<u64>)) -> Result<ContestList> {
    #[cfg(feature = "leetcode")]
    {
        let contests = platform_leetcode::Client::new()?.contests(progress).await?;
        Ok(ContestList {
            contests: contests
                .into_iter()
                .map(from_leetcode)
                .collect::<Result<_>>()?,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = progress;
        Err(crate::Error::LeetCodeDisabled)
    }
}

pub(crate) async fn show(
    contest: ContestSlug,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<Contest> {
    #[cfg(feature = "leetcode")]
    {
        let contest = platform_leetcode::Client::new()?
            .contest(contest.as_ref(), progress)
            .await?;
        from_leetcode(contest)
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (contest, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

pub(crate) async fn status(
    contest: ContestSlug,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ContestRegistration> {
    #[cfg(feature = "leetcode")]
    {
        let registration = platform_leetcode::Client::new()?
            .contest_registration(
                contest.as_ref(),
                &crate::auth::required_credentials()?,
                progress,
            )
            .await?;
        Ok(ContestRegistration {
            id: registration.slug.parse()?,
            registered: registration.registered,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (contest, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
fn from_leetcode(contest: platform_leetcode::ContestSummary) -> Result<Contest> {
    Ok(Contest {
        id: contest.slug.parse()?,
        title: contest.title,
        start_time: contest.start_time,
        duration_seconds: contest.duration_seconds,
        virtual_contest: contest.virtual_contest,
    })
}
