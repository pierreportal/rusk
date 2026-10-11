# Rusk TUI example — Ratatui

A minimal full-screen terminal application used as the reference TUI
application. It needs **no** desktop environment, compositor, or WebKitGTK:
it renders terminal graphics directly with [`ratatui`](https://ratatui.rs) and
[`crossterm`](https://github.com/crossterm-rs/crossterm).

On the Pi it runs on a Linux virtual console (for example `/dev/tty1`) with
`TERM=linux`. The kernel console font limits glyph coverage and colors; this is
expected for a TUI and is why a richer console such as `kmscon` may be offered
later as an optional enhancement.

## Build and run on the Pi

```bash
cargo build --manifest-path examples/ratatui-app/Cargo.toml --release

# Run it fullscreen on the physical console (not over SSH, which has no console):
examples/ratatui-app/target/release/rusk-tui-example
```

Or use the helper:

```bash
./scripts/build-arm64.sh --example tui --native
```

## Keys

| Key            | Action                    |
| -------------- | ------------------------- |
| `SPACE`/`ENTER`| ping (increment counter)  |
| `q` / `ESC`    | quit                      |

See [`docs/installation.md`](../../docs/installation.md) for the full procedure.
