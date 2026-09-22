use crate::{
    domain::{
        CatalogPlatform, CatalogProblem, CatalogProblemList, CatalogProblemSummary, PracticeTopic,
        ProblemId, ProblemSelector, ProblemTag,
    },
    error::{Error, Result},
};

#[cfg(any(feature = "codeforces", feature = "hackerrank"))]
const RESULTS_PER_PAGE: usize = 20;
#[cfg(feature = "project-euler")]
const PROJECT_EULER_RESULTS_PER_PAGE: usize = 50;
#[cfg(feature = "hackerearth")]
const DEFAULT_HACKEREARTH_TOPIC: &str = "basic-programming/input-output/basics-of-input-output";

pub(crate) fn problem_id(
    platform: CatalogPlatform,
    selector: ProblemSelector,
) -> Result<ProblemId> {
    match (platform, selector) {
        (CatalogPlatform::ProjectEuler, ProblemSelector::Number(number)) => {
            number.to_string().parse()
        }
        (CatalogPlatform::ProjectEuler, ProblemSelector::Slug(_)) => {
            Err(Error::InvalidCatalogProblemId {
                platform: platform.label(),
                expected: "a positive problem number",
            })
        }
        (_, ProblemSelector::Slug(id)) => Ok(id),
        (_, ProblemSelector::Number(_)) => Err(Error::InvalidCatalogProblemId {
            platform: platform.label(),
            expected: match platform {
                CatalogPlatform::Codeforces => "a contest number followed by an index, such as 4A",
                CatalogPlatform::HackerEarth => "the slug from its problem URL",
                CatalogPlatform::HackerRank => "the slug from its problem URL",
                CatalogPlatform::ProjectEuler => "a positive problem number",
            },
        }),
    }
}

pub(crate) async fn list(
    platform: CatalogPlatform,
    page: Option<u8>,
    track: Option<ProblemTag>,
    topic: Option<PracticeTopic>,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    match platform {
        CatalogPlatform::Codeforces => codeforces_list(page.unwrap_or(1), progress).await,
        CatalogPlatform::HackerEarth => {
            hackerearth_list(page.unwrap_or(1), topic.as_ref(), progress).await
        }
        CatalogPlatform::HackerRank => {
            hackerrank_list(page.unwrap_or(1), track.as_ref(), progress).await
        }
        CatalogPlatform::ProjectEuler => project_euler_list(page.unwrap_or(1), progress).await,
    }
}

/// List Project Euler's ten newest public problems
pub(crate) async fn project_euler_recent(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    project_euler_recent_inner(progress).await
}

pub(crate) async fn show(
    platform: CatalogPlatform,
    id: ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    match platform {
        CatalogPlatform::Codeforces => codeforces_show(id, progress).await,
        CatalogPlatform::HackerEarth => hackerearth_show(id, progress).await,
        CatalogPlatform::HackerRank => hackerrank_show(id, progress).await,
        CatalogPlatform::ProjectEuler => project_euler_show(id, progress).await,
    }
}

#[cfg(feature = "hackerearth")]
async fn hackerearth_list(
    page: u8,
    topic: Option<&PracticeTopic>,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    let page = platform_hackerearth::Client::new()?
        .practice(
            topic.map_or(DEFAULT_HACKEREARTH_TOPIC, AsRef::as_ref),
            page,
            progress,
        )
        .await?;
    let results = page
        .problems
        .into_iter()
        .map(hackerearth_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CatalogProblemList {
        platform: CatalogPlatform::HackerEarth,
        page: Some(page.page),
        total: results.len() as u32,
        results,
    })
}

#[cfg(not(feature = "hackerearth"))]
async fn hackerearth_list(
    _page: u8,
    _topic: Option<&PracticeTopic>,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    Err(Error::HackerEarthDisabled)
}

