# Required repository settings

These settings are configured in GitHub and cannot be enforced by a tracked file.

## Main branch protection

Apply the following rules to `main`:

- Require a pull request before merging
- Require one approving review and dismiss stale approvals when new commits are pushed
- Require review from Code Owners
- Require all review conversations to be resolved
- Require the `Required checks` status check before merging
- Require branches to be up to date before merging
- Block force pushes and branch deletion
- Include administrators

## Security and maintenance

- Enable private vulnerability reporting and use the repository's security advisory form
- Enable Dependabot version and security updates
- Enable secret scanning and push protection where available
- Register the repository with the Core Infrastructure Initiative Best Practices badge program before adding its badge

## Release publishing

The release workflow uses crates.io trusted publishing through OpenID Connect, so it does not need a long-lived crates.io credential. It builds and attests the native archives, then uploads one verified release bundle as a workflow artifact. It has read-only repository contents permission and never creates a GitHub Release.

A maintainer creates the GitHub Release directly and attaches the verified bundle after the workflow succeeds. Retrospective tags through `v3.1.7` are excluded from the workflow and remain metadata-only releases without binaries.

Before the first automated release, publish the workspace crates manually in this order:

1. `cp-cli-platform-codechef`
2. `cp-cli-platform-codeforces`
3. `cp-cli-platform-hackerearth`
4. `cp-cli-platform-hackerrank`
5. `cp-cli-platform-leetcode`
6. `cp-cli-platform-project-euler`
7. `cp-cli`

For each crate, configure a crates.io trusted publisher with owner `keys-i`, repository `cp-cli`, workflow `release.yml`, and environment `release`.

Protect the GitHub `release` environment with a required reviewer and a deployment rule that permits version tags only.
