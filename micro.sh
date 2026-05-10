#!/usr/bin/env bash
# micro.sh — Build, install YoloFS, and run microbenchmarks.
set -euo pipefail
cd "$(dirname "$0")"
make -C ../filesystem install
yolo reload
cargo run --release -- --micro
yolo unload
