use std::{io::Write, path::PathBuf};

#[cfg(any(feature = "leetcode", feature = "hackerrank"))]
use std::io::IsTerminal;

#[cfg(feature = "leetcode")]
use std::{
    fs::{self, OpenOptions},
    io::Read,
    path::Path,
    time::Duration,
};

#[cfg(feature = "leetcode")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "leetcode")]
use crate::domain::ProblemId;
use crate::{
    domain::{LanguageSlug, ProblemSelector, SolutionFile, SubmissionResult, TestResult},
    error::{Error, Result},
};

#[cfg(feature = "leetcode")]
const MAX_SOURCE_BYTES: u64 = 1024 * 1024;
#[cfg(feature = "leetcode")]
const JUDGE_TIMEOUT: Duration = Duration::from_secs(60);

#[cfg(feature = "leetcode")]
#[derive(Deserialize, Serialize)]
struct SolutionMetadata {
    version: u8,
    id: Box<str>,
    question_id: Box<str>,
    language: Box<str>,
    language_name: Box<str>,
}

#[cfg(feature = "leetcode")]
struct SolutionData {
    path: PathBuf,
    metadata: SolutionMetadata,
    #[cfg(feature = "leetcode")]
    submission: Box<str>,
}

pub(crate) async fn pick(
    problem: ProblemSelector,
    requested_language: Option<LanguageSlug>,
    directory: Option<PathBuf>,
    progress: impl FnMut(usize, Option<u64>),
    ready: impl FnOnce(),
) -> Result<SolutionFile> {
    #[cfg(feature = "leetcode")]
    {
        let mut progress = progress;
        let client = platform_leetcode::Client::new()?;
        let slug = crate::problems::resolve_selector(&client, problem, &mut progress).await?;
        let mut starter = client.starter(slug.as_ref(), &mut progress).await?;
        ready();
        let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        if !interactive && (requested_language.is_none() || directory.is_none()) {
            return Err(Error::InteractivePickRequired);
        }
        let choice = choose_snippet(&starter.snippets, requested_language.as_ref(), interactive)?;
        let snippet = starter.snippets.swap_remove(choice);
        let directory = choose_directory(directory, interactive)?;
        write_solution(
            starter.question_id,
            starter.id.parse()?,
            starter.title,
            snippet,
            &directory,
        )
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (problem, requested_language, directory, progress, ready);
        Err(Error::LeetCodeDisabled)
    }
}

pub(crate) async fn pick_hackerrank(
    problem: ProblemSelector,
    requested_language: Option<LanguageSlug>,
    directory: Option<PathBuf>,
    progress: impl FnMut(usize, Option<u64>),
    ready: impl FnOnce(),
) -> Result<SolutionFile> {
    #[cfg(feature = "hackerrank")]
    {
        use std::fs;

        let id = crate::catalog::problem_id(crate::domain::CatalogPlatform::HackerRank, problem)?;
        let mut problem = platform_hackerrank::Client::new()?
            .problem(id.as_ref(), progress)
            .await?;
        ready();
        let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
        if !interactive && (requested_language.is_none() || directory.is_none()) {
            return Err(Error::InteractivePickRequired);
        }
        let choice =
            choose_hackerrank_starter(&problem.starters, requested_language.as_ref(), interactive)?;
        let starter = problem.starters.swap_remove(choice);
        let directory = choose_directory(directory, interactive)?;
        fs::create_dir_all(&directory).map_err(|source| Error::WorkspaceIo {
            path: directory.clone(),
            source,
        })?;
        let directory = fs::canonicalize(&directory).map_err(|source| Error::WorkspaceIo {
            path: directory.clone(),
            source,
        })?;
        let path = directory.join(format!(
            "{}.{}",
            id.as_ref(),
            language_extension(&starter.language)
        ));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|source| {
                if source.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::SolutionExists(path.clone())
                } else {
                    Error::WorkspaceIo {
                        path: path.clone(),
                        source,
                    }
                }
            })?;
        file.write_all(starter.template.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|source| Error::WorkspaceIo {
                path: path.clone(),
                source,
            })?;
        Ok(SolutionFile {
            platform: "hackerrank".into(),
            id,
            title: problem.summary.title,
            language: starter.language,
            path,
        })
    }
    #[cfg(not(feature = "hackerrank"))]
    {
        let _ = (problem, requested_language, directory, progress, ready);
        Err(Error::HackerRankDisabled)
    }
}

