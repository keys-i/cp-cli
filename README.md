<div align="center">

# cp-cli

[![Crates.io](https://img.shields.io/crates/v/cp-cli.svg)](https://crates.io/crates/cp-cli)
[![Downloads](https://img.shields.io/crates/d/cp-cli.svg)](https://crates.io/crates/cp-cli)
[![License](https://img.shields.io/github/license/keys-i/cp-cli)](LICENSE)

[Install](#installation) · [Usage](#usage) · [Configuration](#configuration) · [Changelog](docs/CHANGELOG.md) · [Contributing](docs/CONTRIBUTING.md) · [Security](docs/SECURITY.md)

</div>

> [!NOTE]
> cp-cli is not affiliated with or endorsed by LeetCode, HackerRank, Codeforces, Exercism, Project Euler, HackerEarth, or CodeChef.

`cp-cli` is a Rust terminal client for coding-problem platforms. It supports public browsing, local starter files, LeetCode testing and submission, structured JSON, and terminal rendering for Markdown, code, tables, and LaTeX.

## Installation

### macOS and Linux

The native installer detects Apple silicon, Intel macOS, x64 Linux, and arm64 Linux; verifies the release checksum; and installs to `~/.local/bin` by default.

```sh
curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSLO https://github.com/keys-i/cp-cli/releases/latest/download/install.sh
sh install.sh
```

Use `--version 3.1.8` for a specific release or `--install-dir <DIRECTORY>` for another location. The installer reports when the directory must be added to `PATH`.

### Windows

PowerShell detects x64 or arm64 Windows, verifies the checksum, installs for the current user, and adds its directory to the user `PATH` when needed.

```powershell
Invoke-WebRequest https://github.com/keys-i/cp-cli/releases/latest/download/install.ps1 -OutFile "$env:TEMP\cp-cli-install.ps1"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$env:TEMP\cp-cli-install.ps1"
```

Add `-Version 3.1.8` or `-InstallDir <DIRECTORY>` after the script path to override defaults. `-ExecutionPolicy Bypass` applies only to that process.

### Manual archives

[GitHub Releases](https://github.com/keys-i/cp-cli/releases/latest) provides these archives. Every release includes `SHA256SUMS`, covering the installers and all archives, plus release provenance.

| Machine | Release archive |
| --- | --- |
| macOS, Apple silicon | `cp-cli-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `cp-cli-x86_64-apple-darwin.tar.gz` |
| Linux, x64 | `cp-cli-x86_64-unknown-linux-musl.tar.gz` |
| Linux, arm64 | `cp-cli-aarch64-unknown-linux-musl.tar.gz` |
| Windows, x64 | `cp-cli-x86_64-pc-windows-msvc.zip` |
| Windows, arm64 | `cp-cli-aarch64-pc-windows-msvc.zip` |

### Cargo and source

```sh
cargo install cp-cli --locked
cp-cli --help
```

Build this checkout with Rust 1.88 or newer:

```sh
cargo build --release --locked
./target/release/cp-cli --help
```

## Usage

```sh
cargo run --locked -- problem daily
cargo run --locked -- --platform hackerrank problem list
cargo run --locked -- --platform codeforces stats tourist
cargo run --locked -- --platform codeforces contest list
cargo run --locked -- --platform exercism problem pick two-fer --track rust
cargo run --locked -- --platform project-euler problem list --recent
cargo run --locked -- --platform codechef discussion list
```

`problem show <ID>` prints a statement and details. LeetCode accepts a number or slug, HackerRank a URL slug, Codeforces a contest/index such as `4A`, and Project Euler a positive number. `--platform leetcode` is optional; use `--format text` (default) or `--format json`. `problem daily` opens the public LeetCode Daily Challenge in the same formats.

`problem list` shows 20 results at a time. LeetCode accepts `--difficulty easy|medium|hard`, `--tag graph`, or both without downloading its full catalogue. HackerRank uses 20-row Algorithms pages by default and accepts `--track`; Codeforces fetches one metadata catalogue and shows 20-row pages; both accept `--page`. Project Euler shows its full compact catalogue in 50-problem pages or its newest ten with `--recent`. HackerEarth defaults to basic input/output practice; select a returned nested path with `--topic`, for example `algorithms/searching/linear-search`. LeetCode-only filters are rejected on other platforms.

`problem search <QUERY>` returns the first 20 matches and a total; quote spaces, for example `problem search "binary tree"`. `problem pick <NUMBER|SLUG>` writes a selected LeetCode or HackerRank starter to `<slug>.<extension>`. Exercism delegates pick, test, and submit to `exercism`; `--dir` is its workspace, and the token is configured at `exercism.org/settings/api_cli`. Redirected LeetCode/HackerRank input needs `--lang` and `--dir`; Exercism needs `--track` and `--dir`.

`problem test <FILE>` runs the marked solution against LeetCode’s generated examples and does not submit. `problem submit <FILE>` submits the marked region once and polls for up to 60 seconds. Both use the file’s selected language. `stats` returns authenticated LeetCode counts by difficulty and ten recent submissions; `--format json` returns the same bounded data. `--platform codeforces stats <HANDLE>` returns a public profile, rating history, and 100 recent submissions; HackerRank and CodeChef accept a public account name in the same position.

`contest list` and `contest show <SLUG>` provide LeetCode’s public upcoming UTC schedule and summary; both support text or JSON. `contest status` reads signed-in registration state. `discussion list [NUMBER|SLUG]` lists trending posts or recent problem solutions; `discussion show <ID>` renders bounded Markdown, including newer LeetCode articles, without raw HTML or unsafe links. Use `--help`, `problem --help`, or a command’s `--help`; there is no `help` subcommand.

In an interactive terminal, lists and recent submissions use a reader that is 60% of terminal width. Arrows move rows and headers, Enter sorts or opens, mouse controls select or sort, and `q`, Escape, or Ctrl-C closes it. At under 80 columns, with redirected output, or in JSON mode, tables are static. Statements use normal scrollback capped at 100 columns. `problem show` accepts only a positive number or a 1–128-character ASCII slug (`1`, `two-sum`), never a full URL.

### Platform support

**Terminal-native** means cp-cli performs the action. **Official CLI** means it invokes the platform tool. **Unavailable** returns an error.

| Platform | Auth and stats | Problems | Contests | Discussions |
| --- | --- | --- | --- | --- |
| LeetCode | Existing environment or saved-session validation and account stats | Terminal-native list, show, starter, test, and submit | Terminal-native list, show, and status; participation and organizer actions unavailable | Read in terminal; writing unavailable without a verified mutation API |
| HackerRank | Terminal-native public profile stats | Terminal-native list, show, and starter | Unavailable | Unavailable |
| Codeforces | Signed API credentials and terminal-native public stats | Terminal-native list and metadata show | Terminal-native list and show | Terminal-native recent blogs and comments |
| Exercism | Guided official CLI token configuration | Official `exercism` CLI pick, test, and submit | Unavailable | Unavailable |
| Project Euler | Unavailable | Terminal-native catalogue list, `--recent`, and statement show | Unavailable | Unavailable |
| HackerEarth | Unavailable | Terminal-native practice-topic list and statement show | Unavailable | Unavailable |
| CodeChef | Terminal-native public profile stats | Unavailable | Unavailable | Terminal-native CodeChef Discuss list and show |

HackerRank statements render Markdown and maths. Codeforces has public metadata but no statements, so its view provides rating, tags, solve count, and an official URL. Project Euler retains required CC BY-NC-SA attribution. HackerEarth renders public practice statements. CodeChef has no dependable public problem catalogue. Contest participation, organizer actions, and community writes remain unavailable where no verified API exists. cp-cli does not copy browser cookies or automate undocumented private APIs.

## Configuration

Colour is automatic; override with `--color always` or `--color never`. `NO_COLOR` disables automatic colour, and `TERM=dumb` also disables enlarged titles. Redirected text and JSON have no terminal controls by default; `--color always` can colour redirected text, but JSON remains clean.

Themes are `possum` (default), `arcade`, `phosphor`, `amber`, and `moonlight`. The possum is an original colour-aware raster glyph using the cell-as-pixel approach described in [Rendering in the Terminal](https://benmandrew.com/articles/terminal-renderer); narrow windows use a text badge and very narrow windows omit it. Problem text keeps the terminal foreground, with theme styling for headings, inputs, outputs, code, mathematics, and links. cp-cli does not change the terminal font, profile colours, or background.

`--background auto` queries the terminal briefly, then tries `COLORFGBG`, then assumes dark (or light for `--theme light`). Override with `--background light` or `--background dark`. The 256-colour palette targets 4.5:1 contrast for the detected background under [WCAG text contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html); custom palettes and translucent backgrounds can change actual contrast.

While connecting, the possum has four bounded 260 ms frames. If the server supplies a response size, the progress bar reports received bytes; it never invents a percentage. `--no-animation` shows a still possum. Progress is disabled for JSON and unless both output streams are terminals. `--sound` emits one terminal bell only after successful text output, never for JSON, redirected output, or failures.

Apple Terminal (`TERM_PROGRAM=Apple_Terminal`) and Windows Terminal (`WT_SESSION`) can use native double-size titles. `--heading-size auto` enables them at 60 columns or wider; `--heading-size large` enables them in narrower windows; `--heading-size normal` disables them. Other terminals use normal bold titles. This changes rendered title size, not the font, and works with `--color never` and `NO_COLOR`.

LaTeX in `$…$`, `$$…$$`, `\(…\)`, and `\[…\]` renders as Unicode math, including fractions, roots, scripts, and matrices, without a browser, graphics protocol, or TeX installation. Multiline equations are blocks; unsupported, oversized, and malformed expressions stay as source. Code preserves literal delimiters. Images are descriptions and links only and are not fetched. The terminal font must contain the required Unicode characters.

Problem JSON has `id` (slug), `title`, and HTML `statement`. Search JSON has `query`, `total`, and compact `results`; list JSON adds active filters. Diagnostics use stderr. Invalid arguments exit 2; request or output failures exit 1; a closed output pipe exits successfully.

### Authentication

Public browsing and starter retrieval never send cookies. LeetCode has no documented OAuth, device callback, or terminal-native login contract; `cp-cli auth login` returns an unavailable-capability error and opens nothing. `auth status` validates active credentials. `LEETCODE_SESSION` plus `LEETCODE_CSRFTOKEN` (or `LEETCODE_CSRF_TOKEN`) take precedence over secure storage; on headless Linux without Secret Service, use environment variables. `auth logout` removes only compatible cp-cli credential records and cannot clear parent-shell variables.

`cp-cli --platform codeforces auth login` accepts a preissued key and hidden secret, verifies the exact signed `user.friends` request, and stores one record in the operating-system credential store. `CODEFORCES_API_KEY` and `CODEFORCES_API_SECRET` take precedence. `cp-cli --platform exercism auth login` accepts a hidden preissued token and runs `exercism configure --token`; an absent CLI reports the required executable.

## Development

```sh
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

cp-cli uses Reqwest with Rustls, `htmd`, `pulldown-cmark-mdcat`, and `term-maths`. Requests have connection, read, and total timeouts; do not follow redirects or retry; and accept at most 2 MiB. Tests use a local HTTP server and do not contact LeetCode. Syntax definitions load only for coloured code blocks with a language tag. Build without LeetCode using `--no-default-features`; its commands then report the platform feature is disabled.

## Security

Do not open a public issue with session cookies or authentication tokens, account credentials, private submission data, a working exploit for an unpatched vulnerability, or sensitive request/response logs. Follow [SECURITY.md](docs/SECURITY.md) for private reporting.

## Contributing and license

Before a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Bug fixes, documentation, tests, and focused features are welcome; [CONTRIBUTING.md](docs/CONTRIBUTING.md) has the review process. Licensed under the [MIT License](LICENSE).
