#!/usr/bin/env bash

set -euo pipefail

for package in \
  cp-cli-platform-codechef \
  cp-cli-platform-codeforces \
  cp-cli-platform-hackerearth \
  cp-cli-platform-hackerrank \
  cp-cli-platform-leetcode \
  cp-cli-platform-project-euler \
  cp-cli; do
  status="$(curl --silent --show-error --output /dev/null --write-out '%{http_code}' \
    --user-agent "cp-cli-release/${VERSION} (https://github.com/${GITHUB_REPOSITORY})" \
    "https://crates.io/api/v1/crates/${package}/${VERSION}")"
  case "$status" in
    200)
      echo "${package} ${VERSION} is already published"
      ;;
    404)
      cargo publish --locked --package "$package"
      ;;
    *)
      echo "crates.io returned HTTP ${status} for ${package} ${VERSION}" >&2
      exit 1
      ;;
  esac
done
