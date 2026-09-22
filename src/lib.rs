mod auth;
mod catalog;
mod cli;
mod codeforces_support;
mod community;
mod contests;
mod discussions;
mod domain;
mod error;
mod exercism_cli;
mod math;
mod output;
mod platform_auth;
mod possum;
mod problems;
mod public_stats;
mod reader;
mod solve;
mod stats;
mod theme;

use std::{
    io::{IsTerminal, Write},
    time::Duration,
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

pub use error::Error;

enum Content {
    AuthLogout(domain::AuthLogout),
    Auth(domain::AuthStatus),
    CatalogList(domain::CatalogProblemList),
    CatalogProblem(domain::CatalogProblem),
    CodeforcesContest(codeforces_support::CodeforcesContest),
    CodeforcesContests(codeforces_support::CodeforcesContestList),
    CodeforcesStats(codeforces_support::CodeforcesStats),
    CommunityDiscussion(domain::CommunityDiscussion),
    CommunityDiscussions(domain::CommunityDiscussionList),
    Contest(domain::Contest),
    ContestRegistration(domain::ContestRegistration),
    Contests(domain::ContestList),
    Discussion(domain::Discussion),
    Discussions(domain::DiscussionList),
    List(domain::ProblemList),
    Problem(domain::Problem),
    PublicStats(domain::PublicAccountStats),
    Search(domain::ProblemSearch),
    Solution(domain::SolutionFile),
    Stats(domain::AccountStats),
    Submission(domain::SubmissionResult),
    Test(domain::TestResult),
    ToolAction(domain::ToolAction),
}

pub async fn run() -> Result<(), Error> {
    let cli = cli::Cli::parse();
    let platform = cli.platform;
    if let cli::Command::Problem { command } = &cli.command {
        match (platform, command) {
            (
                cli::Platform::LeetCode,
                cli::ProblemCommand::List { track: Some(_), .. }
                | cli::ProblemCommand::Show { track: Some(_), .. }
                | cli::ProblemCommand::Pick { track: Some(_), .. },
            ) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem browsing with --track",
                });
            }
            (platform, cli::ProblemCommand::List { topic: Some(_), .. })
                if platform != cli::Platform::HackerEarth =>
            {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --topic",
                });
            }
            (cli::Platform::LeetCode, cli::ProblemCommand::List { page: Some(_), .. }) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --page",
                });
            }
            (cli::Platform::LeetCode, cli::ProblemCommand::List { recent: true, .. }) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --recent",
                });
            }
            (
                cli::Platform::CodeChef,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    track,
                    topic,
                    page,
                    recent,
                },
            ) if difficulty.is_some()
                || tag.is_some()
                || track.is_some()
                || topic.is_some()
                || page.is_some()
                || *recent =>
            {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list filters",
                });
            }
            (
                cli::Platform::HackerEarth,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    track,
                    recent,
                    ..
                },
            ) if difficulty.is_some() || tag.is_some() || track.is_some() || *recent => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --difficulty, --tag, --track or --recent",
                });
            }
            (
                cli::Platform::Codeforces,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    track,
                    recent,
                    ..
                },
            ) if difficulty.is_some() || tag.is_some() || track.is_some() || *recent => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --difficulty, --tag, --track or --recent",
                });
            }
            (
                cli::Platform::HackerRank,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    recent,
                    ..
                },
            ) if difficulty.is_some() || tag.is_some() || *recent => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --difficulty, --tag or --recent",
                });
            }
            (
                cli::Platform::ProjectEuler,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    track,
                    ..
                },
            ) if difficulty.is_some() || tag.is_some() || track.is_some() => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --difficulty, --tag or --track",
                });
            }
            (
                cli::Platform::Exercism,
                cli::ProblemCommand::List {
                    difficulty,
                    tag,
                    page,
                    recent,
                    ..
                },
            ) if difficulty.is_some() || tag.is_some() || page.is_some() || *recent => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem list with --difficulty, --tag or --page",
                });
            }
            (
                cli::Platform::CodeChef
                | cli::Platform::Codeforces
                | cli::Platform::HackerEarth
                | cli::Platform::HackerRank
                | cli::Platform::ProjectEuler,
                cli::ProblemCommand::Show { track: Some(_), .. },
            ) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem show with --track",
                });
            }
            (
                cli::Platform::CodeChef
                | cli::Platform::Codeforces
                | cli::Platform::HackerEarth
                | cli::Platform::HackerRank
                | cli::Platform::ProjectEuler,
                cli::ProblemCommand::Pick { track: Some(_), .. },
            ) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem pick with --track",
                });
            }
            (
                cli::Platform::Exercism,
                cli::ProblemCommand::List { track: None, .. }
                | cli::ProblemCommand::Show { track: None, .. }
                | cli::ProblemCommand::Pick { track: None, .. },
            ) => return Err(Error::ExercismTrackRequired),
            (cli::Platform::Exercism, cli::ProblemCommand::Pick { lang: Some(_), .. }) => {
                return Err(Error::UnsupportedPlatformCommand {
                    platform: platform.label(),
                    command: "problem pick with --lang; the Exercism track chooses the language",
                });
            }
            _ => {}
        }
    }
    let activity = match &cli.command {
        cli::Command::Auth { command } => match command {
            cli::AuthCommand::Login => format!("Configuring {} credentials", platform.label()),
            cli::AuthCommand::Logout => format!("Removing {} credentials", platform.label()),
            cli::AuthCommand::Status => format!("Checking {} credentials", platform.label()),
        },
        cli::Command::Contest { command } => match command {
            cli::ContestCommand::Create => "Checking contest creation support".to_owned(),
            cli::ContestCommand::Delete { contest } => {
                format!("Checking whether {} can be deleted", contest.as_ref())
            }
            cli::ContestCommand::Edit { contest } => {
                format!("Checking whether {} can be edited", contest.as_ref())
            }
            cli::ContestCommand::Join { contest } => {
                format!("Checking registration for {}", contest.as_ref())
            }
            cli::ContestCommand::Leave { contest } => {
                format!("Checking registration for {}", contest.as_ref())
            }
            cli::ContestCommand::List => "Finding upcoming contests".to_owned(),
            cli::ContestCommand::Show { contest } => {
                format!("Loading contest {}", contest.as_ref())
            }
            cli::ContestCommand::Status { contest } => {
                format!("Checking registration for {}", contest.as_ref())
            }
        },
        cli::Command::Discussion { command } => match command {
            cli::DiscussionCommand::Create {
                problem: Some(problem),
            } => match problem {
                domain::ProblemSelector::Number(number) => {
                    format!("Checking discussion support for problem {number}")
                }
                domain::ProblemSelector::Slug(id) => {
                    format!("Checking discussion support for {}", id.as_ref())
                }
            },
            cli::DiscussionCommand::Create { problem: None } => {
                "Checking discussion creation support".to_owned()
            }
            cli::DiscussionCommand::Delete { discussion } => {
                format!(
                    "Checking whether discussion {} can be deleted",
                    discussion.get()
                )
            }
            cli::DiscussionCommand::Edit { discussion } => {
                format!(
                    "Checking whether discussion {} can be edited",
                    discussion.get()
                )
            }
            cli::DiscussionCommand::List {
                problem: Some(problem),
            } => match problem {
                domain::ProblemSelector::Number(number) => {
                    format!("Finding discussions for problem {number}")
                }
                domain::ProblemSelector::Slug(id) => {
                    format!("Finding discussions for {}", id.as_ref())
                }
            },
            cli::DiscussionCommand::List { problem: None } => {
                "Finding trending discussions".to_owned()
            }
            cli::DiscussionCommand::Show { discussion } => {
                format!("Loading discussion {}", discussion.get())
            }
            cli::DiscussionCommand::Reply { discussion } => {
                format!("Checking reply support for discussion {}", discussion.get())
            }
        },
        cli::Command::Problem { command } => match command {
            cli::ProblemCommand::Daily => "Loading the Daily Challenge".to_owned(),
            cli::ProblemCommand::List { .. } => {
                format!("Browsing {} problems", platform.label())
            }
            cli::ProblemCommand::Pick { .. } => "Fetching starter code".to_owned(),
            cli::ProblemCommand::Test(_) if platform == cli::Platform::LeetCode => {
                "Running LeetCode examples".to_owned()
            }
            cli::ProblemCommand::Test(_) if platform == cli::Platform::Exercism => {
                "Running Exercism tests".to_owned()
            }
            cli::ProblemCommand::Test(_) => "Checking platform test support".to_owned(),
            cli::ProblemCommand::Submit(_) if platform == cli::Platform::Exercism => {
                "Submitting with Exercism".to_owned()
            }
            cli::ProblemCommand::Submit(_) => "Submitting solution".to_owned(),
            cli::ProblemCommand::Show { problem, .. } => match problem {
                domain::ProblemSelector::Number(number) => {
                    format!("Loading {} problem {number}", platform.label())
                }
                domain::ProblemSelector::Slug(id) => format!("Loading {}", id.as_ref()),
            },
            cli::ProblemCommand::Search { query } => format!("Searching {}", query.as_ref()),
        },
        cli::Command::Stats { user } => user.as_ref().map_or_else(
            || format!("Loading {} account stats", platform.label()),
            |user| format!("Loading stats for {}", user.as_ref()),
        ),
    };
    let auth_prompt = matches!(
        &cli.command,
        cli::Command::Auth {
            command: cli::AuthCommand::Login | cli::AuthCommand::Logout
        }
    );
    let mut presentation = output::Presentation::detect(cli.color, cli.heading_size);
    presentation.motion = !cli.no_animation;
    let terminal = std::io::stdout().is_terminal()
        && matches!(cli.format, cli::Format::Text)
        && std::env::var("TERM").as_deref() != Ok("dumb");
    presentation.palette = cli.theme.palette(theme::background(
        cli.background,
        terminal && presentation.color,
        cli.theme,
    ));
    let accent = presentation.accent();
    let frames =
        possum::loading_frames(presentation.columns, presentation.color, !cli.no_animation);
    let frame_refs: Vec<_> = frames.iter().map(String::as_str).collect();
    let spinner_style = ProgressStyle::with_template(&format!(
        "{{spinner}}\n{accent}POSSUM// LINK{accent:#}  {{msg}}"
    ))
    .map_err(std::io::Error::other)?
    .tick_strings(&frame_refs);
    let bar_art = possum::render(
        possum::Pose::Idle,
        presentation.columns.min(30),
        presentation.color,
    );
    let bar_template = match bar_art {
        Some(art) => {
            format!("{art}\n{accent}POSSUM// LOAD [{{bar:16}}]{accent:#} {{bytes}}/{{total_bytes}}")
        }
        None => format!("{accent}POSSUM// LOAD [{{bar:16}}]{accent:#} {{bytes}}/{{total_bytes}}"),
    };
    let bar_style = ProgressStyle::with_template(&bar_template)
        .map_err(std::io::Error::other)?
        .progress_chars("█▉▊▋▌▍▎▏ ");
    let progress = if terminal && std::io::stderr().is_terminal() && !auth_prompt {
        let progress = ProgressBar::new_spinner();
        progress.set_style(spinner_style.clone());
        progress.set_message(activity);
        if !cli.no_animation {
            progress.enable_steady_tick(Duration::from_millis(260));
        } else {
            progress.tick();
        }
        Some(progress)
    } else {
        None
    };
    let result = match (cli.platform, cli.command) {
        (
            cli::Platform::Codeforces,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { page, .. },
            },
        ) => catalog::list(
            domain::CatalogPlatform::Codeforces,
            page,
            None,
            None,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
        )
        .await
        .map(Content::CatalogList),
        (
            cli::Platform::HackerRank,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { page, track, .. },
            },
        ) => catalog::list(
            domain::CatalogPlatform::HackerRank,
            page,
            track,
            None,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
        )
        .await
        .map(Content::CatalogList),
        (
            cli::Platform::Codeforces,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => match catalog::problem_id(domain::CatalogPlatform::Codeforces, problem) {
            Ok(id) => catalog::show(domain::CatalogPlatform::Codeforces, id, |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::CatalogProblem),
            Err(error) => Err(error),
        },
        (
            cli::Platform::ProjectEuler,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { recent: true, .. },
            },
        ) => catalog::project_euler_recent(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CatalogList),
        (
            cli::Platform::ProjectEuler,
            cli::Command::Problem {
                command:
                    cli::ProblemCommand::List {
                        page,
                        recent: false,
                        ..
                    },
            },
        ) => catalog::list(
            domain::CatalogPlatform::ProjectEuler,
            page,
            None,
            None,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
        )
        .await
        .map(Content::CatalogList),
        (
            cli::Platform::ProjectEuler,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => match catalog::problem_id(domain::CatalogPlatform::ProjectEuler, problem) {
            Ok(id) => catalog::show(domain::CatalogPlatform::ProjectEuler, id, |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::CatalogProblem),
            Err(error) => Err(error),
        },
        (
            cli::Platform::Exercism,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { track, .. },
            },
        ) => {
            let _ = track;
            Err(Error::CapabilityUnavailable {
                platform: "Exercism",
                capability: "a terminal problem catalogue; use `problem pick --track <track>` with a known exercise slug",
            })
        }
        (
            cli::Platform::Exercism,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, track, .. },
            },
        ) => {
            let _ = (problem, track);
            Err(Error::CapabilityUnavailable {
                platform: "Exercism",
                capability: "terminal problem details before an exercise is downloaded; `problem pick` downloads its README and tests",
            })
        }
        (
            cli::Platform::Exercism,
            cli::Command::Problem {
                command:
                    cli::ProblemCommand::Pick {
                        problem,
                        dir,
                        track: Some(track),
                        ..
                    },
            },
        ) => {
            if matches!(cli.format, cli::Format::Json) {
                Err(Error::CapabilityUnavailable {
                    platform: "Exercism",
                    capability: "JSON output for official CLI downloads",
                })
            } else {
                let id = match problem {
                    domain::ProblemSelector::Slug(id) => Ok(id),
                    domain::ProblemSelector::Number(_) => Err(Error::InvalidCatalogProblemId {
                        platform: "Exercism",
                        expected: "the exercise slug from its URL",
                    }),
                }?;
                let interactive = std::io::stdin().is_terminal() && std::io::stderr().is_terminal();
                let workspace = solve::choose_directory(dir, interactive)?;
                std::fs::create_dir_all(&workspace).map_err(|source| Error::WorkspaceIo {
                    path: workspace.clone(),
                    source,
                })?;
                if let Some(progress) = &progress {
                    progress.finish_and_clear();
                }
                let exercise = exercism_cli::Client::new().download(&workspace, &track, &id)?;
                let path = exercise
                    .solution_files
                    .first()
                    .map(|path| exercise.directory.join(path))
                    .ok_or(Error::CapabilityUnavailable {
                        platform: "Exercism",
                        capability: "a declared solution file for this exercise",
                    })?;
                Ok(Content::Solution(domain::SolutionFile {
                    platform: "exercism".into(),
                    title: id.as_ref().into(),
                    language: track.as_ref().into(),
                    id,
                    path,
                }))
            }
        }
        (
            cli::Platform::Exercism,
            cli::Command::Problem {
                command: cli::ProblemCommand::Test(target),
            },
        ) => {
            if matches!(cli.format, cli::Format::Json) {
                Err(Error::CapabilityUnavailable {
                    platform: "Exercism",
                    capability: "JSON output from the official CLI test runner",
                })
            } else {
                let exercise = exercism_cli::discover_from_solution(&target.path)?;
                if let Some(progress) = &progress {
                    progress.finish_and_clear();
                }
                exercism_cli::Client::new().test(&exercise)?;
                Ok(Content::ToolAction(domain::ToolAction {
                    platform: "Exercism".into(),
                    action: "Tests complete".into(),
                    detail: exercise.directory.display().to_string().into(),
                }))
            }
        }
        (
            cli::Platform::Exercism,
            cli::Command::Problem {
                command: cli::ProblemCommand::Submit(target),
            },
        ) => {
            if matches!(cli.format, cli::Format::Json) {
                Err(Error::CapabilityUnavailable {
                    platform: "Exercism",
                    capability: "JSON output from the official CLI submit command",
                })
            } else {
                let exercise = exercism_cli::discover_from_solution(&target.path)?;
                if let Some(progress) = &progress {
                    progress.finish_and_clear();
                }
                exercism_cli::Client::new().submit(&exercise)?;
                Ok(Content::ToolAction(domain::ToolAction {
                    platform: "Exercism".into(),
                    action: "Submission sent".into(),
                    detail: "The official Exercism CLI accepted the solution files.".into(),
                }))
            }
        }
        (
            cli::Platform::HackerEarth,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { page, topic, .. },
            },
        ) => catalog::list(
            domain::CatalogPlatform::HackerEarth,
            page,
            None,
            topic,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
        )
        .await
        .map(Content::CatalogList),
        (
            cli::Platform::HackerEarth,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => match catalog::problem_id(domain::CatalogPlatform::HackerEarth, problem) {
            Ok(id) => catalog::show(domain::CatalogPlatform::HackerEarth, id, |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::CatalogProblem),
            Err(error) => Err(error),
        },
        (
            cli::Platform::CodeChef,
            cli::Command::Problem {
                command: cli::ProblemCommand::List { .. },
            },
        ) => Err(Error::CapabilityUnavailable {
            platform: "CodeChef",
            capability: "a dependable public problem catalogue API",
        }),
        (
            cli::Platform::CodeChef,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => {
            let _ = problem;
            Err(Error::CapabilityUnavailable {
                platform: "CodeChef",
                capability: "dependable public problem statements through a callable API",
            })
        }
        (
            cli::Platform::HackerRank,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => match catalog::problem_id(domain::CatalogPlatform::HackerRank, problem) {
            Ok(id) => catalog::show(domain::CatalogPlatform::HackerRank, id, |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::CatalogProblem),
            Err(error) => Err(error),
        },
        (
            cli::Platform::HackerRank,
            cli::Command::Problem {
                command:
                    cli::ProblemCommand::Pick {
                        problem, lang, dir, ..
                    },
            },
        ) => solve::pick_hackerrank(
            problem,
            lang,
            dir,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
            || {
                if let Some(progress) = &progress {
                    progress.finish_and_clear();
                }
            },
        )
        .await
        .map(Content::Solution),
        (cli::Platform::Codeforces, cli::Command::Stats { user: Some(user) }) => {
            codeforces_support::stats(user.as_ref(), |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::CodeforcesStats)
        }
        (cli::Platform::Codeforces, cli::Command::Stats { user: None }) => {
            Err(Error::AccountNameRequired {
                platform: "Codeforces",
                capability: "public profile stats",
            })
        }
        (cli::Platform::CodeChef, cli::Command::Stats { user: Some(user) }) => {
            public_stats::codechef(user.as_ref(), |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::PublicStats)
        }
        (cli::Platform::CodeChef, cli::Command::Stats { user: None }) => {
            Err(Error::AccountNameRequired {
                platform: "CodeChef",
                capability: "public profile stats",
            })
        }
        (cli::Platform::HackerRank, cli::Command::Stats { user: Some(user) }) => {
            public_stats::hackerrank(user.as_ref(), |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await
            .map(Content::PublicStats)
        }
        (cli::Platform::HackerRank, cli::Command::Stats { user: None }) => {
            Err(Error::AccountNameRequired {
                platform: "HackerRank",
                capability: "public profile stats",
            })
        }
        (
            cli::Platform::Codeforces,
            cli::Command::Contest {
                command: cli::ContestCommand::List,
            },
        ) => codeforces_support::contests(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CodeforcesContests),
        (
            cli::Platform::Codeforces,
            cli::Command::Contest {
                command: cli::ContestCommand::Show { contest },
            },
        ) => {
            let id = contest
                .as_ref()
                .parse::<u32>()
                .ok()
                .filter(|id| *id > 0)
                .ok_or(Error::InvalidContestSlug)?;
            codeforces_support::contest(id, |bytes, total| {
                update_progress(&progress, &bar_style, bytes, total)
            })
            .await?
            .map(Content::CodeforcesContest)
            .ok_or_else(|| Error::ContestNotFound {
                platform: "Codeforces",
                id: id.to_string().into(),
            })
        }
        (
            cli::Platform::Codeforces,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::List { problem: None },
            },
        ) => codeforces_support::discussions(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CommunityDiscussions),
        (
            cli::Platform::Codeforces,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Show { discussion },
            },
        ) => codeforces_support::discussion(discussion, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CommunityDiscussion),
        (
            cli::Platform::Codeforces,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::List { problem: Some(_) },
            },
        ) => Err(Error::CapabilityUnavailable {
            platform: "Codeforces",
            capability: "problem-filtered blog discussions in its public API",
        }),
        (
            cli::Platform::CodeChef,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::List { problem: None },
            },
        ) => community::codechef_list(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CommunityDiscussions),
        (
            cli::Platform::CodeChef,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Show { discussion },
            },
        ) => community::codechef_show(discussion, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::CommunityDiscussion),
        (
            cli::Platform::Codeforces,
            cli::Command::Auth {
                command: cli::AuthCommand::Login,
            },
        ) => platform_auth::codeforces_login(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::ToolAction),
        (
            cli::Platform::Codeforces,
            cli::Command::Auth {
                command: cli::AuthCommand::Logout,
            },
        ) => platform_auth::codeforces_logout().map(Content::ToolAction),
        (
            cli::Platform::Codeforces,
            cli::Command::Auth {
                command: cli::AuthCommand::Status,
            },
        ) => platform_auth::codeforces_status(|bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::ToolAction),
        (
            cli::Platform::Exercism,
            cli::Command::Auth {
                command: cli::AuthCommand::Login,
            },
        ) => platform_auth::exercism_login().map(Content::ToolAction),
        (
            cli::Platform::Exercism,
            cli::Command::Auth {
                command: cli::AuthCommand::Status,
            },
        ) => platform_auth::exercism_status().map(Content::ToolAction),
        (
            cli::Platform::Exercism,
            cli::Command::Auth {
                command: cli::AuthCommand::Logout,
            },
        ) => Err(Error::CapabilityUnavailable {
            platform: "Exercism",
            capability: "a documented logout operation in the official CLI",
        }),
        (
            cli::Platform::LeetCode,
            cli::Command::Auth {
                command: cli::AuthCommand::Login,
            },
        ) => Err(Error::CapabilityUnavailable {
            platform: "LeetCode",
            capability: "a documented terminal-native authentication flow",
        }),
        (
            cli::Platform::LeetCode,
            cli::Command::Auth {
                command: cli::AuthCommand::Logout,
            },
        ) => auth::logout().map(Content::AuthLogout),
        (
            cli::Platform::LeetCode,
            cli::Command::Auth {
                command: cli::AuthCommand::Status,
            },
        ) => auth::status().await.map(Content::Auth),
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command: cli::ContestCommand::Join { contest },
            },
        ) => {
            let _ = contest;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified contest registration mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command: cli::ContestCommand::Leave { contest },
            },
        ) => {
            let _ = contest;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified contest withdrawal mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command: cli::ContestCommand::List,
            },
        ) => contests::list(|bytes, total| update_progress(&progress, &bar_style, bytes, total))
            .await
            .map(Content::Contests),
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command: cli::ContestCommand::Show { contest },
            },
        ) => contests::show(contest, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Contest),
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command: cli::ContestCommand::Status { contest },
            },
        ) => contests::status(contest, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::ContestRegistration),
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Create { problem },
            },
        ) => {
            let _ = problem;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified discussion creation mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Delete { discussion },
            },
        ) => {
            let _ = discussion;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified discussion deletion mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Edit { discussion },
            },
        ) => {
            let _ = discussion;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified discussion editing mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::List { problem },
            },
        ) => discussions::list(problem, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Discussions),
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Show { discussion },
            },
        ) => discussions::show(discussion, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Discussion),
        (
            cli::Platform::LeetCode,
            cli::Command::Discussion {
                command: cli::DiscussionCommand::Reply { discussion },
            },
        ) => {
            let _ = discussion;
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "a verified discussion reply mutation",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command: cli::ProblemCommand::Daily,
            },
        ) => problems::daily(|bytes, total| update_progress(&progress, &bar_style, bytes, total))
            .await
            .map(Content::Problem),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command:
                    cli::ProblemCommand::List {
                        difficulty, tag, ..
                    },
            },
        ) => problems::list(difficulty.map(Into::into), tag, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::List),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command: cli::ProblemCommand::Show { problem, .. },
            },
        ) => problems::show(problem, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Problem),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command: cli::ProblemCommand::Search { query },
            },
        ) => problems::search(query, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Search),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command:
                    cli::ProblemCommand::Pick {
                        problem, lang, dir, ..
                    },
            },
        ) => solve::pick(
            problem,
            lang,
            dir,
            |bytes, total| update_progress(&progress, &bar_style, bytes, total),
            || {
                if let Some(progress) = &progress {
                    progress.finish_and_clear();
                }
            },
        )
        .await
        .map(Content::Solution),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command: cli::ProblemCommand::Test(target),
            },
        ) => solve::test(target.path, |bytes, total| {
            update_progress(&progress, &bar_style, bytes, total)
        })
        .await
        .map(Content::Test),
        (
            cli::Platform::LeetCode,
            cli::Command::Problem {
                command: cli::ProblemCommand::Submit(target),
            },
        ) => solve::submit(target.path).await.map(Content::Submission),
        (cli::Platform::LeetCode, cli::Command::Stats { user: None }) => {
            stats::load().await.map(Content::Stats)
        }
        (cli::Platform::LeetCode, cli::Command::Stats { user: Some(_) }) => {
            Err(Error::CapabilityUnavailable {
                platform: "LeetCode",
                capability: "public stats for another account",
            })
        }
        (
            cli::Platform::LeetCode,
            cli::Command::Contest {
                command:
                    cli::ContestCommand::Create
                    | cli::ContestCommand::Edit { .. }
                    | cli::ContestCommand::Delete { .. },
            },
        ) => Err(Error::CapabilityUnavailable {
            platform: "LeetCode",
            capability: "self-service contest management",
        }),
        (platform, command) => Err(Error::UnsupportedPlatformCommand {
            platform: platform.label(),
            command: command.path(),
        }),
    };
    if let Some(progress) = &progress {
        progress.finish_and_clear();
    }
    let mut content = result?;
    let selection = if terminal
        && std::io::stdin().is_terminal()
        && presentation.interactive
        && reader::supported()
    {
        match &content {
            Content::CatalogList(list) if !list.results.is_empty() => {
                let (heading, detail) = output::catalog_list_header(list);
                Some(
                    reader::browse_catalog(
                        &heading,
                        &detail,
                        list.platform,
                        &list.results,
                        presentation,
                    )?
                    .map(|id| reader::Target::Catalog(list.platform, id)),
                )
            }
            Content::CodeforcesContests(list) if !list.contests.is_empty() => Some(
                reader::browse_codeforces_contests(
                    &format!(
                        "POSSUM//Contests  {} Codeforces contests",
                        list.contests.len()
                    ),
                    "Select a contest to inspect it",
                    &list.contests,
                    presentation,
                )?
                .map(reader::Target::CodeforcesContest),
            ),
            Content::CodeforcesStats(stats) if !stats.recent_submissions.is_empty() => Some(
                reader::browse_codeforces_submissions(
                    &format!("POSSUM//Stats  {}", stats.profile.handle),
                    &format!(
                        "Rating {} · {} solved in the latest {} submissions",
                        stats
                            .profile
                            .rating
                            .map_or_else(|| "unrated".to_owned(), |rating| rating.to_string()),
                        stats.recent_solved_count,
                        stats.recent_submissions.len()
                    ),
                    &stats.recent_submissions,
                    presentation,
                )?
                .map(|id| reader::Target::Catalog(domain::CatalogPlatform::Codeforces, id)),
            ),
            Content::CommunityDiscussions(list) if !list.discussions.is_empty() => Some(
                reader::browse_community_discussions(
                    &format!(
                        "POSSUM//Discuss  {} {} topics",
                        list.discussions.len(),
                        list.platform
                    ),
                    "Select a discussion to read it",
                    &list.discussions,
                    presentation,
                )?
                .map(|id| {
                    let platform = if list.platform.as_ref() == "Codeforces" {
                        cli::Platform::Codeforces
                    } else {
                        cli::Platform::CodeChef
                    };
                    reader::Target::CommunityDiscussion(platform, id)
                }),
            ),
            Content::List(list) if !list.results.is_empty() => {
                let (heading, detail) = output::list_header(list);
                Some(
                    reader::browse_problems(&heading, &detail, &list.results, presentation)?
                        .map(reader::Target::Problem),
                )
            }
            Content::Search(search) if !search.results.is_empty() => {
                let (heading, detail) = output::search_header(search);
                Some(
                    reader::browse_problems(&heading, &detail, &search.results, presentation)?
                        .map(reader::Target::Problem),
                )
            }
            Content::Contests(list) if !list.contests.is_empty() => {
                let noun = if list.contests.len() == 1 {
                    "contest"
                } else {
                    "contests"
                };
                let heading = format!("POSSUM//Contests  {} upcoming {noun}", list.contests.len());
                Some(
                    reader::browse_contests(
                        &heading,
                        "Select a contest to open its schedule",
                        &list.contests,
                        presentation,
                    )?
                    .map(reader::Target::Contest),
                )
            }
            Content::Discussions(list) if !list.discussions.is_empty() => {
                let shown = list.discussions.len();
                let heading = match (list.problem.as_ref(), list.total) {
                    (Some(_), Some(total)) => {
                        format!("POSSUM//Discuss  {shown} of {total} posts")
                    }
                    (Some(_), None) => format!("POSSUM//Discuss  {shown} posts"),
                    (None, _) => format!("POSSUM//Discuss  {shown} trending posts"),
                };
                let detail = list.problem.as_ref().map_or_else(
                    || "Select a discussion to read it".to_owned(),
                    |problem| format!("Problem: leetcode/{}", problem.as_ref()),
                );
                Some(
                    reader::browse_discussions(&heading, &detail, &list.discussions, presentation)?
                        .map(reader::Target::Discussion),
                )
            }
            Content::Stats(stats) if !stats.recent_submissions.is_empty() => {
                let heading = format!("POSSUM//Stats  {}", stats.username);
                let detail = format!(
                    "{} solved · {} accepted · {} attempts · recent submissions",
                    stats.solved.all, stats.accepted_submissions.all, stats.submissions.all
                );
                Some(
                    reader::browse_submissions(
                        &heading,
                        &detail,
                        &stats.recent_submissions,
                        presentation,
                    )?
                    .map(reader::Target::Problem),
                )
            }
            _ => None,
        }
    } else {
        None
    };
    match selection {
        Some(Some(target)) => {
            if let Some(progress) = &progress {
                progress.set_style(spinner_style.clone());
                progress.reset();
                let activity = match &target {
                    reader::Target::Problem(id) => format!("Loading {}", id.as_ref()),
                    reader::Target::Catalog(platform, id) => {
                        format!("Loading {} on {}", id.as_ref(), platform.label())
                    }
                    reader::Target::Contest(id) => {
                        format!("Loading contest {}", id.as_ref())
                    }
                    reader::Target::CodeforcesContest(id) => {
                        format!("Loading Codeforces contest {id}")
                    }
                    reader::Target::Discussion(id) => {
                        format!("Loading discussion {}", id.get())
                    }
                    reader::Target::CommunityDiscussion(_, id) => {
                        format!("Loading discussion {}", id.get())
                    }
                };
                progress.set_message(activity);
                if presentation.motion {
                    progress.enable_steady_tick(Duration::from_millis(260));
                } else {
                    progress.tick();
                }
            }
            let opened = match target {
                reader::Target::Problem(id) => {
                    problems::show(domain::ProblemSelector::Slug(id), |bytes, total| {
                        update_progress(&progress, &bar_style, bytes, total)
                    })
                    .await
                    .map(Content::Problem)
                }
                reader::Target::Catalog(platform, id) => {
                    let cached = match &content {
                        Content::CatalogList(list)
                            if platform == domain::CatalogPlatform::Codeforces =>
                        {
                            list.results
                                .iter()
                                .find(|problem| problem.id.as_ref() == id.as_ref())
                                .map(|problem| catalog::metadata_problem(platform, problem))
                        }
                        _ => None,
                    };
                    if let Some(problem) = cached {
                        Ok(Content::CatalogProblem(problem))
                    } else {
                        catalog::show(platform, id, |bytes, total| {
                            update_progress(&progress, &bar_style, bytes, total)
                        })
                        .await
                        .map(Content::CatalogProblem)
                    }
                }
                reader::Target::Contest(id) => contests::show(id, |bytes, total| {
                    update_progress(&progress, &bar_style, bytes, total)
                })
                .await
                .map(Content::Contest),
                reader::Target::CodeforcesContest(id) => {
                    codeforces_support::contest(id, |bytes, total| {
                        update_progress(&progress, &bar_style, bytes, total)
                    })
                    .await?
                    .map(Content::CodeforcesContest)
                    .ok_or_else(|| Error::ContestNotFound {
                        platform: "Codeforces",
                        id: id.to_string().into(),
                    })
                }
                reader::Target::Discussion(id) => discussions::show(id, |bytes, total| {
                    update_progress(&progress, &bar_style, bytes, total)
                })
                .await
                .map(Content::Discussion),
                reader::Target::CommunityDiscussion(platform, id) => match platform {
                    cli::Platform::Codeforces => {
                        codeforces_support::discussion(id, |bytes, total| {
                            update_progress(&progress, &bar_style, bytes, total)
                        })
                        .await
                        .map(Content::CommunityDiscussion)
                    }
                    cli::Platform::CodeChef => community::codechef_show(id, |bytes, total| {
                        update_progress(&progress, &bar_style, bytes, total)
                    })
                    .await
                    .map(Content::CommunityDiscussion),
                    _ => unreachable!("community reader only supports native community clients"),
                },
            };
            if let Some(progress) = &progress {
                progress.finish_and_clear();
            }
            content = opened?;
        }
        Some(None) => return Ok(()),
        None => {}
    }
    let color = if presentation.color || presentation.large_heading {
        anstream::ColorChoice::AlwaysAnsi
    } else {
        anstream::ColorChoice::Never
    };
    let stdout = anstream::AutoStream::new(std::io::stdout(), color);
    let mut stdout = stdout.lock();
    match content {
        Content::AuthLogout(logout) => {
            output::auth_logout(&mut stdout, cli.format, &logout, presentation)
        }
        Content::Auth(status) => {
            output::auth_status(&mut stdout, cli.format, &status, presentation)
        }
        Content::CatalogList(list) => {
            output::catalog_list(&mut stdout, cli.format, &list, presentation)
        }
        Content::CatalogProblem(problem) => {
            output::catalog_problem(&mut stdout, cli.format, &problem, presentation)
        }
        Content::CodeforcesContest(contest) => {
            output::codeforces_contest(&mut stdout, cli.format, &contest, presentation)
        }
        Content::CodeforcesContests(contests) => {
            output::codeforces_contests(&mut stdout, cli.format, &contests, presentation)
        }
        Content::CodeforcesStats(stats) => {
            output::codeforces_stats(&mut stdout, cli.format, &stats, presentation)
        }
        Content::CommunityDiscussion(discussion) => {
            output::community_discussion(&mut stdout, cli.format, &discussion, presentation)
        }
        Content::CommunityDiscussions(discussions) => {
            output::community_discussions(&mut stdout, cli.format, &discussions, presentation)
        }
        Content::Contest(contest) => {
            output::contest(&mut stdout, cli.format, &contest, presentation)
        }
        Content::ContestRegistration(registration) => {
            output::contest_registration(&mut stdout, cli.format, &registration, presentation)
        }
        Content::Contests(contests) => {
            output::contests(&mut stdout, cli.format, &contests, presentation)
        }
        Content::Discussion(discussion) => {
            output::discussion(&mut stdout, cli.format, &discussion, presentation)
        }
        Content::Discussions(discussions) => {
            output::discussions(&mut stdout, cli.format, &discussions, presentation)
        }
        Content::List(list) => output::list(&mut stdout, cli.format, &list, presentation),
        Content::Problem(problem) => {
            output::problem(&mut stdout, cli.format, &problem, presentation)
        }
        Content::PublicStats(stats) => {
            output::public_account_stats(&mut stdout, cli.format, &stats, presentation)
        }
        Content::Search(search) => output::search(&mut stdout, cli.format, &search, presentation),
        Content::Solution(solution) => {
            output::solution_file(&mut stdout, cli.format, &solution, presentation)
        }
        Content::Stats(stats) => output::stats(&mut stdout, cli.format, &stats, presentation),
        Content::Submission(submission) => {
            output::submission(&mut stdout, cli.format, &submission, presentation)
        }
        Content::Test(test) => output::test(&mut stdout, cli.format, &test, presentation),
        Content::ToolAction(action) => {
            output::tool_action(&mut stdout, cli.format, &action, presentation)
        }
    }?;
    if cli.sound && terminal && std::io::stderr().is_terminal() {
        std::io::stderr().lock().write_all(b"\x07")?;
    }
    Ok(())
}

fn update_progress(
    progress: &Option<ProgressBar>,
    bar_style: &ProgressStyle,
    bytes: usize,
    total: Option<u64>,
) {
    if let Some(progress) = progress {
        if let Some(total) = total.filter(|total| *total > 0)
            && progress.length().is_none()
        {
            progress.disable_steady_tick();
            progress.set_style(bar_style.clone());
            progress.set_length(total);
        }
        progress.set_position(bytes as u64);
    }
}

#[cfg(test)]
mod tests;
