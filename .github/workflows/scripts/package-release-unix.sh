#!/usr/bin/env bash

set -euo pipefail

archive="cp-cli-${TARGET}.tar.gz"
mkdir -p dist/package dist/smoke
cp "target/${TARGET}/release/${EXECUTABLE}" "dist/package/${EXECUTABLE}"
cp README.md LICENSE dist/package/
tar -C dist/package -czf "dist/${archive}" "$EXECUTABLE" README.md LICENSE
tar -xzf "dist/${archive}" -C dist/smoke
test "$(dist/smoke/cp-cli --version)" = "cp-cli $VERSION"
echo "path=dist/${archive}" >> "$GITHUB_OUTPUT"