#[cfg(feature = "hackerrank")]
fn choose_hackerrank_starter(
    starters: &[platform_hackerrank::Starter],
    requested: Option<&LanguageSlug>,
    interactive: bool,
) -> Result<usize> {
    use std::io::Write as _;

    if starters.is_empty() {
        return Err(Error::CapabilityUnavailable {
            platform: "HackerRank",
            capability: "a starter template for this problem",
        });
    }
    if let Some(requested) = requested {
        return starters
            .iter()
            .position(|starter| starter.language.as_ref() == requested.as_ref())
            .ok_or_else(|| Error::LanguageUnavailable(requested.as_ref().into()));
    }
    if !interactive {
        return Err(Error::InteractivePickRequired);
    }
    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "POSSUM//PICK  Choose a language")?;
    for (index, starter) in starters.iter().enumerate() {
        writeln!(stderr, "  {:>2}. {}", index + 1, starter.language)?;
    }
    for _ in 0..3 {
        write!(stderr, "Language number or slug: ")?;
        stderr.flush()?;
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer)? == 0 {
            return Err(Error::PickCancelled);
        }
        let answer = answer.trim();
        if let Some(index) = answer
            .parse::<usize>()
            .ok()
            .filter(|index| (1..=starters.len()).contains(index))
        {
            return Ok(index - 1);
        }
        if let Some(index) = starters
            .iter()
            .position(|starter| starter.language.as_ref() == answer)
        {
            return Ok(index);
        }
        writeln!(stderr, "Choose one of the listed numbers or slugs.")?;
    }
    Err(Error::PickCancelled)
}

pub(crate) async fn test(
    path: PathBuf,
    progress: impl FnMut(usize, Option<u64>),
) -> Result<TestResult> {
    #[cfg(feature = "leetcode")]
    {
        let solution = read_solution(&path)?;
        let credentials = crate::auth::required_credentials()?;
        let client = platform_leetcode::Client::new()?;
        let cases = client.test_cases(&solution.metadata.id, progress).await?;
        if cases.question_id != solution.metadata.question_id {
            return Err(Error::InvalidWorkspace {
                path: solution.path,
                reason: "solution metadata does not match the LeetCode problem",
            });
        }
        let run_id = client
            .run(
                &credentials,
                &solution.metadata.id,
                &solution.metadata.question_id,
                &solution.metadata.language,
                &solution.submission,
                &cases.input,
            )
            .await?;
        let started = tokio::time::Instant::now();
        loop {
            match client.run_result(&credentials, &run_id).await? {
                platform_leetcode::RunState::Pending => {
                    if started.elapsed() >= JUDGE_TIMEOUT {
                        return Err(Error::TestJudgeTimeout(run_id));
                    }
                    tokio::time::sleep(Duration::from_millis(750)).await;
                }
                platform_leetcode::RunState::Complete(result) => {
                    return Ok(TestResult {
                        run_id: result.id,
                        id: solution.metadata.id.parse()?,
                        path: solution.path,
                        passed: result.passed,
                        status: result.status,
                        runtime: result.runtime,
                        memory: result.memory,
                        passed_cases: result.passed_cases,
                        total_cases: result.total_cases,
                        input: result.input,
                        output: result.output,
                        expected: result.expected,
                        message: result.message,
                    });
                }
            }
        }
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = (path, progress);
        Err(Error::LeetCodeDisabled)
    }
}

pub(crate) async fn submit(path: PathBuf) -> Result<SubmissionResult> {
    #[cfg(feature = "leetcode")]
    {
        let solution = read_solution(&path)?;
        let credentials = crate::auth::required_credentials()?;
        let client = platform_leetcode::Client::new()?;
        let submission_id = client
            .submit(
                &credentials,
                &solution.metadata.id,
                &solution.metadata.question_id,
                &solution.metadata.language,
                &solution.submission,
            )
            .await?;
        let started = tokio::time::Instant::now();
        loop {
            match client.submission(&credentials, submission_id).await? {
                platform_leetcode::SubmissionState::Pending => {
                    if started.elapsed() >= JUDGE_TIMEOUT {
                        return Err(Error::JudgeTimeout(submission_id));
                    }
                    tokio::time::sleep(Duration::from_millis(750)).await;
                }
                platform_leetcode::SubmissionState::Complete(result) => {
                    return Ok(SubmissionResult {
                        id: result.id,
                        status: result.status,
                        accepted: result.accepted,
                        runtime: result.runtime,
                        memory: result.memory,
                        passed: result.passed,
                        total: result.total,
                        message: result.message,
                    });
                }
            }
        }
    }
    #[cfg(not(feature = "leetcode"))]
    {
        let _ = path;
        Err(Error::LeetCodeDisabled)
    }
}

