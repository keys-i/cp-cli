use std::io::{self, Write};

use clap::Parser;
use unicode_width::UnicodeWidthStr;

use crate::{
    cli::{
        AuthCommand, Cli, Color, Command, ContestCommand, DiscussionCommand, Format, HeadingSize,
        ListDifficulty, Platform, ProblemCommand, Theme,
    },
    codeforces_support::{CodeforcesProfile, CodeforcesStats, CodeforcesSubmission},
    domain::{
        AccountStats, AuthLogout, AuthSource, AuthStatus, CatalogPlatform, CatalogProblem,
        CatalogProblemList, CatalogProblemSummary, Contest, ContestList, ContestRegistration,
        Difficulty, DifficultyCounts, Discussion, DiscussionList, DiscussionSummary, LanguageSlug,
        Problem, ProblemId, ProblemList, ProblemQuery, ProblemSearch, ProblemSelector,
        ProblemSummary, ProblemTag, RecentSubmission, SolutionFile, StatementFormat,
        SubmissionResult, TestResult as JudgeTest,
    },
    output::{self, Presentation},
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn presentation(color: bool, large: bool, interactive: bool, columns: u16) -> Presentation {
    Presentation {
        columns,
        color,
        large_heading: large,
        interactive,
        motion: true,
        palette: Theme::Possum.palette([18; 3]),
    }
}

fn render(problem: &Problem, format: Format, presentation: Presentation) -> TestResult<String> {
    let mut bytes = Vec::new();
    output::problem(&mut bytes, format, problem, presentation)?;
    Ok(String::from_utf8(bytes)?)
}

fn render_search(
    search: &ProblemSearch,
    format: Format,
    presentation: Presentation,
) -> TestResult<String> {
    let mut bytes = Vec::new();
    output::search(&mut bytes, format, search, presentation)?;
    Ok(String::from_utf8(bytes)?)
}

fn render_list(
    list: &ProblemList,
    format: Format,
    presentation: Presentation,
) -> TestResult<String> {
    let mut bytes = Vec::new();
    output::list(&mut bytes, format, list, presentation)?;
    Ok(String::from_utf8(bytes)?)
}

fn assert_contains(text: &str, values: &[&str]) {
    for value in values {
        assert!(text.contains(value), "missing {value:?} in:\n{text}");
    }
}

fn assert_excludes(text: &str, values: &[&str]) {
    for value in values {
        assert!(!text.contains(value), "found {value:?} in:\n{text}");
    }
}

#[tokio::test]
async fn cli_rendering_and_terminal_contract() -> TestResult {
    let cli = Cli::try_parse_from([
        "cp-cli",
        "--platform",
        "leetcode",
        "--sound",
        "--no-animation",
        "problem",
        "show",
        "two-sum",
        "--format",
        "json",
    ])?;
    let Command::Problem {
        command: ProblemCommand::Show { problem, track },
    } = cli.command
    else {
        return Err("expected show command".into());
    };
    let ProblemSelector::Slug(id) = problem else {
        return Err("expected show slug".into());
    };
    assert_eq!(id.as_ref(), "two-sum");
    assert!(track.is_none());
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "problem", "show", "1"])?.command,
        Command::Problem {
            command: ProblemCommand::Show {
                problem: ProblemSelector::Number(1),
                track: None,
            }
        }
    ));
    assert!(matches!(cli.format, Format::Json));
    assert!(matches!(cli.theme, Theme::Possum));
    assert!(cli.sound);
    assert!(cli.no_animation);
    let search_cli =
        Cli::try_parse_from(["cp-cli", "problem", "search", "two sum", "--format", "json"])?;
    let Command::Problem {
        command: ProblemCommand::Search { query },
    } = search_cli.command
    else {
        return Err("expected search command".into());
    };
    assert_eq!(query.as_ref(), "two sum");
    assert!(matches!(search_cli.format, Format::Json));
    let daily_cli = Cli::try_parse_from(["cp-cli", "problem", "daily", "--format", "json"])?;
    assert!(matches!(
        daily_cli.command,
        Command::Problem {
            command: ProblemCommand::Daily
        }
    ));
    assert!(matches!(daily_cli.format, Format::Json));
    let list_cli = Cli::try_parse_from([
        "cp-cli",
        "problem",
        "list",
        "--difficulty",
        "medium",
        "--tag",
        "dynamic-programming",
        "--format",
        "json",
    ])?;
    let Command::Problem {
        command:
            ProblemCommand::List {
                difficulty,
                tag,
                track,
                page,
                ..
            },
    } = list_cli.command
    else {
        return Err("expected list command".into());
    };
    assert!(matches!(difficulty, Some(ListDifficulty::Medium)));
    assert_eq!(
        tag.ok_or("missing list tag")?.as_ref(),
        "dynamic-programming"
    );
    assert!(track.is_none());
    assert!(page.is_none());
    assert!(matches!(list_cli.format, Format::Json));
    let exercism_cli = Cli::try_parse_from([
        "cp-cli",
        "--platform",
        "exercism",
        "problem",
        "show",
        "two-fer",
        "--track",
        "rust",
    ])?;
    assert!(matches!(exercism_cli.platform, Platform::Exercism));
    assert!(matches!(
        exercism_cli.command,
        Command::Problem {
            command: ProblemCommand::Show {
                problem: ProblemSelector::Slug(_),
                track: Some(_),
            }
        }
    ));
    let project_euler_cli = Cli::try_parse_from([
        "cp-cli",
        "--platform",
        "project-euler",
        "problem",
        "list",
        "--page",
        "2",
    ])?;
    assert!(matches!(project_euler_cli.platform, Platform::ProjectEuler));
    assert!(matches!(
        project_euler_cli.command,
        Command::Problem {
            command: ProblemCommand::List { page: Some(2), .. }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from([
            "cp-cli",
            "--platform",
            "project-euler",
            "problem",
            "list",
            "--recent",
        ])?
        .command,
        Command::Problem {
            command: ProblemCommand::List { recent: true, .. }
        }
    ));
    let hackerearth_cli = Cli::try_parse_from([
        "cp-cli",
        "--platform",
        "hackerearth",
        "problem",
        "list",
        "--topic",
        "algorithms/searching/linear-search",
    ])?;
    assert!(matches!(
        hackerearth_cli.command,
        Command::Problem {
            command: ProblemCommand::List { topic: Some(_), .. }
        }
    ));
    assert!(
        Cli::try_parse_from([
            "cp-cli",
            "--platform",
            "hackerearth",
            "problem",
            "list",
            "--topic",
            "../private",
        ])
        .is_err()
    );
    for (platform, id, label) in [
        ("hackerearth", "make-an-array-85abd7ad", "HackerEarth"),
        ("codechef", "FLOW001", "CodeChef"),
    ] {
        let cli = Cli::try_parse_from(["cp-cli", "--platform", platform, "problem", "show", id])?;
        assert_eq!(cli.platform.label(), label);
        assert!(matches!(
            cli.command,
            Command::Problem {
                command: ProblemCommand::Show { .. }
            }
        ));
    }
    let pick_cli = Cli::try_parse_from([
        "cp-cli",
        "problem",
        "pick",
        "1",
        "--lang",
        "rust",
        "--dir",
        "solutions",
    ])?;
    let Command::Problem {
        command:
            ProblemCommand::Pick {
                problem,
                lang,
                dir,
                track,
            },
    } = pick_cli.command
    else {
        return Err("expected pick command".into());
    };
    assert!(matches!(problem, ProblemSelector::Number(1)));
    assert_eq!(lang.ok_or("missing language")?.as_ref(), "rust");
    assert_eq!(
        dir.ok_or("missing directory")?,
        std::path::PathBuf::from("solutions")
    );
    assert!(track.is_none());
    for (name, login, logout) in [
        ("login", true, false),
        ("logout", false, true),
        ("status", false, false),
    ] {
        let auth = Cli::try_parse_from(["cp-cli", "auth", name])?;
        assert_eq!(
            matches!(
                &auth.command,
                Command::Auth {
                    command: AuthCommand::Login
                }
            ),
            login
        );
        assert_eq!(
            matches!(
                &auth.command,
                Command::Auth {
                    command: AuthCommand::Logout
                }
            ),
            logout
        );
        assert!(
            login
                || logout
                || matches!(
                    &auth.command,
                    Command::Auth {
                        command: AuthCommand::Status
                    }
                )
        );
    }
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "stats"])?.command,
        Command::Stats { user: None }
    ));
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "--platform", "codeforces", "stats", "tourist"])?.command,
        Command::Stats { user: Some(_) }
    ));
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "contest", "list"])?.command,
        Command::Contest {
            command: ContestCommand::List
        }
    ));
    let contest_cli = Cli::try_parse_from(["cp-cli", "contest", "show", "weekly-contest-500"])?;
    let Command::Contest {
        command: ContestCommand::Show { contest },
    } = contest_cli.command
    else {
        return Err("expected contest show command".into());
    };
    assert_eq!(contest.as_ref(), "weekly-contest-500");
    for action in ["join", "leave", "status"] {
        let contest = Cli::try_parse_from(["cp-cli", "contest", action, "weekly-contest-500"])?;
        assert!(matches!(
            contest.command,
            Command::Contest {
                command: ContestCommand::Join { .. }
                    | ContestCommand::Leave { .. }
                    | ContestCommand::Status { .. }
            }
        ));
    }
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "contest", "create"])?.command,
        Command::Contest {
            command: ContestCommand::Create
        }
    ));
    for action in ["edit", "delete"] {
        let contest = Cli::try_parse_from(["cp-cli", "contest", action, "START256"])?;
        assert!(matches!(
            contest.command,
            Command::Contest {
                command: ContestCommand::Edit { .. } | ContestCommand::Delete { .. }
            }
        ));
    }
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "discussion", "list"])?.command,
        Command::Discussion {
            command: DiscussionCommand::List { problem: None }
        }
    ));
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "discussion", "list", "1"])?.command,
        Command::Discussion {
            command: DiscussionCommand::List {
                problem: Some(ProblemSelector::Number(1))
            }
        }
    ));
    let discussion_cli = Cli::try_parse_from(["cp-cli", "discussion", "show", "1124385"])?;
    let Command::Discussion {
        command: DiscussionCommand::Show { discussion },
    } = discussion_cli.command
    else {
        return Err("expected discussion show command".into());
    };
    assert_eq!(discussion.get(), 1_124_385);
    assert!(matches!(
        Cli::try_parse_from(["cp-cli", "discussion", "create", "two-sum"])?.command,
        Command::Discussion {
            command: DiscussionCommand::Create {
                problem: Some(ProblemSelector::Slug(_))
            }
        }
    ));
    for action in ["delete", "edit", "reply"] {
        let discussion = Cli::try_parse_from(["cp-cli", "discussion", action, "1124385"])?;
        assert!(matches!(
            discussion.command,
            Command::Discussion {
                command: DiscussionCommand::Delete { .. }
                    | DiscussionCommand::Edit { .. }
                    | DiscussionCommand::Reply { .. }
            }
        ));
    }
    for (name, command) in [("test", false), ("submit", true)] {
        let solve = Cli::try_parse_from(["cp-cli", "problem", name, "solutions/two-sum.rs"])?;
        assert!(
            matches!(
                solve.command,
                Command::Problem {
                    command: ProblemCommand::Submit(_)
                }
            ) == command
        );
    }
    let animated = crate::possum::loading_frames(80, true, true);
    let still = crate::possum::loading_frames(80, true, false);
    let blink =
        crate::possum::render(crate::possum::Pose::Blink, 30, true).ok_or("missing blink frame")?;
    let scratch = crate::possum::render(crate::possum::Pose::Scratch, 30, true)
        .ok_or("missing scratch frame")?;
    for (frames, length, moving) in [(&animated, 10, true), (&still, 2, false)] {
        assert_eq!(frames.len(), length);
        assert_eq!(frames.contains(&blink), moving);
        assert_eq!(frames.contains(&scratch), moving);
    }
    assert_eq!(still[0], still[1]);
    let fallback = crate::possum::loading_frames(20, false, true);
    assert_eq!(fallback.len(), 4);
    assert_contains(&fallback.join("\n"), &["P0SSUM//", "POS_SUM//", "POSSUM//"]);
    assert_excludes(&fallback.join("\n"), &["◖•ᴥ•◗⟆"]);

    for (option, values) in [
        (
            "--theme",
            &[
                "possum",
                "arcade",
                "moonlight",
                "phosphor",
                "amber",
                "dark",
                "light",
            ][..],
        ),
        ("--format", &["text", "json"]),
        ("--color", &["auto", "always", "never"]),
        ("--heading-size", &["auto", "normal", "large"]),
        ("--background", &["auto", "dark", "light"]),
        ("--platform", &["leetcode"]),
    ] {
        for value in values {
            assert!(
                Cli::try_parse_from(["cp-cli", option, value, "problem", "show", "two-sum"])
                    .is_ok(),
                "rejected {option} {value}"
            );
        }
    }

    for args in [
        &["cp-cli", "--help"][..],
        &["cp-cli", "problem", "--help"],
        &["cp-cli", "problem", "show", "--help"],
        &["cp-cli", "problem", "search", "--help"],
        &["cp-cli", "problem", "daily", "--help"],
        &["cp-cli", "problem", "list", "--help"],
        &["cp-cli", "problem", "pick", "--help"],
        &["cp-cli", "problem", "test", "--help"],
        &["cp-cli", "problem", "submit", "--help"],
        &["cp-cli", "auth", "login", "--help"],
        &["cp-cli", "auth", "logout", "--help"],
        &["cp-cli", "auth", "status", "--help"],
    ] {
        assert!(matches!(
            Cli::try_parse_from(args),
            Err(error) if error.kind() == clap::error::ErrorKind::DisplayHelp
        ));
    }
    for args in [
        &["cp-cli", "help"][..],
        &["cp-cli", "problem", "help"],
        &["cp-cli", "problem", "show"],
        &["cp-cli", "problem", "search"],
        &["cp-cli", "problem", "daily", "tomorrow"],
        &["cp-cli", "problem", "list", "extra"],
        &["cp-cli", "problem", "list", "--difficulty", "extreme"],
        &["cp-cli", "problem", "list", "--tag", "Dynamic Programming"],
        &["cp-cli", "problem", "pick", "0"],
        &["cp-cli", "problem", "pick", "two-sum", "--lang", "Rust"],
        &["cp-cli", "problem", "test"],
        &["cp-cli", "problem", "submit"],
        &["cp-cli", "solve", "test", "solutions/two-sum.rs"],
        &["cp-cli", "problem", "show", "two-sum", "--format", "yaml"],
        &["cp-cli", "--theme", "crt", "problem", "show", "two-sum"],
    ] {
        assert!(Cli::try_parse_from(args).is_err(), "accepted {args:?}");
    }

    for (slug, valid) in [
        ("a", true),
        ("two-sum", true),
        ("Two_Sum-2", true),
        ("", false),
        ("../two-sum", false),
        ("two sum", false),
        ("two/sum", false),
        ("two\nsum", false),
        ("two\0sum", false),
        ("é", false),
        ("🦡", false),
        ("https://leetcode.com/problems/two-sum/", false),
    ] {
        assert_eq!(slug.parse::<ProblemId>().is_ok(), valid, "slug={slug:?}");
        assert_eq!(
            Cli::try_parse_from(["cp-cli", "problem", "show", slug]).is_ok(),
            valid,
            "slug={slug:?}"
        );
    }
    for (length, valid) in [(127, true), (128, true), (129, false), (1024, false)] {
        assert_eq!("a".repeat(length).parse::<ProblemId>().is_ok(), valid);
    }
    for (query, expected) in [
        ("two sum", "two sum"),
        ("  graph  ", "graph"),
        ("1", "1"),
        ("C++", "C++"),
    ] {
        let query = query.parse::<ProblemQuery>()?;
        assert_eq!(query.as_ref(), expected);
    }
    for query in ["", " ", "two\nsum", "two\tsum", "\u{1b}[31m", "é"] {
        assert!(query.parse::<ProblemQuery>().is_err(), "query={query:?}");
    }
    assert!("a".repeat(101).parse::<ProblemQuery>().is_err());
    for (tag, valid) in [
        ("graph", true),
        ("dynamic-programming", true),
        ("array2", true),
        ("", false),
        ("-graph", false),
        ("graph-", false),
        ("Dynamic", false),
        ("two words", false),
        ("graph/tree", false),
    ] {
        assert_eq!(tag.parse::<ProblemTag>().is_ok(), valid, "tag={tag:?}");
    }
    assert!("a".repeat(65).parse::<ProblemTag>().is_err());
    for (value, valid) in [
        ("1", true),
        ("two-sum", true),
        ("0", false),
        ("two sum", false),
        ("../two-sum", false),
    ] {
        assert_eq!(
            value.parse::<ProblemSelector>().is_ok(),
            valid,
            "selector={value:?}"
        );
    }
    for (value, valid) in [
        ("rust", true),
        ("python3", true),
        ("csharp", true),
        ("", false),
        ("Rust", false),
        ("python 3", false),
    ] {
        assert_eq!(
            value.parse::<LanguageSlug>().is_ok(),
            valid,
            "language={value:?}"
        );
    }

    let plain = presentation(false, false, false, 80);
    let problem = Problem {
        number: 42,
        id,
        title: "A Beautiful Problem\u{1b}\u{7}".into(),
        statement:
            r#"<p>Find <strong>two</strong> numbers with $x_i^2 \leq 10^4$ &amp; return indices.</p>
<h2>Examples</h2><p>&nbsp;</p><p> </p>
<p><strong class="lede example">Example 1:</strong></p>
<div class="panel example-block active">
<p><strong>Input:</strong> <span class="token example-io">nums = [`tick`], target = 1</span></p>
<p><strong>Output:</strong> <span class="example-io">[0]</span></p>
</div>
<p><strong>Constraints:</strong></p>
<ul><li><code>n &gt;= 1</code></li></ul>
<p><strong>Follow-up:</strong> Keep this sentence inline.</p>
<p><strong>Follow up:</strong></p>
<p><strong>Constraints::::</strong></p>
<pre><strong>Input:</strong> nums = [2,7], target = 9
<strong>Output:</strong> [0,1]
<strong>Explanation:</strong> 2 + 7 = 9</pre>
<p><em>Try it</em> with <strong>two numbers</strong>.</p>
<pre><code class="language-rust">first


last
// $x^2$ stays code
</code></pre>
<p>$$\frac{a+b}{c+d}$$</p>
<p>Then \(x^2\), \[\sqrt{n}\], and <span class="math math-inline">x_i + y_j</span>.</p>
<p>$$\begin{pmatrix}a &amp; b \\ c &amp; d\end{pmatrix}$$</p>
<ul><li>Return <code>10<sup>4</sup></code> values</li></ul>
<p>Unknown: $\unknown{x}$; malformed: $\frac{a}{b$.</p>
<p><a href="https://leetcode.com/problems/two-sum/">Read problem</a></p>
<script>__CPCLI_SCRIPT_SENTINEL__</script><style>__CPCLI_STYLE_SENTINEL__</style>
<img src="file:///does-not-exist" alt="diagram">
<table><tr><th>Limit</th></tr><tr><td>$n^2$</td></tr></table>"#
                .into(),
    };
    let text = render(&problem, Format::Text, plain)?;
    assert!(text.starts_with("#42 · leetcode/two-sum\nA Beautiful Problem\n\n"));
    assert_contains(
        &text,
        &[
            "x²ᵢ ≤ 10⁴ & return indices",
            "Example 1",
            "Input: nums = [`tick`], target = 1",
            "Output: [0]",
            "Constraints",
            "n >= 1",
            "Follow-up: Keep this sentence inline.",
            "Follow-up",
            "Constraints::::",
            "Input: nums = [2,7], target = 9\nOutput: [0,1]\nExplanation: 2 + 7 = 9",
            "first\n\n\nlast",
            "$x^2$ stays code",
            "a + b",
            "c + d",
            "Then x²",
            "√",
            "xᵢ + yⱼ",
            "10⁴ values",
            r"$\unknown{x}$",
            r"$\frac{a}{b$",
            "n²",
            "Limit",
            "https://leetcode.com/problems/two-sum/",
            "diagram",
        ],
    );
    assert_excludes(
        &text,
        &[
            "pmatrix",
            "__CPCLI_SCRIPT_SENTINEL__",
            "__CPCLI_STYLE_SENTINEL__",
            "<pre>",
            "\u{1b}",
            "\u{7}",
        ],
    );
    assert!(
        !text
            .split("first")
            .next()
            .ok_or("missing code")?
            .contains("\n\n\n")
    );

    let rich = presentation(true, true, true, 80);
    let styled = render(&problem, Format::Text, rich)?;
    assert_contains(
        &styled,
        &[
            "POSSUM//ARCADE  QUEST",
            "leetcode/two-sum",
            "38;5;95m",
            "38;5;110m",
            "38;5;181m",
            "\x1b[0;5;38;5;",
            "\x1b]8;;https://leetcode.com/problems/two-sum/\x1b\\",
            "\x1b]8;;\x1b\\",
            "\x1b#3",
            "\x1b#4",
            "\x1b#5",
            "\x1b[3mTry it",
            "\x1b[1m two numbers",
            "\x1b[4m",
            "Read problem",
        ],
    );
    for color in [rich.palette.gradient[0], rich.palette.gradient[3]] {
        assert_contains(&styled, &[&format!("38;5;{color}m")]);
    }
    let mut visible_styled = Vec::new();
    anstream::StripStream::new(&mut visible_styled).write_all(styled.as_bytes())?;
    assert_contains(
        &String::from_utf8(visible_styled)?,
        &["A Beautiful Problem"],
    );
    let panel = render(&problem, Format::Text, presentation(true, false, true, 80))?;
    assert_contains(&panel, &["\x1b[31G"]);

    let still_rich = render(
        &problem,
        Format::Text,
        Presentation {
            motion: false,
            ..rich
        },
    )?;
    assert_excludes(&still_rich, &["\x1b[0;5;38;5;", "\x1b[0;6;38;5;"]);
    let redirected = render(&problem, Format::Text, presentation(true, false, false, 80))?;
    assert_excludes(&redirected, &["POSSUM//", "\x1b[0;5;38;5;"]);
    assert_contains(
        &redirected,
        &[
            "38;5;181m",
            "\x1b[38;5;110m nums = [`tick`], target = 1\x1b[0m",
        ],
    );

    let search = ProblemSearch {
        query: "two sum".parse()?,
        total: 23,
        results: vec![
            ProblemSummary {
                number: 1,
                id: "two-sum".parse()?,
                title: "Two Sum".into(),
                difficulty: Difficulty::Easy,
                paid_only: false,
            },
            ProblemSummary {
                number: 1214,
                id: "two-sum-bsts".parse()?,
                title: "Two Sum BSTs".into(),
                difficulty: Difficulty::Medium,
                paid_only: true,
            },
            ProblemSummary {
                number: 1879,
                id: "minimum-xor-sum-of-two-arrays".parse()?,
                title: "Minimum XOR Sum of Two Arrays".into(),
                difficulty: Difficulty::Hard,
                paid_only: false,
            },
        ],
    };
    let search_text = render_search(&search, Format::Text, plain)?;
    assert_contains(
        &search_text,
        &[
            "POSSUM//SCAN  3 of 23 matches",
            "Search: two sum",
            "PROBLEM",
            "LEVEL",
            "SLUG",
            "─┼─",
            "Two Sum",
            "[ EASY ]",
            "leetcode/two-sum",
            "Two Sum BSTs",
            "[MEDIUM]",
            "PREMIUM",
            "[ HARD ]",
            "leetcode/minimum-xor-sum-of-two-",
            "arrays",
            "20 more matches — refine your search",
        ],
    );
    let first_result = search_text
        .lines()
        .find(|line| line.contains("Two Sum") && line.contains("[ EASY ]"))
        .ok_or("missing first result row")?;
    assert_contains(first_result, &["1", "leetcode/two-sum"]);
    assert_excludes(&search_text, &["\u{1b}"]);
    let styled_search = render_search(&search, Format::Text, rich)?;
    assert_contains(
        &styled_search,
        &[
            "POSSUM//SCAN",
            "\x1b]8;;https://leetcode.com/problems/two-sum/\x1b\\",
        ],
    );
    for color in [
        rich.palette.result_number,
        rich.palette.result_title,
        rich.palette.result_slug,
        rich.palette.easy,
        rich.palette.medium,
        rich.palette.hard,
    ] {
        assert_contains(&styled_search, &[&format!("38;5;{color}m")]);
    }
    let search_json: serde_json::Value =
        serde_json::from_str(&render_search(&search, Format::Json, rich)?)?;
    assert_eq!(search_json["query"], "two sum");
    assert_eq!(search_json["total"], 23);
    assert_eq!(search_json["results"][0]["difficulty"], "easy");
    assert_eq!(search_json["results"][1]["paid_only"], true);
    assert_eq!(search_json["results"][2]["difficulty"], "hard");

    let list = ProblemList {
        difficulty: Some(Difficulty::Medium),
        tag: Some("graph".parse()?),
        total: 21,
        results: vec![ProblemSummary {
            number: 133,
            id: "clone-graph".parse()?,
            title: "Clone Graph".into(),
            difficulty: Difficulty::Medium,
            paid_only: false,
        }],
    };
    let list_text = render_list(&list, Format::Text, plain)?;
    assert_contains(
        &list_text,
        &[
            "POSSUM//BROWSE  1 of 21 problems",
            "Filters: MEDIUM · graph",
            "Clone Graph",
            "[MEDIUM]",
            "leetcode/clone-graph",
            "20 more problems — add or adjust filters",
        ],
    );
    let list_json: serde_json::Value =
        serde_json::from_str(&render_list(&list, Format::Json, rich)?)?;
    assert_eq!(list_json["difficulty"], "medium");
    assert_eq!(list_json["tag"], "graph");
    assert_eq!(list_json["total"], 21);
    let narrow_list = render_list(&list, Format::Text, presentation(false, false, false, 16))?;
    assert!(narrow_list.lines().all(|line| line.width() <= 16));

    let catalog = CatalogProblemList {
        platform: CatalogPlatform::HackerRank,
        page: None,
        total: 442,
        results: vec![CatalogProblemSummary {
            id: "solve-me-first".parse()?,
            title: "Solve Me First".into(),
            difficulty: Some(Difficulty::Easy),
            rating: None,
            level: None,
            tags: vec!["Algorithms".into(), "Warmup".into()],
            solved_count: None,
            published_at: None,
            url: "https://www.hackerrank.com/challenges/solve-me-first/problem".into(),
        }],
    };
    let mut catalog_bytes = Vec::new();
    output::catalog_list(&mut catalog_bytes, Format::Text, &catalog, plain)?;
    assert_contains(
        &String::from_utf8(catalog_bytes)?,
        &[
            "POSSUM//Browse  HackerRank · 1 of 442 problems",
            "Solve Me First",
            "[ EASY ]",
            "Algorithms · Warmup",
        ],
    );
    let catalog_problem = CatalogProblem {
        platform: CatalogPlatform::HackerRank,
        id: "solve-me-first".parse()?,
        title: "Solve Me First".into(),
        difficulty: Some(Difficulty::Easy),
        rating: None,
        level: None,
        tags: vec!["Algorithms".into(), "Warmup".into()],
        solved_count: None,
        published_at: None,
        url: "https://www.hackerrank.com/challenges/solve-me-first/problem".into(),
        statement: Some("Add $a + b$. <script>bad()</script>\x1b[31m".into()),
        statement_format: Some(StatementFormat::Markdown),
        input_format: Some("Two integers".into()),
        output_format: Some("Their sum".into()),
        languages: vec!["cpp".into(), "rust".into()],
        attribution: None,
        license: None,
        license_url: None,
    };
    let mut catalog_problem_bytes = Vec::new();
    output::catalog_problem(
        &mut catalog_problem_bytes,
        Format::Text,
        &catalog_problem,
        plain,
    )?;
    let catalog_problem_text = String::from_utf8(catalog_problem_bytes)?;
    assert_contains(
        &catalog_problem_text,
        &[
            "hackerrank/solve-me-first",
            "Languages: 2 available",
            "Input format",
            "Output format",
        ],
    );
    assert_excludes(&catalog_problem_text, &["bad()", "<script>", "\x1b"]);

    let project_euler = CatalogProblem {
        platform: CatalogPlatform::ProjectEuler,
        id: "1".parse()?,
        title: "Multiples of 3 or 5".into(),
        difficulty: None,
        rating: None,
        level: Some("Level 0 · 5%".into()),
        tags: Vec::new(),
        solved_count: Some(1_234_567),
        published_at: Some("5 October 2001".into()),
        url: "https://projecteuler.net/problem=1".into(),
        statement: Some("<p>Find the <strong>sum</strong>.</p><script>bad()</script>".into()),
        statement_format: Some(StatementFormat::Html),
        input_format: None,
        output_format: None,
        languages: Vec::new(),
        attribution: Some("Problem content from Project Euler".into()),
        license: Some("CC BY-NC-SA 4.0".into()),
        license_url: Some("https://creativecommons.org/licenses/by-nc-sa/4.0/".into()),
    };
    let mut project_euler_bytes = Vec::new();
    output::catalog_problem(
        &mut project_euler_bytes,
        Format::Text,
        &project_euler,
        plain,
    )?;
    let project_euler_text = String::from_utf8(project_euler_bytes)?;
    assert_contains(
        &project_euler_text,
        &[
            "project-euler/1",
            "Level 0 · 5%",
            "Published 5 October 2001",
            "Find the sum.",
            "Source: Problem content from Project Euler",
            "License: CC BY-NC-SA 4.0",
        ],
    );
    assert_excludes(&project_euler_text, &["bad()", "<script>"]);

    let project_euler_list = CatalogProblemList {
        platform: CatalogPlatform::ProjectEuler,
        page: Some(2),
        total: 1,
        results: vec![CatalogProblemSummary {
            id: "51".parse()?,
            title: "$N$th digit replacements".into(),
            difficulty: None,
            rating: None,
            level: None,
            tags: Vec::new(),
            solved_count: Some(200_000),
            published_at: Some("29 August 2003".into()),
            url: "https://projecteuler.net/problem=51".into(),
        }],
    };
    let mut project_euler_list_bytes = Vec::new();
    output::catalog_list(
        &mut project_euler_list_bytes,
        Format::Text,
        &project_euler_list,
        plain,
    )?;
    assert_contains(
        &String::from_utf8(project_euler_list_bytes)?,
        &["Catalogue page 2", "/problem=51", "200000", "solved"],
    );
    let project_euler_json: serde_json::Value = serde_json::from_str(&{
        let mut bytes = Vec::new();
        output::catalog_list(&mut bytes, Format::Json, &project_euler_list, plain)?;
        String::from_utf8(bytes)?
    })?;
    assert_eq!(project_euler_json["platform"], "project-euler");
    assert_eq!(
        project_euler_json["results"][0]["title"],
        "$N$th digit replacements"
    );

    let mut auth_bytes = Vec::new();
    output::auth_status(
        &mut auth_bytes,
        Format::Text,
        &AuthStatus {
            configured: true,
            authenticated: true,
            source: Some(AuthSource::SecureStore),
        },
        rich,
    )?;
    assert_contains(&String::from_utf8(auth_bytes)?, &["POSSUM//AUTH", "READY"]);
    let mut logout_bytes = Vec::new();
    output::auth_logout(
        &mut logout_bytes,
        Format::Text,
        &AuthLogout {
            removed: true,
            environment_present: true,
        },
        rich,
    )?;
    assert_contains(
        &String::from_utf8(logout_bytes)?,
        &["SAVED SESSION REMOVED", "still present in this shell"],
    );
    let stats = AccountStats {
        username: "night-possum".into(),
        solved: DifficultyCounts {
            all: 6,
            easy: 3,
            medium: 2,
            hard: 1,
        },
        accepted_submissions: DifficultyCounts {
            all: 9,
            easy: 4,
            medium: 3,
            hard: 2,
        },
        submissions: DifficultyCounts {
            all: 12,
            easy: 5,
            medium: 4,
            hard: 3,
        },
        recent_submissions: vec![RecentSubmission {
            id: 4242,
            status: "Accepted".into(),
            title: "Two Sum".into(),
            problem: "two-sum".parse()?,
            timestamp: 1_700_000_000,
            language: "C++".into(),
            runtime: Some("4 ms".into()),
            memory: Some("16 MB".into()),
            pending: false,
        }],
        has_more_submissions: true,
    };
    let mut stats_bytes = Vec::new();
    output::stats(&mut stats_bytes, Format::Text, &stats, plain)?;
    let stats_text = String::from_utf8(stats_bytes)?;
    assert_contains(
        &stats_text,
        &[
            "POSSUM//Stats  night-possum",
            "Solved",
            "Accepted",
            "Attempts",
            "Recent submissions",
            "Two Sum",
            "C++",
            "4 ms · 16 MB",
            "10 most recent",
        ],
    );
    let codeforces_stats = CodeforcesStats {
        profile: CodeforcesProfile {
            handle: "tourist".into(),
            rank: Some("legendary grandmaster".into()),
            rating: Some(3307),
            max_rank: None,
            max_rating: None,
            contribution: 1,
            friend_of_count: 2,
        },
        rating_history: Vec::new(),
        recent_submissions: vec![CodeforcesSubmission {
            id: 1,
            contest_id: Some(2262),
            timestamp: 1,
            problem_index: "A".into(),
            problem_name: "A problem with a readable title".into(),
            language: "C++23 (GCC 14-64, msys2)".into(),
            verdict: Some("OK".into()),
            passed_test_count: 10,
            runtime_millis: 562,
            memory_bytes: 17_100_800,
        }],
        recent_accepted_count: 1,
        recent_solved_count: 1,
    };
    let mut codeforces_stats_bytes = Vec::new();
    output::codeforces_stats(
        &mut codeforces_stats_bytes,
        Format::Text,
        &codeforces_stats,
        presentation(false, false, false, 80),
    )?;
    let codeforces_stats_text = String::from_utf8(codeforces_stats_bytes)?;
    assert!(codeforces_stats_text.lines().all(|line| line.width() <= 80));
    assert_contains(&codeforces_stats_text, &["msys2)", "562 ms · 16700 KB"]);
    let mut stats_json = Vec::new();
    output::stats(&mut stats_json, Format::Json, &stats, rich)?;
    let stats_json: serde_json::Value = serde_json::from_slice(&stats_json)?;
    assert_eq!(stats_json["username"], "night-possum");
    assert_eq!(stats_json["solved"]["hard"], 1);
    assert_eq!(stats_json["recent_submissions"][0]["problem"], "two-sum");
    let mut narrow_stats = Vec::new();
    output::stats(
        &mut narrow_stats,
        Format::Text,
        &stats,
        presentation(false, false, false, 32),
    )?;
    let narrow_stats = String::from_utf8(narrow_stats)?;
    assert!(narrow_stats.lines().all(|line| line.width() <= 32));
    let contests = ContestList {
        contests: vec![
            Contest {
                id: "weekly-contest-500".parse()?,
                title: "Weekly Contest 500".into(),
                start_time: 1_700_000_000,
                duration_seconds: 5_400,
                virtual_contest: false,
            },
            Contest {
                id: "biweekly-contest-200".parse()?,
                title: "Biweekly Contest 200".into(),
                start_time: 1_700_086_400,
                duration_seconds: 7_200,
                virtual_contest: true,
            },
        ],
    };
    let mut contests_bytes = Vec::new();
    output::contests(&mut contests_bytes, Format::Text, &contests, plain)?;
    let contests_text = String::from_utf8(contests_bytes)?;
    assert_contains(
        &contests_text,
        &[
            "POSSUM//Contests  2 upcoming contests",
            "Starts (UTC)",
            "Weekly Contest 500",
            "2023-11-14 22:13",
            "1h 30m",
            "biweekly-contest-200",
            "Virtual",
        ],
    );
    let mut contests_json = Vec::new();
    output::contests(&mut contests_json, Format::Json, &contests, rich)?;
    let contests_json: serde_json::Value = serde_json::from_slice(&contests_json)?;
    assert_eq!(contests_json["contests"][0]["id"], "weekly-contest-500");
    assert_eq!(contests_json["contests"][0]["duration_seconds"], 5_400);
    let mut contest_bytes = Vec::new();
    output::contest(
        &mut contest_bytes,
        Format::Text,
        &contests.contests[0],
        plain,
    )?;
    assert_contains(
        &String::from_utf8(contest_bytes)?,
        &[
            "POSSUM//Contest",
            "Weekly Contest 500",
            "leetcode/contest/weekly-contest-500",
            "Starts: 2023-11-14 22:13",
            "Duration: 1h 30m",
            "Mode: Official",
        ],
    );
    let registration = ContestRegistration {
        id: "weekly-contest-500".parse()?,
        registered: true,
    };
    let mut registration_bytes = Vec::new();
    output::contest_registration(&mut registration_bytes, Format::Text, &registration, plain)?;
    assert_contains(
        &String::from_utf8(registration_bytes)?,
        &[
            "POSSUM//Contest",
            "weekly-contest-500",
            "Status: Registered",
        ],
    );
    let mut narrow_contests = Vec::new();
    output::contests(
        &mut narrow_contests,
        Format::Text,
        &contests,
        presentation(false, false, false, 32),
    )?;
    let narrow_contests = String::from_utf8(narrow_contests)?;
    assert!(narrow_contests.lines().all(|line| line.width() <= 32));
    for width in [1, 16, 32, 64, 80] {
        for show in [false, true] {
            let mut bytes = Vec::new();
            let display = presentation(false, false, false, width);
            if show {
                output::contest(&mut bytes, Format::Text, &contests.contests[0], display)?;
            } else {
                output::contests(&mut bytes, Format::Text, &contests, display)?;
            }
            let output = String::from_utf8(bytes)?;
            assert!(
                output
                    .lines()
                    .all(|line| line.width() <= usize::from(width)),
                "width={width}: {output}"
            );
        }
    }
    let discussions = DiscussionList {
        problem: Some("two-sum".parse()?),
        total: Some(42),
        discussions: vec![
            DiscussionSummary {
                id: "1124385".parse()?,
                title: "A clean hash map approach".into(),
                author: Some("night-possum".into()),
                created_at: 1_700_000_000,
                views: 1_200,
                comments: 18,
                votes: 73,
            },
            DiscussionSummary {
                id: "1124386".parse()?,
                title: "Why the complement comes first".into(),
                author: None,
                created_at: 1_700_000_001,
                views: 80,
                comments: 2,
                votes: -1,
            },
        ],
    };
    let mut discussions_bytes = Vec::new();
    output::discussions(&mut discussions_bytes, Format::Text, &discussions, plain)?;
    let discussions_text = String::from_utf8(discussions_bytes)?;
    assert_contains(
        &discussions_text,
        &[
            "POSSUM//Discuss  2 of 42 posts",
            "Problem: leetcode/two-sum",
            "Discussion",
            "A clean hash map approach",
            "night-possum",
            "73 votes",
            "Deleted user",
        ],
    );
    let mut discussions_json = Vec::new();
    output::discussions(&mut discussions_json, Format::Json, &discussions, rich)?;
    let discussions_json: serde_json::Value = serde_json::from_slice(&discussions_json)?;
    assert_eq!(discussions_json["problem"], "two-sum");
    assert_eq!(discussions_json["total"], 42);
    assert_eq!(discussions_json["discussions"][0]["id"], 1_124_385);
    let discussion = Discussion {
        id: "1124385".parse()?,
        title: "A clean hash map approach".into(),
        author: Some("night-possum".into()),
        content: "## Idea\n\nUse a **hash map** in $O(n)$ time. [unsafe](javascript:__BAD_LINK__)\n\n<script>__DISCUSSION_SCRIPT__</script>"
            .into(),
        created_at: 1_700_000_000,
        updated_at: Some(1_700_000_060),
        views: 1_200,
        comments: 18,
        votes: 73,
        tags: vec!["C++".into(), "Hash Table".into()],
        pinned: true,
    };
    let mut discussion_bytes = Vec::new();
    output::discussion(&mut discussion_bytes, Format::Text, &discussion, plain)?;
    let discussion_text = String::from_utf8(discussion_bytes)?;
    assert_contains(
        &discussion_text,
        &[
            "POSSUM//Discuss  #1124385",
            "A clean hash map approach",
            "By night-possum · 2023-11-14 22:13 UTC · Pinned",
            "Updated: 2023-11-14 22:14 UTC",
            "73 votes · 18 replies · 1200 views",
            "Tags: C++ · Hash Table",
            "Idea",
            "Use a hash map in O(n) time.",
        ],
    );
    assert_excludes(
        &discussion_text,
        &["__DISCUSSION_SCRIPT__", "javascript:__BAD_LINK__", "\u{1b}"],
    );
    for width in [1, 16, 32, 72, 80] {
        for show in [false, true] {
            let mut bytes = Vec::new();
            let display = presentation(false, false, false, width);
            if show {
                output::discussion(&mut bytes, Format::Text, &discussion, display)?;
            } else {
                output::discussions(&mut bytes, Format::Text, &discussions, display)?;
            }
            let output = String::from_utf8(bytes)?;
            assert!(
                output
                    .lines()
                    .all(|line| line.width() <= usize::from(width)),
                "width={width}: {output}"
            );
        }
    }
    let solution = SolutionFile {
        platform: "leetcode".into(),
        id: "two-sum".parse()?,
        title: "Two Sum".into(),
        language: "Rust".into(),
        path: "solutions/two-sum.rs".into(),
    };
    let mut solution_bytes = Vec::new();
    output::solution_file(&mut solution_bytes, Format::Text, &solution, plain)?;
    assert_contains(
        &String::from_utf8(solution_bytes)?,
        &[
            "POSSUM//PICKED",
            "Two Sum · Rust",
            "problem test solutions/two-sum.rs",
        ],
    );
    let mut test_bytes = Vec::new();
    output::test(
        &mut test_bytes,
        Format::Text,
        &JudgeTest {
            run_id: "run-42".into(),
            id: "two-sum".parse()?,
            path: "solutions/two-sum.rs".into(),
            passed: false,
            status: "Wrong Answer".into(),
            runtime: Some("0 ms".into()),
            memory: Some("2 MB".into()),
            passed_cases: Some(2),
            total_cases: Some(3),
            input: Some("[3,3]\n6".into()),
            output: Some("[1,1]".into()),
            expected: Some("[0,1]".into()),
            message: None,
        },
        plain,
    )?;
    assert_contains(
        &String::from_utf8(test_bytes)?,
        &[
            "POSSUM//TEST  Wrong Answer",
            "LeetCode examples",
            "Cases: 2/3",
            "Input:",
            "Expected:",
            "Output:",
        ],
    );
    let mut submission_bytes = Vec::new();
    output::submission(
        &mut submission_bytes,
        Format::Text,
        &SubmissionResult {
            id: 42,
            status: "Accepted".into(),
            accepted: true,
            runtime: Some("0 ms".into()),
            memory: Some("2 MB".into()),
            passed: Some(63),
            total: Some(63),
            message: None,
        },
        rich,
    )?;
    assert_contains(
        &String::from_utf8(submission_bytes)?,
        &["POSSUM//JUDGE", "Accepted", "Cases: 63/63"],
    );

    let empty_search = ProblemSearch {
        query: "no result".parse()?,
        total: 0,
        results: Vec::new(),
    };
    assert_contains(
        &render_search(&empty_search, Format::Text, plain)?,
        &["0 matches", "No problems found"],
    );
    for width in [1, 2, 10, 40, 48, 80, 100] {
        let output = render_search(
            &search,
            Format::Text,
            presentation(false, false, false, width),
        )?;
        assert!(
            output
                .lines()
                .all(|line| line.width() <= usize::from(width)),
            "width={width}: {output}"
        );
    }
    for width in [16, 32, 48, 80, 100] {
        let output = render_search(
            &search,
            Format::Text,
            presentation(true, false, true, width),
        )?;
        assert_eq!(
            output
                .matches("\x1b]8;;https://leetcode.com/problems/")
                .count(),
            output.matches("\x1b]8;;\x1b\\").count(),
            "width={width}: {output}"
        );
        let mut stripped = Vec::new();
        anstream::StripStream::new(&mut stripped).write_all(output.as_bytes())?;
        let stripped = String::from_utf8(stripped)?;
        assert!(
            stripped
                .lines()
                .all(|line| line.width() <= usize::from(width)),
            "width={width}: {stripped}"
        );
    }

    #[cfg(not(feature = "leetcode"))]
    {
        assert!(matches!(
            crate::problems::search("two".parse()?, |_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::problems::daily(|_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::problems::list(Some(Difficulty::Medium), Some("graph".parse()?), |_, _| {})
                .await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::contests::list(|_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::contests::show("weekly-contest-500".parse()?, |_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::contests::status("weekly-contest-500".parse()?, |_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::discussions::list(None, |_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::discussions::show("1124385".parse()?, |_, _| {}).await,
            Err(crate::Error::LeetCodeDisabled)
        ));
        assert!(matches!(
            crate::solve::pick(
                "two-sum".parse()?,
                Some("rust".parse()?),
                Some(".".into()),
                |_, _| {},
                || {}
            )
            .await,
            Err(crate::Error::LeetCodeDisabled)
        ));
    }

    let poses = [
        crate::possum::Pose::Cursor,
        crate::possum::Pose::Blink,
        crate::possum::Pose::Scratch,
        crate::possum::Pose::Idle,
    ]
    .map(|pose| crate::possum::render(pose, 42, true).ok_or("missing possum frame"))
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    assert_ne!(poses[0], poses[1]);
    assert_ne!(poses[0], poses[2]);
    for art in poses {
        let mut plain_art = Vec::new();
        anstream::StripStream::new(&mut plain_art).write_all(art.as_bytes())?;
        let plain_art = String::from_utf8(plain_art)?;
        assert_eq!(plain_art.lines().count(), 17);
        assert!(plain_art.lines().all(|line| line.width() <= 42));
        assert!(
            plain_art
                .chars()
                .any(|glyph| matches!(glyph, '@' | '%' | '#'))
        );
    }
    assert!(crate::possum::render(crate::possum::Pose::Idle, 27, true).is_none());
    assert!(crate::possum::render(crate::possum::Pose::Idle, 80, false).is_none());
    assert_contains(
        &crate::possum::render_alive(crate::possum::Pose::Cursor, 42, true)
            .ok_or("missing live possum")?,
        &["\x1b[0;5;38;5;"],
    );
    assert_contains(
        &crate::possum::render_alive(crate::possum::Pose::Scratch, 42, true)
            .ok_or("missing scratching possum")?,
        &["\x1b[0;5;38;5;", "\x1b[0;6;38;5;"],
    );

    for (width, visible, wide) in [
        (1, false, false),
        (15, false, false),
        (16, true, false),
        (31, true, false),
        (32, true, true),
        (55, true, true),
        (56, true, true),
        (80, true, true),
    ] {
        let output = render(
            &Problem {
                number: 7,
                id: "pixel-possum".parse()?,
                title: "X".into(),
                statement: "<p>Y</p>".into(),
            },
            Format::Text,
            presentation(false, false, true, width),
        )?;
        assert_eq!(
            output.contains("POSSUM//"),
            visible,
            "width={width}: {output}"
        );
        assert_eq!(
            output.contains("POSSUM//ARCADE"),
            wide,
            "width={width}: {output}"
        );
        assert!(
            output
                .lines()
                .all(|line| line.width() <= usize::from(width)),
            "width={width}: {output}"
        );
    }

    for (theme, background) in [
        (Theme::Possum, [18; 3]),
        (Theme::Arcade, [18; 3]),
        (Theme::Arcade, [245; 3]),
        (Theme::Moonlight, [18; 3]),
        (Theme::Phosphor, [18; 3]),
        (Theme::Amber, [245; 3]),
        (Theme::Dark, [18; 3]),
        (Theme::Light, [245; 3]),
    ] {
        let palette = theme.palette(background);
        let themed = render(&problem, Format::Text, Presentation { palette, ..rich })?;
        assert_contains(
            &themed,
            &["POSSUM//ARCADE", &format!("38;5;{}m", palette.accent)],
        );
    }

    for display in [plain, rich, presentation(false, true, true, 80)] {
        let json = render(&problem, Format::Json, display)?;
        let value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(value["number"], 42);
        assert_eq!(value["id"], "two-sum");
        assert_eq!(value["statement"], problem.statement.as_ref());
        assert!(json.ends_with('\n'));
        assert_excludes(&json, &["\u{1b}", "POSSUM//", "\u{7}"]);
    }
    for format in [Format::Text, Format::Json] {
        let error = output::problem(BrokenPipe, format, &problem, rich)
            .err()
            .ok_or("expected broken pipe")?;
        assert!(error.is_broken_pipe());
        let error = output::search(BrokenPipe, format, &search, rich)
            .err()
            .ok_or("expected search broken pipe")?;
        assert!(error.is_broken_pipe());
    }

    for (color, tty, no_color, terminal, expected) in [
        (Color::Auto, true, false, "xterm-256color", true),
        (Color::Auto, true, true, "xterm-256color", false),
        (Color::Auto, false, false, "xterm-256color", false),
        (Color::Auto, true, false, "dumb", false),
        (Color::Always, false, true, "dumb", true),
        (Color::Never, true, false, "xterm-256color", false),
    ] {
        let actual =
            Presentation::resolve(color, HeadingSize::Auto, tty, no_color, terminal, true, 80);
        assert_eq!(
            (actual.color, actual.interactive),
            (expected, tty && terminal != "dumb")
        );
    }
    for (size, tty, terminal, scalable, width, expected) in [
        (HeadingSize::Auto, true, "xterm", true, 80, true),
        (HeadingSize::Auto, true, "xterm", true, 59, false),
        (HeadingSize::Auto, true, "xterm", false, 80, false),
        (HeadingSize::Auto, false, "xterm", true, 80, false),
        (HeadingSize::Auto, true, "dumb", true, 80, false),
        (HeadingSize::Normal, true, "xterm", true, 80, false),
        (HeadingSize::Large, true, "xterm", true, 4, true),
        (HeadingSize::Large, true, "xterm", true, 3, false),
    ] {
        assert_eq!(
            Presentation::resolve(Color::Never, size, tty, true, terminal, scalable, width,)
                .large_heading,
            expected
        );
    }
    for (columns, expected) in [(0, 1), (1, 1), (80, 80), (100, 100), (101, 100)] {
        assert_eq!(
            Presentation::resolve(
                Color::Never,
                HeadingSize::Normal,
                false,
                false,
                "xterm",
                false,
                columns,
            )
            .columns,
            expected
        );
    }

    let narrow = Problem {
        number: 9,
        id: "narrow".parse()?,
        title: "Narrow".into(),
        statement: "<p>A long sentence with supercalifragilisticexpialidocious that must wrap without losing words.</p>".into(),
    };
    for width in [1, 2, 10, 20, 40, 80] {
        let output = render(
            &narrow,
            Format::Text,
            presentation(false, false, false, width),
        )?;
        assert!(
            output
                .lines()
                .all(|line| line.width() <= usize::from(width)),
            "width={width}: {output}"
        );
    }

    for (source, expected) in [
        ("x^2", "x²"),
        ("x_i", "xᵢ"),
        (r"\sqrt{n}", "√"),
        (r"\frac{a}{b}", "─"),
    ] {
        let rendered = crate::math::render(source)
            .ok_or("expected valid math")?
            .to_string();
        assert!(rendered.contains(expected), "source={source}: {rendered}");
    }
    assert_eq!(crate::math::inline_text("The $x^2$ path"), "The x² path");
    for source in [
        "{".into(),
        "}".into(),
        r"\unknown{x}".into(),
        r"\frac{a}".into(),
        "x^".into(),
        "{".repeat(32),
        "x".repeat(1025),
        "(".repeat(200),
    ] {
        assert!(crate::math::render(&source).is_none(), "source={source:?}");
    }
    Ok(())
}

struct BrokenPipe;

impl Write for BrokenPipe {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
