<div align="center">
# cp-cli

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
> Not affiliated with or endorsed by LeetCode, HackerRank, Codeforces, Exercism, Project Euler, HackerEarth or CodeChef

## Overview

`cp-cli` is a Rust CLI for exploring coding problems from the terminal. LeetCode has the
full solve workflow. Other platforms expose the public reads and supported local actions
that cp-cli can perform in the terminal; unavailable actions fail clearly.

## Features

- Browse LeetCode, HackerRank, Codeforces, HackerEarth and Project Euler problems through their public interfaces
- Use the official Exercism CLI for downloading, testing and submitting exercises
- Read HackerRank and CodeChef public profiles, Codeforces blogs and CodeChef Discuss in the same terminal reader
- Fetch a problem by its platform ID or title slug
- Search public problems by title or number without downloading the full catalogue
- Browse a bounded page by difficulty and tag
- Navigate problem, contest and discussion tables with arrow keys, sortable headers and mouse input
- Open the current LeetCode Daily Challenge with one public request
- Pick any available language starter into a local `<slug>.<extension>` file
- Run LeetCode's generated example cases and submit as separate explicit commands
- Validate and clear a configured LeetCode session without opening a browser
- Read styled Markdown, highlighted code, tables and selectable LaTeX equations inside the terminal
- Automatically enlarge titles in Apple Terminal and Windows Terminal
- A responsive raster-glyph possum HUD and honest animated transfer progress
- Possum, arcade, phosphor, amber and moonlight themes that adapt to the terminal profile
- Emit bounded structured JSON for scripts

## Demo

## Installation

Build the CLI from this workspace with Rust 1.88 or newer:

```sh
cargo build --release --locked
./target/release/cp-cli --help
```

## Quick start

```sh
cargo run --locked -- problem daily
cargo run --locked -- --platform hackerrank problem list
cargo run --locked -- --platform codeforces stats tourist
cargo run --locked -- --platform codeforces contest list
cargo run --locked -- --platform exercism problem pick two-fer --track rust
cargo run --locked -- --platform project-euler problem list --recent
cargo run --locked -- --platform codechef discussion list
```

## Commands

`cp-cli problem daily` opens the current LeetCode Daily Challenge using the same
rich text or JSON output as `problem show`.

`cp-cli problem show <ID>` prints the problem details and available statement. LeetCode
accepts a number or slug, HackerRank accepts its URL slug, Codeforces accepts the contest
number plus index, such as `4A`, and Project Euler accepts a positive number. Use
`--format text` (the default) or `--format json`; `--platform leetcode` is optional.

`cp-cli problem list` browses 20 public problems. LeetCode supports
`--difficulty easy|medium|hard`, `--tag graph`, or both without downloading its full
catalogue. HackerRank reads its Algorithms track by default in 20-row pages; select another
track with `--track`. Codeforces exposes one
full public metadata catalogue, then shows it in bounded 20-row pages. Use `--page` with
either platform.
Project Euler reads its complete compact catalogue in 50-problem pages, including new
problems beyond the historical archive, or its ten newest problems with `--recent`.
HackerEarth defaults to its basic input/output practice path; pass the exact nested path
returned by HackerEarth with `--topic`, such as `algorithms/searching/linear-search`.
HackerRank, Codeforces, HackerEarth and Project Euler reject LeetCode-only filters instead of silently
ignoring them.

HackerRank supplies its statement source, so `problem show` renders its Markdown and
mathematics inside the terminal. Codeforces' public API supplies metadata but no statement;
the terminal view therefore shows the rating, tags, solve count and official clickable URL.
Project Euler renders its public statement in the terminal with its required CC BY-NC-SA
attribution. Exercism integrates its official CLI for downloading, testing and submitting an
exercise after `problem pick --track <TRACK>`. HackerEarth renders public practice statements.
CodeChef does not expose a dependable public problem catalogue, so that command fails clearly.

## Platform support

**Terminal-native** means cp-cli renders or performs the action itself. **Official CLI**
means cp-cli invokes the platform's supported upstream tool. A capability marked
**Unavailable** returns a clear terminal error; no command hands the task off to a website.

