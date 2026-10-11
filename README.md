# Rusk — Rust Kiosk

A reusable runtime for turning a Raspberry Pi into a reliable, single-purpose
graphical device that boots straight into one application.

The goal is to make deploying a graphical Rust application to a Raspberry Pi
almost as straightforward as running a local command — without maintaining a
full desktop environment.

## Status

Early development. This repository currently implements the **Phase 1 reference
platform** ([`PLAN.md`](./PLAN.md)):

- A bootstrap script that turns a clean Raspberry Pi OS Lite 64-bit
  installation into a machine that can run a Rust kiosk application.
- Two runtimes: a **GUI** path (Tauri 2 fullscreen under Wayland/Cage) and a
  **TUI** path (ratatui on a virtual console, with no desktop stack).
- A documented, reproducible installation procedure.

The `kiosk` CLI is scaffolded only; its commands are not implemented yet
(Phase 3). See `PLAN.md` for the full roadmap.

## Reference platform

| Component   | Choice                          |
| ----------- | ------------------------------- |
| Hardware    | Raspberry Pi 4                  |
| OS          | Raspberry Pi OS Lite, 64-bit    |
| Graphics    | Wayland via Cage (kiosk shell)  |
| App runtime | Tauri 2 (Rust backend)          |
| Supervision | systemd                         |

## Quick start

On a clean Raspberry Pi OS Lite 64-bit installation:

```bash
git clone <this-repo> rusk && cd rusk

# Install the runtime stack (GUI) or the terminal stack (TUI):
sudo ./scripts/bootstrap-pi.sh --ui gui --build-deps     # or --ui tui

# Build the GUI example natively on the Pi (it prints the binary path):
BIN=$(./scripts/build-arm64.sh --example gui --native | tail -1)

# Run it fullscreen on the attached display:
cage -- "$BIN"
```

TUI example: `./scripts/build-arm64.sh --example tui --native`, then run the
printed path from the physical console. For exact paths, package lists, and
recovery steps, see [`docs/installation.md`](./docs/installation.md).

## Repository layout

```text
crates/kiosk/          Rust CLI (scaffold; implemented in Phase 3)
config/                Example kiosk configuration
templates/             systemd units and session launcher (Phase 2)
scripts/               Reference platform scripts
examples/tauri-app/    Minimal Tauri 2 reference application
docs/                  Installation, architecture, and operational guides
tests/                 Integration tests and fixtures
```

## Documentation

- [Installation](./docs/installation.md) — reference platform procedure.
- [Configuration](./docs/configuration.md) — the `kiosk.toml` schema.
- [Architecture](./docs/architecture.md) — layers and runtime adapters.
- [Development](./docs/development.md) — building, testing, and extending.
- [`PLAN.md`](./PLAN.md) — project plan and roadmap.

## License

MIT. See [`LICENSE`](./LICENSE).
