use std::{
    borrow::Cow,
    cell::Cell,
    env,
    io::{IsTerminal, Write},
    sync::OnceLock,
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, TextMergeStream};
use pulldown_cmark_mdcat::{
    Environment, Settings, TerminalCapabilities, TerminalSize, push_tty,
    resources::NoopResourceHandler, terminal::capabilities::StyleCapability,
};
use syntect::parsing::SyntaxSet;
use unicode_width::UnicodeWidthStr;

use crate::{
    cli::{Color, Format, HeadingSize},
    codeforces_support::{
        CodeforcesContest, CodeforcesContestList, CodeforcesStats, CodeforcesSubmission,
    },
    domain::{
        AccountStats, AuthLogout, AuthSource, AuthStatus, CatalogPlatform, CatalogProblem,
        CatalogProblemList, CatalogProblemSummary, CommunityDiscussion, CommunityDiscussionList,
        CommunityDiscussionSummary, Contest, ContestList, ContestRegistration, DifficultyCounts,
        Discussion, DiscussionList, DiscussionSummary, Problem, ProblemList, ProblemSearch,
        ProblemSummary, PublicAccountStats, RecentSubmission, SolutionFile, StatementFormat,
        SubmissionResult, TestResult, ToolAction, official_path,
    },
    error::Result,
    math,
};

pub(crate) fn tool_action(
    mut output: impl Write,
    format: Format,
    action: &ToolAction,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, action)?;
        writeln!(output)?;
        return Ok(());
    }
    let accent = presentation.accent();
    let reset = accent.render_reset();
    writeln!(
        output,
        "{accent}POSSUM//{}  {}{reset}",
        clean(&action.platform),
        clean(&action.action)
    )?;
    for line in textwrap::wrap(&clean(&action.detail), usize::from(presentation.columns)) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

pub(crate) fn codeforces_stats(
    mut output: impl Write,
    format: Format,
    stats: &CodeforcesStats,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, stats)?;
        writeln!(output)?;
        return Ok(());
    }
    let columns = usize::from(presentation.columns);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    for line in textwrap::wrap(
        &format!("POSSUM//Stats  {}", clean(&stats.profile.handle)),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    let rank = stats
        .profile
        .rank
        .as_deref()
        .map_or_else(|| "Unrated".to_owned(), clean);
    let rating = stats
        .profile
        .rating
        .map_or_else(|| "unrated".to_owned(), |value| value.to_string());
    for line in textwrap::wrap(
        &format!(
            "Codeforces · {rank} · rating {rating} · contribution {} · {} followers",
            stats.profile.contribution, stats.profile.friend_of_count
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    if let Some(change) = stats.rating_history.last() {
        for line in textwrap::wrap(
            &format!(
                "Latest rating: {} → {} at {}",
                change.old_rating,
                change.new_rating,
                clean(&change.contest_name)
            ),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    writeln!(output)?;
    for line in textwrap::wrap(
        &format!(
            "Latest {} submissions: {} solved · {} accepted",
            stats.recent_submissions.len(),
            stats.recent_solved_count,
            stats.recent_accepted_count
        ),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    if stats.recent_submissions.is_empty() {
        writeln!(output, "No public submissions found.")?;
    } else {
        codeforces_submissions_table(&mut output, &stats.recent_submissions, presentation)?;
    }
    Ok(())
}

pub(crate) fn public_account_stats(
    mut output: impl Write,
    format: Format,
    stats: &PublicAccountStats,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, stats)?;
        writeln!(output)?;
        return Ok(());
    }

    let columns = usize::from(presentation.columns);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    for line in textwrap::wrap(
        &format!(
            "POSSUM//Stats  {} · {}",
            clean(&stats.platform),
            clean(&stats.username)
        ),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    let identity = [stats.name.as_deref(), stats.country.as_deref()]
        .into_iter()
        .flatten()
        .map(clean)
        .collect::<Vec<_>>()
        .join(" · ");
    if !identity.is_empty() {
        for line in textwrap::wrap(&identity, columns) {
            writeln!(output, "{line}")?;
        }
    }
    let mut metrics = Vec::new();
    if let Some(solved) = stats.solved {
        metrics.push(format!("{solved} solved"));
    }
    if let Some(rating) = stats.rating {
        metrics.push(format!("Rating {rating}"));
    }
    if let Some(rating) = stats.highest_rating {
        metrics.push(format!("Peak {rating}"));
    }
    if let Some(level) = stats.level {
        metrics.push(format!("Level {level}"));
    }
    if let Some(contests) = stats.contests {
        metrics.push(format!("{contests} contests"));
    }
    if let Some(events) = stats.events {
        metrics.push(format!("{events} events"));
    }
    if let Some(joined) = &stats.joined_at {
        metrics.push(format!("Joined {}", clean(joined)));
    }
    if !metrics.is_empty() {
        for line in textwrap::wrap(&metrics.join(" · "), columns) {
            writeln!(output, "{line}")?;
        }
    }
    if !metrics.is_empty() || !identity.is_empty() {
        writeln!(output)?;
    }
    url_link(
        &mut output,
        &stats.url,
        &stats.url,
        presentation.color && presentation.interactive,
    )?;
    writeln!(output)?;
    Ok(())
}

pub(crate) fn codeforces_contests(
    mut output: impl Write,
    format: Format,
    list: &CodeforcesContestList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }
    let accent = presentation.accent();
    let reset = accent.render_reset();
    writeln!(
        output,
        "{accent}POSSUM//Contests  {} Codeforces contests{reset}",
        list.contests.len()
    )?;
    if list.contests.is_empty() {
        writeln!(output, "No contests found.")?;
    } else {
        writeln!(output)?;
        codeforces_contests_table(&mut output, &list.contests, presentation)?;
    }
    Ok(())
}

pub(crate) fn codeforces_contest(
    mut output: impl Write,
    format: Format,
    contest: &CodeforcesContest,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, contest)?;
        writeln!(output)?;
        return Ok(());
    }
    let accent = presentation.accent();
    let reset = accent.render_reset();
    writeln!(output, "{accent}POSSUM//Contest  #{}{reset}", contest.id)?;
    heading(&mut output, &contest.name, presentation)?;
    url_link(
        &mut output,
        &contest.url,
        &contest.url,
        presentation.color && presentation.interactive,
    )?;
    writeln!(output)?;
    writeln!(
        output,
        "Phase: {} · Type: {}",
        phase_label(&contest.phase),
        clean(&contest.kind)
    )?;
    if let Some(start) = contest
        .start_time_seconds
        .and_then(|value| u64::try_from(value).ok())
    {
        writeln!(output, "Starts: {} UTC", timestamp_utc(start))?;
    }
    if let Some(duration) = contest.duration_seconds {
        writeln!(output, "Length: {}", duration_label(duration))?;
    }
    Ok(())
}

pub(crate) fn contest_registration(
    mut output: impl Write,
    format: Format,
    registration: &ContestRegistration,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, registration)?;
        writeln!(output)?;
        return Ok(());
    }

    let columns = usize::from(presentation.columns);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    for line in textwrap::wrap("POSSUM//Contest", columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    for line in textwrap::wrap(
        &format!("leetcode/contest/{}", registration.id.as_ref()),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    let status = if registration.registered {
        "Registered"
    } else {
        "Not registered"
    };
    for line in textwrap::wrap(&format!("Status: {status}"), columns) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) struct Presentation {
    pub(crate) columns: u16,
    pub(crate) color: bool,
    pub(crate) large_heading: bool,
    pub(crate) interactive: bool,
    pub(crate) motion: bool,
    pub(crate) palette: crate::theme::Palette,
}

impl Presentation {
    pub(crate) fn accent(self) -> anstyle::Style {
        if self.color {
            crate::theme::foreground(self.palette.accent).bold()
        } else {
            anstyle::Style::new()
        }
    }

    pub(crate) fn detect(color: Color, heading_size: HeadingSize) -> Self {
        let terminal = env::var("TERM").unwrap_or_default();
        let program = env::var("TERM_PROGRAM").unwrap_or_default();
        let tty = std::io::stdout().is_terminal();
        let no_color = env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        let multiplexer = env::var_os("TMUX").is_some()
            || terminal.starts_with("screen")
            || terminal.starts_with("tmux");
        let scalable =
            !multiplexer && (program == "Apple_Terminal" || env::var_os("WT_SESSION").is_some());
        let columns = if tty {
            TerminalSize::detect().unwrap_or_default().columns
        } else {
            80
        };
        Self::resolve(
            color,
            heading_size,
            tty,
            no_color,
            &terminal,
            scalable,
            columns,
        )
    }

    pub(crate) fn resolve(
        color: Color,
        heading_size: HeadingSize,
        tty: bool,
        no_color: bool,
        terminal: &str,
        scalable: bool,
        columns: u16,
    ) -> Self {
        let color = match color {
            Color::Auto => tty && !no_color && terminal != "dumb",
            Color::Always => true,
            Color::Never => false,
        };
        let columns = columns.clamp(1, 100);
        let large_heading = tty
            && terminal != "dumb"
            && scalable
            && match heading_size {
                HeadingSize::Auto => columns >= 60,
                HeadingSize::Normal => false,
                HeadingSize::Large => columns >= 4,
            };
        Self {
            columns,
            color,
            large_heading,
            interactive: tty && terminal != "dumb",
            motion: true,
            palette: crate::cli::Theme::Possum.palette([18; 3]),
        }
    }
}

pub(crate) fn problem(
    mut output: impl Write,
    format: Format,
    problem: &Problem,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, problem)?;
        writeln!(output)?;
        return Ok(());
    }

    problem_header(&mut output, problem, presentation)?;
    statement(&mut output, problem, presentation)
}