| Platform | Auth and stats | Problems | Contests | Discussions |
| --- | --- | --- | --- | --- |
| LeetCode | Existing environment or saved session validation and account stats | Terminal-native list, show, starter, test and submit | List, show and status are terminal-native; participation and organizer actions are unavailable | Read in terminal; writing is unavailable without a verified mutation API |
| HackerRank | Terminal-native public profile stats | Terminal-native list, show and starter | Unavailable | Unavailable |
| Codeforces | Signed API credentials and terminal-native public stats | Terminal-native list and metadata show | Terminal-native list and show | Terminal-native recent blogs and comments |
| Exercism | Guided official CLI token configuration | Official `exercism` CLI pick, test and submit | Unavailable | Unavailable |
| Project Euler | Unavailable | Terminal-native catalogue list, `--recent`, and statement show | Unavailable | Unavailable |
| HackerEarth | Unavailable | Terminal-native practice-topic list and statement show | Unavailable | Unavailable |
| CodeChef | Terminal-native public profile stats | Unavailable | Unavailable | Terminal-native CodeChef Discuss list and show |

cp-cli does not copy browser cookies or automate undocumented private APIs.

`cp-cli problem search <QUERY>` shows the first 20 matching problems in a
responsive table. Number and name use contrasting theme colours, linked slugs
are underlined, and Easy, Medium and Hard remain readable colour-coded badges.
Premium results are marked. Quote queries containing spaces, such as
`problem search "binary tree"`. The total match count makes truncation explicit;
refine the query to narrow a longer result set.

`cp-cli problem pick <NUMBER|SLUG>` fetches every starter variant LeetCode offers,
then asks which language to use and where to write `<slug>.<extension>`. HackerRank uses
the same local starter flow. Exercism delegates download, test and submit to the official
`exercism` executable; the chosen `--dir` becomes its configured workspace. Configure its
API token first at `exercism.org/settings/api_cli`. For redirected input, LeetCode and
HackerRank need `--lang` and `--dir`; Exercism needs `--track` and `--dir`.

`cp-cli problem test <FILE>` sends only the marked solution and LeetCode's generated
example cases to the Run endpoint, then shows the output without submitting. `cp-cli
problem submit <FILE>` is a separate explicit action that submits the marked region once,
then polls the judge for up to 60 seconds. Both commands use the language picked for the
solution file.

`cp-cli stats` uses the authenticated LeetCode account to show solved counts, accepted and total
submission counts by difficulty, and the 10 most recent submissions. Add `--format json`
for the same bounded data as structured output. `--platform codeforces stats <HANDLE>`
shows its public profile, rating history and latest 100 submissions. HackerRank and CodeChef
accept their public account name in the same position.

`cp-cli contest list` shows the bounded upcoming LeetCode schedule with UTC start times
and durations. `cp-cli contest show <SLUG>` opens one contest summary using the slug from
its LeetCode URL. Both are public and support text or JSON output. `contest status` reads
the signed-in account's registration state. Codeforces contest list and show are
terminal-native. Contest participation and organizer actions are unavailable when the
platform does not expose a verified callable API.

