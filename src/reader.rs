use std::{
    cmp::Ordering,
    io::{self, Write},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
        MouseButton, MouseEventKind,
    },
    execute, queue,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    cli::Platform,
    codeforces_support::{CodeforcesContest, CodeforcesSubmission},
    domain::{
        CatalogPlatform, CatalogProblemSummary, CommunityDiscussionSummary, Contest, ContestSlug,
        Difficulty, DiscussionId, DiscussionSummary, ProblemId, ProblemSummary, RecentSubmission,
        official_path,
    },
    error::Result,
    output::Presentation,
};

const HEADER_ROW: u16 = 3;
const FIRST_RESULT_ROW: u16 = HEADER_ROW + 2;
const MIN_TABLE_WIDTH: usize = 48;

pub(crate) enum Target {
    Problem(ProblemId),
    Catalog(CatalogPlatform, ProblemId),
    Contest(ContestSlug),
    CodeforcesContest(u32),
    Discussion(DiscussionId),
    CommunityDiscussion(Platform, DiscussionId),
}
pub(crate) struct Table {
    pub(crate) label: Box<str>,
    pub(crate) detail: Box<str>,
    pub(crate) columns: Vec<Column>,
    pub(crate) rows: Vec<Row>,
}
pub(crate) struct Column {
    pub(crate) label: Box<str>,
    pub(crate) min_width: usize,
    pub(crate) weight: usize,
}
pub(crate) struct Row {
    pub(crate) cells: Vec<Cell>,
    pub(crate) target: Target,
}
pub(crate) struct Cell {
    pub(crate) text: Box<str>,
    pub(crate) sort_key: Box<str>,
    pub(crate) tone: Tone,
}
#[derive(Clone, Copy)]
pub(crate) enum Tone {
    Number,
    Title,
    Difficulty(Difficulty),
    Slug,
    Muted,
    Plain,
}

fn table_width(width: u16) -> usize {
    usize::from(width).saturating_mul(3) / 5
}
pub(crate) fn supported() -> bool {
    terminal::size().is_ok_and(|(width, _)| table_width(width) >= MIN_TABLE_WIDTH)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Header,
    Rows,
}
struct Reader {
    order: Vec<usize>,
    selected: usize,
    scroll: usize,
    focus: Focus,
    header: usize,
    column: usize,
    ascending: bool,
    sorted: bool,
}
impl Reader {
    fn new(rows: &[Row]) -> Self {
        Self {
            order: (0..rows.len()).collect(),
            selected: 0,
            scroll: 0,
            focus: Focus::Rows,
            header: 0,
            column: 0,
            ascending: true,
            sorted: false,
        }
    }
    fn target(&self, rows: &[Row]) -> Target {
        match &rows[self.order[self.selected]].target {
            Target::Problem(id) => Target::Problem(id.clone()),
            Target::Catalog(platform, id) => Target::Catalog(*platform, id.clone()),
            Target::Contest(id) => Target::Contest(id.clone()),
            Target::CodeforcesContest(id) => Target::CodeforcesContest(*id),
            Target::Discussion(id) => Target::Discussion(*id),
            Target::CommunityDiscussion(platform, id) => {
                Target::CommunityDiscussion(*platform, *id)
            }
        }
    }
    fn sort(&mut self, rows: &[Row], column: usize) {
        let selected = self.order[self.selected];
        if self.sorted && self.column == column {
            self.ascending = !self.ascending;
        } else {
            self.column = column;
            self.ascending = true;
        }
        self.header = column;
        self.sorted = true;
        self.order.sort_by(|&left, &right| {
            let order = rows[left].cells[column]
                .sort_key
                .cmp(&rows[right].cells[column].sort_key)
                .then_with(|| left.cmp(&right));
            if self.ascending {
                order
            } else {
                order.reverse()
            }
        });
        self.selected = self
            .order
            .iter()
            .position(|&index| index == selected)
            .unwrap_or(0);
    }
    fn down(&mut self, viewport: usize) {
        if self.focus == Focus::Header {
            self.focus = Focus::Rows;
        } else if self.selected + 1 < self.order.len() {
            self.selected += 1;
        }
        self.visible(viewport);
    }
    fn up(&mut self, viewport: usize) {
        if self.focus == Focus::Rows && self.selected > 0 {
            self.selected -= 1;
        } else if self.focus == Focus::Rows {
            self.focus = Focus::Header;
        }
        self.visible(viewport);
    }
    fn header(&mut self, direction: Ordering, columns: usize) {
        self.focus = Focus::Header;
        self.header = match direction {
            Ordering::Less => self.header.saturating_sub(1),
            Ordering::Greater => (self.header + 1).min(columns - 1),
            Ordering::Equal => self.header,
        };
    }
    fn visible(&mut self, viewport: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll.saturating_add(viewport) {
            self.scroll = self.selected + 1 - viewport;
        }
    }
}

