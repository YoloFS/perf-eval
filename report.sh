#!/usr/bin/env bash
# report.sh — Generate HTML reports from existing results.

set -euo pipefail

cd "$(dirname "$0")"
cargo run --release -- report "$@"
