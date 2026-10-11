# Architecture

Rusk is a reusable toolkit for building single-purpose Linux devices. It does
not provide a full desktop environment; it supervises one application per
device.

## Layers

```text
┌──────────────────────────────────────────────┐
│            Raspberry Pi (OS Lite)            │
│  kernel / drivers / systemd / networking /   │
│  SSH / user accounts                         │
├──────────────────────────────────────────────┤
│              Kiosk management                │
│  configuration · validation · diagnostics ·  │
│  installation · deployment · release mgmt    │
├──────────────────────────────────────────────┤
│              App session (systemd)           │
│  owns a TTY · environment · restart policy   │
├──────────────────────────┬───────────────────┤
│   GUI: Cage / Wayland    │   TUI: (nothing)  │
│   webview / GPU client   │   terminal gfx    │
└──────────────────────────┴───────────────────┘
```

Each layer has one responsibility and is independent:

- **Operating system**: drivers, filesystem, network, SSH, services, accounts.
- **Kiosk management**: reads and validates configuration; installs, checks,
  deploys, supervises, and rolls back releases. This is the `kiosk` CLI.
- **App session**: a systemd-owned unit that starts the application, restarts
  it on failure, and routes logs to the journal.
- **Runtime adapter** (selected by `ui`): decides how the application is driven.
- **Application**: its own UI, logic, and state. The kiosk never needs to know
  what the application does.

## Runtime adapters

The `[kiosk] ui` field selects the adapter and therefore which system packages
and services are required.

### `ui = "gui"` — Wayland via Cage

```
systemd (tty1, seat via logind)
  └─ kiosk-session.sh
       └─ cage -- <application>
```

Requirements: Cage, Mesa/GL, and the application's GUI runtime
(WebKitGTK for Tauri, libraries for winit/SDL). The compositor provides
fullscreen presentation and input. Configuration:

```toml
[kiosk]
ui = "gui"
backend = "cage"

[display]
disable_screen_blanking = true
```

### `ui = "tui"` — terminal graphics

```
systemd (owns tty1, TERM=linux)
  └─ <application>        # ratatui/crossterm, draws to the console
```

No compositor, no seat, no WebKitGTK. The unit claims a TTY path and sets
`TERM`. Rendering is limited by the kernel console font; a richer console
(`kmscon`) may be offered later as an option, not a requirement. Configuration:

```toml
[kiosk]
ui = "tui"

[terminal]
tty = "tty1"
term = "linux"
disable_screen_blanking = true
```

### `ui = "headless"` — no display

A plain systemd service with no TTY and no graphical stack. Useful for service
components on the same device.

## Configuration

- One file per kiosk, TOML, `schema_version` checked.
- Precedence: CLI argument > configuration file > documented defaults.
- Unknown fields and unsupported schema versions are rejected, never silently
  ignored.
- The `ui` field drives which checks `kiosk doctor` performs and which services
  `kiosk install` creates (a TUI kiosk must not install Cage).

## Supervision and lifecycle

- systemd owns process supervision, restart policy, and logging.
- Restart is bounded (`StartLimitBurst`) so a permanently broken application
  reaches a failed state instead of looping forever.
- Recovery paths: SSH for remote administration; stop/restart through the CLI;
  rollback to a previous release after a bad deploy.
- Releases are immutable directories under a release root; `current` is a
  symlink activated atomically. Application data is stored outside releases.

## Security model

- Applications run as an unprivileged user (`kiosk`), given only the device
  groups they need (`video render input audio`).
- Privileged operations happen through narrow `sudo` invocations, not by
  running the CLI or application as root.
- SSH is retained as the recovery and administration path.

## See also

- [`PLAN.md`](../PLAN.md) — full plan, phases, and decision log.
- [`installation.md`](./installation.md) — reference procedure.