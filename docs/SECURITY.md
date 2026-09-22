# Security Policy

We welcome responsible reports that protect users without creating unnecessary risk.

## Supported versions

Security fixes cover the latest published release. `main` may contain unreleased work and is not a supported production release. Older releases are fixed only when an advisory says so.

## Reporting a vulnerability

Keep suspected vulnerabilities out of public issues, pull requests, discussions, commit messages, and social-media posts. Report them privately through GitHub:

1. Open the repository's **Security** tab.
2. Select **Report a vulnerability**.
3. Submit the report privately.

Maintainers must enable private vulnerability reporting before publishing this policy. If it is unavailable, ask a maintainer through a verified private channel where to report; leave vulnerability details out of any public message.

### What to include

A useful report includes:

- the affected version, commit, component, or public API
- the security impact and the asset at risk
- the attacker's required access or capabilities
- clear reproduction steps or a minimal proof of concept
- any relevant configuration, platform, or runtime details
- suggested mitigations, when known
- whether the issue has been disclosed anywhere else

Use synthetic data where possible. Never include credentials, tokens, private keys, production data, or someone else's personal information.

## What happens after a report

We aim to:

- acknowledge receipt within five business days
- confirm the initial severity and next steps within ten business days
- keep the reporter informed when material progress occurs
- prepare a fix, regression tests, release notes, and an advisory when appropriate
- coordinate disclosure after affected users have had a reasonable opportunity to update

Timing depends on complexity, dependencies, and maintainer availability. We may need more information or a working reproduction before confirming an issue.

We may credit reporters publicly unless they prefer anonymity. There is no bug-bounty programme, and we cannot promise payment, rewards, or a CVE assignment.

## Coordinated disclosure

Keep unpatched vulnerabilities confidential until maintainers confirm coordinated disclosure is appropriate.

Use the private security-advisory workflow whenever public development would expose the issue. Before disclosure, keep sensitive details out of public branches, issues, pull requests, test fixtures, logs, and release notes.

After a fix is available, the project may publish an advisory covering affected versions, impact, mitigations, the fixed version, and reporter credit.

## Public security requests

Use a public security feature request only for improvements that are safe to discuss, including:

- safer defaults
- stronger validation or sanitisation
- privacy-preserving behaviour
- dependency or supply-chain hardening
- improved security documentation
- defence-in-depth changes without an undisclosed exploit

If unsure, report privately first. Maintainers can move a non-sensitive request public later.

## Scope

Reports about this project's source code, published packages, release artifacts, supported integrations, and project-controlled infrastructure are in scope when they show a concrete security impact.

The following are generally not treated as vulnerabilities on their own:

- feature requests or general hardening suggestions without a security impact
- reports affecting only unsupported or modified versions
- automated dependency alerts without evidence that this project is exploitable
- missing security headers on documentation or demo sites without a meaningful attack path
- social engineering, phishing, denial-of-service testing, or physical attacks
- findings that require access to another person's account, data, or device without permission

A report outside scope may still be useful, but may be handled as a normal bug or feature request.

## Good-faith research

We support good-faith research. Keep testing proportionate; use accounts and data you control; avoid disruption; collect only the evidence you need; and stop if you encounter sensitive information.

This policy does not authorise access to third-party systems or data, destructive testing, privacy violations, extortion, or illegal conduct. When researchers follow this policy in good faith, the project will not recommend or pursue legal action solely for that research.
