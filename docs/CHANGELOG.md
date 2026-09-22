# Changelog

This ledger reconstructs cp-cli's release milestones from the available source history and
product record. Releases before `3.1.8` are metadata-only historical markers: they describe
when a capability entered the product, but they do not claim that a matching binary or crate
was published at that time. `3.1.8` is the first release built from the current release
pipeline.

The project version advances by one minor step for each command or platform addition and by
one patch step for each distinct correction. Minor steps use the project's decimal sequence,
so `0.9.0` is followed by `1.0.0`.

| Version | Retrospective change |
| --- | --- |
| `0.1.0` | Established the LeetCode `problem show` baseline |
| `0.1.1` | Added usable terminal text scaling |
| `0.1.2` | Removed excessive problem-statement spacing |
| `0.1.3` | Rebalanced terminal colors for sustained reading |
| `0.1.4` | Replaced block art with the raster possum renderer |
| `0.1.5` | Kept possum motion active beyond opening and closing |
| `0.1.6` | Added visual hierarchy to the problem body |
| `0.2.0` | Added `problem list` |
| `0.3.0` | Added `problem daily` |
| `0.4.0` | Added `problem search` |
| `0.5.0` | Added `problem pick` |
| `0.6.0` | Added `auth login` |
| `0.7.0` | Added `auth logout` |
| `0.8.0` | Added `auth status` |
| `0.8.1` | Removed the Safari automation dependency from login |
| `0.8.2` | Reused the person's normal browser session for authentication |
| `0.8.3` | Restored passkey and clipboard support during login |
| `0.8.4` | Removed duplicate keychain prompts |
| `0.9.0` | Added `problem test` |
| `0.9.1` | Accepted complete LeetCode problem data across response variants |
| `0.9.2` | Corrected test-run identifier decoding |
| `1.0.0` | Added `problem submit` |
| `1.0.1` | Corrected submission JSON decoding |
| `1.0.2` | Matched the embedded reader to table density |
| `1.0.3` | Placed the problem identity beside the possum |
| `1.0.4` | Restored rendering and the 60-percent table width |
| `1.0.5` | Restored possum blink and itch motion |
| `1.0.6` | Removed forced uppercase and excessive title tracking |
| `1.1.0` | Added `stats` |
| `1.1.1` | Accepted numeric fields in account statistics |
| `1.2.0` | Added `contest list` |
| `1.3.0` | Added `contest show` |
| `1.4.0` | Added `contest create` |
| `1.5.0` | Added `contest edit` |
| `1.6.0` | Added `contest delete` |
| `1.7.0` | Added `contest join` |
| `1.8.0` | Added `contest leave` |
| `1.9.0` | Added `contest status` |
| `2.0.0` | Added `discussion list` |
| `2.0.1` | Corrected problem-specific discussion requests |
| `2.1.0` | Added `discussion show` |
| `2.1.1` | Rendered discussion bodies instead of schema markers |
| `2.2.0` | Added `discussion create` |
| `2.3.0` | Added `discussion edit` |
| `2.4.0` | Added `discussion delete` |
| `2.5.0` | Added `discussion reply` |
| `2.6.0` | Added HackerRank |
| `2.7.0` | Added Codeforces |
| `2.8.0` | Added Exercism |
| `2.9.0` | Added Project Euler |
| `3.0.0` | Added HackerEarth |
| `3.1.0` | Added CodeChef |
| `3.1.1` | Removed the unsafe `workflow_run` automation path |
| `3.1.2` | Removed the unsafe `pull_request_target` automation path |
| `3.1.3` | Isolated release credentials behind the release environment |
| `3.1.4` | Added a seven-day Dependabot cooldown |
| `3.1.5` | Added native installers for supported desktop targets |
| `3.1.6` | Added release checksum verification and provenance |
| `3.1.7` | Added native Windows installer coverage |
| `3.1.8` | Unified repository governance, tooling layout, and canonical release URLs |
