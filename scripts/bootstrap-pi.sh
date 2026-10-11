#!/usr/bin/env bash
#
# Rusk — Phase 1 reference platform bootstrap.
#
# Turns a clean Raspberry Pi OS Lite (64-bit) installation into a machine that
# can run a Rust kiosk application. The packages installed depend on the chosen
# runtime (`ui`):
#
#   gui   Wayland via Cage + WebKitGTK/GTK runtime (Tauri, winit, SDL)
#   tui   Terminal application on a Linux virtual console (ratatui/crossterm);
#         no compositor and no WebKitGTK
#   both  Install both runtime sets
#
# This script is a direct encoding of docs/installation.md so the procedure
# stays reproducible, and it is the eventual acceptance test for
# `kiosk install` (Phase 3). It is idempotent: running it twice is safe.
#
# Usage:
#   sudo ./scripts/bootstrap-pi.sh [--ui gui|tui|both] [--build-deps]
#                                  [--kiosk-user NAME]
#
# Options:
#   --ui KIND           Runtime(s) to prepare. Default: gui
#   --build-deps        Also install the native build toolchain (dev packages).
#                       Needed only when compiling on the Pi itself.
#   --kiosk-user NAME   Unprivileged user that owns the session. Default: kiosk
#   -h, --help          Show this help.
#
# Reference: PLAN.md §8 (Phase 1).

set -euo pipefail

KIOSK_USER="kiosk"
UI="gui"
BUILD_DEPS=0

# Runtime packages for a GUI kiosk: the compositor, the GL stack, and the
# WebKitGTK/GTK libraries a Tauri app links against at run time.
GUI_RUNTIME_PACKAGES=(
    cage
    libgl1-mesa-dri
    libegl-mesa0
    libgles2
    mesa-utils
    fonts-dejavu-core
    libwebkit2gtk-4.1-0
    libgtk-3-0
    libayatana-appindicator3-1
    librsvg2-2
    libxdo3
    libssl3
)

# A TUI kiosk needs no runtime packages beyond a working Linux console, which
# Raspberry Pi OS Lite already provides. Listed explicitly so future additions
# (for example a richer console such as kmscon) have an obvious home.
TUI_RUNTIME_PACKAGES=()

# Native build dependencies. GUI builds additionally need the WebKitGTK/GTK dev
# headers; TUI builds need only a C toolchain.
COMMON_BUILD_PACKAGES=(build-essential pkg-config git curl)
GUI_BUILD_PACKAGES=(
    wget
    file
    libwebkit2gtk-4.1-dev
    libgtk-3-dev
    libayatana-appindicator3-dev
    librsvg2-dev
    libxdo-dev
    libssl-dev
)

log()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

usage() { sed -n '2,24p' "$0" | sed 's/^# \{0,1\}//'; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ui)          UI="${2:?--ui needs a value}"; shift 2 ;;
        --build-deps)  BUILD_DEPS=1; shift ;;
        --kiosk-user)  KIOSK_USER="${2:?--kiosk-user needs a value}"; shift 2 ;;
        -h|--help)     usage; exit 0 ;;
        *)             die "unknown argument: $1 (try --help)" ;;
    esac
done

case "$UI" in
    gui|tui|both) ;;
    *) die "--ui must be one of: gui, tui, both (got '$UI')" ;;
esac

[[ "$(id -u)" -eq 0 ]] || die "must run as root (use: sudo $0)"
[[ "$(uname -m)" == "aarch64" ]] || die "expected a 64-bit ARM host, found $(uname -m)"

if [[ -r /etc/os-release ]]; then
    # shellcheck disable=SC1091
    . /etc/os-release
    if [[ "${ID:-}" != "raspbian" && "${ID:-}" != "debian" ]]; then
        warn "unexpected OS '${ID:-unknown}'; this script targets Raspberry Pi OS Lite"
    fi
    log "OS: ${PRETTY_NAME:-unknown}"
fi

# Assemble the package sets for the requested runtime(s).
RUNTIME_PACKAGES=()
BUILD_PACKAGES=("${COMMON_BUILD_PACKAGES[@]}")
if [[ "$UI" == "gui" || "$UI" == "both" ]]; then
    RUNTIME_PACKAGES+=("${GUI_RUNTIME_PACKAGES[@]}")
    if [[ "$BUILD_DEPS" -eq 1 ]]; then
        BUILD_PACKAGES+=("${GUI_BUILD_PACKAGES[@]}")
    fi
fi
if [[ "$UI" == "tui" || "$UI" == "both" ]]; then
    RUNTIME_PACKAGES+=("${TUI_RUNTIME_PACKAGES[@]}")
fi

log "Updating package lists"
export DEBIAN_FRONTEND=noninteractive
apt-get update -y

if [[ "${#RUNTIME_PACKAGES[@]}" -gt 0 ]]; then
    log "Installing runtime packages for ui=$UI (${#RUNTIME_PACKAGES[@]})"
    apt-get install -y --no-install-recommends "${RUNTIME_PACKAGES[@]}"
else
    log "No runtime packages required for ui=$UI"
fi

if [[ "$BUILD_DEPS" -eq 1 ]]; then
    log "Installing native build dependencies (${#BUILD_PACKAGES[@]})"
    apt-get install -y --no-install-recommends "${BUILD_PACKAGES[@]}"
fi

log "Ensuring kiosk user '$KIOSK_USER' exists"
if ! id "$KIOSK_USER" >/dev/null 2>&1; then
    adduser --disabled-password --gecos "Rusk kiosk user" "$KIOSK_USER"
    # Allow key-based remote administration for the kiosk user without a password.
    install -d -m 0755 -o "$KIOSK_USER" -g "$KIOSK_USER" "/home/$KIOSK_USER/.ssh"
fi

# Grant only the device access the session needs. video/render/input/audio are
# used by the GUI compositor; the TUI only needs console access, but the extra
# groups are harmless and keep one user model for both runtimes.
for grp in video render input audio plugdev; do
    if getent group "$grp" >/dev/null; then
        usermod -aG "$grp" "$KIOSK_USER"
    fi
done
log "Groups for '$KIOSK_USER': $(id -nG "$KIOSK_USER")"

log "Ensuring SSH is enabled for remote administration"
systemctl enable --now ssh 2>/dev/null || systemctl enable --now sshd 2>/dev/null \
    || warn "could not enable SSH; verify manually"

# Record the exact baseline so later phases can report and reproduce it.
log "Recording reference platform details"
install -d -m 0755 /etc/rusk
{
    echo "recorded_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "hostname=$(hostname)"
    echo "arch=$(uname -m)"
    echo "kernel=$(uname -r)"
    echo "ui=$UI"
    [[ -r /etc/os-release ]] && grep -E '^(PRETTY_NAME|VERSION_ID)=' /etc/os-release
    echo "cage=$(dpkg-query -W -f='${Version}' cage 2>/dev/null || echo not-installed)"
} > /etc/rusk/reference-platform.txt
cat /etc/rusk/reference-platform.txt

log "Bootstrap complete (ui=$UI)"
cat <<EOF

Next steps (see docs/installation.md):
  1. Build an example:   ./scripts/build-arm64.sh --native --example $([[ "$UI" == "tui" ]] && echo tui || echo gui)
  2. Run it manually:
       GUI:  cage -- examples/tauri-app/src-tauri/target/release/rusk-example
       TUI:  examples/ratatui-app/target/release/rusk-tui-example
  3. Verify rendering, input, and that the Rust backend responds.

Do not configure automatic startup until the app works reliably by hand.
EOF