#[cfg(feature = "hackerearth")]
async fn hackerearth_show(
    id: ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    let problem = platform_hackerearth::Client::new()?
        .problem(id.as_ref(), progress)
        .await?;
    let summary = hackerearth_summary(problem.summary)?;
    let mut details = Vec::new();
    if let Some(points) = problem.points {
        details.push(format!("{points} points"));
    }
    if let Some(seconds) = problem.time_limit_seconds {
        details.push(format!("{seconds}s limit"));
    }
    if let Some(memory) = problem.memory_limit_mb {
        details.push(format!("{memory} MB"));
    }
    Ok(CatalogProblem {
        platform: CatalogPlatform::HackerEarth,
        id: summary.id,
        title: summary.title,
        difficulty: summary.difficulty,
        rating: None,
        level: (!details.is_empty()).then(|| details.join(" · ").into()),
        tags: problem.tags,
        solved_count: None,
        published_at: None,
        url: summary.url,
        statement: Some(problem.statement_html),
        statement_format: Some(crate::domain::StatementFormat::Html),
        input_format: None,
        output_format: None,
        languages: Vec::new(),
        attribution: Some("Problem content from HackerEarth".into()),
        license: None,
        license_url: None,
    })
}

#[cfg(not(feature = "hackerearth"))]
async fn hackerearth_show(
    _id: ProblemId,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    Err(Error::HackerEarthDisabled)
}

#[cfg(feature = "hackerearth")]
fn hackerearth_summary(
    problem: platform_hackerearth::ProblemSummary,
) -> Result<CatalogProblemSummary> {
    let difficulty = match problem.difficulty {
        platform_hackerearth::Difficulty::Easy => crate::domain::Difficulty::Easy,
        platform_hackerearth::Difficulty::Medium => crate::domain::Difficulty::Medium,
        platform_hackerearth::Difficulty::Hard => crate::domain::Difficulty::Hard,
    };
    let level = match (problem.success_rate, problem.attempted_by) {
        (Some(rate), Some(attempts)) => {
            Some(format!("{rate}% success · {attempts} attempts").into())
        }
        (Some(rate), None) => Some(format!("{rate}% success").into()),
        (None, Some(attempts)) => Some(format!("{attempts} attempts").into()),
        (None, None) => None,
    };
    Ok(CatalogProblemSummary {
        id: problem.slug.as_ref().parse()?,
        title: problem.title,
        difficulty: Some(difficulty),
        rating: None,
        level,
        tags: Vec::new(),
        solved_count: None,
        published_at: None,
        url: problem.canonical_url,
    })
}

pub(crate) fn metadata_problem(
    platform: CatalogPlatform,
    summary: &CatalogProblemSummary,
) -> CatalogProblem {
    CatalogProblem {
        platform,
        id: summary.id.clone(),
        title: summary.title.clone(),
        difficulty: summary.difficulty,
        rating: summary.rating,
        level: summary.level.clone(),
        tags: summary.tags.clone(),
        solved_count: summary.solved_count,
        published_at: summary.published_at.clone(),
        url: summary.url.clone(),
        statement: None,
        statement_format: None,
        input_format: None,
        output_format: None,
        languages: Vec::new(),
        attribution: None,
        license: None,
        license_url: None,
    }
}

#[cfg(feature = "hackerrank")]
async fn hackerrank_list(
    page: u8,
    track: Option<&ProblemTag>,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    let offset = page_offset(CatalogPlatform::HackerRank, page)? as u32;
    let list = platform_hackerrank::Client::new()?
        .list_track(
            track.map_or("algorithms", AsRef::as_ref),
            offset,
            RESULTS_PER_PAGE as u8,
            progress,
        )
        .await?;
    let results = list
        .problems
        .into_iter()
        .map(hackerrank_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CatalogProblemList {
        platform: CatalogPlatform::HackerRank,
        page: Some(page),
        total: list.total,
        results,
    })
}

#[cfg(not(feature = "hackerrank"))]
async fn hackerrank_list(
    _page: u8,
    _track: Option<&ProblemTag>,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    Err(Error::HackerRankDisabled)
}