#[cfg(feature = "leetcode")]
fn choose_snippet(
    snippets: &[platform_leetcode::CodeSnippet],
    requested: Option<&LanguageSlug>,
    interactive: bool,
) -> Result<usize> {
    if let Some(requested) = requested {
        return snippets
            .iter()
            .position(|snippet| snippet.language_slug.as_ref() == requested.as_ref())
            .ok_or_else(|| Error::LanguageUnavailable(requested.as_ref().into()));
    }
    if !interactive {
        return Err(Error::InteractivePickRequired);
    }

    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "POSSUM//PICK  Choose a language")?;
    for (index, snippet) in snippets.iter().enumerate() {
        writeln!(
            stderr,
            "  {:>2}. {} ({})",
            index + 1,
            snippet.language,
            snippet.language_slug
        )?;
    }
    for _ in 0..3 {
        write!(stderr, "Language number or slug: ")?;
        stderr.flush()?;
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer)? == 0 {
            return Err(Error::PickCancelled);
        }
        let answer = answer.trim();
        if let Some(index) = answer
            .parse::<usize>()
            .ok()
            .filter(|index| (1..=snippets.len()).contains(index))
        {
            return Ok(index - 1);
        }
        if let Some(index) = snippets
            .iter()
            .position(|snippet| snippet.language_slug.as_ref() == answer)
        {
            return Ok(index);
        }
        writeln!(stderr, "Choose one of the listed numbers or slugs.")?;
    }
    Err(Error::PickCancelled)
}

pub(crate) fn choose_directory(directory: Option<PathBuf>, interactive: bool) -> Result<PathBuf> {
    if let Some(directory) = directory {
        return Ok(directory);
    }
    if !interactive {
        return Err(Error::InteractivePickRequired);
    }
    let mut stderr = std::io::stderr().lock();
    write!(stderr, "Destination directory [.]: ")?;
    stderr.flush()?;
    let mut answer = String::new();
    if std::io::stdin().read_line(&mut answer)? == 0 {
        return Err(Error::PickCancelled);
    }
    let answer = answer.trim();
    Ok(if answer.is_empty() {
        PathBuf::from(".")
    } else {
        PathBuf::from(answer)
    })
}

#[cfg(feature = "leetcode")]
fn write_solution(
    question_id: Box<str>,
    id: ProblemId,
    title: Box<str>,
    snippet: platform_leetcode::CodeSnippet,
    directory: &Path,
) -> Result<SolutionFile> {
    fs::create_dir_all(directory).map_err(|source| workspace_io(directory, source))?;
    let directory =
        fs::canonicalize(directory).map_err(|source| workspace_io(directory, source))?;
    let extension = language_extension(&snippet.language_slug);
    let path = directory.join(format!("{}.{}", id.as_ref(), extension));
    let metadata_path = metadata_path(&path)?;
    ensure_absent(&path)?;
    ensure_absent(&metadata_path)?;

    let source = source_file(&snippet.language_slug, &snippet.source);
    let metadata = serde_json::to_vec_pretty(&SolutionMetadata {
        version: 1,
        id: id.as_ref().into(),
        question_id,
        language: snippet.language_slug.clone(),
        language_name: snippet.language.clone(),
    })?;
    let source_staging = staging_path(&path, "source")?;
    let metadata_staging = staging_path(&metadata_path, "metadata")?;
    let result = (|| {
        write_new(&source_staging, source.as_bytes())?;
        write_new(&metadata_staging, &metadata)?;
        ensure_absent(&path)?;
        ensure_absent(&metadata_path)?;
        fs::rename(&source_staging, &path).map_err(|source| workspace_io(&path, source))?;
        if let Err(source) = fs::rename(&metadata_staging, &metadata_path) {
            let _ = fs::remove_file(&path);
            return Err(workspace_io(&metadata_path, source));
        }
        Ok::<(), Error>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&source_staging);
        let _ = fs::remove_file(&metadata_staging);
        return Err(error);
    }
    Ok(SolutionFile {
        platform: "leetcode".into(),
        id,
        title,
        language: snippet.language,
        path,
    })
}

