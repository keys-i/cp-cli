# Security change

## Summary

<!-- Briefly describe the security, privacy, or hardening behaviour introduced by this pull request. -->

## Reusable use case

<!-- Explain why this behaviour belongs in the package rather than in one application or website wrapper. -->

## Related issue or advisory

<!-- Link the public issue, or reference the private security advisory without copying sensitive details here. -->

Closes #

## Threat model

<!--
Describe the risk this change addresses:
- protected asset
- attacker capability
- entry point or trust boundary
- expected security property

Do not include credentials, secrets, working exploit code, or undisclosed vulnerability details.
-->

## Behaviour and API

<!-- Describe secure defaults, opt-in behaviour, failure handling, and any public API changes. -->

```tsx
// Add a minimal usage example when relevant.
```

## Compatibility and migration

<!-- Note breaking changes, changed defaults, browser/runtime constraints, and any required migration steps. Write "None" when not applicable. -->

## Verification

<!-- Include the commands used and relevant results. Cover both expected and hostile or malformed inputs. -->

```text
# Example
npm test
npm run lint
npm run typecheck
```

## Security review checklist

- [ ] The change is covered by unit, integration, or regression tests as appropriate
- [ ] Negative cases and malformed or hostile inputs are tested
- [ ] Defaults are secure and documented
- [ ] Failure behaviour is explicit and fails closed where appropriate
- [ ] Authentication and authorisation checks occur at the correct trust boundary
- [ ] Sensitive values are not exposed through logs, errors, telemetry, snapshots, or fixtures
- [ ] No credentials, tokens, private keys, or production data are committed
- [ ] New runtime dependencies are avoided or justified and reviewed
- [ ] Cryptographic behaviour uses established, maintained primitives rather than custom cryptography
- [ ] Backwards compatibility and migration implications are documented
- [ ] Public documentation and examples have been updated where required
- [ ] This pull request is safe to discuss publicly, or is coordinated through a private security advisory

## Reviewer notes

<!-- Call out the files, assumptions, trade-offs, or attack paths that deserve particular scrutiny. -->
