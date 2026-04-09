#!/usr/bin/env bash
# setup_btrfs.sh — Install dependencies for the btrfs backend.
#
# btrfs requires no build step — only the userspace tools (mkfs.btrfs,
# btrfs subvolume) and rsync (used by btrfs commit to copy changes back
# to the base volume).

set -euo pipefail

info() { echo -e "\033[1;34m==>\033[0m $*"; }

info "Installing btrfs backend dependencies"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    btrfs-progs \
    rsync