`cp-cli discussion list [NUMBER|SLUG]` shows trending posts or the newest solution posts
for one problem. `discussion show <ID>` renders the full bounded Markdown body, including
LeetCode's newer article posts, without executing raw HTML or unsafe links. `discussion
create [NUMBER|SLUG]`, `reply <ID>`, `edit <ID>` and `delete <ID>` fail clearly because
LeetCode does not expose a verified mutation contract for them. Codeforces blogs and comments,
plus CodeChef Discuss list and show, are terminal-native. Community write actions are unavailable.

On an interactive terminal, problem, contest and discussion lists and recent submissions
share the same 60%-width reader. Arrow keys move between rows and headers, Enter sorts a
focused header or opens a focused row, the mouse selects and sorts, and `q` closes the reader.
Piped text and JSON keep the stable non-interactive output.

For `problem show`, use a positive problem number or the slug from its URL, such as
`1` or `two-sum`. Full URLs are not accepted. Slugs accept 1–128 ASCII letters,
digits, hyphens or underscores.

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

Problem prose keeps the terminal foreground for long-form comfort. Section and
example headings use the theme accent, while inputs, outputs and inline code use
a secondary theme colour; mathematics and links retain their own semantic styling.
The same hierarchy remains visible through headings, weight and underline when
colour is unavailable.

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
uses four bounded frames. After the command exits, native blink attributes keep
the final possum blinking and scratching its ear without a resident process, and
the `QUEST` label remains hoverable and clickable where terminal hyperlinks are
supported. When the
server supplies a response size, it settles beside a bracketed arcade bar
showing actual bytes received. It clears before the problem appears; no
percentage is invented while waiting. `--no-animation` replaces it with one
still possum and disables the retained blink. Progress only appears when both
output streams are terminals, and never in JSON mode. Terminals that ignore the
blink or hyperlink controls show the same static, readable output.

`--sound` rings one terminal bell after successful text output. It is opt-in,
never runs for JSON, redirected output or failures, and follows the terminal
profile's own audible or visual bell setting.

Short problem titles sit beside the possum as bold sentence-case text in normal-size
terminals. Apple Terminal (`TERM_PROGRAM=Apple_Terminal`) and Windows Terminal
(`WT_SESSION`) use native double-height, double-width titles above normal-size body
text because terminal sizing applies to a complete row. `--heading-size auto`
enlarges titles at widths of 60 columns or more; `--heading-size large` also enables
them in narrower windows;
`--heading-size normal` disables enlargement. Long titles wrap at the enlarged
size. Other terminals and multiplexers use normal-size, bold titles. This changes
the title's rendered size, not your terminal profile's font setting. Enlargement
still works with `--color never` or `NO_COLOR`. These terminals offer normal and
double-size text, not arbitrary per-word point sizes.

Problem lists and searches open as an embedded table reader when input and output
are attached to a terminal at least 80 columns wide. The table uses 60% of the live
terminal width. Use Up and Down to move, move above the first row to focus the
headers, Left and Right to choose a column, and Enter to toggle its sort direction.
Enter on a row opens that problem. Mouse wheels, row clicks and column-header clicks
work in terminals with mouse reporting. Escape, `q` and Ctrl-C close the reader
immediately. JSON, redirected output and narrow terminals keep the static table
output. Problem statements remain in normal scrollback at a reading width capped at
100 columns.

LaTeX between `$…$`, `$$…$$`, `\(…\)` and `\[…\]` renders as Unicode math,
including stacked fractions, roots, scripts and matrices. It works in both
Apple Terminal and Windows Terminal without a browser, graphics protocol or
external TeX installation. Multiline equations get their own block; unsupported,
oversized or malformed expressions remain readable as their original LaTeX.
This is terminal typesetting, not pixel-perfect TeX. Code examples preserve
literal math delimiters. Images are shown as descriptions and links without
fetching their contents. Your terminal font must include the displayed Unicode
characters.

Use `--help`, `problem --help` or a command's `--help`; there is no `help`
subcommand.

Problem JSON contains `id` (the slug), `title` and `statement` (HTML). Search JSON
contains `query`, `total` and compact `results`; list JSON contains its active
filters, `total` and the same compact results. Diagnostics go to stderr. Invalid
arguments exit with status 2; request and output failures exit with status 1.
Closing an output pipe early exits successfully.

### Authentication

Public browsing and starter retrieval never send cookies. LeetCode does not expose OAuth,
a device callback or another documented terminal-native login contract, so
`cp-cli auth login` returns an explicit unavailable-capability error and opens nothing.
cp-cli does not proxy credentials, read a browser profile or copy browser cookies.

`cp-cli auth status` validates the active credentials. For automation,
`LEETCODE_SESSION` plus `LEETCODE_CSRFTOKEN` (or `LEETCODE_CSRF_TOKEN`) remain supported
and take precedence over secure storage. On headless Linux without Secret Service, use
the environment variables.

`cp-cli auth logout` removes only a compatible credential record saved by an earlier
cp-cli version. It cannot clear the parent shell's environment variables.

Codeforces uses its documented API-key flow. Run
`cp-cli --platform codeforces auth login`; cp-cli accepts a preissued key and secret
with hidden secret input, verifies the exact signed
`user.friends` request, then stores one record in the operating system credential store.
For automation, `CODEFORCES_API_KEY` and `CODEFORCES_API_SECRET` take precedence.

Exercism uses its official CLI configuration. Run
`cp-cli --platform exercism auth login`; cp-cli accepts a preissued token with hidden input
and invokes `exercism configure --token`. If the official CLI is missing, the error says
exactly which executable is required.

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

Build without LeetCode using `--no-default-features`; LeetCode commands then
report that the platform feature is disabled.

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