#[cfg(feature = "leetcode")]
fn read_solution(path: &Path) -> Result<SolutionData> {
    ensure_regular_file(path, "solution must be a real file")?;
    let path = fs::canonicalize(path).map_err(|source| workspace_io(path, source))?;
    let metadata_path = metadata_path(&path)?;
    ensure_regular_file(&metadata_path, "missing solution metadata")?;
    let metadata_bytes = read_limited(&metadata_path, 16 * 1024)?;
    let metadata: SolutionMetadata =
        serde_json::from_slice(&metadata_bytes).map_err(|_| Error::InvalidWorkspace {
            path: metadata_path.clone(),
            reason: "invalid solution metadata",
        })?;
    if metadata.version != 1
        || metadata.id.parse::<ProblemId>().is_err()
        || metadata.language.parse::<LanguageSlug>().is_err()
        || metadata.language_name.is_empty()
        || metadata.language_name.len() > 64
        || metadata.language_name.chars().any(char::is_control)
        || metadata
            .question_id
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
    {
        return Err(Error::InvalidWorkspace {
            path: metadata_path,
            reason: "solution metadata is incomplete or inconsistent",
        });
    }
    let source = String::from_utf8(read_limited(&path, MAX_SOURCE_BYTES)?).map_err(|_| {
        Error::InvalidWorkspace {
            path: path.clone(),
            reason: "solution source must be UTF-8",
        }
    })?;
    #[cfg(feature = "leetcode")]
    let submission = submission_source(&source, &metadata.language, &path)?;
    #[cfg(not(feature = "leetcode"))]
    submission_source(&source, &metadata.language, &path)?;
    Ok(SolutionData {
        path,
        metadata,
        #[cfg(feature = "leetcode")]
        submission,
    })
}

#[cfg(feature = "leetcode")]
fn source_file(language: &str, starter: &str) -> String {
    let prefix = comment_prefix(language);
    let submission_marker = format!("{prefix} == LeetCode Submission ==");
    let test_marker = format!("{prefix} == Local Tests ==");
    let mut source = String::with_capacity(starter.len() + 180);
    if language == "rust" {
        source.push_str("pub struct Solution;\n\n");
    }
    source.push_str(&submission_marker);
    source.push('\n');
    source.push_str(starter);
    if !starter.ends_with('\n') {
        source.push('\n');
    }
    source.push_str(&test_marker);
    source.push('\n');
    if language == "rust" {
        source.push_str(
            "#[cfg(test)]\nmod tests {\n    // Optional local tests; never sent to LeetCode\n}\n",
        );
    } else {
        source.push_str(prefix);
        source.push_str(" Local notes; this section is never sent to LeetCode\n");
    }
    source
}

#[cfg(feature = "leetcode")]
fn submission_source(source: &str, language: &str, path: &Path) -> Result<Box<str>> {
    let prefix = comment_prefix(language);
    let start = format!("{prefix} == LeetCode Submission ==");
    let end = format!("{prefix} == Local Tests ==");
    let normalized = source.replace("\r\n", "\n");
    let lines: Vec<_> = normalized.lines().collect();
    let starts: Vec<_> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == start).then_some(index))
        .collect();
    let ends: Vec<_> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| (*line == end).then_some(index))
        .collect();
    if starts.len() != 1 || ends.len() != 1 || starts[0] >= ends[0] {
        return Err(Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "keep exactly one ordered submission and local-test marker",
        });
    }
    let body = lines[starts[0] + 1..ends[0]].join("\n");
    if body.trim().is_empty() || body.len() as u64 > MAX_SOURCE_BYTES {
        return Err(Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "submission source must contain 1 to 1048576 bytes",
        });
    }
    Ok(body.into())
}

#[cfg(feature = "leetcode")]
fn metadata_path(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "solution filename must be valid Unicode",
        })?;
    Ok(path.with_file_name(format!("{name}.cp-cli.json")))
}

#[cfg(feature = "leetcode")]
fn staging_path(path: &Path, kind: &str) -> Result<PathBuf> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "solution filename must be valid Unicode",
        })?;
    for attempt in 0..16 {
        let candidate = path.with_file_name(format!(
            ".{name}.cp-cli-{kind}-{}-{attempt}",
            std::process::id()
        ));
        if fs::symlink_metadata(&candidate)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        {
            return Ok(candidate);
        }
    }
    Err(Error::InvalidWorkspace {
        path: path.to_owned(),
        reason: "could not reserve a temporary solution filename",
    })
}

#[cfg(feature = "leetcode")]
fn comment_prefix(language: &str) -> &'static str {
    match language {
        "bash" | "elixir" | "python" | "python3" | "r" | "ruby" => "#",
        "lua" | "mssql" | "mysql" | "oraclesql" => "--",
        "racket" | "scheme" => ";;",
        _ => "//",
    }
}

#[cfg(any(feature = "hackerrank", feature = "leetcode"))]
fn language_extension(language: &str) -> &str {
    match language {
        "bash" => "sh",
        "csharp" => "cs",
        "cpp" => "cpp",
        "golang" => "go",
        "javascript" => "js",
        "kotlin" => "kt",
        "mssql" | "mysql" | "oraclesql" => "sql",
        "python" | "python3" => "py",
        "racket" => "rkt",
        "ruby" => "rb",
        "rust" => "rs",
        "typescript" => "ts",
        other => other,
    }
}

