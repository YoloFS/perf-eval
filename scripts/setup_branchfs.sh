#!/usr/bin/env bash
# setup_branchfs.sh — Install the `branchfs` backend and its dependencies.
#
# Builds third_party/branchfs from source in a temporary directory (to
# avoid polluting the submodule working tree) and installs the binary to
# /usr/local/bin.

set -euo pipefail

info() { echo -e "\033[1;34m==>\033[0m $*"; }

BENCH_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BRANCHFS_DIR="$BENCH_DIR/third_party/branchfs"

if [ ! -d "$BRANCHFS_DIR" ]; then
    echo "error: $BRANCHFS_DIR not found — initialize the submodule first:" >&2
    echo "       git submodule update --init $BRANCHFS_DIR" >&2
    exit 1
fi

info "Installing branchfs build dependencies"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    fuse3 \
    libfuse3-dev

info "Building branchfs (in a temporary directory)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
cp -a "$BRANCHFS_DIR/." "$tmp"
cargo build --release --manifest-path "$tmp/Cargo.toml"

info "Installing branchfs to /usr/local/bin"
sudo install -m 755 "$tmp/target/release/branchfs" /usr/local/bin/branchfs
