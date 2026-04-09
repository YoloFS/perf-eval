#!/usr/bin/env bash
# setup_try.sh — Install the `try` backend and its dependencies.
#
# Builds third_party/try from source (autotools) and installs it to
# /usr/local/bin. Also configures kernel settings that `try` requires
# (overlayfs + unprivileged user namespaces via apparmor).

set -euo pipefail

info() { echo -e "\033[1;34m==>\033[0m $*"; }

BENCH_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TRY_DIR="$BENCH_DIR/third_party/try"

if [ ! -d "$TRY_DIR" ]; then
    echo "error: $TRY_DIR not found — initialize the submodule first:" >&2
    echo "       git submodule update --init $TRY_DIR" >&2
    exit 1
fi

info "Installing try build dependencies"
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    autoconf \
    attr \
    pandoc

info "Configuring kernel settings (overlayfs + unprivileged userns)"
sudo modprobe overlay
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0

info "Building try"
cd "$TRY_DIR"
[ -f configure ] || autoconf
[ -f Makefile ] || ./configure --prefix=/usr/local
make

info "Installing try to /usr/local/bin"
sudo make install
