use std::io::{self, Write};

use clap::Parser;
use unicode_width::UnicodeWidthStr;

use crate::{
    cli::{Cli, Color, Command, Format, HeadingSize, ProblemCommand, Theme},
    domain::{Difficulty, Problem, ProblemId, ProblemQuery, ProblemSearch, ProblemSummary},
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
        command: ProblemCommand::Show { id },
    } = cli.command
    else {
        return Err("expected show command".into());
    };
    assert_eq!(id.as_ref(), "two-sum");
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
    let animated = crate::possum::loading_frames(80, true, true);
    let still = crate::possum::loading_frames(80, true, false);
    assert_eq!(animated.len(), 10);
    assert_ne!(animated[0], animated[2]);
    assert_ne!(animated[0], animated[6]);
    assert_eq!(still.len(), 2);
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

    let plain = presentation(false, false, false, 80);
    let problem = Problem {
        id,
        title: "A Beautiful Problem\u{1b}\u{7}".into(),
        statement:
            r#"<p>Find <strong>two</strong> numbers with $x_i^2 \leq 10^4$ &amp; return indices.</p>
<h2>Examples</h2><p>&nbsp;</p><p> </p>
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
    assert!(text.starts_with("A Beautiful Problem\n\n"));
    assert_contains(
        &text,
        &[
            "x²ᵢ ≤ 10⁴ & return indices",
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

    let still_rich = render(
        &problem,
        Format::Text,
        Presentation {
            motion: false,
            ..rich
        },
    )?;
    assert_excludes(&still_rich, &["\x1b[0;5;38;5;"]);
    let redirected = render(&problem, Format::Text, presentation(true, false, false, 80))?;
    assert_excludes(&redirected, &["POSSUM//", "\x1b[0;5;38;5;"]);

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
    assert!(matches!(
        crate::problems::search("two".parse()?, |_, _| {}).await,
        Err(crate::Error::LeetCodeDisabled)
    ));

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
