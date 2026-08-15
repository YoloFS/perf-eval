#!/usr/bin/env bash
# macro.sh — Build, install YoloFS, and run macrobenchmarks.
set -euo pipefail
cd "$(dirname "$0")"
make -C ../filesystem install
yolo reload
# Unload on any exit path (failure, Ctrl-C) so a wedged mount can't outlive the run.
trap 'yolo unload || true' EXIT
cargo run --release -- --macro
