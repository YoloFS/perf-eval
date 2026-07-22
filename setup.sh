#!/usr/bin/env bash
# setup.sh — Install generic dependencies for yolo-bench.
#
# For backend-specific setup, see scripts/setup_branchfs.sh.

set -euo pipefail
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    fio \
    "linux-tools-$(uname -r)" \
    bpftrace
