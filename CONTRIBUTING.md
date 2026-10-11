# Contributing to Rusk

## Ground rules

1. **Automate a working system, don't design an abstract one.**
   Prove a procedure by hand on the reference hardware before scripting it.
2. **Keep it small.** A single crate is preferred until a boundary is proven
   useful. Do not add a crate, adapter, or abstraction for a hypothetical need.
3. **Never interpolate untrusted configuration into a shell.** Prefer direct
   process invocation with explicit arguments.

## Development environment

The primary development machine may be macOS or Linux. The reference target is
ARM64 Linux (Raspberry Pi 4, Raspberry Pi OS Lite 64-bit).

```bash
cargo build            # build the CLI scaffold
cargo test             # unit tests
cargo fmt --all        # formatting
cargo clippy --all-targets -- -D warnings
```

Shell scripts are checked with:

```bash
bash -n scripts/*.sh
shellcheck scripts/*.sh   # if installed
```

## Adding a CLI command

1. Add the variant to `crates/kiosk/src/cli.rs`.
2. Add the handler to `crates/kiosk/src/commands/mod.rs`.
3. Keep parsing separate from business logic; put OS interaction under
   `crates/kiosk/src/system/`.
4. Add a unit or integration test under `tests/`.

## Changing the reference procedure

Anything a developer must do on the Pi by hand belongs in
`docs/installation.md` and, once stable, in `scripts/bootstrap-pi.sh`.
The documented procedure is the acceptance test for `kiosk install`.

## Commit messages

Use the imperative mood and a short summary line, e.g.
`scripts: install cage in bootstrap-pi.sh`.
