#!/usr/bin/env bash

set -euo pipefail

required_results=(
  "$METADATA_RESULT"
  "$QUALITY_RESULT"
  "$TESTS_RESULT"
  "$DOCTESTS_RESULT"
  "$PACKAGE_RESULT"
  "$COVERAGE_RESULT"
  "$BENCH_COMPILE_RESULT"
)

for result in "${required_results[@]}"; do
  if [[ "$result" != "success" ]]; then
    echo "A required job finished with result: $result" >&2
    exit 1
  fi
done

if [[ "$MSRV_RESULT" != "success" && "$MSRV_RESULT" != "skipped" ]]; then
  echo "The MSRV job finished with result: $MSRV_RESULT" >&2
  exit 1
fi
