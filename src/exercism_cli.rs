use std::{
    ffi::OsString,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use serde::Deserialize;

use crate::domain::{ProblemId, ProblemTag};

const MAX_METADATA_BYTES: u64 = 64 * 1024;
const MAX_SOLUTION_FILES: usize = 64;
const MAX_EXERCISE_ANCESTORS: usize = 16;

pub(crate) type Result<T> = std::result::Result<T, ExercismCliError>;

#[derive(Debug)]
pub(crate) struct DownloadedExercise {
    pub(crate) directory: PathBuf,
    pub(crate) solution_files: Vec<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ExercismCliError {
    #[error("the Exercism API token must be 1 to 2048 visible ASCII characters")]
    InvalidToken,
    #[error("the selected Exercism workspace {0} must exist and be accessible")]
    Workspace(PathBuf),
    #[error("the downloaded Exercism exercise escapes its workspace")]
    UnsafeExerciseDirectory,
    #[error("the downloaded Exercism exercise is missing .meta/config.json")]
    MissingMetadata,
    #[error("the Exercism exercise metadata is too large")]
    MetadataTooLarge,
    #[error("the downloaded Exercism exercise metadata is invalid")]
    InvalidMetadata(#[source] serde_json::Error),
    #[error("the Exercism exercise metadata has no usable solution files")]
    InvalidSolutionFiles,
    #[error("could not read the downloaded Exercism exercise")]
    Read(#[source] std::io::Error),
    #[error("could not start the official Exercism CLI for {0}")]
    Start(CommandKind, #[source] std::io::Error),
    #[error("the official `exercism` CLI is not installed or is not on PATH")]
    MissingCli,
    #[error("the official Exercism CLI {operation} command failed{code}")]
    CommandFailed {
        operation: CommandKind,
        code: ExitCode,
    },
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CommandKind {
    Configure,
    Download,
    Test,
    Submit,
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Configure => "configure",
            Self::Download => "download",
            Self::Test => "test",
            Self::Submit => "submit",
        })
    }
}

#[derive(Debug)]
pub(crate) struct ExitCode(Option<i32>);

impl std::fmt::Display for ExitCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(code) => write!(formatter, " (exit code {code})"),
            None => formatter.write_str(" (terminated by signal)"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Client {
    executable: PathBuf,
}

impl Client {
    pub(crate) fn new() -> Self {
        Self {
            executable: PathBuf::from("exercism"),
        }
    }

    pub(crate) fn download(
        &self,
        workspace: &Path,
        track: &ProblemTag,
        exercise: &ProblemId,
    ) -> Result<DownloadedExercise> {
        let workspace = canonical_workspace(workspace)?;
        self.run(configure_invocation(&workspace))?;
        self.run(download_invocation(track, exercise))?;
        discover(&workspace, track, exercise)
    }

    pub(crate) fn configure_token(&self, token: &str) -> Result<()> {
        if !valid_token(token) {
            return Err(ExercismCliError::InvalidToken);
        }
        self.run_quiet(configure_token_invocation(token))
    }

    pub(crate) fn show_configuration(&self) -> Result<()> {
        self.run_quiet(configure_show_invocation())
    }

    pub(crate) fn test(&self, exercise: &DownloadedExercise) -> Result<()> {
        self.run(Invocation::new(CommandKind::Test).in_directory(&exercise.directory))
    }

    pub(crate) fn submit(&self, exercise: &DownloadedExercise) -> Result<()> {
        self.run(
            Invocation::new(CommandKind::Submit)
                .arguments(exercise.solution_files.iter().cloned().map(OsString::from))
                .in_directory(&exercise.directory),
        )
    }

    fn run(&self, invocation: Invocation) -> Result<()> {
        let mut command = Command::new(&self.executable);
        command.args(&invocation.arguments);
        if let Some(directory) = invocation.directory {
            command.current_dir(directory);
        }
        let status = command
            .status()
            .map_err(|source| start_error(invocation.kind, source))?;
        if status.success() {
            Ok(())
        } else {
            Err(ExercismCliError::CommandFailed {
                operation: invocation.kind,
                code: ExitCode(status.code()),
            })
        }
    }

    fn run_quiet(&self, invocation: Invocation) -> Result<()> {
        let mut command = Command::new(&self.executable);
        command
            .args(&invocation.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = command
            .output()
            .map_err(|source| start_error(invocation.kind, source))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(ExercismCliError::CommandFailed {
                operation: invocation.kind,
                code: ExitCode(output.status.code()),
            })
        }
    }
}

fn start_error(kind: CommandKind, source: std::io::Error) -> ExercismCliError {
    if source.kind() == std::io::ErrorKind::NotFound {
        ExercismCliError::MissingCli
    } else {
        ExercismCliError::Start(kind, source)
    }
}

const _: fn(&Client, &str) -> Result<()> = Client::configure_token;
const _: fn(&Client) -> Result<()> = Client::show_configuration;

#[derive(Debug)]
struct Invocation {
    kind: CommandKind,
    arguments: Vec<OsString>,
    directory: Option<PathBuf>,
}

impl Invocation {
    fn new(kind: CommandKind) -> Self {
        Self {
            kind,
            arguments: vec![kind.to_string().into()],
            directory: None,
        }
    }

    fn arguments(mut self, arguments: impl IntoIterator<Item = OsString>) -> Self {
        self.arguments.extend(arguments);
        self
    }

    fn in_directory(mut self, directory: &Path) -> Self {
        self.directory = Some(directory.into());
        self
    }
}

fn download_invocation(track: &ProblemTag, exercise: &ProblemId) -> Invocation {
    Invocation::new(CommandKind::Download).arguments([
        "--track".into(),
        track.as_ref().into(),
        "--exercise".into(),
        exercise.as_ref().into(),
    ])
}

fn configure_invocation(workspace: &Path) -> Invocation {
    Invocation::new(CommandKind::Configure)
        .arguments(["--workspace".into(), workspace.as_os_str().into()])
}

fn configure_token_invocation(token: &str) -> Invocation {
    Invocation::new(CommandKind::Configure).arguments(["--token".into(), token.into()])
}

fn configure_show_invocation() -> Invocation {
    Invocation::new(CommandKind::Configure).arguments(["--show".into()])
}

fn valid_token(token: &str) -> bool {
    !token.is_empty() && token.len() <= 2048 && token.bytes().all(|byte| byte.is_ascii_graphic())
}

pub(crate) fn discover(
    workspace: &Path,
    track: &ProblemTag,
    exercise: &ProblemId,
) -> Result<DownloadedExercise> {
    let workspace = canonical_workspace(workspace)?;
    let directory = fs::canonicalize(workspace.join(track.as_ref()).join(exercise.as_ref()))
        .map_err(|_| ExercismCliError::Workspace(workspace.clone()))?;
    if !directory.starts_with(&workspace) {
        return Err(ExercismCliError::UnsafeExerciseDirectory);
    }
    exercise_from_directory(directory)
}

fn canonical_workspace(workspace: &Path) -> Result<PathBuf> {
    fs::canonicalize(workspace).map_err(|_| ExercismCliError::Workspace(workspace.into()))
}

pub(crate) fn discover_from_solution(path: &Path) -> Result<DownloadedExercise> {
    let target = fs::canonicalize(path).map_err(ExercismCliError::Read)?;
    if !fs::metadata(&target)
        .map_err(ExercismCliError::Read)?
        .is_file()
    {
        return Err(ExercismCliError::InvalidSolutionFiles);
    }

    for directory in target.ancestors().skip(1).take(MAX_EXERCISE_ANCESTORS) {
        if directory.join(".meta/config.json").is_file() {
            let exercise = exercise_from_directory(directory.into())?;
            if exercise
                .solution_files
                .iter()
                .any(|solution| directory.join(solution) == target)
            {
                return Ok(exercise);
            }
            return Err(ExercismCliError::InvalidSolutionFiles);
        }
    }
    Err(ExercismCliError::MissingMetadata)
}

fn exercise_from_directory(directory: PathBuf) -> Result<DownloadedExercise> {
    let metadata_path = directory.join(".meta/config.json");
    let metadata_path = fs::canonicalize(&metadata_path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => ExercismCliError::MissingMetadata,
        _ => ExercismCliError::Read(error),
    })?;
    if !metadata_path.starts_with(&directory) {
        return Err(ExercismCliError::UnsafeExerciseDirectory);
    }
    let metadata = metadata(&metadata_path)?;
    if metadata.files.solution.is_empty() || metadata.files.solution.len() > MAX_SOLUTION_FILES {
        return Err(ExercismCliError::InvalidSolutionFiles);
    }

    let solution_files = metadata
        .files
        .solution
        .into_iter()
        .map(|path| solution_path(&directory, &path))
        .collect::<Result<Vec<_>>>()?;
    Ok(DownloadedExercise {
        directory,
        solution_files,
    })
}

fn metadata(path: &Path) -> Result<ExerciseMetadata> {
    let size = fs::metadata(path).map_err(ExercismCliError::Read)?.len();
    if size > MAX_METADATA_BYTES {
        return Err(ExercismCliError::MetadataTooLarge);
    }
    let mut contents = Vec::with_capacity(size as usize);
    fs::File::open(path)
        .map_err(ExercismCliError::Read)?
        .take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut contents)
        .map_err(ExercismCliError::Read)?;
    if contents.len() as u64 > MAX_METADATA_BYTES {
        return Err(ExercismCliError::MetadataTooLarge);
    }
    serde_json::from_slice(&contents).map_err(ExercismCliError::InvalidMetadata)
}

fn solution_path(directory: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(ExercismCliError::InvalidSolutionFiles);
    }
    let path = fs::canonicalize(directory.join(relative)).map_err(ExercismCliError::Read)?;
    if !path.starts_with(directory)
        || !fs::metadata(&path)
            .map_err(ExercismCliError::Read)?
            .is_file()
    {
        return Err(ExercismCliError::InvalidSolutionFiles);
    }
    path.strip_prefix(directory)
        .map(Path::to_path_buf)
        .map_err(|_| ExercismCliError::InvalidSolutionFiles)
}

#[derive(Deserialize)]
struct ExerciseMetadata {
    files: ExerciseFiles,
}

#[derive(Deserialize)]
struct ExerciseFiles {
    solution: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        fs,
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        CommandKind, ExercismCliError, Invocation, configure_invocation, configure_show_invocation,
        configure_token_invocation, discover, discover_from_solution, download_invocation,
        solution_path,
    };
    use crate::domain::{ProblemId, ProblemTag};

    #[test]
    fn download_uses_the_documented_structured_arguments() -> Result<(), Box<dyn std::error::Error>>
    {
        let track = ProblemTag::from_str("rust")?;
        let exercise = ProblemId::from_str("two-fer")?;
        let invocation = download_invocation(&track, &exercise);
        assert!(matches!(invocation.kind, CommandKind::Download));
        assert_eq!(
            invocation.arguments,
            ["download", "--track", "rust", "--exercise", "two-fer"].map(OsString::from)
        );
        let workspace = std::path::Path::new("workspace");
        let invocation = configure_invocation(workspace);
        assert!(matches!(invocation.kind, CommandKind::Configure));
        assert_eq!(
            invocation.arguments,
            ["configure", "--workspace", "workspace"].map(OsString::from)
        );
        let invocation = configure_show_invocation();
        assert_eq!(
            invocation.arguments,
            ["configure", "--show"].map(OsString::from)
        );
        let invocation = configure_token_invocation("test-token");
        assert_eq!(
            invocation.arguments[..2],
            ["configure", "--token"].map(OsString::from)
        );
        Ok(())
    }

    #[test]
    fn test_and_submit_keep_the_exercise_as_the_working_directory() {
        let directory = std::path::Path::new("exercise");
        assert_eq!(
            Invocation::new(CommandKind::Test)
                .in_directory(directory)
                .directory,
            Some(directory.into())
        );
    }

    #[test]
    fn solution_paths_cannot_escape_the_exercise_directory() {
        let directory = std::path::Path::new("exercise");
        assert!(matches!(
            solution_path(directory, std::path::Path::new("../outside")),
            Err(ExercismCliError::InvalidSolutionFiles)
        ));
        assert!(matches!(
            solution_path(directory, std::path::Path::new("/outside")),
            Err(ExercismCliError::InvalidSolutionFiles)
        ));
    }

    #[test]
    fn discovery_reads_only_declared_solution_files() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "cp-cli-exercism-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let track = ProblemTag::from_str("rust")?;
            let exercise = ProblemId::from_str("two-fer")?;
            let directory = root.join("rust/two-fer");
            fs::create_dir_all(directory.join(".meta"))?;
            fs::create_dir_all(directory.join("src"))?;
            fs::write(
                directory.join(".meta/config.json"),
                r#"{"files":{"solution":["src/lib.rs"]}}"#,
            )?;
            fs::write(directory.join("src/lib.rs"), "pub fn two_fer() {}")?;

            let downloaded = discover(&root, &track, &exercise)?;
            assert_eq!(
                downloaded.solution_files,
                [std::path::PathBuf::from("src/lib.rs")]
            );
            Ok(())
        })();
        let _ = fs::remove_dir_all(&root);
        result
    }

    #[test]
    fn solution_discovery_requires_a_declared_solution_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "cp-cli-exercism-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let directory = root.join("rust/two-fer");
            fs::create_dir_all(directory.join(".meta"))?;
            fs::create_dir_all(directory.join("src"))?;
            fs::write(
                directory.join(".meta/config.json"),
                r#"{"files":{"solution":["src/lib.rs"]}}"#,
            )?;
            let solution = directory.join("src/lib.rs");
            fs::write(&solution, "pub fn two_fer() {}")?;
            fs::write(directory.join("notes.txt"), "not submitted")?;

            assert_eq!(
                discover_from_solution(&solution)?.solution_files,
                [std::path::PathBuf::from("src/lib.rs")]
            );
            assert!(matches!(
                discover_from_solution(&directory.join("notes.txt")),
                Err(ExercismCliError::InvalidSolutionFiles)
            ));
            Ok(())
        })();
        let _ = fs::remove_dir_all(&root);
        result
    }
}
