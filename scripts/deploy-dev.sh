#!/usr/bin/env bash
#
# Rusk — copy a built binary to a Raspberry Pi for manual testing (Phase 1).
#
# This is a convenience script, NOT the production deployment mechanism. Formal
# releases, staging, atomic activation, and rollback arrive with `kiosk deploy`
# in Phase 5.
#
# Usage:
#   ./scripts/deploy-dev.sh <user@host> <binary> [remote_dir]
#
# Example:
#   ./scripts/deploy-dev.sh kiosk@kiosk.local \
#       examples/ratatui-app/target/release/rusk-tui-example
#
# Prints the exact command to run on the Pi afterwards.

set -euo pipefail

die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }
log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }

[[ $# -ge 2 ]] || die "usage: $0 <user@host> <binary> [remote_dir]"

HOST="$1"
BINARY="$2"
REMOTE_DIR="${3:-/home/kiosk/rusk-bin}"

[[ -f "$BINARY" ]] || die "binary not found: $BINARY"
chmod +x "$BINARY" 2>/dev/null || true

REMOTE_NAME="$(basename "$BINARY")"

log "Ensuring $REMOTE_DIR exists on $HOST"
ssh "$HOST" "mkdir -p '$REMOTE_DIR'"

log "Copying $BINARY -> $HOST:$REMOTE_DIR/$REMOTE_NAME"
scp "$BINARY" "$HOST:$REMOTE_DIR/$REMOTE_NAME"
ssh "$HOST" "chmod +x '$REMOTE_DIR/$REMOTE_NAME'"

log "Copied. Run it on the Pi:"
cat <<EOF
  GUI:  ssh $HOST 'cage -- $REMOTE_DIR/$REMOTE_NAME'
  TUI:  ssh $HOST '$REMOTE_DIR/$REMOTE_NAME'    # run from the physical console
EOF