#[cfg(feature = "hackerrank")]
async fn hackerrank_show(
    id: ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    let problem = platform_hackerrank::Client::new()?
        .problem(id.as_ref(), progress)
        .await?;
    let summary = hackerrank_summary(problem.summary)?;
    Ok(CatalogProblem {
        platform: CatalogPlatform::HackerRank,
        id: summary.id,
        title: summary.title,
        difficulty: summary.difficulty,
        rating: None,
        level: None,
        tags: summary.tags,
        solved_count: None,
        published_at: None,
        url: summary.url,
        statement: hackerrank_markdown(problem.statement),
        statement_format: Some(crate::domain::StatementFormat::Markdown),
        input_format: hackerrank_markdown(problem.input_format),
        output_format: hackerrank_markdown(problem.output_format),
        languages: problem.languages,
        attribution: None,
        license: None,
        license_url: None,
    })
}

#[cfg(not(feature = "hackerrank"))]
async fn hackerrank_show(
    _id: ProblemId,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    Err(Error::HackerRankDisabled)
}

#[cfg(feature = "hackerrank")]
fn hackerrank_summary(
    problem: platform_hackerrank::ProblemSummary,
) -> Result<CatalogProblemSummary> {
    let difficulty = match problem.difficulty {
        platform_hackerrank::Difficulty::Easy => Some(crate::domain::Difficulty::Easy),
        platform_hackerrank::Difficulty::Medium => Some(crate::domain::Difficulty::Medium),
        platform_hackerrank::Difficulty::Hard => Some(crate::domain::Difficulty::Hard),
        platform_hackerrank::Difficulty::Other => None,
    };
    let tags = problem.track.map_or_else(Vec::new, |track| {
        if track.name == track.domain_name {
            vec![track.name]
        } else {
            vec![track.domain_name, track.name]
        }
    });
    let url = format!(
        "https://www.hackerrank.com/challenges/{}/problem",
        problem.slug
    );
    Ok(CatalogProblemSummary {
        id: problem.slug.as_ref().parse()?,
        title: problem.title,
        difficulty,
        rating: None,
        level: None,
        tags,
        solved_count: None,
        published_at: None,
        url: url.into(),
    })
}

#[cfg(feature = "codeforces")]
async fn codeforces_list(
    page: u8,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    let offset = page_offset(CatalogPlatform::Codeforces, page)?;
    let problems = platform_codeforces::Client::new()?
        .problems(progress)
        .await?;
    let total = problems.len() as u32;
    let results = problems
        .into_iter()
        .skip(offset)
        .take(RESULTS_PER_PAGE)
        .map(codeforces_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CatalogProblemList {
        platform: CatalogPlatform::Codeforces,
        page: Some(page),
        total,
        results,
    })
}

#[cfg(not(feature = "codeforces"))]
async fn codeforces_list(
    _page: u8,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    Err(Error::CodeforcesDisabled)
}

#[cfg(any(feature = "codeforces", feature = "hackerrank"))]
fn page_offset(platform: CatalogPlatform, page: u8) -> Result<usize> {
    page.checked_sub(1)
        .map(|page| usize::from(page) * RESULTS_PER_PAGE)
        .ok_or(Error::InvalidProblemPage {
            platform: platform.label(),
        })
}

#[cfg(feature = "codeforces")]
async fn codeforces_show(
    id: ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    let (contest, index) = codeforces_id(&id)?;
    let summary = codeforces_summary(
        platform_codeforces::Client::new()?
            .problem(contest, index, progress)
            .await?,
    )?;
    Ok(metadata_problem(CatalogPlatform::Codeforces, &summary))
}

#[cfg(not(feature = "codeforces"))]
async fn codeforces_show(
    _id: ProblemId,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    Err(Error::CodeforcesDisabled)
}

#[cfg(feature = "codeforces")]
fn codeforces_summary(problem: platform_codeforces::Problem) -> Result<CatalogProblemSummary> {
    let id: ProblemId = format!("{}{}", problem.contest_id, problem.index).parse()?;
    Ok(CatalogProblemSummary {
        id,
        title: problem.title,
        difficulty: None,
        rating: problem.rating,
        level: None,
        tags: problem.tags,
        solved_count: problem.solved_count,
        published_at: None,
        url: problem.url,
    })
}

