# Contributing

Thank you for improving `cp-cli`, the Rust CLI for working with coding problems from the terminal.

## Before you start

- Report a suspected vulnerability privately by following [SECURITY.md](SECURITY.md); use the public security form only for safe hardening ideas.
- Open an issue before substantial commands, platform support, dependencies, or behaviour changes.
- Keep a pull request focused. Small fixes and documentation corrections can go directly to a pull request.
- Never include sessions, cookies, API keys, personal data, or undisclosed vulnerability details in public issues or pull requests.

## Development

Install Rust 1.88 or newer, then build and run the CLI from this checkout:

```sh
cargo build --release --locked
./target/release/cp-cli --help
```

Before submitting, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

Add focused tests for behaviour changes and update the README or relevant `docs/` page when a command, output, configuration option, or platform capability changes.

## Pull requests

Describe the user-visible change, the commands you ran, and any compatibility or platform limitations. Keep commits and pull requests reviewable; avoid unrelated formatting, generated files, and dependency updates.

## Licence

By contributing, you confirm that you have the right to provide the material under this repository's existing licence.
