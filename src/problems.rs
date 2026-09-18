use crate::{
    domain::{Problem, ProblemId},
    error::Result,
};

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
