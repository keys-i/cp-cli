use pulldown_cmark::Event;
use term_maths::RenderedBlock;

pub(crate) fn events(text: &str, columns: u16, in_table: bool) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut rest = text;
    while let Some((start, open, close)) = opening(rest) {
        let after_open = &rest[start + open.len()..];
        let Some(end) = closing(after_open, close) else {
            break;
        };
        events.push(Event::Text(rest[..start].to_owned().into()));
        let source = &after_open[..end];
        let original = &rest[start..start + open.len() + end + close.len()];
        if let Some(block) = render(source) {
            let rendered = block.to_string();
            if in_table {
                events.push(Event::Code(
                    if block.height() == 1 {
                        rendered
                    } else {
                        source.to_owned()
                    }
                    .into(),
                ));
            } else if block.width() <= usize::from(columns.saturating_sub(8)) {
                if block.height() > 1 || matches!(open, "$$" | "\\[") {
                    events.push(Event::DisplayMath(rendered.replace('\n', "\n    ").into()));
                } else {
                    events.push(Event::InlineMath(rendered.into()));
                }
            } else {
                events.push(Event::Code(original.to_owned().into()));
            }
        } else {
            events.push(Event::Code(original.to_owned().into()));
        }
        rest = &after_open[end + close.len()..];
    }
    events.push(Event::Text(rest.to_owned().into()));
    events
}

fn opening(text: &str) -> Option<(usize, &'static str, &'static str)> {
    let mut characters = text.char_indices();
    while let Some((index, ch)) = characters.next() {
        match ch {
            '$' if text[index..].starts_with("$$") => return Some((index, "$$", "$$")),
            '$' => return Some((index, "$", "$")),
            '\\' => match characters.next() {
                Some((_, '(')) => return Some((index, "\\(", "\\)")),
                Some((_, '[')) => return Some((index, "\\[", "\\]")),
                _ => {}
            },
            _ => {}
        }
    }
    None
}

fn closing(text: &str, delimiter: &str) -> Option<usize> {
    let mut characters = text.char_indices();
    while let Some((index, ch)) = characters.next() {
        if text[index..].starts_with(delimiter) {
            return Some(index);
        }
        if ch == '\\' {
            characters.next();
        }
    }
    None
}

pub(crate) fn render(source: &str) -> Option<RenderedBlock> {
    // Bound recursive layout work before passing server-provided math to the renderer
    if source.is_empty() || source.len() > 1024 {
        return None;
    }
    if source
        .chars()
        .filter(|ch| {
            matches!(
                ch,
                '{' | '}' | '(' | ')' | '[' | ']' | '\\' | '^' | '_' | '/'
            )
        })
        .count()
        > 128
    {
        return None;
    }
    ratex_parser::parse(source).ok()?;
    let block = term_maths::render(source);
    // Unknown commands survive as backslashes; retain the complete original expression
    if block.is_empty() || block.to_string().contains('\\') {
        None
    } else {
        Some(block)
    }
}
