#!/bin/bash
# Local wrapper: rsync the working tree to `big`, then run cargo there.
# Usage: ./tools/big-cargo.sh [--no-sync] <cargo args...>
#
# Compilation for this crate happens on `big` (24 cores, Linux) rather than the laptop; see
# docs/DEMOS.md "Building on big". The target dir lives on big, one per tree, so repeat runs are warm.
set -euo pipefail

REMOTE_DIR="${BS_REMOTE_DIR:-bs-build}"
REMOTE_TARGET="${BS_REMOTE_TARGET:-bs-target}"
TOOLCHAIN="${BS_TOOLCHAIN:-1.98.1}"
LOCAL_DIR="$(cd "$(dirname "$0")/.." && pwd)"

SYNC=1
if [ "${1:-}" = "--no-sync" ]; then SYNC=0; shift; fi

if [ "$SYNC" = 1 ]; then
    rsync -az --delete \
        --exclude .git --exclude target --exclude 'docs/media/*.gif' --exclude frames \
        "$LOCAL_DIR"/ "big:\$HOME/$REMOTE_DIR/"
fi

# Single-quoted so `$HOME` expands on big, not here.
ssh big "bash -lc 'cd \$HOME/$REMOTE_DIR && RUSTUP_TOOLCHAIN=$TOOLCHAIN CARGO_TARGET_DIR=\$HOME/$REMOTE_TARGET nice -n 5 cargo $*'"
