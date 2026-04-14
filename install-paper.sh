#!/usr/bin/env bash
# install-paper.sh — Install paper artifacts.

set -euo pipefail

cd "$(dirname "$0")"
cargo run --release -- install-paper "$@"
