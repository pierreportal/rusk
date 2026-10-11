# Rusk GUI example — Tauri 2

A minimal fullscreen Tauri 2 application used as the reference GUI application.

It renders a single screen, invokes a Rust command (`greet`) through the Tauri
IPC bridge, and displays the reply — enough to confirm that the frontend, the
webview, and the Rust backend all work on the Raspberry Pi.

## Build and run on the Pi

```bash
# From the repository root, on an ARM64 host:
rustup target add aarch64-unknown-linux-gnu   # not needed when building natively
cargo build --manifest-path examples/tauri-app/src-tauri/Cargo.toml --release

# Run it fullscreen under Cage:
cage -- examples/tauri-app/src-tauri/target/release/rusk-example
```

Or use the helper:

```bash
./scripts/build-arm64.sh --example gui --native
```

## Notes

- No frontend bundler is required: `frontendDist` points directly at
  [`src/`](./src) and the assets are embedded into the binary at compile time.
- `bundle.active` is `false`; this example produces a plain executable rather
  than a Debian package, which is what the kiosk runtime launches.
- The window is configured fullscreen in
  [`src-tauri/tauri.conf.json`](./src-tauri/tauri.conf.json).

See [`docs/installation.md`](../../docs/installation.md) for the full procedure.
