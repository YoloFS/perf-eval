#!/usr/bin/env bash
# paper.sh — Generate paper artifacts from existing results.

set -euo pipefail

cd "$(dirname "$0")"
cargo run --release -- paper "$@"
./plot.py
