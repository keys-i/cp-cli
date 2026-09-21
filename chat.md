# Chat transcript

> This export contains the visible user and assistant messages from the current conversation. Hidden system/developer instructions, tool calls, and private reasoning are not included.

## User

how do i make leetcode into a cli

## Assistant

The quickest way on your Mac is to use an existing Rust CLI rather than reimplementing LeetCode’s private web integration.

## Best terminal-first option

Install `clearloop/leetcode-cli`:

```bash
cargo install leetcode-cli
```

Sign into `leetcode.com` in Chrome once, then:

```bash
leetcode pick 1          # fetch Two Sum and show its description
leetcode edit 1          # generate and open the solution
leetcode test 1          # run sample cases
leetcode test 1 --watch  # rerun whenever you save
leetcode exec 1          # submit to LeetCode
```

It also supports problem filtering, the daily challenge, submission statistics, local caching and shell completions. On macOS and Linux, it can read your existing Chrome LeetCode cookies automatically. ([github.com](https://github.com/clearloop/leetcode-cli))

## Configure it for Rust and VS Code

Run any `leetcode` command once. It creates:

```text
~/.leetcode/leetcode.toml
```

Edit that file:

```toml
[code]
editor = "code"
editor_args = ["--wait"]
lang = "rust"
test = true

# Include the problem statement in the generated file
comment_problem_desc = true
comment_leading = "//"

# Clearly mark the part that gets submitted
edit_code_marker = true
start_marker = "leetcode start"
end_marker = "leetcode end"

[storage]
root = "~/Code/leetcode"
code = "problems"
scripts = "scripts"

[cookies]
csrf = ""
session = ""
site = "leetcode.com"
```

`$VISUAL` and `$EDITOR` override the configured editor. The storage directory can also be moved into a proper Cargo project when you want `cargo check`, full rust-analyzer support and your own local test infrastructure. ([github.com](https://github.com/clearloop/leetcode-cli/blob/main/docs/configuration.md))

Add Zsh completion:

```bash
echo 'eval "$(leetcode completions zsh)"' >> ~/.zshrc
source ~/.zshrc
```

Your normal workflow then becomes:

```bash
leetcode pick --daily
leetcode edit --daily
leetcode test --daily --watch
```

When ready, submit using its problem number:

```bash
leetcode exec 1
```

## Easier Homebrew alternative

`leetgo` is another strong option, particularly when you want generated, locally executable test harnesses:

```bash
brew install leetgo

mkdir -p ~/Code/leetcode
cd ~/Code/leetcode

leetgo init -t us -l rust
leetgo pick today
leetgo edit last
leetgo test last -L
leetgo submit last
```

You can test and submit together:

```bash
leetgo test last -L -s
```

`leetgo` currently provides local-testing support for Rust, C++, Go and Python, while generating submission code for many more languages. It can read cookies from Chrome, Firefox, Safari and Edge; Safari may require Full Disk Access for your terminal. ([github.com](https://github.com/j178/leetgo))

## Building your own CLI

A clean custom command surface would be:

```text
lc daily
lc list --difficulty medium --tag graph
lc get 200 --lang rust
lc test 200
lc submit 200
lc status
lc open 200
```

For a Rust implementation:

```bash
cargo new lc --bin
cd lc

cargo add clap --features derive
cargo add tokio --features macros,rt-multi-thread
cargo add reqwest --features json,cookies,rustls-tls
cargo add serde --features derive
cargo add serde_json anyhow directories keyring
```

Use this architecture:

```text
src/
├── main.rs
├── cli.rs
├── config.rs
├── auth.rs
├── workspace.rs
├── api/
│   ├── mod.rs
│   ├── graphql.rs
│   ├── problems.rs
│   └── submissions.rs
├── commands/
│   ├── daily.rs
│   ├── list.rs
│   ├── get.rs
│   ├── test.rs
│   └── submit.rs
└── runners/
    ├── rust.rs
    ├── cpp.rs
    └── python.rs
```

The important pieces are:

1. **Authentication:** import `LEETCODE_SESSION` and `csrftoken` from the browser or OS keychain. Never commit them.
2. **Problem retrieval:** send GraphQL requests for the title, statement, difficulty, metadata and language skeleton.
3. **Workspace generation:** write something like `problems/0001.two-sum/solution.rs`, `README.md` and `cases.json`.
4. **Local runner:** compile in a temporary directory, enforce a timeout and compare structured outputs.
5. **Submission:** upload only the editable solution block, receive a submission ID and poll until the judge finishes.
6. **Cache:** store problem metadata locally so `lc list` and search are instant.
7. **Rate limiting:** avoid repeatedly downloading the full catalogue.

Community clients use LeetCode’s website GraphQL interface and authenticated session cookies. Login changes have broken integrations before, so isolate the API layer from your command and workspace code. That way, a future LeetCode change only requires replacing one module. ([github.com](https://github.com/j178/leetgo))

For your setup, I would start with **`clearloop/leetcode-cli`**, configure it for Rust, and only build your own once you find a specific workflow it cannot support.

## User

```yaml
name: Feature request
description: Suggest reusable material interaction behaviour.
title: "[FEATURE]: "
body:

- type: input
  id: summary
  attributes:
  label: Summary
  placeholder: Add a low-power material response mode.
  validations:
  required: true
- type: textarea
  id: use\_case
  attributes:
  label: Reusable use case
  description: Explain why this belongs in the package rather than one website wrapper.
  validations:
  required: true
- type: textarea
  id: api
  attributes:
  label: API sketch
  placeholder: |
  \<FeelableSurface ... />
  validations:
  required: false
- type: textarea
  id: acceptance
  attributes:
  label: Acceptance criteria
  placeholder: |
  \- Covered by tests.
  \- Does not add a runtime dependency.
  \- Works with reduced motion.
  validations:
  required: true
```

adapt this for security

## Assistant

For **public security improvements and hardening requests**, use this version. Actual unpatched vulnerabilities should be reported privately through the repository’s **Security → Advisories** area rather than disclosed in a public issue. GitHub supports private vulnerability reporting when it is enabled for the repository. ([docs.github.com](https://docs.github.com/en/code-security/security-advisories/working-with-repository-security-advisories/configuring-private-vulnerability-reporting-for-a-repository))

```yaml
name: Security feature request
description: Suggest reusable security, privacy, or hardening behaviour.
title: "[SECURITY]: "
body:
  - type: markdown
    attributes:
      value: |
        Use this form for security improvements that are safe to discuss publicly.

        Do not include credentials, secrets, exploit code, or details of an
        unpatched vulnerability. Report those privately through the repository's
        Security tab.

  - type: input
    id: summary
    attributes:
      label: Summary
      description: Briefly describe the proposed security improvement.
      placeholder: Add an opt-in Content Security Policy integration.
    validations:
      required: true

  - type: dropdown
    id: category
    attributes:
      label: Security area
      description: Which area does this request primarily affect?
      options:
        - Authentication and authorisation
        - Input validation and sanitisation
        - Data protection and privacy
        - Secure defaults and configuration
        - Dependency and supply-chain security
        - Abuse prevention and rate limiting
        - Logging and sensitive-data handling
        - Other
    validations:
      required: true

  - type: textarea
    id: threat_model
    attributes:
      label: Threat model
      description: Describe the risk, affected assets, attacker capabilities, and relevant trust boundaries.
      placeholder: |
        An application may render untrusted content supplied by...
        The behaviour should protect...
        An attacker is assumed to be able to...
    validations:
      required: true

  - type: textarea
    id: use_case
    attributes:
      label: Reusable use case
      description: Explain why this belongs in the package rather than one application or website wrapper.
      placeholder: |
        Multiple consumers need the same protection because...
        Implementing this independently would make it easier to...
    validations:
      required: true

  - type: textarea
    id: proposed_behaviour
    attributes:
      label: Proposed behaviour
      description: Explain the expected secure behaviour, including defaults and failure handling.
      placeholder: |
        The package should enable...
        The default should be...
        Invalid or unsafe input should...
    validations:
      required: true

  - type: textarea
    id: api
    attributes:
      label: API sketch
      description: Show how consumers might configure or use the behaviour.
      placeholder: |
        <SecureSurface
          policy="strict"
          onViolation={handleViolation}
        />
    validations:
      required: false

  - type: textarea
    id: compatibility
    attributes:
      label: Compatibility and migration
      description: Describe any breaking changes, browser or runtime constraints, and migration requirements.
      placeholder: |
        Existing consumers should continue to...
        Enabling strict mode may require...
    validations:
      required: false

  - type: textarea
    id: acceptance
    attributes:
      label: Acceptance criteria
      placeholder: |
        - Covered by unit and integration tests.
        - Secure by default.
        - Fails closed where appropriate.
        - Does not expose secrets through logs or errors.
        - Does not add a runtime dependency without justification.
        - Documents the threat model and configuration.
        - Includes migration guidance for affected consumers.
    validations:
      required: true

  - type: checkboxes
    id: public_disclosure
    attributes:
      label: Public disclosure confirmation
      options:
        - label: This request is safe to discuss publicly and does not disclose an unpatched vulnerability, credential, secret, or working exploit.
          required: true
```

## User

**make PULL\_REQUEST\_TEMPLATE.yml**

## Assistant

GitHub will **not recognise `PULL_REQUEST_TEMPLATE.yml`** as a pull request form. YAML forms are supported for issues, while pull request templates must be Markdown, normally at `.github/PULL_REQUEST_TEMPLATE.md`. ([docs.github.com](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/creating-a-pull-request-template-for-your-repository))

I made the security-focused template in the correct format:

[Download `PULL_REQUEST_TEMPLATE.md`](sandbox:/mnt/data/PULL_REQUEST_TEMPLATE.md)

This ZIP already contains the correct repository structure:

[Download `.github/PULL_REQUEST_TEMPLATE.md` as a ZIP](sandbox:/mnt/data/security-pr-template.zip)

The template includes sections for the reusable use case, threat model, API and behavioural changes, compatibility, verification commands, sensitive-data handling, dependency review, secure defaults, negative testing, and private-advisory coordination.

## User

make a security.md and CONTRIBUTING.md

## Assistant

Created both files using GitHub’s conventional uppercase `SECURITY.md` filename.

### Files

[Download `SECURITY.md`](sandbox:/mnt/data/SECURITY.md)

Covers private vulnerability reporting, supported versions, response targets, coordinated disclosure, public hardening requests, scope, reporter credit, and good-faith security research.

[Download `CONTRIBUTING.md`](sandbox:/mnt/data/CONTRIBUTING.md)

Covers repository scope, local npm setup, threat modelling, secure implementation practices, dependency review, hostile-input testing, documentation, commits, pull requests, review expectations, and licensing.

### Repository-ready bundle

[Download the complete community-files bundle](sandbox:/mnt/data/security-community-files.zip)

```text
SECURITY.md
CONTRIBUTING.md
.github/
└── PULL_REQUEST_TEMPLATE.md
```

The archive structure, Markdown headings, internal links, and ZIP integrity were verified.

## User

# give me a good README template for LeetCode CLI github repo and short github repo description and topics and show me codeblock for how to configure dependabot for rust cargo repo

## Assistant

## `README.md` template

Replace `OWNER`, `REPO`, `CRATE_NAME`, and the example command name `leetcode`. Remove any feature or command that is not implemented yet.

````markdown
<!--
Before publishing:
- Replace OWNER, REPO, CRATE_NAME, and `leetcode`.
- Remove features and commands that are not implemented.
- Replace the license badge if you are not using MIT.
- Add a terminal recording or screenshot under docs/.
-->

<div align="center">

# LeetCode CLI

A terminal-first LeetCode workflow written in Rust.

[![CI](https://github.com/OWNER/REPO/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/REPO/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/CRATE_NAME.svg)](https://crates.io/crates/CRATE_NAME)
[![Downloads](https://img.shields.io/crates/d/CRATE_NAME.svg)](https://crates.io/crates/CRATE_NAME)
[![License](https://img.shields.io/github/license/OWNER/REPO)](LICENSE)

[Installation](#installation) ·
[Quick start](#quick-start) ·
[Commands](#commands) ·
[Configuration](#configuration) ·
[Contributing](CONTRIBUTING.md) ·
[Security](SECURITY.md)

</div>

> [!NOTE]
> This is an unofficial community project. It is not affiliated with or endorsed by LeetCode.

## Overview

LeetCode CLI brings the problem-solving workflow into your terminal. Browse problems, create solution files, run tests locally, and submit solutions without repeatedly switching between your browser and editor.

The project is designed to provide a fast command-line workflow while keeping generated code, configuration, and authentication data predictable and easy to manage.

## Features

<!-- Keep only features that currently work. -->

- Search and filter LeetCode problems
- Fetch the daily challenge
- Generate starter files for supported languages
- Run sample and custom test cases locally
- Submit solutions from the terminal
- Inspect submission results
- Open problems in your browser or editor
- Cache problem metadata for faster navigation
- Generate shell completions
- Use human-readable configuration files

## Demo

<!-- Replace this with a real terminal recording or screenshot. -->

![LeetCode CLI terminal demo](docs/demo.gif)

## Installation

### Cargo

```shell
cargo install CRATE_NAME
```

### Prebuilt binaries

Download the appropriate archive from the
[latest release](https://github.com/OWNER/REPO/releases/latest).

### From source

```shell
git clone https://github.com/OWNER/REPO.git
cd REPO

cargo build --release

# Optional: install the binary into your Cargo bin directory
cargo install --path .
```

Verify the installation:

```shell
leetcode --version
leetcode --help
```

## Quick start

Authenticate with your LeetCode account:

```shell
leetcode auth login
```

Fetch the daily problem:

```shell
leetcode daily
```

Create or open its solution file:

```shell
leetcode edit last
```

Run its test cases locally:

```shell
leetcode test last
```

Submit the solution:

```shell
leetcode submit last
```

## Commands

| Command | Description |
|---|---|
| `leetcode auth` | Configure and inspect authentication |
| `leetcode daily` | Fetch the current daily challenge |
| `leetcode list` | Search and filter problems |
| `leetcode show <problem>` | Display problem information |
| `leetcode pick <problem>` | Generate a local solution workspace |
| `leetcode edit <problem>` | Open a solution in the configured editor |
| `leetcode test <problem>` | Run local test cases |
| `leetcode submit <problem>` | Submit a solution |
| `leetcode status <submission>` | Inspect a submission result |
| `leetcode open <problem>` | Open a problem in the browser |
| `leetcode completion <shell>` | Generate shell completions |
| `leetcode config` | Inspect or update configuration |

Run the help command for the complete interface:

```shell
leetcode --help
leetcode <command> --help
```

## Problem identifiers

Commands may accept a numeric ID, title slug, URL, or shorthand identifier:

```shell
leetcode show 1
leetcode show two-sum
leetcode show https://leetcode.com/problems/two-sum/
leetcode daily
leetcode test last
```

Document the exact identifier forms supported by your implementation here.

## Authentication

Authentication data should never be committed to the repository or printed in logs.

```shell
leetcode auth login
leetcode auth status
leetcode auth logout
```

Describe the authentication methods supported by the CLI, such as:

- Reading an existing browser session
- Supplying session credentials through environment variables
- Storing credentials in the operating system credential store
- Running without authentication for public problem data

See [SECURITY.md](SECURITY.md) for instructions on reporting authentication or credential-handling vulnerabilities.

## Configuration

The default configuration file is:

```text
~/.config/leetcode-cli/config.toml
```

Example:

```toml
site = "leetcode.com"
language = "rust"
editor = "code --wait"
workspace = "~/Code/leetcode"

[output]
include_description = true
separate_test_cases = true

[testing]
timeout_seconds = 5

[cache]
enabled = true
```

Command-line arguments should take precedence over configuration-file values.

```shell
leetcode config show
leetcode config path
```

## Shell completions

### Zsh

```shell
mkdir -p ~/.zfunc
leetcode completion zsh > ~/.zfunc/_leetcode

echo 'fpath=(~/.zfunc $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

### Bash

```shell
leetcode completion bash > ~/.local/share/bash-completion/completions/leetcode
```

### Fish

```shell
leetcode completion fish > ~/.config/fish/completions/leetcode.fish
```

## Local workspace

An example generated workspace might look like this:

```text
leetcode/
├── 0001-two-sum/
│   ├── README.md
│   ├── solution.rs
│   └── testcases.toml
├── 0002-add-two-numbers/
│   ├── README.md
│   ├── solution.rs
│   └── testcases.toml
└── leetcode.toml
```

Generated files should remain ordinary source files that can be opened, tested, debugged, and version-controlled without requiring a particular editor.

## Supported languages

<!-- Replace this table with the actual support matrix. -->

| Language | Starter generation | Local testing | Submission |
|---|:---:|:---:|:---:|
| Rust | ✅ | ✅ | ✅ |
| C++ | ✅ | ✅ | ✅ |
| Python | ✅ | ✅ | ✅ |
| Go | ✅ | ✅ | ✅ |
| Java | ✅ | Planned | ✅ |
| TypeScript | ✅ | Planned | ✅ |

## Development

Install the Rust toolchain, clone the repository, and run:

```shell
cargo build
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Run the development binary:

```shell
cargo run -- --help
```

Enable debug logging:

```shell
RUST_LOG=debug cargo run -- daily
```

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

```shell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Read [CONTRIBUTING.md](CONTRIBUTING.md) for the complete development and review process.

## Roadmap

Planned work is tracked through
[GitHub Issues](https://github.com/OWNER/REPO/issues) and the
[project board](https://github.com/OWNER/REPO/projects).

Possible areas of development include:

- Additional local language runners
- Contest workflows
- Interactive problem selection
- Offline metadata search
- Improved editor integration
- More secure credential-storage options

## Limitations

- The CLI relies on interfaces exposed by the LeetCode website, which may change.
- Some functionality requires an authenticated LeetCode session.
- Local test execution may not exactly reproduce the remote judge environment.
- Paid or restricted content remains subject to the permissions of the authenticated account.

## License

Licensed under the [MIT License](LICENSE).

## Disclaimer

This project is an independent, unofficial client. Users are responsible for complying with the terms and policies of the services they access through it.
````

## GitHub repository description

> **An unofficial, terminal-first LeetCode client written in Rust for browsing problems, managing solutions, testing locally, and submitting code.**

A slightly tighter alternative:

> **A fast, terminal-first LeetCode workflow written in Rust.**

## GitHub topics

Use these as the core set:

```text
leetcode
leetcode-cli
rust
cargo
cli
command-line-tool
algorithms
data-structures
competitive-programming
coding-challenges
developer-tools
terminal
online-judge
code-generation
```

Add these only when they accurately describe the implementation:

```text
graphql
tui
ratatui
async-rust
tokio
```

GitHub topic names use lowercase letters, numbers, and hyphens, may contain up to 50 characters, and each repository can have at most 20 topics. ([docs.github.com](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/classifying-your-repository-with-topics?utm_source=chatgpt.com))

## Cargo Dependabot configuration

Save this as:

```text
.github/dependabot.yml
```

```yaml
version: 2

updates:
  - package-ecosystem: "cargo"
    directory: "/"

    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
      timezone: "Australia/Brisbane"

    # Limits version-update PRs. Security-update PRs are handled separately.
    open-pull-requests-limit: 10

    commit-message:
      prefix: "deps"

    # Bundle routine upgrades into one PR while keeping major upgrades separate.
    groups:
      cargo-minor-and-patch:
        applies-to: "version-updates"
        patterns:
          - "*"
        update-types:
          - "minor"
          - "patch"
```

GitHub requires the configuration on the default branch under `.github/dependabot.yml` or `.github/dependabot.yaml`. Cargo uses the `cargo` package ecosystem value, and the schedule can specify a weekly day, time, and IANA timezone. ([docs.github.com](https://docs.github.com/en/code-security/concepts/supply-chain-security/about-the-dependabot-yml-file?utm_source=chatgpt.com))

The grouping rule produces a combined pull request for compatible minor and patch version updates. Major updates remain separate, which makes breaking upgrades easier to review individually. ([docs.github.com](https://docs.github.com/en/code-security/tutorials/secure-your-dependencies/optimizing-pr-creation-version-updates?utm_source=chatgpt.com))

For a Rust CLI that also uses GitHub Actions, add this second entry beneath the Cargo entry:

```yaml
  - package-ecosystem: "github-actions"
    directory: "/"

    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:15"
      timezone: "Australia/Brisbane"

    commit-message:
      prefix: "ci"

    groups:
      github-actions:
        applies-to: "version-updates"
        patterns:
          - "*"
```

GitHub Actions is supported as a separate Dependabot ecosystem, with `directory: "/"` covering workflow files in the standard `.github/workflows` location. ([docs.github.com](https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/configure-version-updates?utm_source=chatgpt.com))

## User

## make a actions suite called checks.yml uses code quality and automated tested and benchmarking, release.yml uses release pls, security.yml uses codeQL and other tools



## combined with parllelism, peak security and efficiency and use as much tools as you need to make it better than anything yet

## Assistant

# Rust GitHub Actions suite

Built. This is a repository-ready CI stack rather than three decorative YAML files.

[Download the complete Rust Actions suite](sandbox:/mnt/data/rust-github-actions-suite.zip)

[Download the SHA-256 checksum](sandbox:/mnt/data/rust-github-actions-suite.sha256)

```text
rust-github-actions-suite/
├── .github/
│   ├── dependabot.yml
│   └── workflows/
│       ├── checks.yml
│       ├── release.yml
│       └── security.yml
├── deny.toml
└── CI_SETUP.md
```

## Individual files

- [checks.yml](sandbox:/mnt/data/rust-github-actions-suite/.github/workflows/checks.yml)
- [release.yml](sandbox:/mnt/data/rust-github-actions-suite/.github/workflows/release.yml)
- [security.yml](sandbox:/mnt/data/rust-github-actions-suite/.github/workflows/security.yml)
- [dependabot.yml](sandbox:/mnt/data/rust-github-actions-suite/.github/dependabot.yml)
- [deny.toml](sandbox:/mnt/data/rust-github-actions-suite/deny.toml)
- [CI_SETUP.md](sandbox:/mnt/data/rust-github-actions-suite/CI_SETUP.md)

## `checks.yml`

The normal pull-request path is heavily parallelised:

- `rustfmt`
- `cargo check` across every target and feature
- Clippy with warnings denied
- Rustdoc with warnings denied
- `cargo-nextest` on Linux, macOS and Windows
- Separate default-feature and all-feature Linux runs
- Doctests
- Automatic MSRV detection from `Cargo.toml`
- `cargo package` release-readiness testing
- LLVM coverage with an uploaded LCOV artifact
- Benchmark compilation on every pull request
- Full timed benchmarks on weekly and manual runs
- `cargo-hack` feature-powerset testing on deep runs
- One stable `Checks / Required checks` result for repository rules

Timed benchmarks are kept outside the pull-request gate because shared-runner timing is noisy. Pull requests still compile every benchmark, so broken benches cannot quietly rust in the attic.

## `release.yml`

The release workflow runs the two-part release-plz model:

1. `release-plz release` publishes versions already approved through the release PR.
2. `release-plz release-pr` creates or updates the next release PR.

That follows release-plz’s intended workflow, including its changelog, versioning and workspace support. Trusted publishing works without storing a long-lived crates.io token. ([release-plz.dev](https://release-plz.dev/docs?utm_source=chatgpt.com))

The workflow uses:

- A protected `crates-io` GitHub environment
- crates.io OIDC trusted publishing
- No `CARGO_REGISTRY_TOKEN`
- Complete Git history for release calculation
- Independent concurrency groups for publishing and release-PR maintenance
- An optional short-lived GitHub App installation token
- Explicit token narrowing to only `contents: write` and `pull-requests: write`
- Automatic token revocation at the end of the job
- A fallback to the built-in `GITHUB_TOKEN`

The GitHub App is useful because events created with the repository `GITHUB_TOKEN` normally do not start further workflow runs. ([docs.github.com](https://docs.github.com/en/enterprise-server%403.17/actions/concepts/security/github_token?utm_source=chatgpt.com))

Configure:

```text
Repository variable: RELEASE_APP_CLIENT_ID
Repository secret:   RELEASE_APP_PRIVATE_KEY
```

Without those values, release-plz still works with `GITHUB_TOKEN`, but checks may not automatically start on a release PR created by the workflow.

## `security.yml`

Security runs independently from ordinary testing and layers several tools:

- CodeQL for both Rust source and GitHub Actions workflows
- `security-extended` CodeQL queries
- Pull-request dependency review
- RustSec advisory scanning with `cargo-audit`
- Licence, source, wildcard and dependency policy through `cargo-deny`
- GitHub Actions syntax and shell analysis through `actionlint`
- Workflow-security analysis through `zizmor`
- OpenSSF Scorecard with SARIF upload
- Optional Miri undefined-behaviour testing
- Automatic fuzz-target execution when `fuzz/Cargo.toml` exists
- Uploaded fuzz crash artifacts

Current CodeQL supports Rust analysis with `build-mode: none`, avoiding unnecessary compilation during security analysis. GitHub Actions workflow analysis is included as a second CodeQL language matrix entry. ([docs.github.com](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-for-compiled-languages?utm_source=chatgpt.com))

The security model also includes:

- No `pull_request_target`
- Empty workflow-level permissions by default
- Explicit job-level permissions
- Read-only tokens for untrusted pull-request code
- No direct interpolation of pull-request data into shell commands
- `persist-credentials: false` on every checkout
- Full-SHA action pinning
- Timeouts on every job
- Separate concurrency groups that cancel stale PR analysis
- OpenSSF publishing restrictions respected inside the Scorecard job

GitHub recommends full-length commit pinning because it is the immutable way to reference an action, along with granting the `GITHUB_TOKEN` only the access each job needs. ([docs.github.com](https://docs.github.com/en/code-security/tutorials/secure-your-organization/protect-against-threats?utm_source=chatgpt.com))

## Dependency policy

`deny.toml` starts from a deliberately strict posture:

- Known permissive licences are allowed
- Wildcard dependency versions are denied
- Unknown registries are denied
- Git dependencies are denied unless explicitly approved
- Duplicate dependency versions produce warnings
- Advisory exceptions start empty
- Private unpublished workspace packages are still checked

Review any exception rather than globally loosening the rule. The setup guide includes examples for narrowly allowing an exceptional licence, version or Git source.

## Dependabot

The included Dependabot configuration maintains:

- Cargo dependencies
- Full-SHA GitHub Action references
- Grouped minor and patch Cargo updates
- Grouped GitHub Action updates
- Separate commit prefixes for dependencies and CI
- Brisbane-local weekly schedules

## Required setup

After copying the files:

1. Commit `Cargo.lock`.
2. Set `rust-version`, `license`, `repository` and `description` in `Cargo.toml`.
3. Create a GitHub environment named `crates-io`.
4. Configure the crates.io trusted publisher for `release.yml` and that environment.
5. Add the optional release GitHub App variable and secret.
6. Enable Dependency Graph, Dependabot alerts, code scanning, secret scanning, push protection and private vulnerability reporting.
7. Review the initial `deny.toml` result against your dependency tree.
8. Require `Checks / Required checks` in your repository rules.

For a private repository, CodeQL and published Scorecard results require the corresponding GitHub security features. The suite leaves private Scorecard publication disabled until this repository variable is set:

```text
ENABLE_PRIVATE_SCORECARD=true
```

Miri is similarly opt-in:

```text
ENABLE_MIRI=true
```

## Validation performed

The generated suite was checked with:

- YAML 1.2 parsing
- TOML parsing
- Bash syntax validation for all 26 embedded command blocks
- Verification of all 60 action invocations as full 40-character SHA pins
- Checkout credential-persistence checks
- Job-timeout checks
- Permission-value checks
- Untrusted-expression checks for shell blocks
- Scorecard workflow-restriction checks
- GitHub App current-repository scope and permission checks
- A `pull_request_target` absence check
- ZIP integrity testing

The sandbox could not fetch the upstream `actionlint` binary for a second local execution. The generated `security.yml` installs pinned `actionlint` v1.7.11 and runs it on GitHub. Cargo commands also require the actual repository to execute, so the first real workflow run will reveal any project-specific feature, licence or package-layout adjustments.

Archive SHA-256:

```text
673b4b9babaf3f030ffc322fea497a2b74820a2aa09d7fbe17f6b7b341464892
```

## User

how should i structure src so it has code for all the features and their endpoint registration separate and is clean yet zero cost abstractions and all the code is modular and fast async and agressively compiler optimised for speed and lowest memory and security

```text
src
├── extension.ts
├── leetcode
│   ├── auth.ts
│   ├── boilerplate.ts
│   ├── client.ts
│   ├── index.ts
│   └── types.ts
├── logger
│   ├── Logger.ts
│   ├── index.ts
│   └── types.ts
├── statusBar.ts
├── test
│   ├── runTest.ts
│   └── suite
│       ├── boilerplate.test.ts
│       ├── extension.test.ts
│       ├── index.ts
│       ├── leetcode.test.ts
│       ├── logger.test.ts
│       ├── snapshot.ts
│       ├── statusBar.test.ts
│       ├── testResultsPanel.test.ts
│       └── textRenderer.test.ts
├── tree
│   ├── AllProblemsTreeDataProvider.ts
│   ├── ContestsTreeDataProvider.ts
│   ├── DailyChallengeTreeDataProvider.ts
│   └── StudyListsTreeDataProvider.ts
├── utils
│   └── textRenderer.ts
└── webview
    ├── DiscussionWebview.ts
    ├── ProblemWebview.ts
    └── TestResultsPanel.ts
```

ignore the vscode extension specific stuff and my project is in rust and i want it to be extensible enough to add in hackerrank, codeforces, exercism, project euler, HackerEarth, inside one src and only platforms act like separate crates

## Assistant

## Recommended architecture

Use a **single root application crate** containing your domain, features, CLI, rendering, cache, local execution, configuration, and platform adapters. Put only the remote platform clients in workspace crates:

```text
root application
├── shared domain and feature code
├── CLI command registration
├── concrete platform selection
├── adapters that translate platform models
└── infrastructure

platform crates
├── platform-leetcode
├── platform-codeforces
├── platform-hackerrank
├── platform-exercism
├── platform-project-euler
└── platform-hackerearth
```

The platform crates should understand their respective HTTP APIs, authentication rules, wire formats, and endpoints. They should **not** know about Clap, your cache, terminal rendering, local code execution, or the root application’s domain types.

For dispatch, use a concrete `SelectedPlatform` enum instead of `Arc<dyn Platform>` and `async_trait`. This keeps dispatch statically compiled, avoids boxed futures and vtables, and still permits runtime platform selection. Rust traits containing `async fn` or return-position `impl Future` are not dyn-compatible, which is perfectly acceptable when using concrete enum dispatch. ([doc.rust-lang.org](https://doc.rust-lang.org/reference/items/traits.html?utm_source=chatgpt.com))

## Repository structure

```text
.
├── Cargo.toml
├── Cargo.lock
│
├── src
│   ├── main.rs
│   ├── lib.rs
│   ├── bootstrap.rs
│   ├── error.rs
│   │
│   ├── cli
│   │   ├── mod.rs
│   │   ├── root.rs
│   │   ├── commands.rs
│   │   └── dispatch.rs
│   │
│   ├── domain
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── capability.rs
│   │   ├── contest.rs
│   │   ├── difficulty.rs
│   │   ├── language.rs
│   │   ├── platform.rs
│   │   ├── problem.rs
│   │   ├── solution.rs
│   │   └── submission.rs
│   │
│   ├── ports
│   │   ├── mod.rs
│   │   ├── authentication.rs
│   │   ├── cache.rs
│   │   ├── contests.rs
│   │   ├── problems.rs
│   │   ├── runner.rs
│   │   ├── secrets.rs
│   │   ├── starter_code.rs
│   │   └── submissions.rs
│   │
│   ├── features
│   │   ├── mod.rs
│   │   │
│   │   ├── auth
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── login.rs
│   │   │   ├── logout.rs
│   │   │   └── status.rs
│   │   │
│   │   ├── problems
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── daily.rs
│   │   │   ├── list.rs
│   │   │   ├── pick.rs
│   │   │   ├── search.rs
│   │   │   └── show.rs
│   │   │
│   │   ├── contests
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── list.rs
│   │   │   └── show.rs
│   │   │
│   │   ├── submissions
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── history.rs
│   │   │   ├── status.rs
│   │   │   └── submit.rs
│   │   │
│   │   ├── testing
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── compile.rs
│   │   │   ├── execute.rs
│   │   │   └── compare.rs
│   │   │
│   │   └── workspace
│   │       ├── mod.rs
│   │       ├── generate.rs
│   │       ├── naming.rs
│   │       └── template.rs
│   │
│   ├── platforms
│   │   ├── mod.rs
│   │   ├── kind.rs
│   │   ├── selected.rs
│   │   ├── build.rs
│   │   ├── dispatch.rs
│   │   └── adapters
│   │       ├── mod.rs
│   │       ├── leetcode.rs
│   │       ├── codeforces.rs
│   │       ├── hackerrank.rs
│   │       ├── exercism.rs
│   │       ├── project_euler.rs
│   │       └── hackerearth.rs
│   │
│   ├── infrastructure
│   │   ├── mod.rs
│   │   ├── cache.rs
│   │   ├── config.rs
│   │   ├── filesystem.rs
│   │   ├── http.rs
│   │   ├── process.rs
│   │   ├── secret_store.rs
│   │   └── telemetry.rs
│   │
│   └── output
│       ├── mod.rs
│       ├── format.rs
│       ├── json.rs
│       ├── progress.rs
│       ├── table.rs
│       └── text.rs
│
├── platforms
│   ├── leetcode
│   │   ├── Cargo.toml
│   │   ├── src
│   │   │   ├── lib.rs
│   │   │   ├── auth.rs
│   │   │   ├── client.rs
│   │   │   ├── endpoint.rs
│   │   │   ├── error.rs
│   │   │   ├── model
│   │   │   │   ├── mod.rs
│   │   │   │   ├── contest.rs
│   │   │   │   ├── problem.rs
│   │   │   │   └── submission.rs
│   │   │   ├── endpoints
│   │   │   │   ├── mod.rs
│   │   │   │   ├── authentication.rs
│   │   │   │   ├── contests.rs
│   │   │   │   ├── problems.rs
│   │   │   │   └── submissions.rs
│   │   │   └── wire
│   │   │       ├── mod.rs
│   │   │       ├── graphql.rs
│   │   │       ├── request.rs
│   │   │       └── response.rs
│   │   └── tests
│   │       ├── fixtures
│   │       └── client.rs
│   │
│   ├── codeforces
│   ├── hackerrank
│   ├── exercism
│   ├── project-euler
│   └── hackerearth
│
├── tests
│   ├── cli.rs
│   ├── platform_contracts.rs
│   ├── workspace_generation.rs
│   └── fixtures
│
└── benches
    ├── parsing.rs
    ├── rendering.rs
    └── workspace_generation.rs
```

## The dependency direction

Keep this rule strict:

```text
cli
  │
  ▼
features ──────────────► domain
  │                        ▲
  ▼                        │
ports ◄──────── platform adapters
  ▲                        │
  │                        ▼
infrastructure       platform-* crates
```

In concrete terms:

```text
CLI command
    ↓
feature use case
    ↓
platform-neutral port
    ↓
root adapter
    ↓
platform client crate
    ↓
remote endpoint
```

The critical rule is:

```text
platform-* crates must never depend on the root application crate
```

Otherwise you create a dependency cycle:

```text
root app → platform-leetcode → root app
```

Instead, your root crate owns the traits and adapters. Because the traits are local to the root crate, the root can implement them using types from the platform crates.

## Keep platform clients concrete

Avoid this:

```rust
Arc<dyn Platform>
Box<dyn ProblemCatalog>
#[async_trait]
```

It is convenient, but it introduces boxed futures, virtual dispatch, lifetime friction, and a less transparent call graph.

Use capability-specific traits and a concrete platform enum.

### Problem capability

```rust
// src/ports/problems.rs

use std::future::Future;

use crate::{
    domain::{Problem, ProblemId, ProblemQuery},
    error::Result,
};

pub(crate) trait ProblemCatalog {
    fn problem<'a>(
        &'a self,
        id: &'a ProblemId,
    ) -> impl Future<Output = Result<Problem>> + Send + 'a;

    fn search<'a>(
        &'a self,
        query: &'a ProblemQuery,
    ) -> impl Future<Output = Result<Vec<Problem>>> + Send + 'a;
}
```

This produces concrete futures without `Pin<Box<dyn Future>>`.

### Submission capability

```rust
// src/ports/submissions.rs

use std::future::Future;

use crate::{
    domain::{Submission, SubmissionId, SubmissionRequest},
    error::Result,
};

pub(crate) trait SubmissionService {
    fn submit<'a>(
        &'a self,
        request: &'a SubmissionRequest,
    ) -> impl Future<Output = Result<SubmissionId>> + Send + 'a;

    fn submission<'a>(
        &'a self,
        id: &'a SubmissionId,
    ) -> impl Future<Output = Result<Submission>> + Send + 'a;
}
```

Do not create one enormous `Platform` trait containing 40 methods. Separate capabilities mean feature code only receives what it actually needs.

## Root-level platform adapters

The external platform crate returns its own stable public models. The root adapter converts those models into your application domain.

```rust
// src/platforms/adapters/leetcode.rs

use std::future::Future;

use crate::{
    domain::{Problem, ProblemId, ProblemQuery},
    error::{Error, Result},
    ports::ProblemCatalog,
};

pub(crate) struct LeetCodeAdapter {
    client: platform_leetcode::Client,
}

impl LeetCodeAdapter {
    pub(crate) fn new(client: platform_leetcode::Client) -> Self {
        Self { client }
    }
}

impl ProblemCatalog for LeetCodeAdapter {
    fn problem<'a>(
        &'a self,
        id: &'a ProblemId,
    ) -> impl Future<Output = Result<Problem>> + Send + 'a {
        async move {
            let problem = self
                .client
                .problem(id.native_id())
                .await
                .map_err(Error::leetcode)?;

            Problem::try_from(problem)
        }
    }

    fn search<'a>(
        &'a self,
        query: &'a ProblemQuery,
    ) -> impl Future<Output = Result<Vec<Problem>>> + Send + 'a {
        async move {
            self.client
                .search(query.text())
                .await
                .map_err(Error::leetcode)?
                .into_iter()
                .map(Problem::try_from)
                .collect()
        }
    }
}
```

This extra conversion boundary is useful rather than wasteful. It stops a LeetCode GraphQL field rename from infecting your entire program.

## Platform selection without trait objects

Use one concrete enum:

```rust
// src/platforms/selected.rs

use crate::{
    domain::{Capability, Problem, ProblemId, ProblemQuery},
    error::{Error, Result},
    ports::ProblemCatalog,
    platforms::adapters,
};

pub(crate) enum SelectedPlatform {
    #[cfg(feature = "leetcode")]
    LeetCode(adapters::LeetCodeAdapter),

    #[cfg(feature = "codeforces")]
    Codeforces(adapters::CodeforcesAdapter),

    #[cfg(feature = "hackerrank")]
    HackerRank(adapters::HackerRankAdapter),

    #[cfg(feature = "exercism")]
    Exercism(adapters::ExercismAdapter),

    #[cfg(feature = "project-euler")]
    ProjectEuler(adapters::ProjectEulerAdapter),

    #[cfg(feature = "hackerearth")]
    HackerEarth(adapters::HackerEarthAdapter),
}
```

Then statically dispatch:

```rust
impl ProblemCatalog for SelectedPlatform {
    fn problem<'a>(
        &'a self,
        id: &'a ProblemId,
    ) -> impl Future<Output = Result<Problem>> + Send + 'a {
        async move {
            match self {
                #[cfg(feature = "leetcode")]
                Self::LeetCode(platform) => platform.problem(id).await,

                #[cfg(feature = "codeforces")]
                Self::Codeforces(platform) => platform.problem(id).await,

                #[cfg(feature = "hackerrank")]
                Self::HackerRank(platform) => platform.problem(id).await,

                #[cfg(feature = "exercism")]
                Self::Exercism(platform) => platform.problem(id).await,

                #[cfg(feature = "project-euler")]
                Self::ProjectEuler(platform) => platform.problem(id).await,

                #[cfg(feature = "hackerearth")]
                Self::HackerEarth(platform) => platform.problem(id).await,
            }
        }
    }

    fn search<'a>(
        &'a self,
        query: &'a ProblemQuery,
    ) -> impl Future<Output = Result<Vec<Problem>>> + Send + 'a {
        async move {
            match self {
                #[cfg(feature = "leetcode")]
                Self::LeetCode(platform) => platform.search(query).await,

                #[cfg(feature = "codeforces")]
                Self::Codeforces(platform) => platform.search(query).await,

                #[cfg(feature = "hackerrank")]
                Self::HackerRank(platform) => platform.search(query).await,

                #[cfg(feature = "exercism")]
                Self::Exercism(platform) => platform.search(query).await,

                #[cfg(feature = "project-euler")]
                Self::ProjectEuler(platform) => platform.search(query).await,

                #[cfg(feature = "hackerearth")]
                Self::HackerEarth(platform) => platform.search(query).await,
            }
        }
    }
}
```

That gives you:

- No vtable.
- No `Box<dyn Future>`.
- No heap allocation merely for abstraction.
- Exhaustive compiler checking when adding a platform.
- One predictable branch at the platform boundary.

Do not immediately hide these matches behind a proc macro. Explicit repetition is cheap to audit. Once there are enough identical implementations to become noisy, use one small private `macro_rules!` macro in `platforms/dispatch.rs`.

## Model differing platform capabilities explicitly

Not every platform has:

- Daily challenges.
- Contests.
- Remote submissions.
- Discussions.
- Starter code.
- Authentication.
- Public problem search.
- Local test cases.

Do not fake a universal API where every operation supposedly exists.

```rust
// src/domain/capability.rs

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capabilities(u16);

impl Capabilities {
    pub const AUTH: Self = Self(1 << 0);
    pub const SEARCH: Self = Self(1 << 1);
    pub const DAILY: Self = Self(1 << 2);
    pub const CONTESTS: Self = Self(1 << 3);
    pub const STARTER_CODE: Self = Self(1 << 4);
    pub const REMOTE_TEST: Self = Self(1 << 5);
    pub const SUBMIT: Self = Self(1 << 6);

    pub const fn contains(self, capability: Self) -> bool {
        self.0 & capability.0 == capability.0
    }
}
```

```rust
impl SelectedPlatform {
    pub(crate) const fn capabilities(&self) -> Capabilities {
        match self {
            #[cfg(feature = "leetcode")]
            Self::LeetCode(_) => Capabilities::AUTH
                .union(Capabilities::SEARCH)
                .union(Capabilities::DAILY)
                .union(Capabilities::CONTESTS)
                .union(Capabilities::STARTER_CODE)
                .union(Capabilities::REMOTE_TEST)
                .union(Capabilities::SUBMIT),

            #[cfg(feature = "project-euler")]
            Self::ProjectEuler(_) => Capabilities::SEARCH,
            
            // Other platforms...
        }
    }
}
```

You can implement `union` yourself as a `const fn`, or use `bitflags`. Either approach compiles to integer operations.

A feature should check the capability before invoking the operation:

```rust
platform.require(Capabilities::SUBMIT)?;
```

That produces a proper error such as:

```text
project-euler does not support remote submission
```

rather than burying support differences in random endpoint code.

## Separate command registration from feature implementation

There are three different kinds of registration in this project:

```text
src/cli/commands.rs
    Registers CLI features.

src/platforms/build.rs
    Registers compiled platform implementations.

platforms/<name>/src/endpoints/mod.rs
    Registers remote API operations.
```

Keep those separate.

### CLI registration

```rust
// src/cli/root.rs

use clap::Parser;

use crate::platforms::PlatformKind;

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[arg(long, value_enum, default_value_t = PlatformKind::LeetCode)]
    pub platform: PlatformKind,

    #[command(subcommand)]
    pub command: Command,
}
```

```rust
// src/cli/commands.rs

use clap::Subcommand;

use crate::features;

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Auth(features::auth::Args),
    Problems(features::problems::Args),
    Contests(features::contests::Args),
    Test(features::testing::Args),
    Submit(features::submissions::Args),
    Workspace(features::workspace::Args),
}
```

Clap’s derive API supports nested and flattened command definitions, so each feature can own its argument types while one root enum remains the explicit registration point. ([docs.rs](https://docs.rs/clap/latest/clap/_derive/?utm_source=chatgpt.com))

### CLI dispatch

```rust
// src/cli/dispatch.rs

use crate::{
    cli::Command,
    error::Result,
    App,
};

pub(crate) async fn dispatch(command: Command, app: &App) -> Result<()> {
    match command {
        Command::Auth(args) => {
            crate::features::auth::run(
                args,
                &app.platform,
                &app.secrets,
                &app.output,
            )
            .await
        }

        Command::Problems(args) => {
            crate::features::problems::run(
                args,
                &app.platform,
                &app.workspace,
                &app.output,
            )
            .await
        }

        Command::Contests(args) => {
            crate::features::contests::run(
                args,
                &app.platform,
                &app.output,
            )
            .await
        }

        Command::Test(args) => {
            crate::features::testing::run(
                args,
                &app.runner,
                &app.output,
            )
            .await
        }

        Command::Submit(args) => {
            crate::features::submissions::run(
                args,
                &app.platform,
                &app.output,
            )
            .await
        }

        Command::Workspace(args) => {
            crate::features::workspace::run(
                args,
                &app.platform,
                &app.workspace,
                &app.output,
            )
            .await
        }
    }
}
```

Pass each feature only what it needs. Avoid turning `App` into a global service locator that every module rummages through.

## Feature functions should be generic over their capability

This makes tests simple without runtime mocking frameworks.

```rust
// src/features/problems/show.rs

use crate::{
    domain::{Problem, ProblemId},
    error::Result,
    ports::ProblemCatalog,
};

pub(crate) async fn execute<C>(
    catalog: &C,
    id: &ProblemId,
) -> Result<Problem>
where
    C: ProblemCatalog,
{
    catalog.problem(id).await
}
```

Production:

```rust
execute(&app.platform, &id).await
```

Test:

```rust
execute(&FakeProblemCatalog::new(), &id).await
```

That is static dispatch in both cases.

## Platform crate structure

A platform crate should be a focused protocol client:

```text
platforms/leetcode/src
├── lib.rs
├── auth.rs
├── client.rs
├── endpoint.rs
├── error.rs
├── model
│   ├── mod.rs
│   ├── contest.rs
│   ├── problem.rs
│   └── submission.rs
├── endpoints
│   ├── mod.rs
│   ├── authentication.rs
│   ├── contests.rs
│   ├── problems.rs
│   └── submissions.rs
└── wire
    ├── mod.rs
    ├── graphql.rs
    ├── request.rs
    └── response.rs
```

The distinction is:

```text
wire/
    Exact remote JSON and GraphQL structures.
    Private to the platform crate.

model/
    Stable public models returned by the platform client.

endpoints/
    Typed remote operations and their security policy.

client.rs
    HTTP execution, limits, authentication application, decoding.

auth.rs
    Platform-specific session and CSRF behaviour.
```

Do not export `wire` models. When the remote API changes, only the platform crate should need modification.

## Typed endpoint registration

Instead of scattering paths and retry decisions throughout the client, define typed operations.

```rust
// platforms/leetcode/src/endpoint.rs

use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, Copy)]
pub(crate) enum Verb {
    Get,
    Post,
}

#[derive(Clone, Copy)]
pub(crate) enum Encoding {
    Query,
    Json,
}

#[derive(Clone, Copy)]
pub(crate) enum Authentication {
    None,
    Optional,
    Required,
}

#[derive(Clone, Copy)]
pub(crate) enum RetryClass {
    Never,
    Idempotent,
}

pub(crate) trait Operation {
    type Request: Serialize + Sync;
    type Response: DeserializeOwned;

    const VERB: Verb;
    const PATH: &'static str;
    const ENCODING: Encoding;
    const AUTH: Authentication;
    const RETRY: RetryClass;
    const MAX_RESPONSE_BYTES: usize;
}
```

A concrete endpoint is a zero-sized marker type:

```rust
// platforms/leetcode/src/endpoints/problems.rs

use serde::{Deserialize, Serialize};

use crate::endpoint::{
    Authentication, Encoding, Operation, RetryClass, Verb,
};

pub(crate) struct FetchProblem;

#[derive(Debug, Serialize)]
pub(crate) struct FetchProblemRequest {
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FetchProblemResponse {
    pub data: GraphQlData,
}

impl Operation for FetchProblem {
    type Request = FetchProblemRequest;
    type Response = FetchProblemResponse;

    const VERB: Verb = Verb::Post;
    const PATH: &'static str = "/graphql";
    const ENCODING: Encoding = Encoding::Json;
    const AUTH: Authentication = Authentication::Optional;
    const RETRY: RetryClass = RetryClass::Idempotent;
    const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
}
```

The common client executor becomes generic:

```rust
impl Client {
    pub(crate) async fn execute<O>(
        &self,
        request: &O::Request,
    ) -> Result<O::Response, Error>
    where
        O: Operation,
    {
        let url = self.endpoint_url(O::PATH)?;
        let builder = self.request_builder::<O>(url, request)?;
        let builder = self.auth.apply(builder, O::AUTH)?;

        let response = self
            .send_with_policy::<O>(builder)
            .await?;

        let bytes = read_limited_body(
            response,
            O::MAX_RESPONSE_BYTES,
        )
        .await?;

        serde_json::from_slice(&bytes)
            .map_err(Error::decode)
    }
}
```

Because `O` is known at compile time, the operation policy is monomorphised. Constants such as encoding, authentication requirement, response limit, and retry class can be folded into the generated code.

Submission operations should use:

```rust
const RETRY: RetryClass = RetryClass::Never;
```

Automatically retrying a submission POST can create duplicate submissions.

## Workspace configuration

The root can simultaneously be a package and a workspace root. Cargo explicitly supports a workspace root containing its own package. ([doc.rust-lang.org](https://doc.rust-lang.org/cargo/reference/workspaces.html?utm_source=chatgpt.com))

```toml
[package]
name = "judge-cli"
version = "0.1.0"
edition = "2024"
rust-version = "YOUR_MSRV"

[workspace]
members = ["platforms/*"]
default-members = ["."]
resolver = "3"

[features]
default = ["leetcode"]

all-platforms = [
    "leetcode",
    "codeforces",
    "hackerrank",
    "exercism",
    "project-euler",
    "hackerearth",
]

leetcode = ["dep:platform-leetcode"]
codeforces = ["dep:platform-codeforces"]
hackerrank = ["dep:platform-hackerrank"]
exercism = ["dep:platform-exercism"]
project-euler = ["dep:platform-project-euler"]
hackerearth = ["dep:platform-hackerearth"]

[dependencies]
platform-leetcode = {
    path = "platforms/leetcode",
    optional = true,
}

platform-codeforces = {
    path = "platforms/codeforces",
    optional = true,
}

platform-hackerrank = {
    path = "platforms/hackerrank",
    optional = true,
}

platform-exercism = {
    path = "platforms/exercism",
    optional = true,
}

platform-project-euler = {
    path = "platforms/project-euler",
    optional = true,
}

platform-hackerearth = {
    path = "platforms/hackerearth",
    optional = true,
}
```

Optional dependencies connected through `dep:` features are only compiled when their platform feature is selected. That reduces compile time, binary size, dependency surface, and the amount of protocol code included in a build. ([doc.rust-lang.org](https://doc.rust-lang.org/cargo/reference/features.html?utm_source=chatgpt.com))

Examples:

```bash
# Normal LeetCode build
cargo build --release

# Codeforces-only build
cargo build \
  --release \
  --no-default-features \
  --features codeforces

# Distribution build containing every platform
cargo build \
  --release \
  --features all-platforms
```

Rust 2024 implies Cargo resolver version 3, which considers `rust-version` when choosing dependencies. Keeping it explicit at the workspace level makes the intended resolver clear. ([doc.rust-lang.org](https://doc.rust-lang.org/nightly/edition-guide/rust-2024/cargo-resolver.html?utm_source=chatgpt.com))

## Async runtime design

For a CLI, begin with one current-thread Tokio runtime:

```rust
// src/main.rs

use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    judge_cli::entry().await
}
```

A current-thread runtime still multiplexes concurrent network and process I/O, but does not create a worker thread per CPU. This is often a good memory-conscious starting point for a CLI. Tokio provides both current-thread and multithreaded runtimes, so benchmark before moving to the heavier scheduler. ([docs.rs](https://docs.rs/tokio/latest/tokio/runtime/?utm_source=chatgpt.com))

Use async only for operations that can actually wait:

```text
Use async:
- HTTP
- subprocess completion
- timers
- concurrent requests
- streaming large files

Keep synchronous:
- argument validation
- URL construction
- domain conversion
- small config reads at startup
- rendering calculations
- path validation
```

Do not create a task for every function call.

For a fixed number of independent requests:

```rust
tokio::try_join!(
    client.account(),
    client.daily_problem(),
);
```

For a dynamic request set, use a bounded `JoinSet` and `Semaphore`:

```rust
use std::sync::Arc;

use tokio::{
    sync::Semaphore,
    task::JoinSet,
};

let permits = Arc::new(Semaphore::new(4));
let mut tasks = JoinSet::new();

for id in ids {
    let permit = permits.clone().acquire_owned().await?;
    let client = client.clone();

    tasks.spawn(async move {
        let _permit = permit;
        client.problem(&id).await
    });
}

while let Some(result) = tasks.join_next().await {
    consume(result??);
}
```

A semaphore bounds simultaneous work, while `JoinSet` tracks dynamically spawned tasks and aborts them when the set is dropped. ([docs.rs](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html?utm_source=chatgpt.com))

For local compilers and test executables, use `tokio::process::Command`, not a shell command string:

```rust
let status = tokio::process::Command::new(&compiler)
    .arg(&source)
    .arg("-o")
    .arg(&output)
    .status()
    .await?;
```

Tokio provides asynchronous process spawning and waiting through `tokio::process`. ([docs.rs](https://docs.rs/tokio/latest/tokio/process/?utm_source=chatgpt.com))

## HTTP client strategy

Instantiate only the chosen platform. Do not construct six platform clients at startup.

Each selected platform should reuse one configured `reqwest::Client`. Reqwest clients already maintain an internal connection pool and already use shared ownership internally, so wrapping every client in another `Arc` is unnecessary. ([docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.Client.html?utm_source=chatgpt.com))

The client factory should enforce:

```text
- Explicit connection timeout.
- Explicit read timeout.
- Explicit total request timeout.
- Same-origin redirects or no redirects.
- Disabled automatic Referer when unnecessary.
- Low bounded idle connection count.
- Response body size limits.
- Per-host concurrency limits.
- Authentication applied per request.
- No credentials stored as global default headers.
```

Reqwest otherwise has no overall timeout by default, and its standard redirect policy follows redirects, so both should be deliberately configured for authenticated clients. ([docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html?utm_source=chatgpt.com))

A safe factory shape:

```rust
pub(crate) fn build_http_client() -> Result<reqwest::Client, reqwest::Error> {
    use std::time::Duration;

    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(60))
        .referer(false)
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(2)
        .build()
}
```

A platform that genuinely needs redirects should install a custom same-origin policy rather than globally following arbitrary locations.

## Memory rules

The biggest memory wins will come from architecture rather than tiny string tricks:

1. Construct only the selected platform.
2. Do not fetch entire problem catalogues when pagination is available.
3. Stream large response bodies and enforce maximum sizes.
4. Serialize directly into a locked output writer.
5. Keep concurrency bounded.
6. Avoid global caches unless their size is capped.
7. Avoid `Arc<Mutex<HashMap<...>>>` as the default state container.
8. Do not retain raw response JSON after converting it.
9. Avoid `serde_json::Value` for known schemas.
10. Do not clone problem statements, source files, and test output without need.

For JSON output:

```rust
let stdout = std::io::stdout();
let mut output = std::io::BufWriter::new(stdout.lock());

serde_json::to_writer(&mut output, &result)?;
```

This avoids constructing another complete JSON `String`.

Use ordinary `String` and `Vec<T>` first. Types such as `Box<str>`, `Box<[T]>`, `SmallVec`, and string interning can reduce memory in specific hot structures, but they can also increase complexity or binary size. Introduce them only after measuring.

## Security boundaries

### Crate-level policy

In the root and every platform crate:

```rust
#![forbid(unsafe_code)]
```

Allow unsafe code only in a tiny, separately reviewed module when a benchmark proves it is necessary. Remote API parsing, CLI handling, HTTP calls, and template generation do not require unsafe code.

### Secrets

- Prefer the operating system credential store.
- Wrap in-memory secrets in types that redact `Debug`.
- Never include cookies or bearer tokens in error values.
- Apply credentials only to verified platform origins.
- Use restrictive file permissions for any fallback credential file.
- Never place secrets into command-line arguments, since process lists may expose them.

### URLs and redirects

- Parse base URLs once with `url::Url`.
- Join paths through URL APIs, not string concatenation.
- Reject non-HTTPS origins unless explicitly enabled for local testing.
- Do not forward credentials across origins.
- Disable or constrain redirects.
- Validate endpoint hosts against the selected platform.

### Local execution

Your local runner is a separate security boundary:

```text
- Never invoke through `sh -c` or `cmd /C`.
- Pass every argument separately.
- Use a new temporary directory per execution.
- Restrict execution time.
- Restrict captured output size.
- Kill the complete child process group on timeout.
- Do not inherit sensitive environment variables.
- Do not execute source code downloaded from discussions or editorials.
- Sanitize generated filenames and reject path traversal.
```

### Retries

Retry only operations explicitly marked idempotent:

```text
Safe candidates:
- GET problem
- GET profile
- GraphQL read query
- Submission status polling

Do not automatically retry:
- Submit solution
- Join contest
- Change account state
- Delete cached or remote data
```

## Error structure

Use typed errors internally:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("platform does not support {capability}")]
    Unsupported {
        capability: &'static str,
    },

    #[error("authentication is required")]
    Unauthenticated,

    #[error("resource was not found")]
    NotFound,

    #[error("request was rate limited")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    #[error("remote response exceeded {limit} bytes")]
    ResponseTooLarge {
        limit: usize,
    },

    #[error("remote protocol error: {0}")]
    Protocol(String),

    #[error("network request failed")]
    Transport {
        #[source]
        source: reqwest::Error,
    },

    #[error("local execution failed: {0}")]
    Execution(String),
}
```

Use `anyhow` only at the executable boundary when adding top-level context. Do not pass `anyhow::Error` throughout your domain and platform contracts, because it removes useful error categories.

## Compiler optimisation profiles

You cannot simultaneously maximise runtime speed, minimise binary size, minimise memory, and minimise compilation time. Keep two intentional profiles.

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = "symbols"
incremental = false
overflow-checks = true

[profile.release-max]
inherits = "release"
lto = "fat"

[profile.release-small]
inherits = "release"
opt-level = "z"
lto = "fat"
```

Use:

```bash
# Balanced distribution build
cargo build --release

# Maximum optimisation, slower build
cargo build --profile release-max

# Smaller binary
cargo build --profile release-small
```

Cargo documents that fewer code-generation units may produce faster code at the cost of compilation parallelism, and supports both LTO settings and custom inherited profiles. ([doc.rust-lang.org](https://doc.rust-lang.org/nightly/cargo/reference/profiles.html?utm_source=chatgpt.com))

Do not distribute a binary compiled with:

```bash
RUSTFLAGS="-C target-cpu=native"
```

That is suitable for a local personal build, but it can emit instructions unsupported by another user’s CPU.

## Testing layout

Move tests out of `src/test`.

Use:

```text
src/module.rs
src/module/tests.rs
```

for focused unit tests, and:

```text
tests/
```

for public behaviour and CLI integration tests.

Platform crates should use recorded fixtures or a local mock HTTP server:

```text
platforms/leetcode/tests/fixtures/problem.json
platforms/codeforces/tests/fixtures/problemset.json
```

Create shared platform-contract tests for behavioural consistency:

```rust
async fn problem_catalog_contract<C>(catalog: &C)
where
    C: ProblemCatalog,
{
    // Valid IDs resolve.
    // Missing IDs become Error::NotFound.
    // Invalid input is rejected before an HTTP request.
    // Returned keys retain their platform identity.
}
```

Also fuzz or property-test:

```text
- Problem identifier parsing.
- URL building.
- Filename sanitisation.
- Test-case decoding.
- GraphQL and JSON response conversion.
- Output truncation.
- Template rendering.
```

## Mapping from your current TypeScript tree

```text
extension.ts
    → main.rs + bootstrap.rs + cli/dispatch.rs

leetcode/auth.ts
    → platforms/leetcode/src/auth.rs

leetcode/client.ts
    → platforms/leetcode/src/client.rs

leetcode/types.ts
    → platforms/leetcode/src/wire/*
      + platforms/leetcode/src/model/*

leetcode/boilerplate.ts
    → features/workspace/template.rs
      + ports/starter_code.rs
      + platform-specific starter-code endpoint

logger/*
    → infrastructure/telemetry.rs

statusBar.ts
    → output/progress.rs

utils/textRenderer.ts
    → output/text.rs

tree/AllProblemsTreeDataProvider.ts
    → features/problems/list.rs

tree/ContestsTreeDataProvider.ts
    → features/contests/list.rs

tree/DailyChallengeTreeDataProvider.ts
    → features/problems/daily.rs

webview/TestResultsPanel.ts
    → output/test_report.rs

src/test/*
    → inline unit tests + top-level tests/
```

## The platform addition checklist

Adding Codeforces or Exercism should require exactly these steps:

```text
1. Add platforms/<platform>/Cargo.toml.
2. Implement its concrete HTTP client.
3. Register typed endpoints inside that crate.
4. Expose stable platform models.
5. Add a root adapter.
6. Add a feature-gated SelectedPlatform variant.
7. Declare its capability set.
8. Add its optional Cargo feature.
9. Run the shared platform contract tests.
```

It should not require editing problem search, workspace generation, rendering, local testing, configuration parsing, or unrelated platform clients.

The core decision is therefore:

```text
vertical feature slices in the root crate
+
small capability traits
+
root-owned adapters
+
concrete enum dispatch
+
feature-gated platform crates
+
typed endpoint operations
```

That gives you the extensibility of a plugin architecture without paying for dynamic plugins in every request path.

## User

```text
.
├── Cargo.toml
├── Cargo.lock
│
├── src
│   ├── main.rs
│   ├── lib.rs
│   ├── bootstrap.rs
│   ├── error.rs
│   │
│   ├── cli
│   │   ├── mod.rs
│   │   ├── root.rs
│   │   ├── commands.rs
│   │   └── dispatch.rs
│   │
│   ├── domain
│   │   ├── mod.rs
│   │   ├── account.rs
│   │   ├── capability.rs
│   │   ├── contest.rs
│   │   ├── difficulty.rs
│   │   ├── language.rs
│   │   ├── platform.rs
│   │   ├── problem.rs
│   │   ├── solution.rs
│   │   └── submission.rs
│   │
│   ├── ports
│   │   ├── mod.rs
│   │   ├── authentication.rs
│   │   ├── cache.rs
│   │   ├── contests.rs
│   │   ├── problems.rs
│   │   ├── runner.rs
│   │   ├── secrets.rs
│   │   ├── starter_code.rs
│   │   └── submissions.rs
│   │
│   ├── features
│   │   ├── mod.rs
│   │   │
│   │   ├── auth
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── login.rs
│   │   │   ├── logout.rs
│   │   │   └── status.rs
│   │   │
│   │   ├── problems
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── daily.rs
│   │   │   ├── list.rs
│   │   │   ├── pick.rs
│   │   │   ├── search.rs
│   │   │   └── show.rs
│   │   │
│   │   ├── contests
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── list.rs
│   │   │   └── show.rs
│   │   │
│   │   ├── submissions
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── history.rs
│   │   │   ├── status.rs
│   │   │   └── submit.rs
│   │   │
│   │   ├── testing
│   │   │   ├── mod.rs
│   │   │   ├── args.rs
│   │   │   ├── compile.rs
│   │   │   ├── execute.rs
│   │   │   └── compare.rs
│   │   │
│   │   └── workspace
│   │       ├── mod.rs
│   │       ├── generate.rs
│   │       ├── naming.rs
│   │       └── template.rs
│   │
│   ├── platforms
│   │   ├── mod.rs
│   │   ├── kind.rs
│   │   ├── selected.rs
│   │   ├── build.rs
│   │   ├── dispatch.rs
│   │   └── adapters
│   │       ├── mod.rs
│   │       ├── leetcode.rs
│   │       ├── codeforces.rs
│   │       ├── hackerrank.rs
│   │       ├── exercism.rs
│   │       ├── project_euler.rs
│   │       └── hackerearth.rs
│   │
│   ├── infrastructure
│   │   ├── mod.rs
│   │   ├── cache.rs
│   │   ├── config.rs
│   │   ├── filesystem.rs
│   │   ├── http.rs
│   │   ├── process.rs
│   │   ├── secret_store.rs
│   │   └── telemetry.rs
│   │
│   └── output
│       ├── mod.rs
│       ├── format.rs
│       ├── json.rs
│       ├── progress.rs
│       ├── table.rs
│       └── text.rs
│
├── platforms
│   ├── leetcode
│   │   ├── Cargo.toml
│   │   ├── src
│   │   │   ├── lib.rs
│   │   │   ├── auth.rs
│   │   │   ├── client.rs
│   │   │   ├── endpoint.rs
│   │   │   ├── error.rs
│   │   │   ├── model
│   │   │   │   ├── mod.rs
│   │   │   │   ├── contest.rs
│   │   │   │   ├── problem.rs
│   │   │   │   └── submission.rs
│   │   │   ├── endpoints
│   │   │   │   ├── mod.rs
│   │   │   │   ├── authentication.rs
│   │   │   │   ├── contests.rs
│   │   │   │   ├── problems.rs
│   │   │   │   └── submissions.rs
│   │   │   └── wire
│   │   │       ├── mod.rs
│   │   │       ├── graphql.rs
│   │   │       ├── request.rs
│   │   │       └── response.rs
│   │   └── tests
│   │       ├── fixtures
│   │       └── client.rs
│   │
│   ├── codeforces
│   ├── hackerrank
│   ├── exercism
│   ├── project-euler
│   └── hackerearth
│
├── tests
│   ├── cli.rs
│   ├── platform_contracts.rs
│   ├── workspace_generation.rs
│   └── fixtures
│
└── benches
    ├── parsing.rs
    ├── rendering.rs
    └── workspace_generation.rs
```

make it into mkdir  -p and touch commands

## Assistant

Run this from the repository root:

```bash
mkdir -p \
  src/cli \
  src/domain \
  src/ports \
  src/features/auth \
  src/features/problems \
  src/features/contests \
  src/features/submissions \
  src/features/testing \
  src/features/workspace \
  src/platforms/adapters \
  src/infrastructure \
  src/output \
  platforms/leetcode/src/model \
  platforms/leetcode/src/endpoints \
  platforms/leetcode/src/wire \
  platforms/leetcode/tests/fixtures \
  platforms/codeforces \
  platforms/hackerrank \
  platforms/exercism \
  platforms/project-euler \
  platforms/hackerearth \
  tests/fixtures \
  benches
```

```bash
touch \
  Cargo.toml \
  Cargo.lock \
  src/main.rs \
  src/lib.rs \
  src/bootstrap.rs \
  src/error.rs \
  src/cli/mod.rs \
  src/cli/root.rs \
  src/cli/commands.rs \
  src/cli/dispatch.rs \
  src/domain/mod.rs \
  src/domain/account.rs \
  src/domain/capability.rs \
  src/domain/contest.rs \
  src/domain/difficulty.rs \
  src/domain/language.rs \
  src/domain/platform.rs \
  src/domain/problem.rs \
  src/domain/solution.rs \
  src/domain/submission.rs \
  src/ports/mod.rs \
  src/ports/authentication.rs \
  src/ports/cache.rs \
  src/ports/contests.rs \
  src/ports/problems.rs \
  src/ports/runner.rs \
  src/ports/secrets.rs \
  src/ports/starter_code.rs \
  src/ports/submissions.rs \
  src/features/mod.rs \
  src/features/auth/mod.rs \
  src/features/auth/args.rs \
  src/features/auth/login.rs \
  src/features/auth/logout.rs \
  src/features/auth/status.rs \
  src/features/problems/mod.rs \
  src/features/problems/args.rs \
  src/features/problems/daily.rs \
  src/features/problems/list.rs \
  src/features/problems/pick.rs \
  src/features/problems/search.rs \
  src/features/problems/show.rs \
  src/features/contests/mod.rs \
  src/features/contests/args.rs \
  src/features/contests/list.rs \
  src/features/contests/show.rs \
  src/features/submissions/mod.rs \
  src/features/submissions/args.rs \
  src/features/submissions/history.rs \
  src/features/submissions/status.rs \
  src/features/submissions/submit.rs \
  src/features/testing/mod.rs \
  src/features/testing/args.rs \
  src/features/testing/compile.rs \
  src/features/testing/execute.rs \
  src/features/testing/compare.rs \
  src/features/workspace/mod.rs \
  src/features/workspace/generate.rs \
  src/features/workspace/naming.rs \
  src/features/workspace/template.rs \
  src/platforms/mod.rs \
  src/platforms/kind.rs \
  src/platforms/selected.rs \
  src/platforms/build.rs \
  src/platforms/dispatch.rs \
  src/platforms/adapters/mod.rs \
  src/platforms/adapters/leetcode.rs \
  src/platforms/adapters/codeforces.rs \
  src/platforms/adapters/hackerrank.rs \
  src/platforms/adapters/exercism.rs \
  src/platforms/adapters/project_euler.rs \
  src/platforms/adapters/hackerearth.rs \
  src/infrastructure/mod.rs \
  src/infrastructure/cache.rs \
  src/infrastructure/config.rs \
  src/infrastructure/filesystem.rs \
  src/infrastructure/http.rs \
  src/infrastructure/process.rs \
  src/infrastructure/secret_store.rs \
  src/infrastructure/telemetry.rs \
  src/output/mod.rs \
  src/output/format.rs \
  src/output/json.rs \
  src/output/progress.rs \
  src/output/table.rs \
  src/output/text.rs \
  platforms/leetcode/Cargo.toml \
  platforms/leetcode/src/lib.rs \
  platforms/leetcode/src/auth.rs \
  platforms/leetcode/src/client.rs \
  platforms/leetcode/src/endpoint.rs \
  platforms/leetcode/src/error.rs \
  platforms/leetcode/src/model/mod.rs \
  platforms/leetcode/src/model/contest.rs \
  platforms/leetcode/src/model/problem.rs \
  platforms/leetcode/src/model/submission.rs \
  platforms/leetcode/src/endpoints/mod.rs \
  platforms/leetcode/src/endpoints/authentication.rs \
  platforms/leetcode/src/endpoints/contests.rs \
  platforms/leetcode/src/endpoints/problems.rs \
  platforms/leetcode/src/endpoints/submissions.rs \
  platforms/leetcode/src/wire/mod.rs \
  platforms/leetcode/src/wire/graphql.rs \
  platforms/leetcode/src/wire/request.rs \
  platforms/leetcode/src/wire/response.rs \
  platforms/leetcode/tests/client.rs \
  tests/cli.rs \
  tests/platform_contracts.rs \
  tests/workspace_generation.rs \
  benches/parsing.rs \
  benches/rendering.rs \
  benches/workspace_generation.rs
```

Or as one copy-pasteable block:

```bash
set -euo pipefail

mkdir -p \
  src/{cli,domain,ports,infrastructure,output} \
  src/features/{auth,problems,contests,submissions,testing,workspace} \
  src/platforms/adapters \
  platforms/leetcode/src/{model,endpoints,wire} \
  platforms/leetcode/tests/fixtures \
  platforms/{codeforces,hackerrank,exercism,project-euler,hackerearth} \
  tests/fixtures \
  benches

touch \
  Cargo.toml Cargo.lock \
  src/{main.rs,lib.rs,bootstrap.rs,error.rs} \
  src/cli/{mod.rs,root.rs,commands.rs,dispatch.rs} \
  src/domain/{mod.rs,account.rs,capability.rs,contest.rs,difficulty.rs,language.rs,platform.rs,problem.rs,solution.rs,submission.rs} \
  src/ports/{mod.rs,authentication.rs,cache.rs,contests.rs,problems.rs,runner.rs,secrets.rs,starter_code.rs,submissions.rs} \
  src/features/mod.rs \
  src/features/auth/{mod.rs,args.rs,login.rs,logout.rs,status.rs} \
  src/features/problems/{mod.rs,args.rs,daily.rs,list.rs,pick.rs,search.rs,show.rs} \
  src/features/contests/{mod.rs,args.rs,list.rs,show.rs} \
  src/features/submissions/{mod.rs,args.rs,history.rs,status.rs,submit.rs} \
  src/features/testing/{mod.rs,args.rs,compile.rs,execute.rs,compare.rs} \
  src/features/workspace/{mod.rs,generate.rs,naming.rs,template.rs} \
  src/platforms/{mod.rs,kind.rs,selected.rs,build.rs,dispatch.rs} \
  src/platforms/adapters/{mod.rs,leetcode.rs,codeforces.rs,hackerrank.rs,exercism.rs,project_euler.rs,hackerearth.rs} \
  src/infrastructure/{mod.rs,cache.rs,config.rs,filesystem.rs,http.rs,process.rs,secret_store.rs,telemetry.rs} \
  src/output/{mod.rs,format.rs,json.rs,progress.rs,table.rs,text.rs} \
  platforms/leetcode/Cargo.toml \
  platforms/leetcode/src/{lib.rs,auth.rs,client.rs,endpoint.rs,error.rs} \
  platforms/leetcode/src/model/{mod.rs,contest.rs,problem.rs,submission.rs} \
  platforms/leetcode/src/endpoints/{mod.rs,authentication.rs,contests.rs,problems.rs,submissions.rs} \
  platforms/leetcode/src/wire/{mod.rs,graphql.rs,request.rs,response.rs} \
  platforms/leetcode/tests/client.rs \
  tests/{cli.rs,platform_contracts.rs,workspace_generation.rs} \
  benches/{parsing.rs,rendering.rs,workspace_generation.rs}
```

To confirm the generated structure:

```bash
tree -a -I target
```

## User

how to make it module by module and make it efficient, optimised, secure, modular and fast async and agressively compiler optimised for speed and lowest memory and security, zero cost abstractions and least lines first

## Assistant

# Build vertically, not directory-first

Treat your large tree as the **destination**, not a checklist. The cleanest, fastest path is one complete command first:

```text
judge problem show two-sum
```

That command should travel through:

```text
CLI
  → feature
  → selected platform
  → LeetCode adapter
  → LeetCode client
  → domain model
  → output
```

Once that path works, add `search`, `daily`, authentication, submission, local testing, caching, then the second platform. Empty modules are architectural confetti.

## 1. Start with this smaller tree

Do not create all the final modules yet:

```text
src/
├── main.rs
├── lib.rs
├── cli.rs
├── domain.rs
├── domain/
│   └── problem.rs
├── error.rs
├── features.rs
├── features/
│   └── problems.rs
├── platforms.rs
├── platforms/
│   └── leetcode.rs
└── output.rs

platforms/
└── leetcode/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── client.rs
        ├── error.rs
        └── model.rs
```

Prefer the modern layout:

```text
features.rs
features/problems.rs
features/problems/show.rs
```

over:

```text
features/mod.rs
features/problems/mod.rs
features/problems/show.rs
```

Both work, but the first avoids a small blizzard of `mod.rs` files.

Only split a file when it has at least two distinct responsibilities or becomes hard to navigate. A rough threshold of 200 to 300 lines is useful, but responsibility matters more than line count.

---

# 2. Establish the dependency rules first

Keep this direction:

```text
main
  ↓
lib / composition
  ↓
cli ────────→ features ────────→ domain
                  ↓
               platforms
                  ↓
              adapters
                  ↓
          platform client crate

output ─────→ domain
infrastructure ─────→ ports
```

Never allow:

```text
domain → reqwest
domain → clap
domain → tokio
platform crate → root application
feature → concrete HTTP request
output → platform client
```

The module rules should be:

| Module | Owns | Must not own |
|---|---|---|
| `main.rs` | Runtime and exit code | Business logic |
| `lib.rs` | Application composition | HTTP details |
| `cli.rs` | Parsing commands | Executing commands |
| `domain` | Validated business types | I/O, async, Clap |
| `features` | Use-case orchestration | Raw HTTP |
| `platforms` | Platform selection and adapters | CLI parsing |
| Platform crate | HTTP protocol and wire formats | Root domain types |
| `output` | Rendering and serialization | Fetching data |
| `infrastructure` | Cache, filesystem, process, secrets | Use-case policy |

Default everything to `pub(crate)`. Make something `pub` only because another crate genuinely needs it.

---

# 3. Module 1: workspace and compiler policy

Start with a small root `Cargo.toml`:

```toml
[package]
name = "judge-cli"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[workspace]
members = ["platforms/*"]
default-members = ["."]
resolver = "3"

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"
repository = "https://github.com/OWNER/REPOSITORY"

[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
url = "2"

[features]
default = ["leetcode"]
leetcode = ["dep:platform-leetcode"]

[dependencies]
clap = { version = "4", features = ["derive"] }
serde.workspace = true
serde_json = "1"
thiserror.workspace = true

tokio = {
    version = "1",
    default-features = false,
    features = ["macros", "rt", "time"]
}

platform-leetcode = {
    path = "platforms/leetcode",
    optional = true
}

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = "debuginfo"
incremental = false
overflow-checks = true

[profile.release-max]
inherits = "release"
lto = "fat"

[profile.release-small]
inherits = "release"
opt-level = "s"
lto = "fat"
```

Cargo workspaces share one lockfile and target directory, while optional dependencies and features let you omit unused platform crates from a build. Resolver version 3 is the Edition 2024 default and considers incompatible Rust versions during dependency resolution. ([doc.rust-lang.org](https://doc.rust-lang.org/cargo/reference/features.html?utm_source=chatgpt.com))

Use:

```bash
# Normal release
cargo build --release

# Maximum whole-program optimisation
cargo build --profile release-max

# Smaller binary
cargo build --profile release-small

# Codeforces-only build later
cargo build \
  --release \
  --no-default-features \
  --features codeforces
```

`opt-level = 3` is not guaranteed to outperform `2`, and size modes are not guaranteed to produce the smallest binary. Measure all serious release profiles. Thin LTO usually provides much of the benefit of whole-program optimisation with substantially lower link cost than fat LTO, while one code-generation unit permits broader optimisation at the cost of slower compilation. ([doc.rust-lang.org](https://doc.rust-lang.org/nightly/cargo/reference/profiles.html?utm_source=chatgpt.com))

Each platform crate should inherit the lints:

```toml
[lints]
workspace = true
```

---

# 4. Module 2: the smallest domain

Keep domain code synchronous, dependency-light and validated.

```rust
// src/domain.rs

mod problem;

pub(crate) use problem::{Problem, ProblemId};
```

```rust
// src/domain/problem.rs

use serde::Serialize;

const MAX_PROBLEM_ID_LEN: usize = 128;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
#[serde(transparent)]
pub(crate) struct ProblemId(Box<str>);

impl ProblemId {
    pub(crate) fn parse(value: String) -> Option<Self> {
        let valid = !value.is_empty()
            && value.len() <= MAX_PROBLEM_ID_LEN
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
            });

        valid.then(|| Self(value.into_boxed_str()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Problem {
    pub(crate) id: ProblemId,
    pub(crate) title: Box<str>,
    pub(crate) statement: Box<str>,
}
```

Important choices:

- Do not derive `Clone` for `Problem` unless something actually needs to duplicate complete problem statements.
- Use `Box<str>` for immutable, long-lived strings.
- Use `String` for text still being constructed or mutated.
- Validate identifiers once at their boundary.
- Do not put platform-specific numeric IDs directly into universal domain types.
- Do not put raw JSON in domain objects.
- Do not put authentication state in domain objects.

A future universal ID can be:

```rust
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct ProblemKey {
    pub(crate) platform: PlatformKind,
    pub(crate) native: Box<str>,
}
```

---

# 5. Module 3: one typed application error

Use one root error enum. Never include cookies, tokens, response headers or complete response bodies in it.

```rust
// src/error.rs

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("invalid problem identifier")]
    InvalidProblemId,

    #[error("{platform} does not support {capability}")]
    Unsupported {
        platform: &'static str,
        capability: &'static str,
    },

    #[cfg(feature = "leetcode")]
    #[error(transparent)]
    LeetCode(#[from] platform_leetcode::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

Avoid:

```rust
anyhow::Result<T>
```

through the entire codebase. `anyhow` is useful at an executable boundary, but typed errors are much better for:

- Retry decisions
- Exit codes
- User-facing messages
- Authentication recovery
- Rate-limit handling
- Tests
- Avoiding accidental secret leakage

Also avoid `String` errors such as:

```rust
Err("request failed".to_owned())
```

because the compiler cannot help you distinguish failure categories.

---

# 6. Module 4: the first platform crate

The platform crate owns only LeetCode protocol behaviour.

```toml
# platforms/leetcode/Cargo.toml

[package]
name = "platform-leetcode"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true

[lints]
workspace = true

[dependencies]
reqwest = {
    version = "0.13",
    default-features = false,
    features = ["rustls", "json"]
}

serde.workspace = true
serde_json = "1"
thiserror.workspace = true
url.workspace = true
```

Disabling default features and explicitly selecting the TLS and protocol features prevents unused Reqwest facilities from quietly entering the dependency graph. Cargo features are additive, so the whole workspace must consistently disable unwanted defaults. Reqwest documents separate features for TLS, cookies, compression, query encoding, JSON and streaming. ([doc.rust-lang.org](https://doc.rust-lang.org/cargo/reference/features.html?utm_source=chatgpt.com))

## Platform model

```rust
// platforms/leetcode/src/model.rs

#[derive(Debug)]
pub struct Problem {
    pub id: Box<str>,
    pub title: Box<str>,
    pub statement: Box<str>,
}
```

This is a stable public model owned by the LeetCode crate. It is **not** the raw GraphQL response.

## Platform error

```rust
// platforms/leetcode/src/error.rs

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid LeetCode base URL")]
    InvalidBaseUrl(#[source] url::ParseError),

    #[error("LeetCode request failed")]
    Transport(#[from] reqwest::Error),

    #[error("LeetCode returned an unsuccessful status")]
    Status,

    #[error("LeetCode response exceeded {limit} bytes")]
    ResponseTooLarge { limit: usize },

    #[error("LeetCode returned an invalid response")]
    Decode(#[from] serde_json::Error),

    #[error("LeetCode returned incomplete problem data")]
    InvalidResponse,
}
```

## HTTP client

```rust
// platforms/leetcode/src/client.rs

use std::time::Duration;

use reqwest::{redirect::Policy, Client as HttpClient};
use url::Url;

use crate::{error::Result, model::Problem, Error};

const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base: Url,
}

impl Client {
    pub fn new() -> Result<Self> {
        let http = HttpClient::builder()
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(20))
            .timeout(Duration::from_secs(30))
            .redirect(Policy::none())
            .pool_max_idle_per_host(2)
            .build()?;

        let base = Url::parse("https://leetcode.com/")
            .map_err(Error::InvalidBaseUrl)?;

        Ok(Self { http, base })
    }

    pub async fn problem(&self, slug: &str) -> Result<Problem> {
        let url = self
            .base
            .join("graphql/")
            .map_err(Error::InvalidBaseUrl)?;

        let request = ProblemRequest::new(slug);

        let response = self
            .http
            .post(url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let wire: ProblemResponse = read_json(response).await?;
        wire.try_into()
    }
}
```

Reuse one Reqwest client for the selected platform. Reqwest maintains its own connection pool and already uses shared ownership internally, so another `Arc<reqwest::Client>` is normally unnecessary. ([docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.Client.html?utm_source=chatgpt.com))

Reqwest has no total, read or connection timeout by default, and follows redirects by default. Authenticated clients should therefore configure those policies explicitly rather than accepting invisible network behaviour. ([docs.rs](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html?utm_source=chatgpt.com))

## Bounded response decoding

Do not call `response.bytes()` on an untrusted, unlimited response.

```rust
async fn read_json<T>(
    mut response: reqwest::Response,
) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(Error::ResponseTooLarge {
            limit: MAX_RESPONSE_BYTES,
        });
    }

    let mut body = Vec::with_capacity(8 * 1024);

    while let Some(chunk) = response.chunk().await? {
        let next = body
            .len()
            .checked_add(chunk.len())
            .ok_or(Error::ResponseTooLarge {
                limit: MAX_RESPONSE_BYTES,
            })?;

        if next > MAX_RESPONSE_BYTES {
            return Err(Error::ResponseTooLarge {
                limit: MAX_RESPONSE_BYTES,
            });
        }

        body.extend_from_slice(&chunk);
    }

    Ok(serde_json::from_slice(&body)?)
}
```

That prevents a hostile or broken endpoint from turning a tiny CLI request into a memory balloon.

Keep the initial request and response structs in `client.rs`. Split them into:

```text
wire/
endpoints/
endpoint.rs
```

only after you have approximately three endpoints and obvious repetition.

---

# 7. Module 5: root adapter and platform selection

The root owns conversion from a platform model into the application domain.

```rust
// src/platforms.rs

#[cfg(feature = "leetcode")]
mod leetcode;

use crate::{
    domain::{Problem, ProblemId},
    error::Result,
};

pub(crate) enum Selected {
    #[cfg(feature = "leetcode")]
    LeetCode(platform_leetcode::Client),
}

impl Selected {
    pub(crate) fn leetcode() -> Result<Self> {
        Ok(Self::LeetCode(platform_leetcode::Client::new()?))
    }

    pub(crate) async fn problem(
        &self,
        id: &ProblemId,
    ) -> Result<Problem> {
        match self {
            #[cfg(feature = "leetcode")]
            Self::LeetCode(client) => {
                let problem = client.problem(id.as_str()).await?;
                Ok(problem.into())
            }
        }
    }
}
```

```rust
// src/platforms/leetcode.rs

use crate::domain::{Problem, ProblemId};

impl From<platform_leetcode::Problem> for Problem {
    fn from(value: platform_leetcode::Problem) -> Self {
        Self {
            id: ProblemId::parse(value.id.into())
                .expect("platform client returned a validated identifier"),
            title: value.title,
            statement: value.statement,
        }
    }
}
```

For stricter code, use `TryFrom` and remove the `expect`:

```rust
impl TryFrom<platform_leetcode::Problem> for Problem {
    type Error = crate::Error;

    fn try_from(value: platform_leetcode::Problem) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ProblemId::parse(value.id.into())
                .ok_or(crate::Error::InvalidProblemId)?,
            title: value.title,
            statement: value.statement,
        })
    }
}
```

Start with this concrete enum. It is fewer lines than a full port architecture and already gives you:

- Runtime platform selection
- Exhaustive matching
- Static concrete calls
- No vtable
- No boxed future
- No service locator

---

# 8. Module 6: the first feature

A feature should perform orchestration, not protocol work.

```rust
// src/features.rs

pub(crate) mod problems;
```

```rust
// src/features/problems.rs

use crate::{
    domain::{Problem, ProblemId},
    error::Result,
    platforms::Selected,
};

pub(crate) async fn show(
    platform: &Selected,
    id: ProblemId,
) -> Result<Problem> {
    platform.problem(&id).await
}
```

That is intentionally tiny.

Do not create:

```text
service.rs
repository.rs
handler.rs
manager.rs
controller.rs
use_case.rs
```

around a single call. Those names often become a paper labyrinth before they provide any isolation.

Split `problems.rs` only when it gains several operations:

```text
features/
├── problems.rs
└── problems/
    ├── daily.rs
    ├── list.rs
    ├── search.rs
    └── show.rs
```

---

# 9. Module 7: CLI parsing

The CLI parses typed data and performs no HTTP.

```rust
// src/cli.rs

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[arg(long, default_value_t = Platform::LeetCode)]
    pub(crate) platform: Platform,

    #[arg(long, default_value_t = Format::Text)]
    pub(crate) format: Format,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Platform {
    LeetCode,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Format {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Show {
        id: String,
    },
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}
```

Clap’s derive API supports typed parsers, reusable argument groups and nested subcommands. Keep argument definitions beside the feature once they become substantial, but retain a single visible command-registration enum. ([docs.rs](https://docs.rs/clap/latest/clap/_derive/?utm_source=chatgpt.com))

---

# 10. Module 8: output

Return structured values from features. Render at the edge.

```rust
// src/output.rs

use std::io::{self, Write};

use crate::{
    cli::Format,
    domain::Problem,
    error::Result,
};

pub(crate) fn problem(
    format: Format,
    problem: &Problem,
) -> Result<()> {
    match format {
        Format::Json => {
            let stdout = io::stdout();
            serde_json::to_writer(stdout.lock(), problem)?;
        }

        Format::Text => {
            let stdout = io::stdout();
            let mut output = stdout.lock();

            writeln!(output, "{}\n", problem.title)?;
            writeln!(output, "{}", problem.statement)?;
        }
    }

    Ok(())
}
```

Writing directly into a locked output stream avoids constructing a second complete JSON or text buffer.

Keep colour, progress indicators and terminal detection out until the plain output works. JSON output must never contain progress messages.

---

# 11. Module 9: composition

Your initial `lib.rs` can be the composition root. You do not need `bootstrap.rs` yet.

```rust
// src/lib.rs

mod cli;
mod domain;
mod error;
mod features;
mod output;
mod platforms;

use error::Result;

pub async fn run() -> Result<()> {
    let cli = cli::parse();

    let platform = match cli.platform {
        cli::Platform::LeetCode => platforms::Selected::leetcode()?,
    };

    match cli.command {
        cli::Command::Show { id } => {
            let id = domain::ProblemId::parse(id)
                .ok_or(error::Error::InvalidProblemId)?;

            let problem = features::problems::show(
                &platform,
                id,
            )
            .await?;

            output::problem(cli.format, &problem)
        }
    }
}
```

```rust
// src/main.rs

use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match judge_cli::run().await {
        Ok(()) => ExitCode::SUCCESS,

        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
```

A current-thread Tokio runtime creates no worker threads and can still drive asynchronous network I/O, timers and subprocess waiting. It is a sensible starting point for a mostly network-bound CLI, although Tokio recommends benchmarking against its multithreaded runtime when workloads become CPU-heavy. ([docs.rs](https://docs.rs/tokio/latest/tokio/runtime/?utm_source=chatgpt.com))

Create `bootstrap.rs` only when construction has become substantial, for example:

```text
config
HTTP clients
secret store
cache
workspace manager
local runner
telemetry
```

At that point:

```rust
pub(crate) struct App {
    pub(crate) platform: Selected,
    pub(crate) cache: Cache,
    pub(crate) runner: Runner,
    pub(crate) secrets: SecretStore,
}
```

Do not pass the complete `App` into every feature. Pass only the capabilities a feature needs.

---

# 12. Add ports only when the second implementation exists

Do not begin with eight empty traits.

When you add Codeforces, or when a feature genuinely needs a fake implementation for testing, extract a capability:

```rust
// src/ports/problems.rs

use crate::{
    domain::{Problem, ProblemId},
    error::Result,
};

pub(crate) trait ProblemSource {
    async fn problem(
        &self,
        id: &ProblemId,
    ) -> Result<Problem>;
}
```

Then make the feature generic:

```rust
pub(crate) async fn show<S>(
    source: &S,
    id: ProblemId,
) -> Result<Problem>
where
    S: ProblemSource,
{
    source.problem(&id).await
}
```

Implement it for the selected enum:

```rust
impl ProblemSource for Selected {
    async fn problem(
        &self,
        id: &ProblemId,
    ) -> Result<Problem> {
        match self {
            Self::LeetCode(client) => {
                Ok(client.problem(id.as_str()).await?.try_into()?)
            }

            Self::Codeforces(client) => {
                Ok(client.problem(id.as_str()).await?.try_into()?)
            }
        }
    }
}
```

Internal async traits plus generic callers give you static dispatch without `Box<dyn Future>`. Async trait methods and return-position `impl Trait` are not dyn-compatible, so use generic or enum dispatch rather than trying to force these traits behind `dyn`. ([doc.rust-lang.org](https://doc.rust-lang.org/reference/items/traits.html?utm_source=chatgpt.com))

Use small capability traits:

```text
ProblemSource
ContestSource
Authenticator
StarterCodeSource
SubmissionService
RemoteTester
```

Avoid a giant trait:

```rust
trait Platform {
    async fn login(...);
    async fn logout(...);
    async fn daily(...);
    async fn problem(...);
    async fn search(...);
    async fn contests(...);
    async fn submit(...);
    async fn discussions(...);
    // forty more methods
}
```

Such a trait forces Project Euler to pretend it supports authentication or contests, and forces every feature to depend on everything.

---

# 13. Fast async rules

## Do not spawn by default

For fixed independent operations:

```rust
let (profile, daily) = tokio::try_join!(
    client.profile(),
    client.daily(),
)?;
```

Spawning would require `'static` ownership, often causing extra `Arc`s and clones for no gain.

Use `tokio::spawn` only when:

- Work has an independent lifecycle.
- You have dynamic fan-out.
- Cancellation ownership is explicit.
- Parallel execution materially improves latency.

## Bound dynamic fan-out

```rust
use std::sync::Arc;

use tokio::{
    sync::Semaphore,
    task::JoinSet,
};

async fn fetch_many(
    client: platform_leetcode::Client,
    ids: Vec<Box<str>>,
) -> Result<Vec<platform_leetcode::Problem>, platform_leetcode::Error> {
    let permits = Arc::new(Semaphore::new(4));
    let mut tasks = JoinSet::new();

    for id in ids {
        let permit = permits.clone().acquire_owned().await
            .expect("semaphore remains open");

        let client = client.clone();

        tasks.spawn(async move {
            let _permit = permit;
            client.problem(&id).await
        });
    }

    let mut problems = Vec::with_capacity(tasks.len());

    while let Some(result) = tasks.join_next().await {
        problems.push(result.expect("task did not panic")?);
    }

    Ok(problems)
}
```

For production, convert task panics and semaphore closure into typed errors rather than using `expect`.

A Tokio semaphore bounds concurrent resource use, and `JoinSet` owns a dynamic task collection whose tasks are aborted if the set is dropped. ([docs.rs](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html?utm_source=chatgpt.com))

## Do not hold locks across `.await`

Avoid:

```rust
let mut cache = cache.lock().await;
let problem = client.problem(id).await?;
cache.insert(id, problem);
```

Instead:

```rust
let problem = client.problem(id).await?;

let mut cache = cache.lock().await;
cache.insert(id, problem);
```

Better still, use a disk cache with short critical sections rather than one global `Mutex<HashMap<...>>`.

## Keep CPU and blocking work off the async executor

Use:

```rust
tokio::process::Command
```

for compilers and test executables.

Use `spawn_blocking` only for unavoidable synchronous CPU or filesystem libraries. Be aware that an already-running `spawn_blocking` operation cannot be aborted like a normal async task. ([docs.rs](https://docs.rs/tokio/latest/tokio/task/?utm_source=chatgpt.com))

---

# 14. Lowest-memory rules

Apply these in order:

1. **Never load an unlimited response body.**
2. **Do not cache an entire platform catalogue in RAM.**
3. **Bound concurrency.**
4. **Reuse one HTTP client.**
5. **Write output directly to the destination.**
6. **Do not derive `Clone` indiscriminately.**
7. **Drop raw wire responses after conversion.**
8. **Use pagination rather than bulk downloads.**
9. **Use immutable boxed strings for retained models.**
10. **Construct only the selected platform client.**

Avoid premature cleverness:

```text
SmallVec
ArrayVec
string interning
arena allocation
custom global allocators
memory mapping
manual unsafe parsing
```

Those may help after profiling, but they can also increase binary size, complexity and attack surface.

Static generics are runtime-cheap, but excessive monomorphisation can increase binary size. Keep generic dispatch at capability boundaries, then use concrete types inside each implementation.

---

# 15. Security rules per layer

## Domain

- Validate lengths and character sets.
- Represent valid state with newtypes.
- Use checked arithmetic for attacker-controlled sizes.
- Do not accept arbitrary filesystem paths as problem IDs.

## HTTP client

- HTTPS only by default.
- Explicit timeouts.
- Disable redirects or permit same-origin redirects only.
- Cap response body size.
- Cap decompressed data size.
- Do not log request headers.
- Retry only idempotent reads.
- Never automatically retry a solution submission.
- Never attach credentials through universal default headers if a client may contact multiple origins.

## Authentication

Use a redacted secret type:

```rust
pub(crate) struct SessionToken(Box<str>);

impl std::fmt::Debug for SessionToken {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        formatter.write_str("SessionToken([REDACTED])")
    }
}
```

Store persistent credentials in the operating-system credential store. Avoid plaintext TOML credentials.

## Filesystem

- Canonicalise the workspace root.
- Reject `..`, absolute paths and path separators in generated names.
- Write temporary files, then atomically rename.
- Refuse symlink escapes where relevant.
- Apply restrictive permissions to credential and session files.

## Local execution

Never do:

```rust
Command::new("sh")
    .arg("-c")
    .arg(user_controlled_command);
```

Do:

```rust
Command::new(&compiler)
    .arg("--edition=2024")
    .arg(&source)
    .arg("-o")
    .arg(&binary);
```

Also:

- Use a unique temporary directory.
- Set a timeout.
- Bound stdout and stderr.
- Clear sensitive environment variables.
- Kill the child process tree on timeout.
- Do not execute downloaded editorial or discussion code.
- Add OS sandboxing before claiming execution is untrusted-code-safe.

## Crate boundary

Place this in every crate:

```rust
#![forbid(unsafe_code)]
```

Also inherit workspace lints so a platform crate cannot quietly reintroduce unsafe code.

---

# 16. When each original directory should appear

## `ports/`

Create it when there are two implementations or a real test fake.

```text
ports/problems.rs
ports/submissions.rs
ports/authentication.rs
```

Not before.

## `infrastructure/`

Create a module only when the concrete service exists:

```text
infrastructure/cache.rs
infrastructure/process.rs
infrastructure/secret_store.rs
```

`http.rs` probably does not belong here if every platform owns materially different HTTP policy.

## `platforms/<name>/wire/`

Create after several endpoint response types exist. Keep every wire type private.

```text
wire/graphql.rs
wire/problem.rs
wire/submission.rs
```

## `platforms/<name>/endpoints/`

Create when common request execution is repeated.

At that point, introduce a zero-sized typed operation:

```rust
pub(crate) trait Operation {
    type Request: serde::Serialize + Sync;
    type Response: serde::de::DeserializeOwned;

    const PATH: &'static str;
    const AUTHENTICATED: bool;
    const RETRYABLE: bool;
    const MAX_RESPONSE_BYTES: usize;
}
```

The compiler can specialise those associated constants for each concrete operation.

## `bootstrap.rs`

Create when composition no longer fits comfortably in `lib.rs`.

## `capability.rs`

Create when the second platform exposes different functionality.

Use a bitset or small enum set rather than a heap-backed `HashSet`.

---

# 17. Recommended implementation order

## Milestone 1: one read-only command

```text
problem show
```

Modules:

```text
domain/problem
platform-leetcode/client
platform adapter
feature show
CLI
text output
```

No authentication, cache, runner or generic ports.

## Milestone 2: read-only problem workflow

```text
problem show
problem search
problem daily
problem list
```

Now split:

```text
features/problems/
platforms/leetcode/endpoints/
platforms/leetcode/wire/
```

## Milestone 3: authentication

Add:

```text
ports/authentication.rs
ports/secrets.rs
infrastructure/secret_store.rs
platforms/leetcode/auth.rs
features/auth/
```

## Milestone 4: submissions

Add:

```text
domain/submission.rs
ports/submissions.rs
features/submissions/
platform endpoint submission code
```

Submission writes are non-retryable unless the platform provides a genuine idempotency mechanism.

## Milestone 5: local testing

Add:

```text
ports/runner.rs
infrastructure/process.rs
features/testing/
domain/solution.rs
```

## Milestone 6: disk cache

Add it after profiling confirms network or repeated parsing is a problem.

```text
ports/cache.rs
infrastructure/cache.rs
```

Prefer a bounded disk cache over a permanent in-memory catalogue.

## Milestone 7: Codeforces

Only now extract shared capability traits and platform contract tests.

```text
platforms/codeforces/
src/platforms/codeforces.rs
```

This validates whether the abstractions are genuinely universal or merely LeetCode concepts wearing fake moustaches.

## Milestone 8: remaining platforms

Add one at a time:

```text
Exercism
Project Euler
HackerRank
HackerEarth
```

Do not stub them all simultaneously.

---

# 18. Aggressive optimisation comes last

Once you have representative benchmarks, add PGO. Rust’s PGO workflow compiles an instrumented binary, runs representative workloads, merges the collected profile and recompiles with that profile to guide inlining, branch layout and register allocation. ([doc.rust-lang.org](https://doc.rust-lang.org/rustc/profile-guided-optimization.html?utm_source=chatgpt.com))

A useful profile corpus for this CLI would include:

```bash
judge --help
judge problem show two-sum
judge problem search graph
judge problem daily
judge --format json problem show two-sum
judge test ./solution.rs
```

Do not train PGO only on `--help`, or you will produce a world-class help screen and a merely adequate application.

For distributed binaries, do not use:

```bash
RUSTFLAGS="-C target-cpu=native"
```

That may emit instructions unavailable on another user’s CPU. Reserve native CPU builds for your own machine or publish separate target-specific artifacts. Rust’s target-feature documentation explicitly treats manually selected CPU features as architecture-dependent and potentially unsafe when used incorrectly. ([doc.rust-lang.org](https://doc.rust-lang.org/rustc/codegen-options/?utm_source=chatgpt.com))

---

# 19. The shortest set of enduring rules

```text
Build one vertical slice first.
Create abstractions on the second use.
Keep domain synchronous and pure.
Keep wire models private.
Use concrete enum dispatch at runtime boundaries.
Use generic capability traits inside features.
Never use unbounded concurrency.
Never read unbounded network or process output.
Reuse one HTTP client.
Do not derive Clone automatically.
Do not use dyn or async_trait on hot paths.
Do not add unsafe code before measurement proves necessity.
Do not add a cache before measuring.
Do not add a custom allocator before profiling.
Do not use inline(always) as seasoning.
Do not optimise code that has no benchmark.
```

The first concrete target should therefore contain only enough code for:

```bash
judge --platform leetcode problem show two-sum
```

Everything else grows outward from that one verified path.

## User

export entire chat history to chat.md
