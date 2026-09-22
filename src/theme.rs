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
    pub(crate) result_number: u8,
    pub(crate) result_title: u8,
    pub(crate) result_slug: u8,
    pub(crate) easy: u8,
    pub(crate) medium: u8,
    pub(crate) hard: u8,
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
            (Self::Possum, false) => ([252, 252, 251, 251], 181, 110),
            (Self::Possum, true) => ([238, 238, 239, 239], 95, 24),
            (Self::Arcade, false) => ([51, 87, 201, 198], 51, 87),
            (Self::Arcade, true) => ([24, 25, 90, 89], 25, 24),
            (Self::Moonlight, false) => ([252, 252, 251, 251], 110, 181),
            (Self::Moonlight, true) => ([238, 238, 239, 239], 24, 95),
            (Self::Phosphor, false) => ([252, 252, 251, 251], 108, 151),
            (Self::Phosphor, true) => ([238, 238, 239, 239], 28, 22),
            (Self::Amber, false) => ([252, 252, 251, 251], 179, 223),
            (Self::Amber, true) => ([238, 238, 239, 239], 94, 58),
            (_, false) => ([252, 252, 251, 251], 250, 110),
            (_, true) => ([238, 238, 239, 239], 239, 24),
        };
        let background = luminance(background);
        let gradient = gradient.map(|color| readable(color, background));
        let accent = readable(accent, background);
        let code = readable_distinct(code, background, accent);
        let result_title = gradient
            .into_iter()
            .find(|color| *color != accent)
            .unwrap_or(accent);
        let result_title = readable_distinct(result_title, background, accent);
        Palette {
            gradient,
            accent,
            code,
            math: accent,
            link: accent,
            result_number: accent,
            result_title,
            result_slug: if matches!(self, Self::Arcade) {
                gradient[1]
            } else {
                accent
            },
            easy: readable(if light { 28 } else { 108 }, background),
            medium: readable(if light { 94 } else { 179 }, background),
            hard: readable(if light { 124 } else { 174 }, background),
            shadow: readable(if light { 250 } else { 239 }, background),
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
        .min_by_key(|candidate| color_distance(rgb(*candidate), original))
        .unwrap_or(if background > 0.179 { 16 } else { 231 })
}

fn readable_distinct(index: u8, background: f64, other: u8) -> u8 {
    const MIN_DISTANCE: i32 = 30 * 30;
    let original = rgb(index);
    let other = rgb(other);
    let candidate = readable(index, background);
    if color_distance(rgb(candidate), other) >= MIN_DISTANCE {
        return candidate;
    }
    (16..=255)
        .filter(|candidate| {
            contrast(luminance(rgb(*candidate)), background) >= 4.5
                && color_distance(rgb(*candidate), other) >= MIN_DISTANCE
        })
        .min_by_key(|candidate| color_distance(rgb(*candidate), original))
        .unwrap_or(candidate)
}

fn color_distance(first: [u8; 3], second: [u8; 3]) -> i32 {
    first
        .into_iter()
        .zip(second)
        .map(|(a, b)| (i32::from(a) - i32::from(b)).pow(2))
        .sum()
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
                palette.result_number,
                palette.result_title,
                palette.result_slug,
                palette.easy,
                palette.medium,
                palette.hard,
                palette.shadow,
            ]) {
                assert!(contrast(luminance(rgb(color)), luminance(background)) >= 4.5);
            }
            assert_ne!(palette.result_number, palette.result_title);
            assert_ne!(palette.accent, palette.code);
        }
    }
}
