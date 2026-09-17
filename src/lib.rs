mod cli;
mod domain;
mod error;
mod math;
mod output;
mod possum;
mod problems;
mod theme;

use std::{
    io::{IsTerminal, Write},
    time::Duration,
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

pub use error::Error;

pub async fn run() -> Result<(), Error> {
    let cli = cli::Cli::parse();
    let cli::Command::Problem { command } = cli.command;
    let cli::ProblemCommand::Show { id } = command;
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
    let progress = if terminal && std::io::stderr().is_terminal() {
        let progress = ProgressBar::new_spinner();
        progress.set_style(spinner_style);
        progress.set_message(format!("Opening {}", id.as_ref()));
        if !cli.no_animation {
            progress.enable_steady_tick(Duration::from_millis(260));
        } else {
            progress.tick();
        }
        Some(progress)
    } else {
        None
    };
    let result = match cli.platform {
        cli::Platform::LeetCode => {
            problems::show(&id, |bytes, total| {
                if let Some(progress) = &progress {
                    if let Some(total) = total.filter(|total| *total > 0)
                        && progress.length().is_none()
                    {
                        progress.disable_steady_tick();
                        progress.set_style(bar_style.clone());
                        progress.set_length(total);
                    }
                    progress.set_position(bytes as u64);
                }
            })
            .await
        }
    };
    if let Some(progress) = progress {
        progress.finish_and_clear();
    }
    let problem = result?;
    let color = if presentation.color || presentation.large_heading {
        anstream::ColorChoice::AlwaysAnsi
    } else {
        anstream::ColorChoice::Never
    };
    let stdout = anstream::AutoStream::new(std::io::stdout(), color);
    output::problem(stdout.lock(), cli.format, &problem, presentation)?;
    if cli.sound && terminal && std::io::stderr().is_terminal() {
        std::io::stderr().lock().write_all(b"\x07")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
