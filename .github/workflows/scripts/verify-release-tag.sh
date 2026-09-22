#!/usr/bin/env bash

set -euo pipefail

metadata="$(cargo metadata --locked --no-deps --format-version 1)"
version="$(jq -er '.packages[] | select(.name == "cp-cli") | .version' <<<"$metadata")"

for package in \
  cp-cli-platform-codechef \
  cp-cli-platform-codeforces \
  cp-cli-platform-hackerearth \
  cp-cli-platform-hackerrank \
  cp-cli-platform-leetcode \
  cp-cli-platform-project-euler; do
  package_version="$(jq -er --arg package "$package" \
    '.packages[] | select(.name == $package) | .version' <<<"$metadata")"
  test "$package_version" = "$version"
done

if [[ "$TAG" != "v$version" ]]; then
  echo "release tag ${TAG} does not match workspace version ${version}" >&2
  exit 1
fi

echo "value=$version" >> "$GITHUB_OUTPUT"
