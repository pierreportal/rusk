# Configuration

Configuration is the main interface between you and the kiosk runtime:
a single TOML file per device, checked into version control, that describes
*what* to run and *how* it should be supervised.

- **Format:** TOML
- **Default path:** `./kiosk.toml`
- **Override:** `kiosk --config <path> <command>`
- **Schema version:** `1`

Unknown fields and unsupported schema versions are rejected rather than
silently ignored. Every struct uses `deny_unknown_fields`, so a typo is a
parse error.

## Precedence

1. Explicit CLI arguments (Phase 3).
2. The selected configuration file.
3. Documented built-in defaults.

Today only the file and defaults are implemented; CLI overrides arrive with the
operational commands.

## Complete example

```toml
schema_version = 1

[kiosk]
name = "my-kiosk"
ui = "gui"                 # "gui" | "tui" | "headless"
backend = "cage"           # gui only
fullscreen = true
restart_policy = "always"  # "always" | "on-failure" | "never"
restart_delay_seconds = 2

[application]
name = "my-app"
command = "/opt/kiosk/releases/initial/bin/my-app"
working_directory = "/opt/kiosk"
arguments = ["--fullscreen"]
environment = { RUST_LOG = "info" }

[display]                  # gui only
disable_screen_blanking = true
width = 800
height = 480

[terminal]                 # tui only (required when ui = "tui")
tty = "tty1"
term = "linux"
disable_screen_blanking = true

[network]
ssh_enabled = true

[deployment]
release_directory = "/opt/kiosk/releases"
keep_releases = 3
```

Two ready-to-use files live in [`config/`](../config):
`kiosk.example.toml` (GUI) and `kiosk.tui.example.toml` (TUI).

## `[kiosk]`

| Field                    | Type    | Default    | Notes                                              |
| ------------------------ | ------- | ---------- | -------------------------------------------------- |
| `name`                   | string  | —          | Required, non-empty. Identifies the kiosk.          |
| `ui`                     | enum    | `gui`      | `gui`, `tui`, or `headless`. Selects the runtime.   |
| `backend`                | string  | `cage`     | GUI only. Only `"cage"` is supported today.        |
| `fullscreen`             | bool    | `true`     | Fullscreen presentation.                           |
| `restart_policy`         | enum    | `always`   | `always`, `on-failure`, or `never`.                |
| `restart_delay_seconds`  | integer | `2`        | Delay between restart attempts.                    |

`ui` is the most important field: it decides which system dependencies and
services are required. A `tui` kiosk must not install Cage or WebKitGTK; a
`gui` kiosk does.

## `[application]`

| Field               | Type              | Default | Notes                                                |
| ------------------- | ----------------- | ------- | ---------------------------------------------------- |
| `name`              | string            | —       | Required, non-empty.                                 |
| `command`           | path              | —       | Required, absolute path to the executable.           |
| `working_directory` | path              | unset   | Process working directory.                           |
| `arguments`         | array of strings  | `[]`    | Passed verbatim; never expanded by a shell.          |
| `environment`       | table of strings  | `{}`    | Extra environment variables.                         |

The command is launched directly, not through a shell. Configuration values
are never interpolated into a shell command line.

When release management is in use, `command` points at the stable `current`
symlink, not at a versioned release directory.

## `[display]` — GUI only

| Field                     | Type    | Default | Notes                                        |
| ------------------------- | ------- | ------- | -------------------------------------------- |
| `disable_screen_blanking` | bool    | `true`  | Prevent the display from sleeping.           |
| `width`                   | integer | unset   | Optional hint; capabilities are inspected.   |
| `height`                  | integer | unset   | Optional hint; capabilities are inspected.   |

`width`/`height` are hints, not enforced modes, until the runtime can inspect
the connected display.

## `[terminal]` — TUI only

Required when `ui = "tui"`. A TUI runs directly on a Linux virtual console with
no compositor.

| Field                     | Type   | Default | Notes                                            |
| ------------------------- | ------ | ------- | ------------------------------------------------ |
| `tty`                     | string | `tty1`  | Console the session claims.                      |
| `term`                    | string | `linux` | Value exported as `TERM`.                        |
| `disable_screen_blanking` | bool   | `true`  | Prevent console blanking.                        |

## `[network]`

| Field         | Type | Default | Notes                                        |
| ------------- | ---- | ------- | -------------------------------------------- |
| `ssh_enabled` | bool | `true`  | Keep SSH available as the recovery path.     |

## `[deployment]`

| Field               | Type    | Default | Notes                                        |
| ------------------- | ------- | ------- | -------------------------------------------- |
| `release_directory` | path    | —       | Root under which releases are staged.        |
| `keep_releases`     | integer | `3`     | How many previous releases to retain.        |

## Validation

`kiosk config --config <path> validate` loads, parses, and validates a file.
Current rules:

- `schema_version` must equal the supported version (currently `1`).
- `kiosk.name`, `application.name`, and `application.command` must be non-empty.
- With `ui = "gui"`, `backend` — if present — must be `"cage"`.
- With `ui = "tui"`, a `[terminal]` section is required and its `tty`/`term`
  must be non-empty.
- Unknown fields are a parse error.
- Invalid enum values (e.g. an unknown `restart_policy`) are a parse error.

Validation collects *all* problems and reports them together:

```text
$ kiosk config --config bad.toml validate
error: invalid configuration:
  - unsupported schema_version 99 (this build supports 1)
  - kiosk.name must not be empty
```

## Implementation status

The configuration model and its validation are implemented and tested
(`crates/kiosk/src/config.rs`). Fields not yet consumed by `install`/`doctor`
are accepted but inert until their phases land — see [`PLAN.md`](../PLAN.md).
`width`, `height`, and `restart_delay_seconds` in particular are placeholders
for behavior delivered later.

## See also

- [`architecture.md`](./architecture.md) — how the `ui` field drives runtime,
  dependencies, and services.
- [`installation.md`](./installation.md) — the reference procedure.
