use crate::{
    domain::{Problem, ProblemId, ProblemQuery, ProblemSearch},
    error::Result,
};

#[cfg(feature = "leetcode")]
use crate::domain::{Difficulty, ProblemSummary};

pub(crate) async fn show(
    id: &ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<Problem> {
    #[cfg(feature = "leetcode")]
    {
        let problem = platform_leetcode::Client::new()?
            .problem(id.as_ref(), progress)
            .await?;
        Ok(Problem {
            id: problem.id.parse()?,
            title: problem.title,
            statement: problem.statement,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (id, progress);
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
        let results = search
            .problems
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
            .collect::<Result<Vec<_>>>()?;
        Ok(ProblemSearch {
            query,
            total: search.total,
            results,
        })
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (query, progress);
        Err(crate::Error::LeetCodeDisabled)
    }
}
