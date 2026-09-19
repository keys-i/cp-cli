use std::{
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
    domain::Problem,
    error::Result,
    math,
};

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

    hud(&mut output, problem.id.as_ref(), presentation)?;
    heading(&mut output, &problem.title, presentation)?;
    statement(&mut output, problem, presentation)
}

fn hud(output: &mut impl Write, id: &str, presentation: Presentation) -> std::io::Result<()> {
    if !presentation.interactive {
        return Ok(());
    }
    if presentation.columns < 16 {
        return Ok(());
    }
    let art = if presentation.motion {
        crate::possum::render_alive(
            crate::possum::Pose::Cursor,
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
        const PREFIX: &str = "POSSUM//ARCADE  QUEST ";
        write!(output, "{accent}{PREFIX}{reset}")?;
        for (index, line) in textwrap::wrap(
            &format!("leetcode/{id}"),
            usize::from(presentation.columns.saturating_sub(PREFIX.width() as u16)).max(1),
        )
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                write!(output, "\n{accent}{}{reset}", " ".repeat(PREFIX.width()))?;
            }
            problem_link(output, &line, id, presentation.color)?;
        }
        writeln!(output)?;
    } else {
        let metadata = format!("POSSUM// leetcode/{id}");
        for line in textwrap::wrap(&metadata, usize::from(presentation.columns)) {
            write!(output, "{accent}")?;
            problem_link(output, &line, id, presentation.color)?;
            writeln!(output, "{reset}")?;
        }
    }
    Ok(())
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

fn statement(output: &mut impl Write, problem: &Problem, presentation: Presentation) -> Result<()> {
    let markdown = markdown(problem)?;
    render_markdown(output, &markdown, presentation)?;
    Ok(())
}

fn markdown(problem: &Problem) -> std::io::Result<String> {
    let markdown = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "iframe", "object"])
        .add_handler(
            vec!["p"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                let content = handlers.walk_children(element.node).content;
                (!content.trim().is_empty())
                    .then(|| format!("\n\n{}\n\n", content.trim()).into())
            },
        )
        .add_handler(
            vec!["pre"],
            |handlers: &dyn htmd::element_handler::Handlers, element: htmd::Element<'_>| {
                use markup5ever_rcdom::NodeData;
                let contains_code = element.node.children.borrow().iter().any(|node| {
                    matches!(&node.data, NodeData::Element { name, .. } if name.local.as_ref() == "code")
                });
                if contains_code {
                    return handlers.fallback(element);
                }
                let mut content = String::new();
                let mut nodes = vec![element.node.clone()];
                while let Some(node) = nodes.pop() {
                    match &node.data {
                        NodeData::Text { contents } => content.push_str(&contents.borrow()),
                        NodeData::Element { name, .. } if name.local.as_ref() == "br" => {
                            content.push('\n');
                        }
                        _ => nodes.extend(node.children.borrow().iter().rev().cloned()),
                    }
                }
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
                let content = handlers.walk_children(element.node).content;
                let class = element
                    .attrs
                    .iter()
                    .find(|attr| attr.name.local.as_ref() == "class")
                    .map(|attr| attr.value.as_ref());
                Some(
                    match class {
                        Some("math math-inline") => format!("${content}$"),
                        Some("math math-display") => format!("$${content}$$"),
                        _ => content,
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
        .convert(&problem.statement)?;
    Ok(clean(&markdown))
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
        let shadow = crate::theme::foreground(presentation.palette.shadow);
        writeln!(
            output,
            "{shadow} ╰{}╴{shadow:#}",
            "─".repeat(longest.saturating_sub(3))
        )?;
    }
    writeln!(output)
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
    static SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    static PLAIN_SYNTAXES: OnceLock<SyntaxSet> = OnceLock::new();
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let highlight = presentation.color && Parser::new_ext(markdown, options).any(|event| {
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
        base_url: "https://leetcode.com/"
            .parse()
            .map_err(std::io::Error::other)?,
        hostname: "localhost".into(),
    };
    let mut in_code = false;
    let mut in_table = false;
    let preserve_blank = Cell::new(false);
    let events = TextMergeStream::new(Parser::new_ext(markdown, options)).flat_map(|event| {
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code = true,
            Event::End(TagEnd::CodeBlock) => in_code = false,
            Event::Start(Tag::Table(_)) => in_table = true,
            Event::End(TagEnd::Table) => in_table = false,
            _ => {}
        }
        preserve_blank.set(in_code && matches!(event, Event::Text(_)));
        match event {
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
