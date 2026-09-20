# Contributing

Thank you for helping improve the project. Contributions are welcome when they make the package safer, clearer, more reusable, or easier to maintain.

## Start in the right place

Before opening an issue or pull request:

- Report suspected vulnerabilities privately by following [`SECURITY.md`](SECURITY.md).
- Use the security feature-request form for hardening ideas that are safe to discuss publicly.
- Open a normal bug report for reproducible behaviour that does not expose users to a security risk.
- Discuss substantial API changes, new runtime dependencies, or breaking behaviour in an issue before implementation.
- Small documentation corrections and narrowly scoped fixes may go directly to a pull request.

Never place credentials, secrets, production data, personal information, or details of an undisclosed vulnerability in a public issue or pull request.

## Package scope

A change belongs in this package when it solves a reusable problem for multiple consumers rather than encoding the policy or layout of one website.

Strong contributions generally:

- provide secure and predictable defaults
- keep the public API small and explicit
- preserve backwards compatibility where practical
- make failure behaviour clear and safe
- avoid unnecessary runtime dependencies
- work across the project's supported environments
- include tests and documentation for user-visible behaviour

Application-specific composition, branding, analytics, routing, and deployment policy usually belong in the consuming application instead.

## Development setup

Fork and clone the repository, then create a focused branch from the current default branch:

```bash
git switch -c security/short-description
```

Install dependencies using the package manager selected by the repository lockfile. For an npm-based checkout, run:

```bash
npm ci
```

Before submitting a change, run the checks provided by the repository. The usual commands are:

```bash
npm test
npm run lint
npm run typecheck
npm run build
```

If a command is unavailable, use the equivalent script defined in `package.json` and record the commands you ran in the pull request.

## Designing security changes

Security-sensitive changes should begin with a concise threat model:

- What asset or user action is being protected?
- What can the attacker control?
- Where is the relevant trust boundary?
- What property should remain true during failure or attack?
- Is the behaviour secure by default, opt-in, or backwards-compatible?

Prefer simple, auditable designs over clever ones. Validate data at the boundary where trust changes, enforce authorisation at the protected operation, and fail closed when continuing would create a security risk.

Do not introduce custom cryptography. Use established, maintained platform capabilities or libraries, and explain why the selected primitive and configuration are appropriate.

## Dependencies

Avoid adding a runtime dependency when the behaviour can be implemented clearly and safely with the existing stack.

When a dependency is necessary, document:

- why it is required
- why a smaller or existing alternative is insufficient
- its maintenance and release status
- its licence and runtime footprint
- any new transitive dependencies
- the security-sensitive APIs used by the change

Do not bundle copied third-party code merely to avoid declaring a dependency.

## Tests

Every behavioural change should include tests at the level where a regression would be detected most reliably.

For security changes, cover both expected and hostile inputs. Relevant cases may include:

- malformed, missing, duplicated, oversized, or unexpected input
- unauthenticated and unauthorised access
- unsafe defaults and configuration transitions
- escaping, encoding, parsing, or canonicalisation boundaries
- error paths that could leak sensitive values
- race conditions or repeated operations
- backwards-compatibility and migration behaviour

When practical, add a regression test that fails before the fix and passes afterwards. Tests must use synthetic credentials and data.

## Documentation

Update the README, API reference, examples, migration guidance, and security notes whenever users must understand or configure new behaviour.

Documentation should explain:

- the default behaviour
- the security property being provided
- configuration and opt-out consequences
- failure behaviour
- compatibility constraints
- a minimal usage example

Examples must not contain real credentials, identifiers, endpoints, or production data.

## Commits

Keep commits focused and reviewable. Use clear, imperative commit messages, for example:

```text
Validate redirect targets at the trust boundary
Add regression test for token leakage
Document strict policy migration
```

Do not mix unrelated refactoring, formatting, generated files, or dependency updates into the same change unless they are required for the contribution.

## Pull requests

Use the repository pull-request template and include:

- a concise summary
- the reusable package-level use case
- the related public issue or private advisory
- the threat model and security assumptions
- public API or behavioural changes
- compatibility and migration notes
- the exact verification commands and relevant results
- any files, attack paths, or trade-offs requiring special reviewer attention

Keep undisclosed vulnerability details inside the private security-advisory workflow. A public pull request must be safe for immediate disclosure.

Draft pull requests are welcome for early design feedback, but they should still explain the intended behaviour and current limitations. Keep each pull request centred on one coherent change.

## Review expectations

Maintainers review contributions for correctness, security, API design, test quality, compatibility, dependency cost, documentation, and long-term maintenance burden.

A pull request may be asked to change even when its immediate behaviour works. Reviewers may request a smaller API, additional negative tests, clearer failure handling, migration support, or removal of unrelated work.

All required automated checks and requested reviews must pass before merge. Maintainers decide when a contribution is ready and may close proposals that do not fit the package scope.

## Respectful collaboration

Discuss ideas and code directly, respectfully, and with enough evidence for others to reproduce the result. Assume mistakes are possible, avoid personal criticism, and give maintainers and reporters room to coordinate sensitive work privately.

## Licence

By submitting a contribution, you agree that it may be distributed under the repository's existing licence and that you have the right to provide the contributed material.
