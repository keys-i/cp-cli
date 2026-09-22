use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::domain::{
    AccountName, ContestSlug, Difficulty, DiscussionId, LanguageSlug, PracticeTopic, ProblemQuery,
    ProblemSelector, ProblemTag,
};

#[derive(Parser)]
#[command(
    version,
    disable_help_subcommand = true,
    about = "Solve problems across coding platforms from your terminal",
    after_help = "Get started:\n  cp-cli problem daily\n  cp-cli problem list --difficulty medium --tag graph\n  cp-cli problem search \"two sum\"\n  cp-cli problem show 1\n  cp-cli --platform hackerrank problem show solve-me-first\n  cp-cli --platform codeforces problem show 4A\n  cp-cli --platform exercism problem pick two-fer --track rust\n  cp-cli --platform project-euler problem show 1\n  cp-cli --platform hackerearth problem list --topic algorithms/searching/linear-search\n  cp-cli --platform codechef stats ksun48\n  cp-cli problem pick 1\n  cp-cli stats\n  cp-cli contest list\n  cp-cli discussion list\n  cp-cli discussion show 8533478\n\nUse the identifiers and paths returned by each platform.\nFor scripts, add --format json."
)]
pub(crate) struct Cli {
    #[arg(long, value_enum, global = true, default_value = "leetcode")]
    pub(crate) platform: Platform,
    /// Readable text by default; JSON for scripts
    #[arg(long, value_enum, global = true, default_value = "text")]
    pub(crate) format: Format,
    /// Colorize terminal output (auto respects NO_COLOR)
    #[arg(long, value_enum, global = true, default_value = "auto")]
    pub(crate) color: Color,
    /// Enlarge titles in supported terminals when space permits
    #[arg(long, value_enum, global = true, default_value = "auto")]
    pub(crate) heading_size: HeadingSize,
    /// Possum, arcade, phosphor, amber, moonlight, or neutral colors
    #[arg(long, value_enum, global = true, default_value = "possum")]
    pub(crate) theme: Theme,
    /// Match the terminal profile, or explicitly select its background tone
    #[arg(long, value_enum, global = true, default_value = "auto")]
    pub(crate) background: Background,
    /// Disable terminal motion
    #[arg(long, global = true)]
    pub(crate) no_animation: bool,
    /// Ring the terminal bell when the problem is ready
    #[arg(long, global = true)]
    pub(crate) sound: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Platform {
    #[value(name = "codechef")]
    CodeChef,
    #[value(name = "codeforces")]
    Codeforces,
    #[value(name = "exercism")]
    Exercism,
    #[value(name = "hackerearth")]
    HackerEarth,
    #[value(name = "hackerrank")]
    HackerRank,
    #[value(name = "leetcode")]
    LeetCode,
    #[value(name = "project-euler")]
    ProjectEuler,
}

impl Platform {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::CodeChef => "CodeChef",
            Self::Codeforces => "Codeforces",
            Self::Exercism => "Exercism",
            Self::HackerEarth => "HackerEarth",
            Self::HackerRank => "HackerRank",
            Self::LeetCode => "LeetCode",
            Self::ProjectEuler => "Project Euler",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Format {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Color {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum HeadingSize {
    Auto,
    Normal,
    Large,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Theme {
    Possum,
    Arcade,
    Moonlight,
    Phosphor,
    Amber,
    Dark,
    Light,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Background {
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ListDifficulty {
    Easy,
    Medium,
    Hard,
}

impl From<ListDifficulty> for Difficulty {
    fn from(value: ListDifficulty) -> Self {
        match value {
            ListDifficulty::Easy => Self::Easy,
            ListDifficulty::Medium => Self::Medium,
            ListDifficulty::Hard => Self::Hard,
        }
    }
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Configure or inspect supported terminal authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// Browse and participate in platform contests
    Contest {
        #[command(subcommand)]
        command: ContestCommand,
    },
    /// Browse public discussions
    Discussion {
        #[command(subcommand)]
        command: DiscussionCommand,
    },
    /// Browse public problems
    Problem {
        #[command(subcommand)]
        command: ProblemCommand,
    },
    /// Show account progress and recent submissions
    Stats {
        /// Public account name when the platform cannot identify an authenticated session
        #[arg(value_name = "ACCOUNT")]
        user: Option<AccountName>,
    },
}

impl Command {
    pub(crate) fn path(&self) -> &'static str {
        match self {
            Self::Auth { command } => match command {
                AuthCommand::Login => "auth login",
                AuthCommand::Logout => "auth logout",
                AuthCommand::Status => "auth status",
            },
            Self::Contest { command } => match command {
                ContestCommand::Create => "contest create",
                ContestCommand::Delete { .. } => "contest delete",
                ContestCommand::Edit { .. } => "contest edit",
                ContestCommand::Join { .. } => "contest join",
                ContestCommand::Leave { .. } => "contest leave",
                ContestCommand::List => "contest list",
                ContestCommand::Show { .. } => "contest show",
                ContestCommand::Status { .. } => "contest status",
            },
            Self::Discussion { command } => match command {
                DiscussionCommand::Create { .. } => "discussion create",
                DiscussionCommand::Delete { .. } => "discussion delete",
                DiscussionCommand::Edit { .. } => "discussion edit",
                DiscussionCommand::List { .. } => "discussion list",
                DiscussionCommand::Show { .. } => "discussion show",
                DiscussionCommand::Reply { .. } => "discussion reply",
            },
            Self::Problem { command } => match command {
                ProblemCommand::Daily => "problem daily",
                ProblemCommand::List { .. } => "problem list",
                ProblemCommand::Pick { .. } => "problem pick",
                ProblemCommand::Test(_) => "problem test",
                ProblemCommand::Submit(_) => "problem submit",
                ProblemCommand::Show { .. } => "problem show",
                ProblemCommand::Search { .. } => "problem search",
            },
            Self::Stats { .. } => "stats",
        }
    }
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub(crate) enum DiscussionCommand {
    /// Create a discussion through a verified platform API
    Create {
        /// Optional problem number or slug for a solution post
        #[arg(value_name = "NUMBER|SLUG")]
        problem: Option<ProblemSelector>,
    },
    /// Delete a discussion through a verified platform API
    Delete {
        /// Numeric topic ID from the platform discussion URL
        #[arg(value_name = "ID")]
        discussion: DiscussionId,
    },
    /// Edit a discussion through a verified platform API
    Edit {
        /// Numeric topic ID from the platform discussion URL
        #[arg(value_name = "ID")]
        discussion: DiscussionId,
    },
    /// List trending posts, or recent posts for one problem
    List {
        /// Optional problem number or slug
        #[arg(value_name = "NUMBER|SLUG")]
        problem: Option<ProblemSelector>,
    },
    /// Show one discussion by its numeric topic ID
    Show {
        /// Numeric topic ID from a LeetCode discussion URL
        #[arg(value_name = "ID")]
        discussion: DiscussionId,
    },
    /// Reply through a verified platform API
    Reply {
        /// Numeric topic ID from a LeetCode discussion URL
        #[arg(value_name = "ID")]
        discussion: DiscussionId,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub(crate) enum ContestCommand {
    /// Create a contest through a verified platform API
    Create,
    /// Edit a contest through a verified platform API
    Edit {
        /// Contest ID or slug from its platform URL
        #[arg(value_name = "ID")]
        contest: ContestSlug,
    },
    /// Delete a contest through a verified platform API
    Delete {
        /// Contest ID or slug from its platform URL
        #[arg(value_name = "ID")]
        contest: ContestSlug,
    },
    /// Join a contest through a verified platform API
    Join {
        /// Contest slug from its LeetCode URL
        #[arg(value_name = "SLUG")]
        contest: ContestSlug,
    },
    /// Leave a contest through a verified platform API
    Leave {
        /// Contest slug from its LeetCode URL
        #[arg(value_name = "SLUG")]
        contest: ContestSlug,
    },
    /// List upcoming contests
    List,
    /// Show one contest's schedule
    Show {
        /// Contest slug from its LeetCode URL
        #[arg(value_name = "SLUG")]
        contest: ContestSlug,
    },
    /// Show whether the signed-in account joined a contest
    Status {
        /// Contest slug from its LeetCode URL
        #[arg(value_name = "SLUG")]
        contest: ContestSlug,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub(crate) enum AuthCommand {
    /// Configure supported account credentials
    Login,
    /// Remove credentials saved by this CLI
    Logout,
    /// Check whether configured credentials authenticate successfully
    Status,
}

#[derive(Args)]
pub(crate) struct SolutionTarget {
    /// Solution file created by problem pick
    #[arg(value_name = "FILE")]
    pub(crate) path: PathBuf,
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub(crate) enum ProblemCommand {
    /// Show the platform's daily challenge when available
    Daily,
    /// Browse public problems
    List {
        /// Easy, medium or hard
        #[arg(long, value_enum)]
        difficulty: Option<ListDifficulty>,
        /// LeetCode tag slug, such as graph or dynamic-programming
        #[arg(long, value_name = "TAG")]
        tag: Option<ProblemTag>,
        /// Exercism or HackerRank track slug
        #[arg(long, value_name = "TRACK")]
        track: Option<ProblemTag>,
        /// HackerEarth practice path, such as algorithms/searching/linear-search
        #[arg(long, value_name = "PATH")]
        topic: Option<PracticeTopic>,
        /// Catalogue page
        #[arg(long, value_name = "PAGE")]
        page: Option<u8>,
        /// Show Project Euler's ten newest problems
        #[arg(long, conflicts_with = "page")]
        recent: bool,
    },
    /// Choose a language starter and save it locally
    Pick {
        /// Positive problem number or slug from the problem URL
        #[arg(value_name = "NUMBER|SLUG")]
        problem: ProblemSelector,
        /// Platform language slug; omitted values are prompted for when needed
        #[arg(long, value_name = "LANG")]
        lang: Option<LanguageSlug>,
        /// Destination directory; omitted values are prompted for
        #[arg(long, value_name = "DIR")]
        dir: Option<PathBuf>,
        /// Exercism track slug, such as rust or python
        #[arg(long, value_name = "TRACK")]
        track: Option<ProblemTag>,
    },
    /// Run the platform's supported local or remote tests
    Test(SolutionTarget),
    /// Submit the marked solution through the supported platform workflow
    Submit(SolutionTarget),
    /// Show a problem's details and available statement
    Show {
        /// Problem ID from the platform URL
        #[arg(value_name = "ID")]
        problem: ProblemSelector,
        /// Exercism track slug, such as rust or python
        #[arg(long, value_name = "TRACK")]
        track: Option<ProblemTag>,
    },
    /// Search public problems by title or number
    Search {
        /// Search text, such as two sum or 1
        #[arg(value_name = "QUERY")]
        query: ProblemQuery,
    },
}