pub(crate) fn catalog_problem(
    mut output: impl Write,
    format: Format,
    problem: &CatalogProblem,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, problem)?;
        writeln!(output)?;
        return Ok(());
    }

    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(
        &format!(
            "POSSUM//Arcade  {}/{}",
            problem.platform.slug(),
            problem.id.as_ref()
        ),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    heading(
        &mut output,
        &math::inline_text(&problem.title),
        presentation,
    )?;
    let link_style = result_link_style(presentation, presentation.palette.result_slug);
    for line in textwrap::wrap(problem.url.as_ref(), columns) {
        write!(output, "{link_style}")?;
        url_link(
            &mut output,
            &line,
            &problem.url,
            presentation.color && presentation.interactive,
        )?;
        writeln!(output, "{}", link_style.render_reset())?;
    }
    let mut details = Vec::new();
    if let Some(difficulty) = problem.difficulty {
        details.push(difficulty.label().to_owned());
    }
    if let Some(rating) = problem.rating {
        details.push(format!("Rating {rating}"));
    }
    if let Some(level) = &problem.level {
        details.push(clean(level));
    }
    if let Some(solved) = problem.solved_count {
        details.push(format!("Solved by {solved}"));
    }
    if let Some(published) = &problem.published_at {
        details.push(format!("Published {published}"));
    }
    if !problem.tags.is_empty() {
        details.push(
            problem
                .tags
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join(" · "),
        );
    }
    if !details.is_empty() {
        for line in textwrap::wrap(&details.join(" · "), columns) {
            writeln!(output, "{line}")?;
        }
    }
    if !problem.languages.is_empty() {
        for line in textwrap::wrap(
            &format!("Languages: {} available", problem.languages.len()),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    writeln!(output)?;
    if let Some(statement) = &problem.statement {
        let statement = match problem.statement_format {
            Some(StatementFormat::Html) => Cow::Owned(markdown_from_html(statement)?),
            Some(StatementFormat::Markdown) | None => Cow::Borrowed(statement.as_ref()),
        };
        render_markdown_at(&mut output, &statement, presentation, &problem.url)?;
        if let Some(input) = &problem.input_format {
            render_markdown_at(
                &mut output,
                &format!("## Input format\n\n{input}"),
                presentation,
                &problem.url,
            )?;
        }
        if let Some(output_format) = &problem.output_format {
            render_markdown_at(
                &mut output,
                &format!("## Output format\n\n{output_format}"),
                presentation,
                &problem.url,
            )?;
        }
    } else {
        for line in textwrap::wrap(
            &format!(
                "{} did not include a statement in its public API response. The metadata above is the complete terminal view available through that API.",
                problem.platform.label()
            ),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    if let Some(attribution) = &problem.attribution {
        writeln!(output)?;
        for line in textwrap::wrap(&format!("Source: {}", clean(attribution)), columns) {
            writeln!(output, "{line}")?;
        }
    }
    if let Some(license) = &problem.license {
        write!(output, "License: ")?;
        if let Some(url) = &problem.license_url {
            url_link(
                &mut output,
                &clean(license),
                url,
                presentation.color && presentation.interactive,
            )?;
            writeln!(output)?;
        } else {
            writeln!(output, "{}", clean(license))?;
        }
    }
    Ok(())
}

pub(crate) fn catalog_list(
    mut output: impl Write,
    format: Format,
    list: &CatalogProblemList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }

    let (heading, detail) = catalog_list_header(list);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&heading, columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    for line in textwrap::wrap(&detail, columns) {
        writeln!(output, "{line}")?;
    }
    if list.results.is_empty() {
        writeln!(output, "No public problems found.")?;
        return Ok(());
    }
    writeln!(output)?;
    if columns >= 48 {
        catalog_results_table(&mut output, &list.results, presentation)?;
    } else {
        catalog_results_compact(&mut output, &list.results, presentation)?;
    }
    Ok(())
}

pub(crate) fn catalog_list_header(list: &CatalogProblemList) -> (String, String) {
    let shown = list.results.len();
    let count = if list.total as usize > shown {
        format!("{shown} of {} problems", list.total)
    } else if list.total == 1 {
        "1 problem".to_owned()
    } else {
        format!("{} problems", list.total)
    };
    let detail = list.page.map_or_else(
        || {
            if list.platform == CatalogPlatform::ProjectEuler {
                "Newest problems · solve counts include visible profiles only".to_owned()
            } else {
                "Select a problem to open its details".to_owned()
            }
        },
        |page| {
            let kind = if list.platform == CatalogPlatform::ProjectEuler {
                "Catalogue page"
            } else {
                "Page"
            };
            format!("{kind} {page} · select a problem to open its details")
        },
    );
    (
        format!("POSSUM//Browse  {} · {count}", list.platform.label()),
        detail,
    )
}

pub(crate) fn search(
    mut output: impl Write,
    format: Format,
    search: &ProblemSearch,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, search)?;
        writeln!(output)?;
        return Ok(());
    }

    let shown = search.results.len();
    let (heading, detail) = search_header(search);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&heading, columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    for line in textwrap::wrap(&detail, columns) {
        writeln!(output, "{line}")?;
    }

    if search.results.is_empty() {
        writeln!(output)?;
        for line in textwrap::wrap(
            "No problems found. Try a shorter title or problem number.",
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
        return Ok(());
    }

    writeln!(output)?;
    if columns >= 48 {
        results_table(&mut output, &search.results, presentation)?;
    } else {
        results_compact(&mut output, &search.results, presentation)?;
    }

    if search.total > shown as u32 {
        writeln!(output)?;
        let remaining = search.total - shown as u32;
        for line in textwrap::wrap(
            &format!("{remaining} more matches — refine your search to narrow the list."),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

pub(crate) fn list(
    mut output: impl Write,
    format: Format,
    list: &ProblemList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }

    let shown = list.results.len();
    let (heading, detail) = list_header(list);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&heading, columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    for line in textwrap::wrap(&detail, columns) {
        writeln!(output, "{line}")?;
    }

    if list.results.is_empty() {
        writeln!(output)?;
        for line in textwrap::wrap("No problems found. Adjust the difficulty or tag.", columns) {
            writeln!(output, "{line}")?;
        }
        return Ok(());
    }

    writeln!(output)?;
    if columns >= 48 {
        results_table(&mut output, &list.results, presentation)?;
    } else {
        results_compact(&mut output, &list.results, presentation)?;
    }

    if list.total > shown as u32 {
        writeln!(output)?;
        let remaining = list.total - shown as u32;
        for line in textwrap::wrap(
            &format!("{remaining} more problems — add or adjust filters to narrow the list."),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

pub(crate) fn contests(
    mut output: impl Write,
    format: Format,
    list: &ContestList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }

    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    let noun = if list.contests.len() == 1 {
        "contest"
    } else {
        "contests"
    };
    for line in textwrap::wrap(
        &format!("POSSUM//Contests  {} upcoming {noun}", list.contests.len()),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    if list.contests.is_empty() {
        writeln!(output, "No upcoming contests.")?;
        return Ok(());
    }

    writeln!(output)?;
    if columns >= 64 {
        contests_table(&mut output, &list.contests, presentation)?;
    } else {
        contests_compact(&mut output, &list.contests, presentation)?;
    }
    Ok(())
}

pub(crate) fn contest(
    mut output: impl Write,
    format: Format,
    contest: &Contest,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, contest)?;
        writeln!(output)?;
        return Ok(());
    }

    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap("POSSUM//Contest", columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    heading(&mut output, &contest.title, presentation)?;
    let slug = format!("leetcode/contest/{}", contest.id.as_ref());
    let slug_style = result_link_style(presentation, presentation.palette.result_slug);
    for line in textwrap::wrap(&slug, columns) {
        write!(output, "{slug_style}")?;
        contest_link(
            &mut output,
            &line,
            contest.id.as_ref(),
            presentation.color && presentation.interactive,
        )?;
        writeln!(output, "{}", slug_style.render_reset())?;
    }
    for line in textwrap::wrap(
        &format!("Starts: {} UTC", timestamp_utc(contest.start_time)),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    for line in textwrap::wrap(
        &format!("Duration: {}", duration_label(contest.duration_seconds)),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    let mode = if contest.virtual_contest {
        "Virtual"
    } else {
        "Official"
    };
    for line in textwrap::wrap(&format!("Mode: {mode}"), columns) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

fn contests_table(
    output: &mut impl Write,
    contests: &[Contest],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let starts_width = 16;
    let duration_width = 8;
    let content_width = columns - starts_width - duration_width - 9;
    let title_width = (content_width * 2 / 5).max(12);
    let slug_width = content_width - title_width;
    let rule_style = result_style(presentation, presentation.palette.shadow);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let slug_style = result_link_style(presentation, presentation.palette.result_slug);

    write_cell(output, "Starts (UTC)", starts_width, presentation.accent())?;
    write_divider(output, rule_style)?;
    write_cell(output, "Length", duration_width, presentation.accent())?;
    write_divider(output, rule_style)?;
    write_cell(output, "Contest", title_width, title_style)?;
    write_divider(output, rule_style)?;
    write_styled(output, "Slug", slug_style, None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(starts_width),
        "─".repeat(duration_width),
        "─".repeat(title_width),
        "─".repeat(slug_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for contest in contests {
        let starts = timestamp_utc(contest.start_time);
        let duration = duration_label(contest.duration_seconds);
        let title = clean(&contest.title);
        let slug = format!(
            "{}{}",
            contest.id.as_ref(),
            if contest.virtual_contest {
                " · Virtual"
            } else {
                ""
            }
        );
        let title_lines = textwrap::wrap(&title, title_width);
        let slug_lines = textwrap::wrap(&slug, slug_width);
        let height = title_lines.len().max(slug_lines.len());
        for row in 0..height {
            write_cell(
                output,
                if row == 0 { &starts } else { "" },
                starts_width,
                result_style(presentation, presentation.palette.result_number),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                if row == 0 { &duration } else { "" },
                duration_width,
                presentation.accent(),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                title_lines.get(row).map_or("", AsRef::as_ref),
                title_width,
                title_style,
            )?;
            write_divider(output, rule_style)?;
            let slug_line = slug_lines.get(row).map_or("", AsRef::as_ref);
            write!(output, "{slug_style}")?;
            contest_link(
                output,
                slug_line,
                contest.id.as_ref(),
                presentation.color && presentation.interactive,
            )?;
            writeln!(output, "{}", slug_style.render_reset())?;
        }
    }
    Ok(())
}

fn contests_compact(
    output: &mut impl Write,
    contests: &[Contest],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let slug_style = result_link_style(presentation, presentation.palette.result_slug);
    for contest in contests {
        for line in textwrap::wrap(&clean(&contest.title), columns) {
            writeln!(output, "{title_style}{line}{}", title_style.render_reset())?;
        }
        let detail = format!(
            "{} · {}{}",
            timestamp_utc(contest.start_time),
            duration_label(contest.duration_seconds),
            if contest.virtual_contest {
                " · Virtual"
            } else {
                ""
            }
        );
        for line in textwrap::wrap(&detail, columns) {
            writeln!(output, "{line}")?;
        }
        let slug = format!("leetcode/contest/{}", contest.id.as_ref());
        for line in textwrap::wrap(&slug, columns) {
            write!(output, "{slug_style}")?;
            contest_link(
                output,
                &line,
                contest.id.as_ref(),
                presentation.color && presentation.interactive,
            )?;
            writeln!(output, "{}", slug_style.render_reset())?;
        }
    }
    Ok(())
}

pub(crate) fn timestamp_utc(timestamp: u64) -> String {
    let days = (timestamp / 86_400) as i64;
    let day_seconds = timestamp % 86_400;
    let (year, month, day) = civil_date(days);
    let hour = day_seconds / 3_600;
    let minute = day_seconds % 3_600 / 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
}

fn civil_date(days_since_epoch: i64) -> (i64, i64, i64) {
    let days = days_since_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

pub(crate) fn duration_label(seconds: u32) -> String {
    let hours = seconds / 3_600;
    let minutes = seconds % 3_600 / 60;
    match (hours, minutes) {
        (0, minutes) => format!("{minutes}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, minutes) => format!("{hours}h {minutes}m"),
    }
}

pub(crate) fn discussions(
    mut output: impl Write,
    format: Format,
    list: &DiscussionList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }

    let shown = list.discussions.len();
    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    let heading = match (list.problem.as_ref(), list.total) {
        (Some(_), Some(total)) => format!("POSSUM//Discuss  {shown} of {total} posts"),
        (Some(_), None) => format!("POSSUM//Discuss  {shown} posts"),
        (None, _) => format!("POSSUM//Discuss  {shown} trending posts"),
    };
    for line in textwrap::wrap(&heading, columns) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    if let Some(problem) = &list.problem {
        for line in textwrap::wrap(&format!("Problem: leetcode/{}", problem.as_ref()), columns) {
            writeln!(output, "{line}")?;
        }
    }
    if list.discussions.is_empty() {
        writeln!(output, "No discussions found.")?;
        return Ok(());
    }

    writeln!(output)?;
    if columns >= 72 {
        discussions_table(&mut output, &list.discussions, presentation)?;
    } else {
        discussions_compact(&mut output, &list.discussions, presentation)?;
    }
    Ok(())
}

pub(crate) fn discussion(
    mut output: impl Write,
    format: Format,
    discussion: &Discussion,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, discussion)?;
        writeln!(output)?;
        return Ok(());
    }

    let columns = usize::from(presentation.columns);
    let accent = presentation.accent();
    let reset = accent.render_reset();
    for line in textwrap::wrap(
        &format!("POSSUM//Discuss  #{}", discussion.id.get()),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    heading(&mut output, &discussion.title, presentation)?;
    let link = format!("leetcode/discuss/post/{}", discussion.id.get());
    let link_style = result_link_style(presentation, presentation.palette.result_slug);
    for line in textwrap::wrap(&link, columns) {
        write!(output, "{link_style}")?;
        discussion_link(
            &mut output,
            &line,
            discussion.id.get(),
            presentation.color && presentation.interactive,
        )?;
        writeln!(output, "{}", link_style.render_reset())?;
    }
    let author = discussion.author.as_deref().unwrap_or("Deleted user");
    for line in textwrap::wrap(
        &format!(
            "By {} · {} UTC{}",
            clean(author),
            timestamp_utc(discussion.created_at),
            if discussion.pinned { " · Pinned" } else { "" }
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    if let Some(updated_at) = discussion
        .updated_at
        .filter(|updated_at| *updated_at != discussion.created_at)
    {
        for line in textwrap::wrap(
            &format!("Updated: {} UTC", timestamp_utc(updated_at)),
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    for line in textwrap::wrap(
        &format!(
            "{} votes · {} replies · {} views",
            discussion.votes, discussion.comments, discussion.views
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    if !discussion.tags.is_empty() {
        let tags = discussion
            .tags
            .iter()
            .map(|tag| clean(tag))
            .collect::<Vec<_>>()
            .join(" · ");
        for line in textwrap::wrap(&format!("Tags: {tags}"), columns) {
            writeln!(output, "{line}")?;
        }
    }
    writeln!(output)?;
    render_markdown(&mut output, &clean(&discussion.content), presentation)?;
    Ok(())
}

pub(crate) fn community_discussions(
    mut output: impl Write,
    format: Format,
    list: &CommunityDiscussionList,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, list)?;
        writeln!(output)?;
        return Ok(());
    }
    let columns = usize::from(presentation.columns);
    let heading = format!(
        "POSSUM//Discuss  {} {} topics",
        list.discussions.len(),
        clean(&list.platform)
    );
    for line in textwrap::wrap(&heading, columns) {
        writeln!(
            output,
            "{}{line}{}",
            presentation.accent(),
            presentation.accent().render_reset()
        )?;
    }
    if list.discussions.is_empty() {
        writeln!(output, "No discussions found.")?;
        return Ok(());
    }
    writeln!(output)?;
    if columns >= 72 {
        community_discussions_table(&mut output, &list.discussions, presentation)?;
    } else {
        community_discussions_compact(&mut output, &list.discussions, presentation)?;
    }
    Ok(())
}

pub(crate) fn community_discussion(
    mut output: impl Write,
    format: Format,
    discussion: &CommunityDiscussion,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, discussion)?;
        writeln!(output)?;
        return Ok(());
    }
    let columns = usize::from(presentation.columns);
    let summary = &discussion.summary;
    writeln!(
        output,
        "{}POSSUM//Discuss  #{}{}",
        presentation.accent(),
        summary.id.get(),
        presentation.accent().render_reset()
    )?;
    heading(&mut output, &summary.title, presentation)?;
    for line in textwrap::wrap(&summary.url, columns) {
        url_link(
            &mut output,
            &line,
            &summary.url,
            presentation.color && presentation.interactive,
        )?;
        writeln!(output)?;
    }
    let author = clean(summary.author.as_deref().unwrap_or("Unknown author"));
    for line in textwrap::wrap(
        &format!("By {author} · {}", clean(&summary.created_at)),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    for line in textwrap::wrap(
        &format!(
            "{} votes · {} replies · {} views · active {}",
            summary.votes,
            summary.replies,
            summary.views,
            clean(&summary.activity_at)
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    writeln!(output)?;
    let content = match discussion.content_format {
        StatementFormat::Html => Cow::Owned(markdown_from_html(&discussion.content)?),
        StatementFormat::Markdown => Cow::Borrowed(discussion.content.as_ref()),
    };
    render_markdown_at(&mut output, &content, presentation, &summary.url)?;
    Ok(())
}

fn codeforces_contests_table(
    output: &mut impl Write,
    contests: &[CodeforcesContest],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let starts_width = 16;
    let phase_width = 9;
    let path_width = contests
        .iter()
        .map(|contest| contest.id.to_string().len() + 3 + official_path(&contest.url).width())
        .max()
        .unwrap_or(9)
        .clamp(9, 28);
    let fixed = starts_width + phase_width + path_width + 9;
    let title_width = columns.saturating_sub(fixed).max(12);
    let rule_style = result_style(presentation, presentation.palette.shadow);
    for (label, width) in [
        ("Starts (UTC)", starts_width),
        ("Phase", phase_width),
        ("Contest", title_width),
    ] {
        write_cell(output, label, width, presentation.accent())?;
        write_divider(output, rule_style)?;
    }
    write_styled(output, "Id · path", presentation.accent(), None, false)?;
    writeln!(output)?;
    writeln!(
        output,
        "{rule_style}{}─┼─{}─┼─{}─┼─{}{reset}",
        "─".repeat(starts_width),
        "─".repeat(phase_width),
        "─".repeat(title_width),
        "─".repeat(path_width),
        reset = rule_style.render_reset()
    )?;
    for contest in contests {
        let starts = contest
            .start_time_seconds
            .and_then(|value| u64::try_from(value).ok())
            .map_or_else(|| "TBD".to_owned(), timestamp_utc);
        let name = clean(&contest.name);
        let title_lines = textwrap::wrap(&name, title_width);
        let path = format!("{} · {}", contest.id, official_path(&contest.url));
        let path_lines = textwrap::wrap(&path, path_width);
        let height = title_lines.len().max(path_lines.len());
        for row in 0..height {
            write_cell(
                output,
                if row == 0 { &starts } else { "" },
                starts_width,
                result_style(presentation, presentation.palette.result_number),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                if row == 0 {
                    phase_label(&contest.phase)
                } else {
                    ""
                },
                phase_width,
                result_style(presentation, presentation.palette.result_slug),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                title_lines.get(row).map_or("", AsRef::as_ref),
                title_width,
                result_bold(presentation, presentation.palette.result_title),
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                path_lines.get(row).map_or("", AsRef::as_ref),
                result_bold(presentation, presentation.palette.result_number),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn codeforces_submissions_table(
    output: &mut impl Write,
    submissions: &[CodeforcesSubmission],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let verdict_width = 12;
    let language_width = 18;
    let result_width = 18;
    let title_width = columns
        .saturating_sub(verdict_width + language_width + result_width + 9)
        .max(12);
    let rule_style = result_style(presentation, presentation.palette.shadow);
    for (label, width) in [
        ("Status", verdict_width),
        ("Problem", title_width),
        ("Language", language_width),
    ] {
        write_cell(output, label, width, presentation.accent())?;
        write_divider(output, rule_style)?;
    }
    write_styled(output, "Result", presentation.accent(), None, false)?;
    writeln!(output)?;
    writeln!(
        output,
        "{rule_style}{}─┼─{}─┼─{}─┼─{}{reset}",
        "─".repeat(verdict_width),
        "─".repeat(title_width),
        "─".repeat(language_width),
        "─".repeat(result_width),
        reset = rule_style.render_reset()
    )?;
    for submission in submissions.iter().take(10) {
        let verdict = submission
            .verdict
            .as_deref()
            .map_or("Pending", verdict_label);
        let problem = clean(&submission.problem_name);
        let language = clean(&submission.language);
        let result = format!(
            "{} ms · {} KB",
            submission.runtime_millis,
            submission.memory_bytes / 1024
        );
        let verdict_lines = textwrap::wrap(verdict, verdict_width);
        let problem_lines = textwrap::wrap(&problem, title_width);
        let language_lines = textwrap::wrap(&language, language_width);
        let result_lines = textwrap::wrap(&result, result_width);
        let height = verdict_lines
            .len()
            .max(problem_lines.len())
            .max(language_lines.len())
            .max(result_lines.len());
        for row in 0..height {
            write_cell(
                output,
                verdict_lines.get(row).map_or("", AsRef::as_ref),
                verdict_width,
                if submission.verdict.as_deref() == Some("OK") {
                    result_bold(presentation, presentation.palette.easy)
                } else {
                    result_bold(presentation, presentation.palette.hard)
                },
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                problem_lines.get(row).map_or("", AsRef::as_ref),
                title_width,
                result_bold(presentation, presentation.palette.result_title),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                language_lines.get(row).map_or("", AsRef::as_ref),
                language_width,
                result_style(presentation, presentation.palette.result_slug),
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                result_lines.get(row).map_or("", AsRef::as_ref),
                result_style(presentation, presentation.palette.result_number),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn phase_label(phase: &str) -> &str {
    match phase {
        "BEFORE" => "Upcoming",
        "CODING" => "Live",
        "PENDING_SYSTEM_TEST" => "Testing",
        "SYSTEM_TEST" => "Testing",
        "FINISHED" => "Finished",
        other => other,
    }
}

fn verdict_label(verdict: &str) -> &str {
    match verdict {
        "OK" => "Accepted",
        "WRONG_ANSWER" => "Wrong answer",
        "TIME_LIMIT_EXCEEDED" => "Time limit",
        "MEMORY_LIMIT_EXCEEDED" => "Memory limit",
        "COMPILATION_ERROR" => "Compile error",
        other => other,
    }
}

fn community_discussions_table(
    output: &mut impl Write,
    discussions: &[CommunityDiscussionSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let id_width = discussions
        .iter()
        .map(|discussion| discussion.id.get().to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let author_width = 16;
    let activity_width = 20;
    let fixed_width = id_width + author_width + activity_width + 9;
    if columns < fixed_width + 12 {
        return community_discussions_compact(output, discussions, presentation);
    }
    let title_width = columns - fixed_width;
    let rule_style = result_style(presentation, presentation.palette.shadow);
    let id_style = result_bold(presentation, presentation.palette.result_number);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let author_style = result_style(presentation, presentation.palette.result_slug);
    for (label, width, style) in [
        ("Id", id_width, presentation.accent()),
        ("Discussion", title_width, title_style),
        ("Author", author_width, author_style),
    ] {
        write_cell(output, label, width, style)?;
        write_divider(output, rule_style)?;
    }
    write_styled(output, "Activity", presentation.accent(), None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(id_width),
        "─".repeat(title_width),
        "─".repeat(author_width),
        "─".repeat(activity_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;
    for discussion in discussions {
        let id = discussion.id.get().to_string();
        let title = clean(&discussion.title);
        let author = clean(discussion.author.as_deref().unwrap_or("Unknown author"));
        let activity = format!(
            "{} votes · {} replies · {} views",
            discussion.votes, discussion.replies, discussion.views
        );
        let title_lines = textwrap::wrap(&title, title_width);
        let author_lines = textwrap::wrap(&author, author_width);
        let activity_lines = textwrap::wrap(&activity, activity_width);
        let height = title_lines
            .len()
            .max(author_lines.len())
            .max(activity_lines.len());
        for row in 0..height {
            write_cell(output, if row == 0 { &id } else { "" }, id_width, id_style)?;
            write_divider(output, rule_style)?;
            let title_line = title_lines.get(row).map_or("", AsRef::as_ref);
            write!(output, "{title_style}")?;
            url_link(
                output,
                title_line,
                &discussion.url,
                presentation.color && presentation.interactive,
            )?;
            write!(
                output,
                "{}{}",
                " ".repeat(title_width.saturating_sub(title_line.width())),
                title_style.render_reset()
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                author_lines.get(row).map_or("", AsRef::as_ref),
                author_width,
                author_style,
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                activity_lines.get(row).map_or("", AsRef::as_ref),
                presentation.accent(),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn community_discussions_compact(
    output: &mut impl Write,
    discussions: &[CommunityDiscussionSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    for discussion in discussions {
        let title = clean(&discussion.title);
        for line in textwrap::wrap(&title, columns) {
            url_link(
                output,
                &line,
                &discussion.url,
                presentation.color && presentation.interactive,
            )?;
            writeln!(output)?;
        }
        let detail = format!(
            "#{} · {} · {} replies · {} views",
            discussion.id.get(),
            clean(discussion.author.as_deref().unwrap_or("Unknown author")),
            discussion.replies,
            discussion.views
        );
        for line in textwrap::wrap(&detail, columns) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

fn discussions_table(
    output: &mut impl Write,
    discussions: &[DiscussionSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let id_width = discussions
        .iter()
        .map(|discussion| discussion.id.get().to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let author_width = 16;
    let activity_width = 20;
    let fixed_width = id_width + author_width + activity_width + 9;
    if columns < fixed_width + 12 {
        return discussions_compact(output, discussions, presentation);
    }
    let title_width = columns - fixed_width;
    let rule_style = result_style(presentation, presentation.palette.shadow);
    let id_style = result_bold(presentation, presentation.palette.result_number);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let author_style = result_style(presentation, presentation.palette.result_slug);

    write_cell(output, "Id", id_width, presentation.accent())?;
    write_divider(output, rule_style)?;
    write_cell(output, "Discussion", title_width, title_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Author", author_width, author_style)?;
    write_divider(output, rule_style)?;
    write_styled(output, "Activity", presentation.accent(), None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(id_width),
        "─".repeat(title_width),
        "─".repeat(author_width),
        "─".repeat(activity_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for discussion in discussions {
        let id = discussion.id.get().to_string();
        let title = clean(&discussion.title);
        let author = clean(discussion.author.as_deref().unwrap_or("Deleted user"));
        let activity = format!(
            "{} votes · {} replies · {} views",
            discussion.votes, discussion.comments, discussion.views
        );
        let title_lines = textwrap::wrap(&title, title_width);
        let author_lines = textwrap::wrap(&author, author_width);
        let activity_lines = textwrap::wrap(&activity, activity_width);
        let height = title_lines
            .len()
            .max(author_lines.len())
            .max(activity_lines.len());
        for row in 0..height {
            write_cell(output, if row == 0 { &id } else { "" }, id_width, id_style)?;
            write_divider(output, rule_style)?;
            let title_line = title_lines.get(row).map_or("", AsRef::as_ref);
            write!(output, "{title_style}")?;
            discussion_link(
                output,
                title_line,
                discussion.id.get(),
                presentation.color && presentation.interactive,
            )?;
            write!(
                output,
                "{}{}",
                " ".repeat(title_width.saturating_sub(title_line.width())),
                title_style.render_reset()
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                author_lines.get(row).map_or("", AsRef::as_ref),
                author_width,
                author_style,
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                activity_lines.get(row).map_or("", AsRef::as_ref),
                presentation.accent(),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn discussions_compact(
    output: &mut impl Write,
    discussions: &[DiscussionSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    for discussion in discussions {
        let title = clean(&discussion.title);
        for line in textwrap::wrap(&title, columns) {
            write!(output, "{title_style}")?;
            discussion_link(
                output,
                &line,
                discussion.id.get(),
                presentation.color && presentation.interactive,
            )?;
            writeln!(output, "{}", title_style.render_reset())?;
        }
        let author = clean(discussion.author.as_deref().unwrap_or("Deleted user"));
        let detail = format!(
            "#{} · {author} · {} votes · {} replies · {} views",
            discussion.id.get(),
            discussion.votes,
            discussion.comments,
            discussion.views
        );
        for line in textwrap::wrap(&detail, columns) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

pub(crate) fn search_header(search: &ProblemSearch) -> (String, String) {
    let shown = search.results.len();
    let count = if search.total == 1 {
        "1 match".to_owned()
    } else if search.total > shown as u32 {
        format!("{shown} of {} matches", search.total)
    } else {
        format!("{} matches", search.total)
    };
    (
        format!("POSSUM//SCAN  {count}"),
        format!("Search: {}", search.query.as_ref()),
    )
}

pub(crate) fn list_header(list: &ProblemList) -> (String, String) {
    let shown = list.results.len();
    let count = if list.total == 1 {
        "1 problem".to_owned()
    } else if list.total > shown as u32 {
        format!("{shown} of {} problems", list.total)
    } else {
        format!("{} problems", list.total)
    };
    let filters = match (&list.difficulty, &list.tag) {
        (Some(difficulty), Some(tag)) => {
            format!("{} · {}", difficulty.label(), tag.as_ref())
        }
        (Some(difficulty), None) => difficulty.label().to_owned(),
        (None, Some(tag)) => tag.as_ref().to_owned(),
        (None, None) => "all public problems".to_owned(),
    };
    (
        format!("POSSUM//BROWSE  {count}"),
        format!("Filters: {filters}"),
    )
}

pub(crate) fn auth_status(
    mut output: impl Write,
    format: Format,
    status: &AuthStatus,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, status)?;
        writeln!(output)?;
        return Ok(());
    }

    let source = match status.source {
        Some(AuthSource::Environment) => "environment",
        Some(AuthSource::SecureStore) => "saved",
        None => "configured",
    };
    let (label, detail, color) = match (status.configured, status.authenticated) {
        (true, true) => (
            "READY",
            format!("LeetCode accepted the {source} session."),
            presentation.palette.easy,
        ),
        (true, false) => (
            "EXPIRED",
            format!("LeetCode did not accept the {source} session."),
            presentation.palette.hard,
        ),
        (false, _) => (
            "NOT CONFIGURED",
            "Set LEETCODE_SESSION and LEETCODE_CSRFTOKEN together; LeetCode does not expose a terminal-native login flow."
                .to_owned(),
            presentation.palette.medium,
        ),
    };
    let style = result_bold(presentation, color);
    let reset = style.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&format!("POSSUM//AUTH  {label}"), columns) {
        writeln!(output, "{style}{line}{reset}")?;
    }
    for line in textwrap::wrap(&detail, columns) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

pub(crate) fn auth_logout(
    mut output: impl Write,
    format: Format,
    logout: &AuthLogout,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, logout)?;
        writeln!(output)?;
        return Ok(());
    }

    let (label, detail) = if logout.removed {
        (
            "SAVED SESSION REMOVED",
            "The saved LeetCode session was removed from secure storage.",
        )
    } else {
        (
            "NO SAVED SESSION",
            "There was no saved LeetCode session in secure storage.",
        )
    };
    let style = result_bold(presentation, presentation.palette.easy);
    let reset = style.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&format!("POSSUM//AUTH  {label}"), columns) {
        writeln!(output, "{style}{line}{reset}")?;
    }
    for line in textwrap::wrap(detail, columns) {
        writeln!(output, "{line}")?;
    }
    if logout.environment_present {
        for line in textwrap::wrap(
            "LeetCode credential variables are still present in this shell; unset them separately.",
            columns,
        ) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

pub(crate) fn stats(
    mut output: impl Write,
    format: Format,
    stats: &AccountStats,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, stats)?;
        writeln!(output)?;
        return Ok(());
    }

    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(
        &format!("POSSUM//Stats  {}", clean(&stats.username)),
        columns,
    ) {
        writeln!(output, "{accent}{line}{reset}")?;
    }
    writeln!(output)?;
    if columns >= 48 {
        stats_table(&mut output, stats, presentation)?;
    } else {
        stats_compact(&mut output, stats, columns)?;
    }

    writeln!(output)?;
    writeln!(output, "{accent}Recent submissions{reset}")?;
    if stats.recent_submissions.is_empty() {
        writeln!(output, "No submissions yet.")?;
    } else if columns >= 72 {
        submissions_table(&mut output, &stats.recent_submissions, presentation)?;
    } else {
        submissions_compact(&mut output, &stats.recent_submissions, presentation)?;
    }
    if stats.has_more_submissions {
        writeln!(output)?;
        for line in textwrap::wrap("Showing the 10 most recent submissions.", columns) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

fn stats_table(
    output: &mut impl Write,
    stats: &AccountStats,
    presentation: Presentation,
) -> std::io::Result<()> {
    let level_width = 6;
    let solved_width = count_width("Solved", stats.solved);
    let accepted_width = count_width("Accepted", stats.accepted_submissions);
    let attempts_width = count_width("Attempts", stats.submissions);
    let rule_style = result_style(presentation, presentation.palette.shadow);
    let header_style = presentation.accent();

    write_cell(output, "Level", level_width, header_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Solved", solved_width, header_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Accepted", accepted_width, header_style)?;
    write_divider(output, rule_style)?;
    write_styled(output, "Attempts", header_style, None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(level_width),
        "─".repeat(solved_width),
        "─".repeat(accepted_width),
        "─".repeat(attempts_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for (label, solved, accepted, attempts, style) in [
        (
            "All",
            stats.solved.all,
            stats.accepted_submissions.all,
            stats.submissions.all,
            presentation.accent(),
        ),
        (
            "Easy",
            stats.solved.easy,
            stats.accepted_submissions.easy,
            stats.submissions.easy,
            difficulty_style(&crate::domain::Difficulty::Easy, presentation),
        ),
        (
            "Medium",
            stats.solved.medium,
            stats.accepted_submissions.medium,
            stats.submissions.medium,
            difficulty_style(&crate::domain::Difficulty::Medium, presentation),
        ),
        (
            "Hard",
            stats.solved.hard,
            stats.accepted_submissions.hard,
            stats.submissions.hard,
            difficulty_style(&crate::domain::Difficulty::Hard, presentation),
        ),
    ] {
        write_cell(output, label, level_width, style)?;
        write_divider(output, rule_style)?;
        write_cell(output, &solved.to_string(), solved_width, style)?;
        write_divider(output, rule_style)?;
        write_cell(output, &accepted.to_string(), accepted_width, style)?;
        write_divider(output, rule_style)?;
        write_styled(output, &attempts.to_string(), style, None, false)?;
        writeln!(output)?;
    }
    Ok(())
}

fn count_width(label: &str, counts: DifficultyCounts) -> usize {
    [counts.all, counts.easy, counts.medium, counts.hard]
        .into_iter()
        .map(|count| count.to_string().len())
        .max()
        .unwrap_or(1)
        .max(label.len())
}

fn stats_compact(
    output: &mut impl Write,
    stats: &AccountStats,
    columns: usize,
) -> std::io::Result<()> {
    for line in textwrap::wrap(
        &format!(
            "Solved: {} total · {} easy · {} medium · {} hard",
            stats.solved.all, stats.solved.easy, stats.solved.medium, stats.solved.hard
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    for line in textwrap::wrap(
        &format!(
            "Submissions: {} accepted · {} attempts",
            stats.accepted_submissions.all, stats.submissions.all
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

fn submissions_table(
    output: &mut impl Write,
    submissions: &[RecentSubmission],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let id_width = submissions
        .iter()
        .map(|submission| submission.id.to_string().len())
        .max()
        .unwrap_or(2)
        .max(2);
    let status_width = 12;
    let language_width = 9;
    let result_width = 17;
    let fixed_width = id_width + status_width + language_width + result_width + 12;
    if columns < fixed_width + 7 {
        return submissions_compact(output, submissions, presentation);
    }
    let title_width = columns - fixed_width;
    let rule_style = result_style(presentation, presentation.palette.shadow);
    let header_style = presentation.accent();

    write_cell(output, "Id", id_width, header_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Status", status_width, header_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Problem", title_width, header_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Language", language_width, header_style)?;
    write_divider(output, rule_style)?;
    write_styled(output, "Result", header_style, None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(id_width),
        "─".repeat(status_width),
        "─".repeat(title_width),
        "─".repeat(language_width),
        "─".repeat(result_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for submission in submissions {
        let id = submission.id.to_string();
        let status = clean(&submission.status);
        let title = clean(&submission.title);
        let language = clean(&submission.language);
        let result = submission_result(submission);
        let status_lines = textwrap::wrap(&status, status_width);
        let title_lines = textwrap::wrap(&title, title_width);
        let language_lines = textwrap::wrap(&language, language_width);
        let result_lines = textwrap::wrap(&result, result_width);
        let height = status_lines
            .len()
            .max(title_lines.len())
            .max(language_lines.len())
            .max(result_lines.len());
        let status_style = submission_style(submission, presentation);
        let title_style = result_bold(presentation, presentation.palette.result_title);
        for row in 0..height {
            write_cell(
                output,
                if row == 0 { &id } else { "" },
                id_width,
                result_bold(presentation, presentation.palette.result_number),
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                status_lines.get(row).map_or("", AsRef::as_ref),
                status_width,
                status_style,
            )?;
            write_divider(output, rule_style)?;
            let title_line = title_lines.get(row).map_or("", AsRef::as_ref);
            write_styled(
                output,
                title_line,
                title_style,
                Some(submission.problem.as_ref()),
                presentation.color && presentation.interactive,
            )?;
            write!(
                output,
                "{}",
                " ".repeat(title_width.saturating_sub(title_line.width()))
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                language_lines.get(row).map_or("", AsRef::as_ref),
                language_width,
                result_style(presentation, presentation.palette.result_slug),
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                result_lines.get(row).map_or("", AsRef::as_ref),
                result_style(presentation, presentation.palette.result_slug),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn submissions_compact(
    output: &mut impl Write,
    submissions: &[RecentSubmission],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    for submission in submissions {
        let status = clean(&submission.status);
        let title = clean(&submission.title);
        let style = submission_style(submission, presentation);
        for line in textwrap::wrap(&format!("{status} · {title}"), columns) {
            writeln!(output, "{style}{line}{}", style.render_reset())?;
        }
        let detail = format!(
            "#{} · {} · {}",
            submission.id,
            clean(&submission.language),
            submission_result(submission)
        );
        let indent = if columns > 2 { "  " } else { "" };
        for line in textwrap::wrap(&detail, columns.saturating_sub(indent.len()).max(1)) {
            writeln!(output, "{indent}{line}")?;
        }
    }
    Ok(())
}

fn submission_result(submission: &RecentSubmission) -> String {
    match (&submission.runtime, &submission.memory) {
        (Some(runtime), Some(memory)) => format!("{} · {}", clean(runtime), clean(memory)),
        (Some(runtime), None) => clean(runtime),
        (None, Some(memory)) => clean(memory),
        (None, None) if submission.pending => "Pending".to_owned(),
        (None, None) => "—".to_owned(),
    }
}

fn submission_style(submission: &RecentSubmission, presentation: Presentation) -> anstyle::Style {
    let color = if submission.pending {
        presentation.palette.medium
    } else if submission.status.eq_ignore_ascii_case("accepted") {
        presentation.palette.easy
    } else {
        presentation.palette.hard
    };
    result_bold(presentation, color)
}

pub(crate) fn solution_file(
    mut output: impl Write,
    format: Format,
    solution: &SolutionFile,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, solution)?;
        writeln!(output)?;
        return Ok(());
    }

    let accent = presentation.accent();
    let reset = accent.render_reset();
    let columns = usize::from(presentation.columns);
    writeln!(output, "{accent}POSSUM//PICKED{reset}")?;
    for line in textwrap::wrap(
        &format!("{} · {}", clean(&solution.title), clean(&solution.language)),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    let path = clean(&solution.path.display().to_string());
    for line in textwrap::wrap(&format!("Solution: {path}"), columns) {
        writeln!(output, "{line}")?;
    }
    for line in textwrap::wrap(
        &format!(
            "Next: cp-cli --platform {} problem test {path}",
            clean(&solution.platform)
        ),
        columns,
    ) {
        writeln!(output, "{line}")?;
    }
    Ok(())
}

pub(crate) fn test(
    mut output: impl Write,
    format: Format,
    test: &TestResult,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, test)?;
        writeln!(output)?;
        return Ok(());
    }
    let color = if test.passed {
        presentation.palette.easy
    } else {
        presentation.palette.hard
    };
    let style = result_bold(presentation, color);
    let reset = style.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(&format!("POSSUM//TEST  {}", clean(&test.status)), columns) {
        writeln!(output, "{style}{line}{reset}")?;
    }
    writeln!(output, "Problem: {} · LeetCode examples", test.id.as_ref())?;
    writeln!(output, "Run: {}", clean(&test.run_id))?;
    if let (Some(passed), Some(total)) = (test.passed_cases, test.total_cases) {
        writeln!(output, "Cases: {passed}/{total}")?;
    }
    if let Some(runtime) = &test.runtime {
        writeln!(output, "Runtime: {}", clean(runtime))?;
    }
    if let Some(memory) = &test.memory {
        writeln!(output, "Memory: {}", clean(memory))?;
    }
    if !test.passed {
        for (label, value) in [
            ("Input", test.input.as_deref()),
            ("Expected", test.expected.as_deref()),
            ("Output", test.output.as_deref()),
        ] {
            if let Some(value) = value {
                labeled_block(&mut output, label, value, columns)?;
            }
        }
    }
    if let Some(message) = &test.message {
        labeled_block(&mut output, "Detail", message, columns)?;
    }
    Ok(())
}

fn labeled_block(
    output: &mut impl Write,
    label: &str,
    value: &str,
    columns: usize,
) -> std::io::Result<()> {
    writeln!(output, "{label}:")?;
    for source_line in clean(value).lines() {
        for line in textwrap::wrap(source_line, columns.saturating_sub(2).max(1)) {
            writeln!(output, "  {line}")?;
        }
    }
    Ok(())
}

pub(crate) fn submission(
    mut output: impl Write,
    format: Format,
    submission: &SubmissionResult,
    presentation: Presentation,
) -> Result<()> {
    if matches!(format, Format::Json) {
        serde_json::to_writer(&mut output, submission)?;
        writeln!(output)?;
        return Ok(());
    }

    let color = if submission.accepted {
        presentation.palette.easy
    } else {
        presentation.palette.hard
    };
    let style = result_bold(presentation, color);
    let reset = style.render_reset();
    let columns = usize::from(presentation.columns);
    for line in textwrap::wrap(
        &format!("POSSUM//JUDGE  {}", clean(&submission.status)),
        columns,
    ) {
        writeln!(output, "{style}{line}{reset}")?;
    }
    writeln!(output, "Submission: {}", submission.id)?;
    if let (Some(passed), Some(total)) = (submission.passed, submission.total) {
        writeln!(output, "Cases: {passed}/{total}")?;
    }
    if let Some(runtime) = &submission.runtime {
        writeln!(output, "Runtime: {}", clean(runtime))?;
    }
    if let Some(memory) = &submission.memory {
        writeln!(output, "Memory: {}", clean(memory))?;
    }
    if let Some(message) = &submission.message {
        for line in textwrap::wrap(&format!("Detail: {}", clean(message)), columns) {
            writeln!(output, "{line}")?;
        }
    }
    Ok(())
}

fn results_table(
    output: &mut impl Write,
    results: &[ProblemSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let number_width = results
        .iter()
        .map(|problem| problem.number.to_string().width())
        .max()
        .unwrap_or(1)
        .max(1);
    let level_width = 8;
    let content_width = columns - number_width - level_width - 9;
    let title_width = (content_width * 2 / 5).max(10);
    let slug_width = content_width - title_width;
    let number_style = result_bold(presentation, presentation.palette.result_number);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let slug_style = result_link_style(presentation, presentation.palette.result_slug);
    let rule_style = result_style(presentation, presentation.palette.shadow);

    write_cell(output, "#", number_width, number_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "PROBLEM", title_width, title_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "LEVEL", level_width, presentation.accent())?;
    write_divider(output, rule_style)?;
    write_styled(output, "SLUG", slug_style, None, false)?;
    writeln!(output)?;

    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(number_width),
        "─".repeat(title_width),
        "─".repeat(level_width),
        "─".repeat(slug_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for problem in results {
        let number = problem.number.to_string();
        let title = clean(problem.title.as_ref());
        let badge = format!("[{:^6}]", problem.difficulty.label());
        let slug = format!(
            "leetcode/{}{}",
            problem.id.as_ref(),
            if problem.paid_only { " · PREMIUM" } else { "" }
        );
        let title_lines = textwrap::wrap(&title, title_width);
        let slug_lines = textwrap::wrap(&slug, slug_width);
        let row_height = title_lines.len().max(slug_lines.len());
        let level_style = difficulty_style(&problem.difficulty, presentation);

        for row in 0..row_height {
            write_cell(
                output,
                if row == 0 { &number } else { "" },
                number_width,
                number_style,
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                title_lines.get(row).map_or("", AsRef::as_ref),
                title_width,
                title_style,
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                if row == 0 { &badge } else { "" },
                level_width,
                level_style,
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                slug_lines.get(row).map_or("", AsRef::as_ref),
                slug_style,
                Some(problem.id.as_ref()),
                presentation.color && presentation.interactive,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn catalog_results_table(
    output: &mut impl Write,
    results: &[CatalogProblemSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let id_width = results
        .iter()
        .map(|problem| problem.id.as_ref().width())
        .max()
        .unwrap_or(2)
        .clamp(2, 14);
    let level_width = 8;
    let content_width = columns.saturating_sub(id_width + level_width + 9);
    let title_width = (content_width * 3 / 5).max(7).min(content_width);
    let tags_width = content_width.saturating_sub(title_width).max(1);
    let id_style = result_link_style(presentation, presentation.palette.result_slug);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let rule_style = result_style(presentation, presentation.palette.shadow);

    write_cell(output, "Id", id_width, id_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Problem", title_width, title_style)?;
    write_divider(output, rule_style)?;
    write_cell(output, "Level", level_width, presentation.accent())?;
    write_divider(output, rule_style)?;
    write_styled(output, "Path · details", presentation.accent(), None, false)?;
    writeln!(output)?;
    let rule = format!(
        "{}─┼─{}─┼─{}─┼─{}",
        "─".repeat(id_width),
        "─".repeat(title_width),
        "─".repeat(level_width),
        "─".repeat(tags_width),
    );
    writeln!(output, "{rule_style}{rule}{}", rule_style.render_reset())?;

    for problem in results {
        let id = clean(problem.id.as_ref());
        let title = clean(&math::inline_text(&problem.title));
        let (level, level_style) = catalog_level(problem, presentation);
        let tags = catalog_context(problem);
        let id_lines = textwrap::wrap(&id, id_width);
        let title_lines = textwrap::wrap(&title, title_width);
        let tags_lines = textwrap::wrap(if tags.is_empty() { "—" } else { &tags }, tags_width);
        let height = id_lines.len().max(title_lines.len()).max(tags_lines.len());
        for row in 0..height {
            let id_line = id_lines.get(row).map_or("", AsRef::as_ref);
            write!(output, "{id_style}")?;
            url_link(
                output,
                id_line,
                &problem.url,
                presentation.color && presentation.interactive,
            )?;
            write!(output, "{}", id_style.render_reset())?;
            write!(
                output,
                "{}",
                " ".repeat(id_width.saturating_sub(id_line.width()))
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                title_lines.get(row).map_or("", AsRef::as_ref),
                title_width,
                title_style,
            )?;
            write_divider(output, rule_style)?;
            write_cell(
                output,
                if row == 0 { &level } else { "" },
                level_width,
                level_style,
            )?;
            write_divider(output, rule_style)?;
            write_styled(
                output,
                tags_lines.get(row).map_or("", AsRef::as_ref),
                result_style(presentation, presentation.palette.result_slug),
                None,
                false,
            )?;
            writeln!(output)?;
        }
    }
    Ok(())
}

fn catalog_results_compact(
    output: &mut impl Write,
    results: &[CatalogProblemSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let id_style = result_link_style(presentation, presentation.palette.result_slug);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    for problem in results {
        for line in textwrap::wrap(
            &format!(
                "{}  {}",
                problem.id.as_ref(),
                clean(&math::inline_text(&problem.title))
            ),
            columns,
        ) {
            write!(output, "{title_style}")?;
            url_link(
                output,
                &line,
                &problem.url,
                presentation.color && presentation.interactive,
            )?;
            writeln!(output, "{}", title_style.render_reset())?;
        }
        let (level, level_style) = catalog_level(problem, presentation);
        write_styled(output, &level, level_style, None, false)?;
        let context = catalog_context(problem);
        if !context.is_empty() {
            write!(output, "  {id_style}")?;
            for line in textwrap::wrap(&context, columns.saturating_sub(level.width() + 2).max(1)) {
                write!(output, "{line}")?;
            }
            write!(output, "{}", id_style.render_reset())?;
        }
        writeln!(output)?;
    }
    Ok(())
}

fn catalog_level(
    problem: &CatalogProblemSummary,
    presentation: Presentation,
) -> (String, anstyle::Style) {
    if let Some(level) = &problem.level {
        return (
            clean(level),
            result_bold(presentation, presentation.palette.result_number),
        );
    }
    if let Some(difficulty) = problem.difficulty {
        return (
            format!("[{:^6}]", difficulty.label()),
            difficulty_style(&difficulty, presentation),
        );
    }
    if let Some(rating) = problem.rating {
        return (
            rating.to_string(),
            result_bold(presentation, presentation.palette.result_number),
        );
    }
    (
        "—".to_owned(),
        result_style(presentation, presentation.palette.shadow),
    )
}

fn catalog_context(problem: &CatalogProblemSummary) -> String {
    let context = if problem.tags.is_empty() {
        problem.solved_count.map_or_else(
            || {
                problem
                    .published_at
                    .as_deref()
                    .map_or_else(String::new, |published| format!("Published {published}"))
            },
            |solved| format!("{solved} solved"),
        )
    } else {
        clean(
            &problem
                .tags
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join(" · "),
        )
    };
    let path = official_path(&problem.url);
    if context.is_empty() {
        path
    } else {
        format!("{path} · {context}")
    }
}

fn results_compact(
    output: &mut impl Write,
    results: &[ProblemSummary],
    presentation: Presentation,
) -> std::io::Result<()> {
    let columns = usize::from(presentation.columns);
    let number_style = result_bold(presentation, presentation.palette.result_number);
    let title_style = result_bold(presentation, presentation.palette.result_title);
    let slug_style = result_link_style(presentation, presentation.palette.result_slug);

    for problem in results {
        let number = problem.number.to_string();
        let title = clean(problem.title.as_ref());
        let title_indent = number.width() + 2;
        if title_indent < columns {
            write_styled(output, &number, number_style, None, false)?;
            write!(output, "  ")?;
            for (row, line) in textwrap::wrap(&title, columns - title_indent)
                .into_iter()
                .enumerate()
            {
                if row > 0 {
                    write!(output, "{}", " ".repeat(title_indent))?;
                }
                write_styled(output, &line, title_style, None, false)?;
                writeln!(output)?;
            }
        } else {
            for line in textwrap::wrap(&number, columns) {
                writeln!(
                    output,
                    "{number_style}{line}{}",
                    number_style.render_reset()
                )?;
            }
            for line in textwrap::wrap(&title, columns) {
                writeln!(output, "{title_style}{line}{}", title_style.render_reset())?;
            }
        }

        let badge = format!("[{:^6}]", problem.difficulty.label());
        let level_style = difficulty_style(&problem.difficulty, presentation);
        let slug = format!(
            "leetcode/{}{}",
            problem.id.as_ref(),
            if problem.paid_only { " · PREMIUM" } else { "" }
        );
        let detail_indent = badge.width() + 2;
        if detail_indent < columns {
            write_styled(output, &badge, level_style, None, false)?;
            write!(output, "  ")?;
            for (row, line) in textwrap::wrap(&slug, columns - detail_indent)
                .into_iter()
                .enumerate()
            {
                if row > 0 {
                    write!(output, "{}", " ".repeat(detail_indent))?;
                }
                write_styled(
                    output,
                    &line,
                    slug_style,
                    Some(problem.id.as_ref()),
                    presentation.color && presentation.interactive,
                )?;
                writeln!(output)?;
            }
        } else {
            for line in textwrap::wrap(&badge, columns) {
                writeln!(output, "{level_style}{line}{}", level_style.render_reset())?;
            }
            for line in textwrap::wrap(&slug, columns) {
                write_styled(
                    output,
                    &line,
                    slug_style,
                    Some(problem.id.as_ref()),
                    presentation.color && presentation.interactive,
                )?;
                writeln!(output)?;
            }
        }
    }
    Ok(())
}

fn result_style(presentation: Presentation, color: u8) -> anstyle::Style {
    if presentation.color {
        crate::theme::foreground(color)
    } else {
        anstyle::Style::new()
    }
}

fn result_bold(presentation: Presentation, color: u8) -> anstyle::Style {
    if presentation.color {
        crate::theme::foreground(color).bold()
    } else {
        anstyle::Style::new()
    }
}

fn result_link_style(presentation: Presentation, color: u8) -> anstyle::Style {
    if presentation.color {
        crate::theme::foreground(color).underline()
    } else {
        anstyle::Style::new()
    }
}

fn difficulty_style(
    difficulty: &crate::domain::Difficulty,
    presentation: Presentation,
) -> anstyle::Style {
    let color = match difficulty {
        crate::domain::Difficulty::Easy => presentation.palette.easy,
        crate::domain::Difficulty::Medium => presentation.palette.medium,
        crate::domain::Difficulty::Hard => presentation.palette.hard,
    };
    result_bold(presentation, color)
}

fn write_cell(
    output: &mut impl Write,
    text: &str,
    width: usize,
    style: anstyle::Style,
) -> std::io::Result<()> {
    write_styled(output, text, style, None, false)?;
    write!(output, "{}", " ".repeat(width.saturating_sub(text.width())))
}

fn write_divider(output: &mut impl Write, style: anstyle::Style) -> std::io::Result<()> {
    write!(output, " {style}│{} ", style.render_reset())
}

fn write_styled(
    output: &mut impl Write,
    text: &str,
    style: anstyle::Style,
    link: Option<&str>,
    links: bool,
) -> std::io::Result<()> {
    if text.is_empty() {
        return Ok(());
    }
    write!(output, "{style}")?;
    if let Some(id) = link {
        problem_link(output, text, id, links)?;
    } else {
        write!(output, "{text}")?;
    }
    write!(output, "{}", style.render_reset())
}

fn hud(
    output: &mut impl Write,
    number: u32,
    id: &str,
    presentation: Presentation,
) -> std::io::Result<()> {
    if !presentation.interactive || presentation.columns < 16 {
        let accent = presentation.accent();
        let reset = accent.render_reset();
        for line in textwrap::wrap(
            &format!("#{number} · leetcode/{id}"),
            usize::from(presentation.columns),
        ) {
            writeln!(output, "{accent}{line}{reset}")?;
        }
        return Ok(());
    }
    let art = if presentation.motion {
        crate::possum::render_alive(
            crate::possum::Pose::Scratch,
            presentation.columns,
            presentation.color,
        )
    } else {
        crate::possum::render(
            crate::possum::Pose::Cursor,
            presentation.columns,
            presentation.color,
        )
    };
    if let Some(art) = art {
        writeln!(output, "{art}")?;
    }
    let accent = presentation.accent();
    let reset = accent.render_reset();
    if presentation.columns >= 32 {
        let prefix = format!("POSSUM//ARCADE  QUEST #{number} · ");
        write!(output, "{accent}{prefix}{reset}")?;
        for (index, line) in textwrap::wrap(
            &format!("leetcode/{id}"),
            usize::from(presentation.columns.saturating_sub(prefix.width() as u16)).max(1),
        )
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                write!(output, "\n{accent}{}{reset}", " ".repeat(prefix.width()))?;
            }
            problem_link(output, &line, id, presentation.color)?;
        }
        writeln!(output)?;
    } else {
        let metadata = format!("POSSUM// #{number} · leetcode/{id}");
        for line in textwrap::wrap(&metadata, usize::from(presentation.columns)) {
            write!(output, "{accent}")?;
            problem_link(output, &line, id, presentation.color)?;
            writeln!(output, "{reset}")?;
        }
    }
    Ok(())
}

fn problem_header(
    output: &mut impl Write,
    problem: &Problem,
    presentation: Presentation,
) -> std::io::Result<()> {
    let art_width = 28;
    let text_column = art_width + 3;
    let text_width = usize::from(presentation.columns).saturating_sub(text_column - 1);
    let title: String = problem
        .title
        .chars()
        .filter(|character| !character.is_control())
        .collect();
    let prefix = format!("POSSUM//ARCADE  QUEST #{} · ", problem.number);
    let slug = format!("leetcode/{}", problem.id.as_ref());
    let art = if presentation.interactive
        && presentation.color
        && !presentation.large_heading
        && prefix.width() + slug.width() <= text_width
        && title.width() <= text_width
    {
        if presentation.motion {
            crate::possum::render_alive(
                crate::possum::Pose::Scratch,
                art_width as u16,
                presentation.color,
            )
        } else {
            crate::possum::render(
                crate::possum::Pose::Cursor,
                art_width as u16,
                presentation.color,
            )
        }
    } else {
        None
    };

    if let Some(art) = art {
        let text_row = art.lines().count().saturating_sub(3) / 2;
        for (row, line) in art.lines().enumerate() {
            write!(output, "{line}")?;
            if (text_row..text_row + 3).contains(&row) {
                write!(output, "\x1b[{text_column}G")?;
            }
            match row.saturating_sub(text_row) {
                0 if row >= text_row => {
                    let accent = presentation.accent();
                    write!(output, "{accent}{prefix}{}", accent.render_reset())?;
                    problem_link(output, &slug, problem.id.as_ref(), presentation.interactive)?;
                }
                1 if row >= text_row => title_line(output, &title, presentation)?,
                2 if row >= text_row => title_rule(output, title.width(), presentation)?,
                _ => {}
            }
            writeln!(output)?;
        }
        return writeln!(output);
    }

    hud(output, problem.number, problem.id.as_ref(), presentation)?;
    heading(output, &problem.title, presentation)
}

fn problem_link(
    output: &mut impl Write,
    text: &str,
    id: &str,
    enabled: bool,
) -> std::io::Result<()> {
    if enabled {
        write!(
            output,
            "\x1b]8;;https://leetcode.com/problems/{id}/\x1b\\{text}\x1b]8;;\x1b\\"
        )
    } else {
        write!(output, "{text}")
    }
}

fn url_link(output: &mut impl Write, text: &str, url: &str, enabled: bool) -> std::io::Result<()> {
    if enabled {
        write!(output, "\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
    } else {
        write!(output, "{text}")
    }
}

fn contest_link(
    output: &mut impl Write,
    text: &str,
    id: &str,
    enabled: bool,
) -> std::io::Result<()> {
    if enabled {
        write!(
            output,
            "\x1b]8;;https://leetcode.com/contest/{id}/\x1b\\{text}\x1b]8;;\x1b\\"
        )
    } else {
        write!(output, "{text}")
    }
}

fn discussion_link(
    output: &mut impl Write,
    text: &str,
    id: u32,
    enabled: bool,
) -> std::io::Result<()> {
    if enabled {
        write!(
            output,
            "\x1b]8;;https://leetcode.com/discuss/post/{id}/\x1b\\{text}\x1b]8;;\x1b\\"
        )
    } else {
        write!(output, "{text}")
    }
}

fn statement(output: &mut impl Write, problem: &Problem, presentation: Presentation) -> Result<()> {
    let markdown = markdown(problem)?;
    render_markdown(output, &markdown, presentation)?;
    Ok(())
}

fn markdown(problem: &Problem) -> std::io::Result<String> {
    markdown_from_html(&problem.statement)
}

fn markdown_from_html(html: &str) -> std::io::Result<String> {
    let markdown = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "iframe", "object"])
        .add_handler(
            vec!["p"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                let content = handlers.walk_children(element.node).content;
                let content = content.trim();
                if let Some(label) = statement_heading(content) {
                    Some(format!("\n\n## {label}\n\n").into())
                } else {
                    (!content.is_empty()).then(|| format!("\n\n{content}\n\n").into())
                }
            },
        )
        .add_handler(
            vec!["pre"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                let contains_code = element.node.children.borrow().iter().any(|node| {
                    matches!(&node.data, markup5ever_rcdom::NodeData::Element { name, .. } if name.local.as_ref() == "code")
                });
                if contains_code {
                    return handlers.fallback(element);
                }
                let content = element_text(element.node);
                let mut block = String::with_capacity(content.len() + 16);
                block.push_str("\n\n");
                for line in content.trim_matches('\n').lines() {
                    block.push_str("    ");
                    block.push_str(line);
                    block.push('\n');
                }
                block.push('\n');
                Some(block.into())
            },
        )
        .add_handler(
            vec!["span"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                let class = element
                    .attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "class")
                    .map(|attr| attr.value.as_ref())
                    .unwrap_or_default();
                if class
                    .split_ascii_whitespace()
                    .any(|class| class == "example-io")
                {
                    let content = element_text(element.node);
                    if content.contains('\n') {
                        return handlers.fallback(element);
                    }
                    return Some(markdown_code(&content).into());
                }
                let content = handlers.walk_children(element.node).content;
                Some(
                    if class
                        .split_ascii_whitespace()
                        .any(|class| class == "math-inline")
                    {
                        format!("${content}$")
                    } else if class
                        .split_ascii_whitespace()
                        .any(|class| class == "math-display")
                    {
                        format!("$${content}$$")
                    } else {
                        content
                    }
                    .into(),
                )
            },
        )
        .add_handler(
            vec!["sup", "sub"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                let operator = if element.tag == "sup" { '^' } else { '_' };
                let source = format!(
                    "{operator}{{{}}}",
                    handlers.walk_children(element.node).content
                );
                Some(
                    match math::render(&format!("x{source}")) {
                        Some(block) if block.height() == 1 => {
                            let text = block.to_string();
                            text.strip_prefix('x').unwrap_or(&text).to_owned()
                        }
                        _ => source,
                    }
                    .into(),
                )
            },
        )
        .build()
        .convert(html)?;
    Ok(clean(&markdown))
}

fn element_text(root: &markup5ever_rcdom::Handle) -> String {
    use markup5ever_rcdom::NodeData;

    let mut content = String::new();
    let mut nodes = vec![root.clone()];
    while let Some(node) = nodes.pop() {
        match &node.data {
            NodeData::Text { contents } => content.push_str(&contents.borrow()),
            NodeData::Element { name, .. } if name.local.as_ref() == "br" => content.push('\n'),
            _ => nodes.extend(node.children.borrow().iter().rev().cloned()),
        }
    }
    content
}

fn statement_heading(markdown: &str) -> Option<String> {
    let label = markdown.strip_prefix("**")?.strip_suffix("**")?;
    let label = label.strip_suffix(':').unwrap_or(label);
    if label.ends_with(':') {
        return None;
    }
    match label {
        "Constraints" => Some(label.to_owned()),
        "Follow-up" | "Follow up" => Some("Follow-up".to_owned()),
        _ => {
            let number = label.strip_prefix("Example ")?;
            (!number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| format!("Example {number}"))
        }
    }
}

fn markdown_code(content: &str) -> String {
    let fence = "`".repeat(
        content
            .split(|character| character != '`')
            .map(str::len)
            .max()
            .unwrap_or(0)
            + 1,
    );
    format!("{fence} {content} {fence}")
}

fn heading(
    output: &mut impl Write,
    title: &str,
    presentation: Presentation,
) -> std::io::Result<()> {
    let title: String = title.chars().filter(|ch| !ch.is_control()).collect();
    let columns = usize::from(presentation.columns);
    let large = presentation.large_heading
        && title
            .split_whitespace()
            .all(|word| word.width() < columns / 2);
    let width = if large {
        columns / 2 - 1
    } else {
        columns.saturating_sub(1).max(1)
    };
    let mut longest = 0;
    for line in textwrap::wrap(&title, width) {
        longest = longest.max(line.width() * if large { 2 } else { 1 });
        if large {
            write!(output, "\r\x1b#3")?;
            title_line(output, &line, presentation)?;
            write!(output, "\n\r\x1b#4")?;
            title_line(output, &line, presentation)?;
            write!(output, "\n\r\x1b#5")?;
        } else {
            title_line(output, &line, presentation)?;
            writeln!(output)?;
        }
    }
    if presentation.color && longest > 3 {
        title_rule(output, longest, presentation)?;
        writeln!(output)?;
    }
    writeln!(output)
}

fn title_rule(
    output: &mut impl Write,
    width: usize,
    presentation: Presentation,
) -> std::io::Result<()> {
    if presentation.color && width > 3 {
        let shadow = crate::theme::foreground(presentation.palette.shadow);
        write!(
            output,
            "{shadow} ╰{}╴{shadow:#}",
            "─".repeat(width.saturating_sub(3))
        )?;
    }
    Ok(())
}

fn title_line(
    output: &mut impl Write,
    line: &str,
    presentation: Presentation,
) -> std::io::Result<()> {
    if !presentation.color {
        return write!(output, "{line}");
    }
    let palette = presentation.palette;
    let count = line.chars().count().max(1);
    let mut previous = None;
    for (index, ch) in line.chars().enumerate() {
        let color = palette.gradient[index * palette.gradient.len() / count];
        if previous != Some(color) {
            write!(output, "{}", crate::theme::foreground(color).bold())?;
            previous = Some(color);
        }
        write!(output, "{ch}")?;
    }
    write!(output, "\x1b[0m")
}

fn render_markdown(
    output: &mut impl Write,
    markdown: &str,
    presentation: Presentation,
) -> std::io::Result<()> {
    render_markdown_at(output, markdown, presentation, "https://leetcode.com/")
}

fn render_markdown_at(
    output: &mut impl Write,
    markdown: &str,
    presentation: Presentation,
    base_url: &str,
) -> std::io::Result<()> {
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    static PLAIN_SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let markdown = strip_terminal_controls(strip_dangerous_html(markdown));
    if presentation.columns < 8 {
        return render_narrow_markdown(output, &markdown, options, presentation.columns);
    }
    let highlight = presentation.color && Parser::new_ext(&markdown, options).any(|event| {
        matches!(event, Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(language))) if !language.is_empty())
    });
    let syntax_set = if highlight {
        SYNTAXES.get_or_init(SyntaxSet::load_defaults_newlines)
    } else {
        PLAIN_SYNTAXES.get_or_init(SyntaxSet::new)
    };
    let settings = Settings {
        terminal_capabilities: TerminalCapabilities {
            style: presentation.color.then_some(StyleCapability::Ansi),
            image: None,
            marks: None,
        },
        terminal_size: TerminalSize {
            columns: presentation.columns,
            ..TerminalSize::default()
        },
        syntax_set,
        theme: presentation.palette.markdown(),
        syntax_theme: None,
    };
    let environment = Environment {
        base_url: base_url.parse().map_err(std::io::Error::other)?,
        hostname: "localhost".into(),
    };
    let mut in_code = false;
    let mut in_table = false;
    let mut unsafe_link = 0_u8;
    let preserve_blank = Cell::new(false);
    let events = TextMergeStream::new(Parser::new_ext(&markdown, options)).flat_map(|event| {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code = true,
            Event::End(TagEnd::CodeBlock) => in_code = false,
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => in_table = false,
            _ => {}
        }
        preserve_blank.set(in_code && matches!(event, Event::Text(_)));
        match event {
            Event::Html(_) | Event::InlineHtml(_) => Vec::new(),
            Event::Start(Tag::Link { ref dest_url, .. }) if !safe_markdown_url(dest_url) => {
                unsafe_link = unsafe_link.saturating_add(1);
                Vec::new()
            }
            Event::End(TagEnd::Link) if unsafe_link > 0 => {
                unsafe_link -= 1;
                Vec::new()
            }
            Event::Text(text) if !in_code => math::events(&text, presentation.columns, in_table)
                .into_iter()
                .flat_map(|event| match event {
                    Event::Text(text) if !in_table => wrap_long_words(&text, presentation.columns),
                    other => vec![other],
                })
                .collect(),
            other => vec![other.into_static()],
        }
    });
    let mut compact = CompactWriter {
        output,
        preserve_blank: &preserve_blank,
        line: Vec::new(),
        plain: Vec::new(),
        last_blank: true,
    };
    push_tty(
        &settings,
        &environment,
        &NoopResourceHandler,
        &mut compact,
        events,
    )?;
    compact.flush()
}

fn strip_dangerous_html(markdown: &str) -> Cow<'_, str> {
    const TAGS: [(&str, &str); 4] = [
        ("<script", "</script"),
        ("<style", "</style"),
        ("<iframe", "</iframe"),
        ("<object", "</object"),
    ];
    let lowercase = markdown.to_ascii_lowercase();
    let mut cursor = 0;
    let mut cleaned = None::<String>;
    loop {
        let found = TAGS
            .iter()
            .filter_map(|(open, close)| {
                lowercase[cursor..]
                    .find(open)
                    .map(|offset| (cursor + offset, *close))
            })
            .min_by_key(|(start, _)| *start);
        let Some((start, close)) = found else {
            break;
        };
        let output = cleaned.get_or_insert_with(|| String::with_capacity(markdown.len()));
        output.push_str(&markdown[cursor..start]);
        let Some(open_end) = lowercase[start..].find('>').map(|offset| start + offset) else {
            cursor = markdown.len();
            break;
        };
        if lowercase[start..=open_end].trim_end().ends_with("/>") {
            cursor = open_end + 1;
            continue;
        }
        let Some(close_start) = lowercase[open_end + 1..]
            .find(close)
            .map(|offset| open_end + 1 + offset)
        else {
            cursor = markdown.len();
            break;
        };
        cursor = lowercase[close_start..]
            .find('>')
            .map_or(markdown.len(), |offset| close_start + offset + 1);
    }
    match cleaned {
        Some(mut cleaned) => {
            cleaned.push_str(&markdown[cursor..]);
            Cow::Owned(cleaned)
        }
        None => Cow::Borrowed(markdown),
    }
}

fn strip_terminal_controls(markdown: Cow<'_, str>) -> Cow<'_, str> {
    if !markdown
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return markdown;
    }
    match markdown {
        Cow::Borrowed(markdown) => Cow::Owned(
            markdown
                .chars()
                .filter(|character| {
                    !character.is_control() || matches!(character, '\n' | '\r' | '\t')
                })
                .collect(),
        ),
        Cow::Owned(mut markdown) => {
            markdown.retain(|character| {
                !character.is_control() || matches!(character, '\n' | '\r' | '\t')
            });
            Cow::Owned(markdown)
        }
    }
}

fn render_narrow_markdown(
    output: &mut impl Write,
    markdown: &str,
    options: Options,
    columns: u16,
) -> std::io::Result<()> {
    let mut plain = String::new();
    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Text(text) | Event::Code(text) => plain.push_str(&text),
            Event::SoftBreak | Event::HardBreak => plain.push('\n'),
            Event::End(
                TagEnd::Paragraph
                | TagEnd::Heading(_)
                | TagEnd::Item
                | TagEnd::CodeBlock
                | TagEnd::TableRow,
            ) => plain.push('\n'),
            Event::Rule => plain.push_str("—\n"),
            Event::Html(_) | Event::InlineHtml(_) => {}
            _ => {}
        }
    }
    let width = usize::from(columns).max(1);
    for line in plain.lines() {
        if line.is_empty() {
            writeln!(output)?;
        } else {
            for wrapped in textwrap::wrap(line, width) {
                writeln!(output, "{wrapped}")?;
            }
        }
    }
    Ok(())
}

fn safe_markdown_url(url: &str) -> bool {
    let url = url.trim();
    !url.is_empty()
        && !url.chars().any(char::is_control)
        && (url.starts_with("https://")
            || url.starts_with("http://")
            || url.starts_with("mailto:")
            || url.starts_with('/')
            || url.starts_with('#')
            || !url
                .split(['/', '?', '#'])
                .next()
                .is_some_and(|prefix| prefix.contains(':')))
}

struct CompactWriter<'a, W> {
    output: &'a mut W,
    preserve_blank: &'a Cell<bool>,
    line: Vec<u8>,
    plain: Vec<u8>,
    last_blank: bool,
}

impl<W: Write> CompactWriter<'_, W> {
    fn line(&mut self) -> std::io::Result<()> {
        self.plain.clear();
        anstream::StripStream::new(&mut self.plain).write_all(&self.line)?;
        let blank = String::from_utf8_lossy(&self.plain).trim().is_empty();
        if !blank || !self.last_blank || self.preserve_blank.get() {
            self.output.write_all(&self.line)?;
        }
        self.last_blank = blank;
        self.line.clear();
        Ok(())
    }
}

impl<W: Write> Write for CompactWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        for chunk in bytes.split_inclusive(|byte| *byte == b'\n') {
            self.line.extend_from_slice(chunk);
            if chunk.last() == Some(&b'\n') {
                self.line()?;
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.line.is_empty() {
            self.line()?;
        }
        self.output.flush()
    }
}

fn wrap_long_words(text: &str, columns: u16) -> Vec<Event<'static>> {
    let width = usize::from(columns).max(1);
    let mut events = Vec::new();
    let mut pending = String::new();
    for word in text.split_inclusive(char::is_whitespace) {
        if word.trim_end().width() > width {
            for (index, part) in textwrap::wrap(word, width).into_iter().enumerate() {
                if index > 0 {
                    events.push(Event::Text(std::mem::take(&mut pending).into()));
                    events.push(Event::HardBreak);
                }
                pending.push_str(&part);
            }
            pending.push_str(&word[word.trim_end().len()..]);
        } else {
            pending.push_str(word);
        }
    }
    events.push(Event::Text(pending.into()));
    events
}

fn clean(input: &str) -> String {
    input
        .chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect()
}
