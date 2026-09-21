<div align="center">
# LeetCode CLI

[![Crates.io](https://img.shields.io/crates/v/leetcode-cli).svg)](https://crates.io/crates/leetcode-cli)
[![Downloads](https://img.shields.io/crates/d/leetcode-cli).svg)](https://crates.io/crates/leetcode-cli)
[![License](https://img.shields.io/github/license/keys-i/leetcode-cli)](LICENSE)

[Installation](#installation) ·
[Quick start](#quick-start) ·
[Commands](#commands) ·
[Configuration](#configuration) ·
[Contributing](CONTRIBUTING.md) ·
[Security](SECURITY.md)

</div>

> [!NOTE]
> Not affiliated with or endorsed by LeetCode

## Overview

`cp-cli` is a Rust CLI for finding and reading public LeetCode problems from the terminal.

## Features

- Fetch a problem by its title slug
- Search public problems by title or number without downloading the full catalogue
- Read styled Markdown, highlighted code, tables and selectable LaTeX equations inside the terminal
- Automatically enlarge titles in Apple Terminal and Windows Terminal
- A responsive raster-glyph possum HUD and honest animated transfer progress
- Possum, arcade, phosphor, amber and moonlight themes that adapt to the terminal profile
- Emit JSON containing the original HTML for scripts

## Demo

## Installation

Build the CLI from this workspace with Rust 1.88 or newer:

```sh
cargo build --release --locked
./target/release/cp-cli --help
```

## Quick Start

```sh
cargo run --locked -- problem search "two sum"
cargo run --locked -- problem show two-sum
cargo run --locked -- --theme arcade problem show two-sum
cargo run --locked -- problem show two-sum --heading-size large
cargo run --locked -- --platform leetcode --format json problem show two-sum
```

## Commands

`cp-cli problem show <SLUG>` prints a problem's title and statement. Use
`--format text` (the default) or `--format json`; `--platform leetcode` is optional.

`cp-cli problem search <QUERY>` shows the first 20 matching problems in a
responsive table. Number and name use contrasting theme colours, linked slugs
are underlined, and Easy, Medium and Hard remain readable colour-coded badges.
Premium results are marked. Quote queries containing spaces, such as
`problem search "binary tree"`. The total match count makes truncation explicit;
refine the query to narrow a longer result set.

For `problem show`, use the slug from the problem URL, such as `two-sum`.
Numeric problem-number lookup and full URLs are not accepted by that command.
Slugs accept 1–128 ASCII letters, digits, hyphens or underscores.

### Terminal appearance

Color is automatic. Use `--color always` or `--color never` to override it;
`NO_COLOR` disables automatic color; `TERM=dumb` also disables enlarged titles.
Redirected text and JSON
contain no terminal controls by default. `--color always` can color redirected
text, but JSON always remains clean.

The default `possum` theme uses soft neutral text with a muted pink accent. On a
color terminal, an original possum sprite is sampled into aspect-corrected,
color-aware glyphs using the cell-as-pixel approach from
[Rendering in the Terminal](https://benmandrew.com/articles/terminal-renderer),
rather than hand-shaped block art. It keeps the grey fur,
cream face, dark eyes, pink nose, paws and curling tail legible in Apple Terminal
and Windows Terminal. Narrow windows receive a text badge and very constrained
windows omit decoration. The `POSSUM//ARCADE` HUD, `QUEST leetcode/two-sum`
label, double-size title and offset shadow form one retro grammar without
surrounding the statement in chrome.

`--theme arcade` supplies the vivid cyan, violet and pink 80s palette.
`--theme phosphor` uses subdued terminal green, `--theme amber` uses warm amber,
and `--theme moonlight` uses muted blue. The default keeps long reading calm;
neon remains an explicit choice. The CLI cannot replace a terminal profile's
font; arcade lettering comes from the monospace composition, double-size title
and compact system labels, so content stays selectable and searchable.

`--background auto` queries the terminal background with a short timeout, then
tries `COLORFGBG`, then assumes dark (light for `--theme light`). Override it with
`--background light` or `--background dark` when detection is unavailable or a
slow remote connection times out. Body text keeps the profile's foreground;
the CLI never changes profile colors or paints a background. The 256-color
accent palette targets at least 4.5:1 contrast against the detected background,
following [WCAG's text contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).
Custom remapping of the extended palette and translucent backgrounds can change
the actual contrast. No palette is universally the most comfortable.

While a request connects, the possum looks toward the live status cursor,
blinks, and occasionally scratches an ear. The animation updates at 260 ms and
uses four bounded frames. After the command exits, the terminal keeps the final
possum's eyes blinking without a resident process, and the `QUEST` label remains
hoverable and clickable where terminal hyperlinks are supported. When the
server supplies a response size, it settles beside a bracketed arcade bar
showing actual bytes received. It clears before the problem appears; no
percentage is invented while waiting. `--no-animation` replaces it with one
still possum and disables the retained blink. Progress only appears when both
output streams are terminals, and never in JSON mode. Terminals that ignore the
blink or hyperlink controls show the same static, readable output.

`--sound` rings one terminal bell after successful text output. It is opt-in,
never runs for JSON, redirected output or failures, and follows the terminal
profile's own audible or visual bell setting.

Apple Terminal (`TERM_PROGRAM=Apple_Terminal`) and Windows Terminal (`WT_SESSION`)
use native double-height, double-width titles above normal-size body text.
`--heading-size auto` enlarges titles at widths of 60 columns or more;
`--heading-size large` also enables them in narrower windows;
`--heading-size normal` disables enlargement. Long titles wrap at the enlarged
size. Other terminals and multiplexers use normal-size, bold titles. This changes
the title's rendered size, not your terminal profile's font setting. Enlargement
still works with `--color never` or `NO_COLOR`. These terminals offer normal and
double-size text, not arbitrary per-word point sizes.

Layout uses the terminal width at invocation, capped at 100 columns for reading.
Run the command again after resizing. Output stays in normal scrollback and the
command exits without opening a reader or waiting for keys.

LaTeX between `$…$`, `$$…$$`, `\(…\)` and `\[…\]` renders as Unicode math,
including stacked fractions, roots, scripts and matrices. It works in both
Apple Terminal and Windows Terminal without a browser, graphics protocol or
external TeX installation. Multiline equations get their own block; unsupported,
oversized or malformed expressions remain readable as their original LaTeX.
This is terminal typesetting, not pixel-perfect TeX. Code examples preserve
literal math delimiters. Images are shown as descriptions and links without
fetching their contents. Your terminal font must include the displayed Unicode
characters.

Use `--help`, `problem --help` or `problem show --help`; there is no `help`
subcommand.

Problem JSON contains `id` (the slug), `title` and `statement` (HTML). Search JSON
contains `query`, `total` and compact `results`. Diagnostics go to stderr. Invalid
arguments exit with status 2; request and output failures exit with status 1.
Closing an output pipe early exits successfully.

### Authentication

This command uses public problem data without cookies. Premium-only statements
that require a session cannot be fetched yet.

### Config

### Shell completions

## Development

```sh
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

The client uses Reqwest with Rustls for HTTPS, `htmd` and `pulldown-cmark-mdcat`
for Markdown, and `term-maths` for built-in Unicode math. Requests have connection,
read and total timeouts, do not follow
redirects or retry, and accept at most 2 MiB of response data. Tests use a local
HTTP server and do not contact LeetCode. Syntax definitions load only for colored
code blocks with a language tag; plain examples and JSON avoid that allocation.

Build without LeetCode using `--no-default-features`; requesting a problem then
reports that the platform feature is disabled.

## Security

Do not open a public issue containing:
- Session cookies or authentication tokens
- Account credentials
- Private submission data
- A working exploit for an unpatched vulnerability
- Sensitive request or response logs

Follow the private reporting instructions in [SECURITY.md](SECURITY.md).

## Contributing

Bug fixes, documentation improvements, tests, and focused feature additions are welcome.

Before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) for the complete development and review process.

## License

Licensed under the [MIT License](LICENSE)
