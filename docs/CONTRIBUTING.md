# Contributing

Thanks for improving `cp-cli`.

## Before you start

- Report vulnerabilities privately through [SECURITY.md](SECURITY.md).
  Use public reports only for safe hardening ideas.
- Discuss substantial commands, platform support, dependencies, or behaviour changes
  in an issue first.
- Small fixes and doc corrections can go straight to a focused pull request.
- Never publish sessions, cookies, API keys, personal data, or undisclosed vulnerability details.

## Development

Install Rust 1.88+, then build and run:

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

Add focused tests for behaviour changes. Update the README or relevant docs when
commands, output, configuration, or platform support change.

## Pull requests

Say what changed for users, what you ran, and any compatibility or platform limits.
Keep the diff reviewable. Leave unrelated formatting, generated files, and dependency
updates out.

## Licence

By contributing, you confirm that you have the right to provide the material under this
repository's existing licence.
