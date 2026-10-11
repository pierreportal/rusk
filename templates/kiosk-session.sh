#!/usr/bin/env bash
#
# Rusk graphical session launcher.
#
# TEMPLATE — deployed by `kiosk install` in Phase 2. Kept here so the reference
# procedure and the generated unit stay in sync.
#
# Invoked by kiosk-session.service as the unprivileged kiosk user on a local
# TTY. It must `exec` the compositor so systemd tracks a single process and can
# observe the real exit status.
set -euo pipefail

# Stable path to the application. When release management is in use this points
# at a symlink under /opt/kiosk/current, never at a versioned release directory.
APP="${RUSK_APP:-/opt/kiosk/current/bin/my-app}"

if [[ ! -x "$APP" ]]; then
    echo "rusk: application is not executable: $APP" >&2
    exit 1
fi

# Cage is the kiosk compositor. Everything after `--` becomes the client
# command line; no shell interpolation is performed here on purpose.
exec cage -- "$APP"