#[cfg(feature = "leetcode")]
fn ensure_absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(Error::SolutionExists(path.to_owned())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(workspace_io(path, source)),
    }
}

#[cfg(feature = "leetcode")]
fn ensure_regular_file(path: &Path, reason: &'static str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(Error::InvalidWorkspace {
            path: path.to_owned(),
            reason,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err(Error::InvalidWorkspace {
                path: path.to_owned(),
                reason,
            })
        }
        Err(source) => Err(workspace_io(path, source)),
    }
}

#[cfg(feature = "leetcode")]
fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let length = fs::metadata(path)
        .map_err(|source| workspace_io(path, source))?
        .len();
    if length > limit {
        return Err(Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "solution file is too large",
        });
    }
    let mut bytes = Vec::with_capacity(length as usize);
    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|source| workspace_io(path, source))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| workspace_io(path, source))?;
    if bytes.len() as u64 > limit {
        return Err(Error::InvalidWorkspace {
            path: path.to_owned(),
            reason: "solution file is too large",
        });
    }
    Ok(bytes)
}

#[cfg(feature = "leetcode")]
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| workspace_io(path, source))?;
    file.write_all(bytes)
        .map_err(|source| workspace_io(path, source))?;
    file.sync_all().map_err(|source| workspace_io(path, source))
}

#[cfg(feature = "leetcode")]
fn workspace_io(path: &Path, source: std::io::Error) -> Error {
    Error::WorkspaceIo {
        path: path.to_owned(),
        source,
    }
}

#[cfg(all(test, feature = "leetcode"))]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn solution_contract() -> TestResult {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root =
            std::env::temp_dir().join(format!("cp-cli-solution-{}-{nonce}", std::process::id()));
        let result = async {
            let snippet = platform_leetcode::CodeSnippet {
                language: "Rust".into(),
                language_slug: "rust".into(),
                source: "impl Solution { pub fn answer() -> i32 { 42 } }".into(),
            };
            let solution = write_solution(
                "1".into(),
                "two-sum".parse()?,
                "Two Sum".into(),
                snippet,
                &root,
            )?;
            assert_eq!(
                solution.path.file_name().and_then(|name| name.to_str()),
                Some("two-sum.rs")
            );
            let source = fs::read_to_string(&solution.path)?;
            assert!(source.starts_with("pub struct Solution;\n\n// == LeetCode Submission ==\n"));
            assert!(source.contains("// == Local Tests ==\n#[cfg(test)]"));
            let loaded = read_solution(&solution.path)?;
            assert_eq!(
                loaded.submission.as_ref(),
                "impl Solution { pub fn answer() -> i32 { 42 } }"
            );
            let duplicate = write_solution(
                "1".into(),
                "two-sum".parse()?,
                "Two Sum".into(),
                platform_leetcode::CodeSnippet {
                    language: "Rust".into(),
                    language_slug: "rust".into(),
                    source: "impl Solution {}".into(),
                },
                &root,
            );
            assert!(matches!(duplicate, Err(Error::SolutionExists(_))));

            for (language, source, valid) in [
                (
                    "rust",
                    "// == LeetCode Submission ==\nimpl Solution {}\n// == Local Tests ==",
                    true,
                ),
                (
                    "python3",
                    "# == LeetCode Submission ==\nclass Solution: pass\n# == Local Tests ==",
                    true,
                ),
                (
                    "rust",
                    "// == LeetCode Submission ==\n// == Local Tests ==",
                    false,
                ),
                (
                    "rust",
                    "// == Local Tests ==\nimpl Solution {}\n// == LeetCode Submission ==",
                    false,
                ),
            ] {
                assert_eq!(
                    submission_source(source, language, Path::new("solution")).is_ok(),
                    valid,
                    "language={language}"
                );
            }
            #[cfg(feature = "hackerrank")]
            {
                let starters = vec![
                    platform_hackerrank::Starter {
                        language: "cpp".into(),
                        template: "int main() {}".into(),
                    },
                    platform_hackerrank::Starter {
                        language: "rust".into(),
                        template: "fn main() {}".into(),
                    },
                ];
                let language: LanguageSlug = "rust".parse()?;
                assert_eq!(
                    choose_hackerrank_starter(&starters, Some(&language), false)?,
                    1
                );
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        }
        .await;
        let _ = fs::remove_dir_all(&root);
        result
    }
}