#[cfg(feature = "codeforces")]
fn codeforces_id(id: &ProblemId) -> Result<(u32, &str)> {
    let value = id.as_ref();
    let split = value
        .bytes()
        .position(|byte| !byte.is_ascii_digit())
        .ok_or(Error::InvalidCatalogProblemId {
            platform: "Codeforces",
            expected: "a contest number followed by an index, such as 4A",
        })?;
    let (contest, index) = value.split_at(split);
    let contest = contest
        .parse::<u32>()
        .ok()
        .filter(|contest| *contest > 0)
        .ok_or(Error::InvalidCatalogProblemId {
            platform: "Codeforces",
            expected: "a contest number followed by an index, such as 4A",
        })?;
    if index.is_empty() {
        return Err(Error::InvalidCatalogProblemId {
            platform: "Codeforces",
            expected: "a contest number followed by an index, such as 4A",
        });
    }
    Ok((contest, index))
}

#[cfg(feature = "project-euler")]
async fn project_euler_list(
    page: u8,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    let offset = page
        .checked_sub(1)
        .map(|page| usize::from(page) * PROJECT_EULER_RESULTS_PER_PAGE)
        .ok_or(Error::InvalidProblemPage {
            platform: CatalogPlatform::ProjectEuler.label(),
        })?;
    let catalog = platform_project_euler::Client::new()?
        .catalog(progress)
        .await?;
    let total = catalog.problems.len() as u32;
    if offset >= catalog.problems.len() && !catalog.problems.is_empty() {
        return Err(Error::ProblemPageNotFound {
            platform: CatalogPlatform::ProjectEuler.label(),
            page,
            last: catalog
                .problems
                .len()
                .div_ceil(PROJECT_EULER_RESULTS_PER_PAGE),
        });
    }
    let results = catalog
        .problems
        .into_iter()
        .skip(offset)
        .take(PROJECT_EULER_RESULTS_PER_PAGE)
        .map(project_euler_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CatalogProblemList {
        platform: CatalogPlatform::ProjectEuler,
        page: Some(page),
        total,
        results,
    })
}

#[cfg(feature = "project-euler")]
async fn project_euler_recent_inner(
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    let recent = platform_project_euler::Client::new()?
        .recent(progress)
        .await?;
    let results = recent
        .problems
        .into_iter()
        .map(project_euler_summary)
        .collect::<Result<Vec<_>>>()?;
    Ok(CatalogProblemList {
        platform: CatalogPlatform::ProjectEuler,
        page: None,
        total: results.len() as u32,
        results,
    })
}

#[cfg(not(feature = "project-euler"))]
async fn project_euler_recent_inner(
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    Err(Error::ProjectEulerDisabled)
}

