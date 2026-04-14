#!/usr/bin/env bash
# run.sh — Build and run yolo-bench workloads.

set -euo pipefail

BENCH_DIR="$(cd "$(dirname "$0")" && pwd)"
AGFS_ROOT="$(cd "$BENCH_DIR/.." && pwd)"

mode="${1:-}"
case "$mode" in
    "")
        bench_flags=(--micro --macro)
        ;;
    micro)
        bench_flags=(--micro)
        ;;
    macro)
        bench_flags=(--macro)
        ;;
    *)
        echo "usage: $0 [micro|macro]" >&2
        exit 1
        ;;
esac

make -C "$AGFS_ROOT" install
yolo reload

for bench_flag in "${bench_flags[@]}"; do
    cargo run --release --manifest-path "$BENCH_DIR/Cargo.toml" -- "$bench_flag"
done

yolo unload
