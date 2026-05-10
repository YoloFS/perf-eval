#!/usr/bin/env bash
# macro.sh — Build, install YoloFS, and run macrobenchmarks.
set -euo pipefail
cd "$(dirname "$0")"
make -C ../filesystem install
yolo reload
cargo run --release -- --macro
yolo unload
