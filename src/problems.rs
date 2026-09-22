use crate::{
    domain::{
        Difficulty, Problem, ProblemList, ProblemQuery, ProblemSearch, ProblemSelector, ProblemTag,
    },
    error::Result,
};

#[cfg(feature = "leetcode")]
use crate::{
    domain::{ProblemId, ProblemSummary},
    error::Error,
};

pub(crate) async fn show(
    problem: ProblemSelector,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<Problem> {
    #[cfg(feature = "leetcode")]
    {
        let mut progress = progress;
        let client = platform_leetcode::Client::new()?;
        let id = resolve_selector(&client, problem, &mut progress).await?;
        let problem = client.problem(id.as_ref(), progress).await?;
        from_leetcode(problem)
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (problem, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
pub(crate) async fn resolve_selector(
    client: &platform_leetcode::Client,
    problem: ProblemSelector,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ProblemId> {
    match problem {
        ProblemSelector::Slug(id) => Ok(id),
        ProblemSelector::Number(number) => client
            .search(&number.to_string(), progress)
            .await?
            .problems
            .into_iter()
            .find(|problem| problem.number == number)
            .map(|problem| problem.id)
            .ok_or(Error::ProblemNumberNotFound)?
            .parse(),
    }
}

pub(crate) async fn daily(progress: impl FnMut(usize, Option<u64>)) -> Result<Problem> {
    #[cfg(feature = "leetcode")]
    {
        from_leetcode(platform_leetcode::Client::new()?.daily(progress).await?)
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = progress;
        Err(crate::Error::LeetCodeDisabled)
    }
}

pub(crate) async fn list(
    difficulty: Option<Difficulty>,
    tag: Option<ProblemTag>,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ProblemList> {
    #[cfg(feature = "leetcode")]
    {
        let list = platform_leetcode::Client::new()?
            .list(
                difficulty.map(to_leetcode_difficulty),
                tag.as_ref().map(AsRef::as_ref),
                progress,
            )
            .await?;
        Ok(ProblemList {
            difficulty,
            tag,
            total: list.total,
            results: summaries_from_leetcode(list.problems)?,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (difficulty, tag, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

pub(crate) async fn search(
    query: ProblemQuery,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<ProblemSearch> {
    #[cfg(feature = "leetcode")]
    {
        let search = platform_leetcode::Client::new()?
            .search(query.as_ref(), progress)
            .await?;
        Ok(ProblemSearch {
            query,
            total: search.total,
            results: summaries_from_leetcode(search.problems)?,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (query, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
fn summaries_from_leetcode(
    problems: Vec<platform_leetcode::ProblemSummary>,
) -> Result<Vec<ProblemSummary>> {
    problems
        .into_iter()
        .map(|problem| {
            Ok(ProblemSummary {
                number: problem.number,
                id: problem.id.parse()?,
                title: problem.title,
                difficulty: match problem.difficulty {
                    platform_leetcode::Difficulty::Easy => Difficulty::Easy,
                    platform_leetcode::Difficulty::Medium => Difficulty::Medium,
                    platform_leetcode::Difficulty::Hard => Difficulty::Hard,
                },
                paid_only: problem.paid_only,
            })
        })
        .collect()
}

#[cfg(feature = "leetcode")]
fn to_leetcode_difficulty(difficulty: Difficulty) -> platform_leetcode::Difficulty {
    match difficulty {
        Difficulty::Easy => platform_leetcode::Difficulty::Easy,
        Difficulty::Medium => platform_leetcode::Difficulty::Medium,
        Difficulty::Hard => platform_leetcode::Difficulty::Hard,
    }
}

#[cfg(feature = "leetcode")]
fn from_leetcode(problem: platform_leetcode::Problem) -> Result<Problem> {
    Ok(Problem {
        number: problem.number,
        id: problem.id.parse()?,
        title: problem.title,
        statement: problem.statement,
    })
}
