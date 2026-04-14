#!/usr/bin/env bash
# rerender.sh — Regenerate HTML reports from existing results.

set -euo pipefail

cd "$(dirname "$0")"
cargo run --release -- rerender "$@"
