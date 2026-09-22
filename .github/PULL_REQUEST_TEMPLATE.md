# Summary

<!-- Describe the cp-cli change and the user-facing result -->

Closes #

## Compatibility

<!-- Note changed commands, output, configuration, platform behaviour, or migration; write "None" when not applicable -->

## Verification

<!-- Include commands run and relevant results -->

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

## Checklist

- [ ] The change is focused and documented where users need it
- [ ] Tests cover changed behaviour or the reason they are unnecessary is stated above
- [ ] No secrets, sessions, or private vulnerability details are included
