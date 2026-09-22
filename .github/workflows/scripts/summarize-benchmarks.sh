#!/usr/bin/env bash

set -euo pipefail

if [[ ! -f benchmark-output.txt ]]; then
  exit 0
fi

{
  echo "## Benchmark output"
  echo
  echo '```text'
  tail -n 200 benchmark-output.txt
  echo '```'
} >> "$GITHUB_STEP_SUMMARY"
