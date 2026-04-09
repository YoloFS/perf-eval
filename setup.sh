#!/usr/bin/env bash
# setup.sh — Install generic dependencies for yolo-bench.
#
# For backend-specific setup, see scripts/setup_{try,branchfs,btrfs}.sh.

set -euo pipefail

info() { echo -e "\033[1;34m==>\033[0m $*"; }

info "Installing generic benchmark dependencies"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    fio \
    "linux-tools-$(uname -r)" \
    bpftrace
