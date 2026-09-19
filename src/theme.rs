use std::{io::IsTerminal, time::Duration};

use anstyle::{Ansi256Color, Style};
use pulldown_cmark_mdcat::Theme as MarkdownTheme;

use crate::cli::{Background, Theme};

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) gradient: [u8; 4],
    pub(crate) accent: u8,
    code: u8,
    math: u8,
    link: u8,
    pub(crate) shadow: u8,
    light: bool,
}

pub(crate) fn background(mode: Background, query: bool, theme: Theme) -> [u8; 3] {
    match mode {
        Background::Dark => return [18; 3],
        Background::Light => return [245; 3],
        Background::Auto => {}
    }
    if query && std::io::stdin().is_terminal() && std::io::stderr().is_terminal() {
        let mut options = terminal_colorsaurus::QueryOptions::default();
        options.timeout = Duration::from_millis(150);
        if let Ok(color) = terminal_colorsaurus::background_color(options) {
            let (r, g, b) = color.scale_to_8bit();
            return [r, g, b];
        }
    }
    // Profiles without OSC support can advertise their background through COLORFGBG
    if let Some(index) = std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| value.rsplit(';').next()?.parse::<u8>().ok())
    {
        return if index >= 16 {
            rgb(index)
        } else if matches!(index, 7 | 15) {
            [245; 3]
        } else {
            [18; 3]
        };
    }
    if matches!(theme, Theme::Light) {
        [245; 3]
    } else {
        [18; 3]
    }
}

impl Theme {
    pub(crate) fn palette(self, background: [u8; 3]) -> Palette {
        let light = luminance(background) > 0.179;
        let (gradient, accent, code) = match (self, light) {
            (Self::Possum, false) => ([252, 252, 251, 251], 181, 250),
            (Self::Possum, true) => ([238, 238, 239, 239], 95, 239),
            (Self::Arcade, false) => ([51, 87, 201, 198], 51, 250),
            (Self::Arcade, true) => ([24, 25, 90, 89], 25, 239),
            (Self::Moonlight, false) => ([252, 252, 251, 251], 110, 250),
            (Self::Moonlight, true) => ([238, 238, 239, 239], 24, 239),
            (Self::Phosphor, false) => ([252, 252, 251, 251], 108, 250),
            (Self::Phosphor, true) => ([238, 238, 239, 239], 28, 239),
            (Self::Amber, false) => ([252, 252, 251, 251], 179, 250),
            (Self::Amber, true) => ([238, 238, 239, 239], 94, 239),
            (_, false) => ([252, 252, 251, 251], 250, 250),
            (_, true) => ([238, 238, 239, 239], 239, 239),
        };
        let background = luminance(background);
        let gradient = gradient.map(|color| readable(color, background));
        Palette {
            gradient,
            accent: readable(accent, background),
            code: readable(code, background),
            math: readable(code, background),
            link: readable(accent, background),
            shadow: if light { 250 } else { 239 },
            light,
        }
    }
}

impl Palette {
    pub(crate) fn markdown(self) -> MarkdownTheme {
        let mut theme = if self.light {
            MarkdownTheme::light()
        } else {
            MarkdownTheme::dark()
        };
        let heading = foreground(self.accent).bold();
        theme.h2_style = heading;
        theme.h3_style = heading;
        theme.h4_style = heading;
        theme.h5_style = heading;
        theme.h6_style = heading;
        theme.h1_text_style = heading;
        theme.h1_prefix_style = Style::new();
        theme.code_style = foreground(self.code);
        theme.math_style = foreground(self.math);
        theme.link_style = foreground(self.link).underline();
        theme.image_link_style = theme.link_style;
        theme.footnote_style = foreground(self.link);
        theme.quote_border_style = foreground(self.accent);
        theme.rule_color = Ansi256Color(self.accent).into();
        theme
    }
}

pub(crate) fn foreground(index: u8) -> Style {
    Style::new().fg_color(Some(Ansi256Color(index).into()))
}

fn rgb(index: u8) -> [u8; 3] {
    if index >= 232 {
        return [8 + 10 * (index - 232); 3];
    }
    let index = index.saturating_sub(16);
    let steps = [0, 95, 135, 175, 215, 255];
    [
        steps[usize::from(index / 36)],
        steps[usize::from(index / 6 % 6)],
        steps[usize::from(index % 6)],
    ]
}

fn luminance(color: [u8; 3]) -> f64 {
    let [r, g, b] = color.map(|channel| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast(first: f64, second: f64) -> f64 {
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn readable(index: u8, background: f64) -> u8 {
    let original = rgb(index);
    if contrast(luminance(original), background) >= 4.5 {
        return index;
    }
    // Preserve the nearest available color that meets normal-text contrast
    (16..=255)
        .filter(|candidate| contrast(luminance(rgb(*candidate)), background) >= 4.5)
        .min_by_key(|candidate| {
            rgb(*candidate)
                .into_iter()
                .zip(original)
                .map(|(a, b)| (i32::from(a) - i32::from(b)).pow(2))
                .sum::<i32>()
        })
        .unwrap_or(if background > 0.179 { 16 } else { 231 })
}

#[cfg(test)]
#[test]
fn palettes_follow_profile_contrast() {
    for background in [
        [0; 3],
        [18; 3],
        [245; 3],
        [255; 3],
        [128; 3],
        [45, 65, 85],
        [200, 175, 150],
    ] {
        for theme in [
            Theme::Possum,
            Theme::Arcade,
            Theme::Moonlight,
            Theme::Phosphor,
            Theme::Amber,
            Theme::Dark,
            Theme::Light,
        ] {
            let palette = theme.palette(background);
            for color in palette.gradient.into_iter().chain([
                palette.accent,
                palette.code,
                palette.math,
                palette.link,
            ]) {
                assert!(contrast(luminance(rgb(color)), luminance(background)) >= 4.5);
            }
        }
    }
}