#[cfg(not(feature = "project-euler"))]
async fn project_euler_list(
    _page: u8,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblemList> {
    Err(Error::ProjectEulerDisabled)
}

#[cfg(feature = "project-euler")]
async fn project_euler_show(
    id: ProblemId,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    let number = id
        .as_ref()
        .parse::<u16>()
        .ok()
        .filter(|number| *number > 0)
        .ok_or(Error::InvalidCatalogProblemId {
            platform: "Project Euler",
            expected: "a positive problem number no greater than 65535",
        })?;
    let problem = platform_project_euler::Client::new()?
        .problem(number, progress)
        .await?;
    let summary = project_euler_summary(problem.summary)?;
    Ok(CatalogProblem {
        platform: CatalogPlatform::ProjectEuler,
        id: summary.id,
        title: summary.title,
        difficulty: None,
        rating: None,
        level: problem.difficulty.map(|difficulty| {
            format!("Level {} · {}%", difficulty.level, difficulty.percentage).into()
        }),
        tags: Vec::new(),
        solved_count: summary.solved_count,
        published_at: summary.published_at,
        url: summary.url,
        statement: Some(problem.statement_html),
        statement_format: Some(crate::domain::StatementFormat::Html),
        input_format: None,
        output_format: None,
        languages: Vec::new(),
        attribution: Some(problem.attribution.into()),
        license: Some(problem.license.into()),
        license_url: Some(problem.license_url.into()),
    })
}

#[cfg(not(feature = "project-euler"))]
async fn project_euler_show(
    _id: ProblemId,
    _progress: impl FnMut(usize, Option<u64>),
) -> Result<CatalogProblem> {
    Err(Error::ProjectEulerDisabled)
}

#[cfg(feature = "project-euler")]
fn project_euler_summary(
    problem: platform_project_euler::ProblemSummary,
) -> Result<CatalogProblemSummary> {
    Ok(CatalogProblemSummary {
        id: problem.number.to_string().parse()?,
        title: problem.title,
        difficulty: None,
        rating: None,
        level: None,
        tags: Vec::new(),
        solved_count: problem.solved_count,
        published_at: problem.published_at,
        url: problem.canonical_url,
    })
}

#[cfg(feature = "hackerrank")]
fn hackerrank_markdown(value: Box<str>) -> Option<Box<str>> {
    if value.trim().is_empty() {
        return None;
    }
    let mut output = String::with_capacity(value.len());
    let mut rest = value.as_ref();
    while let Some(start) = rest.find('$') {
        output.push_str(&rest[..start]);
        let math = &rest[start..];
        if let Some(display) = math.strip_prefix("$$") {
            output.push_str("$$");
            rest = display;
            continue;
        }
        let Some(after) = math.strip_prefix('$') else {
            break;
        };
        let Some(end) = after.find('$') else {
            output.push_str(math);
            rest = "";
            break;
        };
        let source = &after[..end];
        if let Some(code) = hackerrank_code(source) {
            output.push('`');
            output.push_str(&code);
            output.push('`');
        } else {
            output.push('$');
            output.push_str(source);
            output.push('$');
        }
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Some(output.into())
}

#[cfg(feature = "hackerrank")]
fn hackerrank_code(source: &str) -> Option<String> {
    let source = source.replace("\\\\", " ").replace("\\ ", " ");
    if !source
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b" _[]*&".contains(&byte))
    {
        return None;
    }
    let mut words = source.split_whitespace();
    let first = words.next()?.trim_end_matches("[]");
    if !matches!(
        first,
        "bool"
            | "boolean"
            | "char"
            | "double"
            | "float"
            | "int"
            | "long"
            | "short"
            | "string"
            | "unsigned"
    ) {
        return None;
    }
    Some(source.split_whitespace().collect::<Vec<_>>().join(" "))
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "hackerrank")]
    use super::hackerrank_markdown;
    #[cfg(feature = "project-euler")]
    use super::problem_id;
    #[cfg(feature = "codeforces")]
    use super::{codeforces_id, page_offset};
    #[cfg(any(feature = "codeforces", feature = "project-euler"))]
    use crate::domain::CatalogPlatform;
    #[cfg(feature = "project-euler")]
    use crate::domain::ProblemSelector;

    #[cfg(feature = "codeforces")]
    #[test]
    fn codeforces_problem_ids_split_at_the_index() -> Result<(), Box<dyn std::error::Error>> {
        for (input, expected) in [("4A", (4, "A")), ("1932B2", (1932, "B2"))] {
            assert_eq!(codeforces_id(&input.parse()?)?, expected);
        }
        for input in ["4", "A", "0A"] {
            assert!(codeforces_id(&input.parse()?).is_err());
        }
        assert_eq!(page_offset(CatalogPlatform::Codeforces, 2)?, 20);
        assert!(page_offset(CatalogPlatform::Codeforces, 0).is_err());
        Ok(())
    }

    #[cfg(feature = "hackerrank")]
    #[test]
    fn hackerrank_type_notation_stays_code() {
        for (source, expected) in [
            ("$int$", "`int`"),
            ("$int\\\\ value$", "`int value`"),
            ("$a = 7$", "$a = 7$"),
        ] {
            assert_eq!(
                hackerrank_markdown(source.into()).as_deref(),
                Some(expected)
            );
        }
    }

    #[cfg(feature = "project-euler")]
    #[test]
    fn project_euler_requires_a_numeric_problem() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            problem_id(CatalogPlatform::ProjectEuler, ProblemSelector::Number(42))?.as_ref(),
            "42"
        );
        assert!(
            problem_id(
                CatalogPlatform::ProjectEuler,
                ProblemSelector::Slug("forty-two".parse()?)
            )
            .is_err()
        );
        Ok(())
    }
}
