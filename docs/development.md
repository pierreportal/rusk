# Development Guide

How to work on Rusk: build it, test it, run the examples, and add to it.

The primary development machine may be macOS or Linux. The reference target is
ARM64 Linux (Raspberry Pi 4, Raspberry Pi OS Lite 64-bit) — see
[`installation.md`](./installation.md).

## Toolchain

[`rust-toolchain.toml`](../rust-toolchain.toml) pins the `stable` channel with
`rustfmt` and `clippy`. With [rustup](https://rustup.rs) installed, the correct
toolchain is selected automatically when you enter the repository.

The workspace targets Rust `1.77` or newer (`rust-version` in
[`Cargo.toml`](../Cargo.toml)) because the Tauri 2 example requires it.

## Repository layout

```text
crates/kiosk/          The `kiosk` CLI
  src/main.rs          Entry point; parses argv and sets the exit code
  src/cli.rs           clap definitions (argument parsing only)
  src/commands/        Dispatch; business logic per command
  src/config.rs        Configuration model + validation
  src/error.rs         Structured error types
  src/system/          OS interaction (packages, services, display) — Phase 3
config/                Example kiosk.toml files
templates/             systemd units + session launcher
scripts/               Reference-platform scripts
examples/              Reference applications (Tauri GUI, Ratatui TUI)
docs/                  This documentation
```

Keep parsing, business logic, and OS interaction in separate places. Command
handlers should read like a description of the operation, not a pile of
`Command::new("systemctl")`.

## Build, test, lint

```bash
cargo build                       # workspace (the CLI)
cargo test                        # unit tests
cargo fmt --all                   # format
cargo fmt --all -- --check        # verify formatting (CI)
cargo clippy --all-targets -- -D warnings
```

The examples are separate workspaces (each declares its own `[workspace]`) so
their larger dependency trees do not burden the CLI:

```bash
cargo build --manifest-path examples/ratatui-app/Cargo.toml
cargo build --manifest-path examples/tauri-app/src-tauri/Cargo.toml
```

## Running the examples

On a desktop (macOS/Linux) the TUI example runs in any terminal:

```bash
cargo run --manifest-path examples/ratatui-app/Cargo.toml
```

On the Pi:

```bash
# GUI (needs Cage + WebKitGTK)
cage -- examples/tauri-app/src-tauri/target/release/rusk-example

# TUI (run from the physical console, not over SSH)
TERM=linux examples/ratatui-app/target/release/rusk-tui-example
```

## Building for ARM64

Native build on the Pi itself is the supported Phase 1 path:

```bash
./scripts/build-arm64.sh --example gui --native
./scripts/build-arm64.sh --example tui --native
```

Cross-compiling from macOS or x86 Linux uses `cargo-zigbuild` + `zig`:

```bash
cargo install cargo-zigbuild
brew install zig                         # or see https://ziglang.org/download/
./scripts/build-arm64.sh --example tui --cross
```

A cross-built binary must still be verified on the reference hardware; in
particular the GUI example links WebKitGTK/GTK at run time.

`--manifest <path> --bin <name>` builds an arbitrary application instead of a
bundled example.

## Adding a CLI command

1. Add a variant to the `Command` enum in
   [`crates/kiosk/src/cli.rs`](../crates/kiosk/src/cli.rs). Prefer a small
   `Args` struct for options.
2. Add the matching arm in
   [`crates/kiosk/src/commands/mod.rs`](../crates/kiosk/src/commands/mod.rs).
3. Keep the handler thin: put OS interaction in a `system::` module
   (e.g. `system::services`) so it can be unit-tested with mocked inputs.
4. Add a unit or integration test.
5. Update the docs that describe the command.

Parsing must not perform work, and handlers must not construct shell strings
from configuration. When a command changes the system, support `--dry-run` and
make it idempotent.

## Changing service templates

The systemd units under [`templates/`](../templates) are the source of truth
that `kiosk install` will render in Phase 2/3. When you change one:

- Keep placeholders in the `@@NAME@@` form.
- Update `docs/installation.md` if the manual procedure changes.
- Update `templates/kiosk-session.sh` alongside `kiosk-session.service`.
- Remember the two runtimes differ: the GUI unit uses `PAMName=login` and
  starts `cage`; the TUI unit claims a TTY and sets `TERM` with no compositor.

## Testing deployment and rollback

Deployment and rollback are Phase 5–6 and not implemented yet. The intended
approach is temporary directories plus mocked system operations for unit tests,
and the reference Pi for hardware acceptance tests. Until then,
[`scripts/deploy-dev.sh`](../scripts/deploy-dev.sh) copies a single binary over
SSH for manual testing.

## Testing strategy

- **Unit tests** run anywhere (configuration parsing, defaults, invalid input,
  release-identifier validation). Add them next to the code or under
  [`tests/`](../).
- **Integration tests** cover generated unit files, exit codes, dry-run
  behavior, staging, and rollback using temporary directories.
- **Hardware acceptance tests** run on the reference Pi (cold boot, crash,
  missing executable, network loss, interrupted update, rollback). See
  [`PLAN.md`](../PLAN.md) §15.

## Scripts

| Script                          | Purpose                                            |
| ------------------------------- | -------------------------------------------------- |
| `scripts/bootstrap-pi.sh`       | Prepare a clean Pi OS Lite install (`--ui`).       |
| `scripts/build-arm64.sh`        | Build an example for ARM64 (native or cross).      |
| `scripts/deploy-dev.sh`         | Copy a binary to a Pi over SSH for manual testing. |

Shell scripts must be `bash -n` clean and, when available, `shellcheck` clean.

## See also

- [`configuration.md`](./configuration.md) — the configuration schema.
- [`architecture.md`](./architecture.md) — layers and runtime adapters.
- [`PLAN.md`](../PLAN.md) — phases, milestones, and the decision log.
