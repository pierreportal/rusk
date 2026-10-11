#!/usr/bin/env bash
#
# Rusk — build a Rust kiosk application for ARM64 Linux.
#
# Two modes:
#   native  Build on an ARM64 Linux host (the Pi itself, or an ARM64 container
#           or VM). This is the supported path in Phase 1.
#   cross   Build from any host using cargo-zigbuild + zig as the linker. Handy
#           from a macOS development machine, but the produced binary must still
#           be verified on the reference hardware.
#
# Usage:
#   ./scripts/build-arm64.sh [--native|--cross] [--example NAME]
#                            [--manifest PATH] [--bin NAME] [--debug]
#                            [--target TRIPLE]
#
# --example selects a bundled example and its manifest/binary:
#   gui | tauri     Tauri 2 GUI example       (examples/tauri-app)
#   tui | ratatui   Ratatui TUI example       (examples/ratatui-app)
#
# Defaults: mode auto (native on aarch64, cross otherwise), example gui.
#
# Reference: PLAN.md §8, §12.

set -euo pipefail

TARGET="aarch64-unknown-linux-gnu"
MODE="auto"
EXAMPLE=""
MANIFEST=""
BIN_NAME=""
CARGO_PROFILE="--release"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

usage() { sed -n '2,24p' "$0" | sed 's/^# \{0,1\}//'; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --native)   MODE="native"; shift ;;
        --cross)    MODE="cross"; shift ;;
        --example)  EXAMPLE="${2:?--example needs a value}"; shift 2 ;;
        --manifest) MANIFEST="${2:?--manifest needs a value}"; shift 2 ;;
        --bin)      BIN_NAME="${2:?--bin needs a value}"; shift 2 ;;
        --debug)    CARGO_PROFILE=""; shift ;;
        --target)   TARGET="${2:?--target needs a value}"; shift 2 ;;
        -h|--help)  usage; exit 0 ;;
        *)          die "unknown argument: $1 (try --help)" ;;
    esac
done

# Resolve the example's manifest and binary name unless overridden.
case "${EXAMPLE:-gui}" in
    gui|tauri)
        MANIFEST="${MANIFEST:-examples/tauri-app/src-tauri/Cargo.toml}"
        BIN_NAME="${BIN_NAME:-rusk-example}" ;;
    tui|ratatui)
        MANIFEST="${MANIFEST:-examples/ratatui-app/Cargo.toml}"
        BIN_NAME="${BIN_NAME:-rusk-tui-example}" ;;
    "")
        MANIFEST="${MANIFEST:?specify --example or --manifest}"
        BIN_NAME="${BIN_NAME:?specify --bin when using --manifest}" ;;
    *)  die "--example must be one of: gui, tauri, tui, ratatui (got '$EXAMPLE')" ;;
esac
: "${BIN_NAME:?specify --bin when using --manifest}"
[[ -f "$MANIFEST" ]] || die "manifest not found: $MANIFEST"

# Resolve a mode when left on auto.
if [[ "$MODE" == "auto" ]]; then
    if [[ "$(uname -s)" == "Linux" && "$(uname -m)" == "aarch64" ]]; then
        MODE="native"
    else
        MODE="cross"
    fi
fi

PROFILE_DIR="release"; [[ -z "$CARGO_PROFILE" ]] && PROFILE_DIR="debug"
MANIFEST_DIR="$(cd "$(dirname "$MANIFEST")" && pwd)"

if [[ "$MODE" == "native" ]]; then
    [[ "$(uname -m)" == "aarch64" ]] \
        || die "native mode must run on aarch64 (use --cross or build on the Pi)"
    log "Building natively ($(uname -m))"
    # shellcheck disable=SC2086
    cargo build --manifest-path "$MANIFEST" $CARGO_PROFILE
    BIN_PATH="$MANIFEST_DIR/target/$PROFILE_DIR/$BIN_NAME"
else
    log "Cross-compiling to $TARGET"
    command -v cargo-zigbuild >/dev/null \
        || die "cargo-zigbuild not found. Install it with:
    cargo install cargo-zigbuild
  and install zig (https://ziglang.org/download/ or 'brew install zig')."
    command -v zig >/dev/null \
        || die "zig not found. Install zig (https://ziglang.org/download/ or 'brew install zig')."

    rustup target add "$TARGET" >/dev/null 2>&1 || true
    # shellcheck disable=SC2086
    cargo zigbuild --manifest-path "$MANIFEST" $CARGO_PROFILE --target "$TARGET"
    BIN_PATH="$MANIFEST_DIR/target/$TARGET/$PROFILE_DIR/$BIN_NAME"
fi

[[ -f "$BIN_PATH" ]] || die "build finished but binary not found at $BIN_PATH"
file "$BIN_PATH" | grep -qi 'aarch64\|ARM aarch64' \
    || printf '\033[1;33mwarning:\033[0m binary does not look like aarch64: %s\n' "$(file "$BIN_PATH")"

log "Built: $BIN_PATH"
echo "$BIN_PATH"
