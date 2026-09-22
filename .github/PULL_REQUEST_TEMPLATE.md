# Summary

<!-- What changed for users? -->

Closes #

## Compatibility

<!-- Commands, output, config, platform behaviour, or migration. Write "None" if unchanged. -->

## Verification

<!-- Commands run and relevant results -->

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

## Checklist

- [ ] Focused change; user docs updated where needed
- [ ] Tests cover changed behaviour, or the reason they are unnecessary is above
- [ ] No secrets, sessions, or private vulnerability details