struct TerminalGuard;
impl TerminalGuard {
    fn enter(out: &mut impl Write) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let guard = Self;
        if let Err(error) = execute!(out, EnterAlternateScreen, EnableMouseCapture, Hide) {
            drop(guard);
            return Err(error);
        }
        Ok(guard)
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, Show, DisableMouseCapture, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

struct Layout {
    widths: Vec<usize>,
}
impl Layout {
    fn new(width: usize, table: &Table) -> Self {
        let mut widths: Vec<_> = table
            .columns
            .iter()
            .map(|column| column.label.width().max(column.min_width))
            .collect();
        let available = width.saturating_sub(2 + table.columns.len().saturating_sub(1) * 3);
        let minimum = widths.iter().sum::<usize>();
        if available > minimum {
            let extra = available - minimum;
            let total = table
                .columns
                .iter()
                .map(|column| column.weight)
                .sum::<usize>();
            if total > 0 {
                let mut used = 0;
                for (width, column) in widths.iter_mut().zip(&table.columns) {
                    let share = extra
                        .saturating_mul(column.weight)
                        .checked_div(total)
                        .unwrap_or(0);
                    *width += share;
                    used += share;
                }
                if let Some(last) = widths.last_mut() {
                    *last += extra - used;
                }
            } else if let Some(last) = widths.last_mut() {
                *last += extra;
            }
        }
        Self { widths }
    }
    fn width(&self) -> usize {
        2 + self.widths.iter().sum::<usize>() + self.widths.len().saturating_sub(1) * 3
    }
    fn header_at(&self, x: u16) -> Option<usize> {
        let mut start = 2;
        for (index, width) in self.widths.iter().enumerate() {
            if usize::from(x) >= start && usize::from(x) < start + width {
                return Some(index);
            }
            start += width + 3;
        }
        None
    }
}

pub(crate) fn browse_table(table: &Table, presentation: Presentation) -> Result<Option<Target>> {
    if table.rows.is_empty()
        || table.columns.is_empty()
        || table
            .rows
            .iter()
            .any(|row| row.cells.len() != table.columns.len())
    {
        return Ok(None);
    }
    let mut out = io::stdout();
    let _terminal = TerminalGuard::enter(&mut out)?;
    let mut reader = Reader::new(&table.rows);
    loop {
        let (width, height) = terminal::size()?;
        let pane = table_width(width);
        if pane < MIN_TABLE_WIDTH {
            queue!(out, crossterm::cursor::MoveTo(0, 0), Clear(ClearType::All))?;
            let accent = style(presentation, presentation.palette.accent, true, false);
            write!(
                out,
                "{accent}POSSUM//READER{}\r\n{}\r\n",
                accent.render_reset(),
                truncate(
                    "Widen terminal to 80 columns · q close",
                    usize::from(width).max(1)
                )
            )?;
            out.flush()?;
            loop {
                match event::read()? {
                    Event::Key(key)
                        if key.kind != KeyEventKind::Release
                            && (matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
                                || key.code == KeyCode::Char('c')
                                    && key.modifiers.contains(KeyModifiers::CONTROL)) =>
                    {
                        return Ok(None);
                    }
                    Event::Resize(width, _) if table_width(width) >= MIN_TABLE_WIDTH => break,
                    _ => {}
                }
            }
            continue;
        }
        let layout = Layout::new(pane, table);
        let viewport = usize::from(height.saturating_sub(FIRST_RESULT_ROW + 1)).max(1);
        reader.visible(viewport);
        draw(&mut out, table, presentation, &reader, &layout, viewport)?;
        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                KeyCode::Up => reader.up(viewport),
                KeyCode::Down => reader.down(viewport),
                KeyCode::Left => reader.header(Ordering::Less, table.columns.len()),
                KeyCode::Right => reader.header(Ordering::Greater, table.columns.len()),
                KeyCode::Enter => match reader.focus {
                    Focus::Header => reader.sort(&table.rows, reader.header),
                    Focus::Rows => return Ok(Some(reader.target(&table.rows))),
                },
                _ => {}
            },
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                if mouse.row == HEADER_ROW {
                    if let Some(column) = layout.header_at(mouse.column) {
                        reader.focus = Focus::Header;
                        reader.sort(&table.rows, column);
                    }
                } else if mouse.row >= FIRST_RESULT_ROW {
                    let clicked = reader.scroll + usize::from(mouse.row - FIRST_RESULT_ROW);
                    if clicked < reader.order.len() {
                        reader.selected = clicked;
                        reader.focus = Focus::Rows;
                    }
                }
            }
            Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::ScrollUp) => {
                reader.up(viewport)
            }
            Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::ScrollDown) => {
                reader.down(viewport)
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

pub(crate) fn browse_problems(
    label: &str,
    detail: &str,
    rows: &[ProblemSummary],
    presentation: Presentation,
) -> Result<Option<ProblemId>> {
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "#".into(),
                min_width: rows
                    .iter()
                    .map(|problem| problem.number.to_string().width())
                    .max()
                    .unwrap_or(1),
                weight: 0,
            },
            Column {
                label: "Problem".into(),
                min_width: 4,
                weight: 2,
            },
            Column {
                label: "Level".into(),
                min_width: 8,
                weight: 0,
            },
            Column {
                label: "Slug".into(),
                min_width: 4,
                weight: 3,
            },
        ],
        rows: rows.iter().map(problem_row).collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::Problem(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

pub(crate) fn browse_catalog(
    label: &str,
    detail: &str,
    platform: CatalogPlatform,
    rows: &[CatalogProblemSummary],
    presentation: Presentation,
) -> Result<Option<ProblemId>> {
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "Id".into(),
                min_width: 8,
                weight: 2,
            },
            Column {
                label: "Problem".into(),
                min_width: 7,
                weight: 3,
            },
            Column {
                label: "Level".into(),
                min_width: 8,
                weight: 0,
            },
            Column {
                label: "Path · details".into(),
                min_width: 4,
                weight: 2,
            },
        ],
        rows: rows
            .iter()
            .map(|problem| {
                let (level, tone, level_sort) = if let Some(level) = &problem.level {
                    (level.to_string(), Tone::Number, level.to_string())
                } else {
                    match (problem.difficulty, problem.rating) {
                        (Some(difficulty), _) => (
                            format!("[{:^6}]", difficulty.label()),
                            Tone::Difficulty(difficulty),
                            format!(
                                "{:05}",
                                match difficulty {
                                    Difficulty::Easy => 1,
                                    Difficulty::Medium => 2,
                                    Difficulty::Hard => 3,
                                }
                            ),
                        ),
                        (None, Some(rating)) => {
                            (rating.to_string(), Tone::Number, format!("{rating:05}"))
                        }
                        (None, None) => ("—".to_owned(), Tone::Muted, String::new()),
                    }
                };
                let context = if problem.tags.is_empty() {
                    problem.solved_count.map_or_else(
                        || {
                            problem
                                .published_at
                                .as_deref()
                                .map_or_else(|| "—".to_owned(), |date| format!("Published {date}"))
                        },
                        |solved| format!("{solved} solved"),
                    )
                } else {
                    problem
                        .tags
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<_>>()
                        .join(" · ")
                };
                let path = official_path(&problem.url);
                let details = if context == "—" {
                    path
                } else {
                    format!("{path} · {context}")
                };
                Row {
                    cells: vec![
                        Cell {
                            text: problem.id.as_ref().into(),
                            sort_key: problem.id.as_ref().into(),
                            tone: Tone::Slug,
                        },
                        Cell {
                            text: crate::math::inline_text(&problem.title).into(),
                            sort_key: problem.title.clone(),
                            tone: Tone::Title,
                        },
                        Cell {
                            text: level.clone().into(),
                            sort_key: level_sort.into(),
                            tone,
                        },
                        Cell {
                            text: details.clone().into(),
                            sort_key: details.into(),
                            tone: Tone::Plain,
                        },
                    ],
                    target: Target::Catalog(platform, problem.id.clone()),
                }
            })
            .collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::Catalog(_, id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

pub(crate) fn browse_contests(
    label: &str,
    detail: &str,
    contests: &[Contest],
    presentation: Presentation,
) -> Result<Option<ContestSlug>> {
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "Starts (UTC)".into(),
                min_width: 10,
                weight: 1,
            },
            Column {
                label: "Length".into(),
                min_width: 6,
                weight: 0,
            },
            Column {
                label: "Contest".into(),
                min_width: 8,
                weight: 2,
            },
            Column {
                label: "Slug".into(),
                min_width: 4,
                weight: 2,
            },
        ],
        rows: contests
            .iter()
            .map(|contest| Row {
                cells: vec![
                    Cell {
                        text: crate::output::timestamp_utc(contest.start_time).into(),
                        sort_key: format!("{:020}", contest.start_time).into(),
                        tone: Tone::Number,
                    },
                    Cell {
                        text: crate::output::duration_label(contest.duration_seconds).into(),
                        sort_key: format!("{:010}", contest.duration_seconds).into(),
                        tone: Tone::Plain,
                    },
                    Cell {
                        text: contest.title.clone(),
                        sort_key: contest.title.clone(),
                        tone: Tone::Title,
                    },
                    Cell {
                        text: format!(
                            "{}{}",
                            contest.id.as_ref(),
                            if contest.virtual_contest {
                                " · Virtual"
                            } else {
                                ""
                            }
                        )
                        .into(),
                        sort_key: contest.id.as_ref().into(),
                        tone: Tone::Slug,
                    },
                ],
                target: Target::Contest(contest.id.clone()),
            })
            .collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::Contest(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

pub(crate) fn browse_codeforces_contests(
    label: &str,
    detail: &str,
    contests: &[CodeforcesContest],
    presentation: Presentation,
) -> Result<Option<u32>> {
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "Starts (UTC)".into(),
                min_width: 10,
                weight: 1,
            },
            Column {
                label: "Phase".into(),
                min_width: 6,
                weight: 0,
            },
            Column {
                label: "Contest".into(),
                min_width: 12,
                weight: 3,
            },
            Column {
                label: "Id · path".into(),
                min_width: 9,
                weight: 1,
            },
        ],
        rows: contests
            .iter()
            .map(|contest| {
                let starts = contest
                    .start_time_seconds
                    .and_then(|value| u64::try_from(value).ok())
                    .map_or_else(|| "TBD".to_owned(), crate::output::timestamp_utc);
                Row {
                    cells: vec![
                        Cell {
                            text: starts.into(),
                            sort_key: format!(
                                "{:020}",
                                contest.start_time_seconds.unwrap_or(i64::MAX)
                            )
                            .into(),
                            tone: Tone::Number,
                        },
                        Cell {
                            text: contest.phase.clone(),
                            sort_key: contest.phase.clone(),
                            tone: Tone::Muted,
                        },
                        Cell {
                            text: contest.name.clone(),
                            sort_key: contest.name.clone(),
                            tone: Tone::Title,
                        },
                        Cell {
                            text: format!("{} · {}", contest.id, official_path(&contest.url))
                                .into(),
                            sort_key: format!("{:010}", contest.id).into(),
                            tone: Tone::Slug,
                        },
                    ],
                    target: Target::CodeforcesContest(contest.id),
                }
            })
            .collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::CodeforcesContest(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

pub(crate) fn browse_discussions(
    label: &str,
    detail: &str,
    discussions: &[DiscussionSummary],
    presentation: Presentation,
) -> Result<Option<DiscussionId>> {
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "Id".into(),
                min_width: discussions
                    .iter()
                    .map(|discussion| discussion.id.get().to_string().width())
                    .max()
                    .unwrap_or(2),
                weight: 0,
            },
            Column {
                label: "Discussion".into(),
                min_width: 10,
                weight: 3,
            },
            Column {
                label: "Author".into(),
                min_width: 6,
                weight: 1,
            },
            Column {
                label: "Activity".into(),
                min_width: 8,
                weight: 2,
            },
        ],
        rows: discussions
            .iter()
            .map(|discussion| Row {
                cells: vec![
                    Cell {
                        text: discussion.id.get().to_string().into(),
                        sort_key: format!("{:010}", discussion.id.get()).into(),
                        tone: Tone::Number,
                    },
                    Cell {
                        text: discussion.title.clone(),
                        sort_key: discussion.title.clone(),
                        tone: Tone::Title,
                    },
                    Cell {
                        text: discussion
                            .author
                            .clone()
                            .unwrap_or_else(|| "Deleted user".into()),
                        sort_key: discussion
                            .author
                            .clone()
                            .unwrap_or_else(|| "Deleted user".into()),
                        tone: Tone::Plain,
                    },
                    Cell {
                        text: format!(
                            "{} votes · {} replies · {} views",
                            discussion.votes, discussion.comments, discussion.views
                        )
                        .into(),
                        sort_key: format!("{:010}", i64::from(discussion.votes) + 1_000_000_000)
                            .into(),
                        tone: Tone::Muted,
                    },
                ],
                target: Target::Discussion(discussion.id),
            })
            .collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::Discussion(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

pub(crate) fn browse_community_discussions(
    label: &str,
    detail: &str,
    discussions: &[CommunityDiscussionSummary],
    presentation: Presentation,
) -> Result<Option<DiscussionId>> {
    let rows = discussions
        .iter()
        .map(|discussion| DiscussionSummary {
            id: discussion.id,
            title: discussion.title.clone(),
            author: discussion.author.clone(),
            created_at: 0,
            views: discussion.views,
            comments: discussion.replies,
            votes: discussion.votes,
        })
        .collect::<Vec<_>>();
    browse_discussions(label, detail, &rows, presentation)
}

pub(crate) fn browse_codeforces_submissions(
    label: &str,
    detail: &str,
    submissions: &[CodeforcesSubmission],
    presentation: Presentation,
) -> Result<Option<ProblemId>> {
    let rows = submissions
        .iter()
        .filter_map(|submission| {
            let id = format!("{}{}", submission.contest_id?, submission.problem_index)
                .parse()
                .ok()?;
            Some(RecentSubmission {
                id: submission.id,
                status: submission
                    .verdict
                    .clone()
                    .unwrap_or_else(|| "Pending".into()),
                title: submission.problem_name.clone(),
                problem: id,
                timestamp: u64::try_from(submission.timestamp).unwrap_or_default(),
                language: submission.language.clone(),
                runtime: Some(format!("{} ms", submission.runtime_millis).into()),
                memory: Some(format!("{} KB", submission.memory_bytes / 1024).into()),
                pending: submission.verdict.is_none(),
            })
        })
        .collect::<Vec<_>>();
    browse_submissions(label, detail, &rows, presentation)
}

pub(crate) fn browse_submissions(
    label: &str,
    detail: &str,
    submissions: &[RecentSubmission],
    presentation: Presentation,
) -> Result<Option<ProblemId>> {
    let id_width = submissions
        .iter()
        .map(|submission| submission.id.to_string().len())
        .max()
        .unwrap_or(2)
        .clamp(2, 11);
    let table = Table {
        label: label.into(),
        detail: detail.into(),
        columns: vec![
            Column {
                label: "Id".into(),
                min_width: id_width,
                weight: 0,
            },
            Column {
                label: "Status".into(),
                min_width: 6,
                weight: 1,
            },
            Column {
                label: "Problem".into(),
                min_width: 7,
                weight: 2,
            },
            Column {
                label: "Language".into(),
                min_width: 4,
                weight: 1,
            },
            Column {
                label: "Result".into(),
                min_width: 6,
                weight: 1,
            },
        ],
        rows: submissions
            .iter()
            .map(|submission| {
                let result = match (&submission.runtime, &submission.memory) {
                    (Some(runtime), Some(memory)) => format!("{runtime} · {memory}"),
                    (Some(runtime), None) => runtime.to_string(),
                    (None, Some(memory)) => memory.to_string(),
                    (None, None) if submission.pending => "Pending".to_owned(),
                    (None, None) => "—".to_owned(),
                };
                Row {
                    cells: vec![
                        Cell {
                            text: submission.id.to_string().into(),
                            sort_key: format!("{:020}", submission.id).into(),
                            tone: Tone::Number,
                        },
                        Cell {
                            text: submission.status.clone(),
                            sort_key: submission.status.clone(),
                            tone: Tone::Title,
                        },
                        Cell {
                            text: submission.title.clone(),
                            sort_key: submission.title.clone(),
                            tone: Tone::Slug,
                        },
                        Cell {
                            text: submission.language.clone(),
                            sort_key: submission.language.clone(),
                            tone: Tone::Plain,
                        },
                        Cell {
                            text: result.clone().into(),
                            sort_key: result.into(),
                            tone: Tone::Muted,
                        },
                    ],
                    target: Target::Problem(submission.problem.clone()),
                }
            })
            .collect(),
    };
    match browse_table(&table, presentation)? {
        Some(Target::Problem(id)) => Ok(Some(id)),
        _ => Ok(None),
    }
}

fn problem_row(problem: &ProblemSummary) -> Row {
    let level = format!("[{:^6}]", problem.difficulty.label());
    let slug = format!(
        "leetcode/{}{}",
        problem.id.as_ref(),
        if problem.paid_only { " · PREMIUM" } else { "" }
    );
    Row {
        cells: vec![
            Cell {
                text: problem.number.to_string().into(),
                sort_key: format!("{:010}", problem.number).into(),
                tone: Tone::Number,
            },
            Cell {
                text: problem.title.clone(),
                sort_key: problem.title.clone(),
                tone: Tone::Title,
            },
            Cell {
                text: level.clone().into(),
                sort_key: difficulty_order(problem.difficulty).to_string().into(),
                tone: Tone::Difficulty(problem.difficulty),
            },
            Cell {
                text: slug.into(),
                sort_key: problem.id.as_ref().into(),
                tone: Tone::Slug,
            },
        ],
        target: Target::Problem(problem.id.clone()),
    }
}
fn difficulty_order(difficulty: Difficulty) -> u8 {
    match difficulty {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
    }
}

fn draw(
    out: &mut impl Write,
    table: &Table,
    presentation: Presentation,
    reader: &Reader,
    layout: &Layout,
    viewport: usize,
) -> io::Result<()> {
    queue!(out, crossterm::cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    let accent = style(presentation, presentation.palette.accent, true, false);
    write!(
        out,
        "{accent}{}{}\r\n{}\r\n\r\n",
        truncate(&table.label, layout.width()),
        accent.render_reset(),
        truncate(&table.detail, layout.width())
    )?;
    write!(out, "  ")?;
    for (index, column) in table.columns.iter().enumerate() {
        if index > 0 {
            write!(out, " │ ")?;
        }
        let active = reader.focus == Focus::Header && reader.header == index;
        let available = layout.widths[index].saturating_sub(2);
        let label = truncate(&column.label, available);
        let style = style(presentation, presentation.palette.accent, true, false);
        write!(
            out,
            "{style}{}{: <available$}{}{}",
            if active { "[" } else { " " },
            label,
            if active { "]" } else { " " },
            style.render_reset()
        )?;
    }
    write!(out, "\r\n")?;
    let rule = format!(
        "  {}",
        layout
            .widths
            .iter()
            .map(|width| "─".repeat(*width))
            .collect::<Vec<_>>()
            .join("─┼─")
    );
    let rule_style = style(presentation, presentation.palette.shadow, false, false);
    write!(out, "{rule_style}{rule}{}\r\n", rule_style.render_reset())?;
    for index in reader.scroll..reader.order.len().min(reader.scroll + viewport) {
        let row = &table.rows[reader.order[index]];
        let marker = selected_style(
            style(presentation, presentation.palette.accent, true, false),
            index == reader.selected,
            presentation,
        );
        write!(
            out,
            "{marker}{}{}",
            if index == reader.selected { "> " } else { "  " },
            marker.render_reset()
        )?;
        for (column, cell_value) in row.cells.iter().enumerate() {
            if column > 0 {
                write!(out, " │ ")?;
            }
            let text = truncate(&cell_value.text, layout.widths[column]);
            let cell = selected_style(
                tone_style(presentation, cell_value.tone),
                index == reader.selected,
                presentation,
            );
            write!(
                out,
                "{cell}{text}{}{}",
                cell.render_reset(),
                " ".repeat(layout.widths[column].saturating_sub(text.width()))
            )?;
        }
        write!(out, "\r\n")?;
    }
    let order = if reader.sorted {
        format!(
            "{} {}",
            table.columns[reader.column].label,
            if reader.ascending { "↑" } else { "↓" }
        )
    } else {
        "server order".to_owned()
    };
    let focus = match reader.focus {
        Focus::Header => "header",
        Focus::Rows => "rows",
    };
    write!(
        out,
        "{}\r\n",
        truncate(
            &format!(
                "q close · {}/{} · {focus} · {order} · ↑↓ rows · ←→ headers · enter",
                reader.selected + 1,
                table.rows.len()
            ),
            layout.width()
        )
    )?;
    out.flush()
}
fn tone_style(p: Presentation, tone: Tone) -> anstyle::Style {
    match tone {
        Tone::Number => style(p, p.palette.result_number, true, false),
        Tone::Title => style(p, p.palette.result_title, true, false),
        Tone::Difficulty(Difficulty::Easy) => style(p, p.palette.easy, true, false),
        Tone::Difficulty(Difficulty::Medium) => style(p, p.palette.medium, true, false),
        Tone::Difficulty(Difficulty::Hard) => style(p, p.palette.hard, true, false),
        Tone::Slug => style(p, p.palette.result_slug, false, true),
        Tone::Muted => style(p, p.palette.shadow, false, false),
        Tone::Plain => anstyle::Style::new(),
    }
}
fn style(p: Presentation, color: u8, bold: bool, underline: bool) -> anstyle::Style {
    if !p.color {
        return anstyle::Style::new();
    }
    let style = crate::theme::foreground(color);
    match (bold, underline) {
        (true, true) => style.bold().underline(),
        (true, false) => style.bold(),
        (false, true) => style.underline(),
        (false, false) => style,
    }
}
fn selected_style(style: anstyle::Style, selected: bool, p: Presentation) -> anstyle::Style {
    if selected && p.color {
        style.invert()
    } else {
        style
    }
}
fn truncate(value: &str, width: usize) -> String {
    let visible_width = value
        .chars()
        .filter(|character| !character.is_control())
        .filter_map(UnicodeWidthChar::width)
        .sum::<usize>();
    if visible_width <= width {
        return value
            .chars()
            .filter(|character| !character.is_control())
            .collect();
    }
    if width == 0 {
        return String::new();
    }
    if width <= 1 {
        return "…".to_owned();
    }
    let mut result = String::with_capacity(width);
    let mut used = 0;
    for character in value.chars().filter(|character| !character.is_control()) {
        let w = character.width().unwrap_or(0);
        if used + w >= width {
            break;
        }
        result.push(character);
        used += w;
    }
    result.push('…');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn generic_reader_sorts_and_preserves_target() -> Result<()> {
        let rows = [
            (2, "beta", Difficulty::Hard),
            (1, "gamma", Difficulty::Medium),
            (3, "alpha", Difficulty::Easy),
        ]
        .into_iter()
        .map(|(number, title, difficulty)| {
            Ok(ProblemSummary {
                number,
                id: ProblemId::from_str(title)?,
                title: title.into(),
                difficulty,
                paid_only: false,
            })
        })
        .collect::<Result<Vec<_>>>()?;
        let table = Table {
            label: "table".into(),
            detail: "detail".into(),
            columns: vec![
                Column {
                    label: "#".into(),
                    min_width: 1,
                    weight: 0,
                },
                Column {
                    label: "PROBLEM".into(),
                    min_width: 4,
                    weight: 1,
                },
            ],
            rows: rows
                .iter()
                .map(problem_row)
                .map(|mut row| {
                    row.cells.truncate(2);
                    row
                })
                .collect(),
        };
        let mut reader = Reader::new(&table.rows);
        reader.selected = 1;
        for column in 0..table.columns.len() {
            reader.sort(&table.rows, column);
            assert!(
                matches!(reader.target(&table.rows), Target::Problem(ref id) if id.as_ref() == "gamma")
            );
            reader.sort(&table.rows, column);
            assert!(!reader.ascending);
        }
        let layout = Layout::new(80, &table);
        assert_eq!(Layout::new(48, &table).width(), 48);
        assert_eq!(table_width(100), 60);
        assert_eq!(layout.header_at(2), Some(0));
        assert_eq!(layout.header_at((2 + layout.widths[0] + 3) as u16), Some(1));
        assert_eq!(truncate("safe\x1b[31m", 20), "safe[31m");
        assert_eq!(truncate("line\nbreak", 20), "linebreak");
        Ok(())
    }
}
