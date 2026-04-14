#!/usr/bin/env bash
# run.sh — Build and run yolo-bench workloads.

set -euo pipefail

cd "$(dirname "$0")"

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

make -C .. install
yolo reload

for bench_flag in "${bench_flags[@]}"; do
    cargo run --release -- "$bench_flag"
done

yolo unload
