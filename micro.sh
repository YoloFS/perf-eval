#!/usr/bin/env bash
# micro.sh — Build, install YoloFS, and run microbenchmarks.
set -euo pipefail
cd "$(dirname "$0")"
make -C ../filesystem install
yolo reload
# Unload on any exit path (failure, Ctrl-C) so a wedged mount can't outlive the run.
trap 'yolo unload || true' EXIT
cargo run --release -- --micro
