use clap::{Parser, Subcommand, ValueEnum};

use crate::domain::{ProblemId, ProblemQuery};

#[derive(Parser)]
#[command(
    version,
    disable_help_subcommand = true,
    about = "Find and read LeetCode problems in your terminal",
    after_help = "Get started:\n  cp-cli problem search \"two sum\"\n  cp-cli problem show two-sum\n\nUse a title or number to search, then open its slug.\nFor scripts, add --format json."
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

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Platform {
    #[value(name = "leetcode")]
    LeetCode,
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

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Browse public problems
    Problem {
        #[command(subcommand)]
        command: ProblemCommand,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub(crate) enum ProblemCommand {
    /// Show a problem's title and statement
    Show {
        /// Slug from the problem URL, such as two-sum (not its numeric number)
        #[arg(value_name = "SLUG")]
        id: ProblemId,
    },
    /// Search public problems by title or number
    Search {
        /// Search text, such as two sum or 1
        #[arg(value_name = "QUERY")]
        query: ProblemQuery,
    },
}
