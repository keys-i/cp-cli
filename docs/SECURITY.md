# Security Policy

Security is part of this project's public contract. We welcome responsible reports that help protect users without exposing them to unnecessary risk.

## Supported versions

Security fixes are provided for the latest published release.

The `main` branch may contain unreleased changes and should not be treated as a supported production release. Older releases receive fixes only when a security advisory explicitly says otherwise.

## Reporting a vulnerability

Do not report a suspected vulnerability in a public issue, pull request, discussion, commit message, or social-media post.

Use GitHub's private vulnerability reporting instead:

1. Open the repository's **Security** tab.
2. Select **Report a vulnerability**.
3. Submit the report privately.

Maintainers should enable private vulnerability reporting before publishing this policy. If the button is unavailable, contact a maintainer through a verified private channel and ask where to send the report. Do not include vulnerability details in the initial public message.

### What to include

A useful report contains:

- the affected version, commit, component, or public API
- the security impact and the asset at risk
- the attacker's required access or capabilities
- clear reproduction steps or a minimal proof of concept
- any relevant configuration, platform, or runtime details
- suggested mitigations, when known
- whether the issue has been disclosed anywhere else

Use synthetic data wherever possible. Never include credentials, access tokens, private keys, production data, or another person's personal information.

## What happens after a report

We aim to:

- acknowledge receipt within five business days
- confirm the initial severity and next steps within ten business days
- keep the reporter informed when material progress occurs
- prepare a fix, regression tests, release notes, and an advisory when appropriate
- coordinate disclosure after affected users have had a reasonable opportunity to update

Timelines depend on complexity, affected dependencies, and maintainer availability. We may ask for additional information or a working reproduction before confirming the issue.

Reporters may receive public credit unless they prefer to remain anonymous. This project does not currently operate a bug-bounty programme and cannot promise payment, rewards, or a CVE assignment.

## Coordinated disclosure

Please keep an unpatched vulnerability confidential until the maintainers confirm that coordinated disclosure is appropriate.

Security fixes should be developed through the private security-advisory workflow when public development would reveal the vulnerability. Do not copy sensitive advisory details into public branches, issues, pull requests, test fixtures, logs, or release notes before disclosure.

Once a fix is available, the project may publish an advisory describing the affected versions, impact, mitigations, fixed version, and reporter credit.

## Public security requests

Use a public security feature request for improvements that are safe to discuss openly, such as:

- safer defaults
- stronger validation or sanitisation
- privacy-preserving behaviour
- dependency or supply-chain hardening
- improved security documentation
- defence-in-depth changes without an undisclosed exploit

When uncertain, report privately first. Maintainers can move a non-sensitive idea into the public tracker later.

## Scope

Reports concerning this project's source code, published packages, release artifacts, supported integrations, and project-controlled infrastructure are in scope when they demonstrate a concrete security impact.

The following are generally not treated as vulnerabilities on their own:

- feature requests or general hardening suggestions without a security impact
- reports affecting only unsupported or modified versions
- automated dependency alerts without evidence that this project is exploitable
- missing security headers on documentation or demo sites without a meaningful attack path
- social engineering, phishing, denial-of-service testing, or physical attacks
- findings that require access to another person's account, data, or device without permission

A report may still be valuable even when it falls outside this scope, but it may be handled as a normal bug or feature request.

## Good-faith research

We support research performed in good faith. Keep testing proportionate, use accounts and data you control, avoid service disruption, collect only the minimum evidence needed, and stop if you encounter sensitive information.

This policy does not authorise access to third-party systems or data, destructive testing, privacy violations, extortion, or conduct prohibited by applicable law. When a researcher follows this policy and acts in good faith, the project will not recommend or pursue legal action solely because of the security research.
