# Installation — Reference Platform (Phase 1)

This procedure turns a clean Raspberry Pi OS Lite (64-bit) installation into a
device that can run a Rust kiosk application, either **GUI** (Wayland via Cage
+ WebKitGTK) or **TUI** (terminal graphics on a virtual console, no desktop
stack).

It is the acceptance test for the automation that arrives in later phases:
scripts in Phase 1, then `kiosk install` in Phase 3.

---

## 1. What you need

- Raspberry Pi 4 (2 GB or more recommended)
- A display with HDMI, plus keyboard and mouse (GUI)
- A 32 GB (or larger) microSD card
- Another computer on the same network (to flash and SSH in)

## 2. Flash Raspberry Pi OS Lite 64-bit

1. Install the [Raspberry Pi Imager](https://www.raspberrypi.com/software/).
2. Choose **Raspberry Pi OS Lite (64-bit)**.
3. Before writing, open the Imager settings:
   - Enable SSH (password or a public key you have).
   - Set a username and password for your Pi user.
   - Configure Wi-Fi if you are not using Ethernet.
4. Write the image and boot the Pi.

## 3. First boot

Find the Pi on your network (e.g. `ping raspberrypi.local`) and connect:

```bash
ssh <user>@raspberrypi.local
sudo apt update && sudo apt full-upgrade -y
sudo reboot
```

## 4. Bootstrap the reference platform

Clone this repository and run the bootstrap script. Choose the runtime(s):

```bash
# GUI kiosk (Cage + WebKitGTK) — the default
sudo ./scripts/bootstrap-pi.sh --ui gui --build-deps

# TUI kiosk (no compositor, no WebKitGTK)
sudo ./scripts/bootstrap-pi.sh --ui tui --build-deps
```

What it does:

- Installs runtime packages for the selected `--ui`
  (GUI: `cage`, Mesa GL, and the WebKitGTK/GTK runtime; TUI: nothing extra).
- With `--build-deps`, installs the native build toolchain
  (`gcc`, `pkg-config`, dev headers) needed to compile on the Pi.
- Creates an unprivileged `kiosk` user and adds it to the
  `video render input audio plugdev` groups.
- Ensures `ssh` is enabled so the device stays remotely reachable.
- Writes the baseline to `/etc/rusk/reference-platform.txt`.

The script is idempotent and safe to re-run.

## 5. Build an example on the Pi

Compile natively on the Pi (recommended starting point):

```bash
# GUI example (Tauri 2)
./scripts/build-arm64.sh --example gui --native

# TUI example (ratatui)
./scripts/build-arm64.sh --example tui --native
```

Output binary locations:

| Example   | Path                                                    |
| --------- | ------------------------------------------------------- |
| GUI       | `examples/tauri-app/src-tauri/target/release/rusk-example` |
| TUI       | `examples/ratatui-app/target/release/rusk-tui-example`     |

The GUI example needs this repository's WebKitGTK dev headers, installed by
`--build-deps`. Verify with `pkg-config --exists webkit2gtk-4.1 && echo ok`.

## 6. Run manually (before automating anything)

### GUI — under Cage

Run from the physical console (not over SSH), e.g. as your user:

```bash
cage -- examples/tauri-app/src-tauri/target/release/rusk-example
```

Verify:

- A fullscreen window appears without a desktop environment.
- The page shows `Hello, Rusk! The Rust kiosk backend is running.` — this is
  the Rust `greet` command round-tripping through the webview.
- Keyboard, mouse, and touchscreen (where applicable) work.
- Closing/quitting returns to the console cleanly.

### TUI — on the virtual console

Run from the physical console with `TERM=linux`:

```bash
TERM=linux examples/ratatui-app/target/release/rusk-tui-example
```

Verify:

- A bordered fullscreen view renders with `pings: 0`.
- `SPACE`/`ENTER` increments the counter; `q`/`ESC` restores the console.

> A TUI is not meant to run over SSH for display; it draws to a console. It can
> be *started* via SSH, but render it on the attached screen.

## 7. Verify without a network connection

Disconnect Ethernet/Wi-Fi and reboot. Both runtimes are local-only at this
stage; the application should start and render exactly as before.

## 8. Record the baseline

The bootstrap script already writes `/etc/rusk/reference-platform.txt`. Keep a
copy of that file and of the exact commands above — it is the "known-good
baseline" used to verify later automation and to document the Phase 1
deliverable.

## 9. Troubleshooting (Phase 1)

| Symptom                     | Likely cause / fix                                      |
| --------------------------- | ------------------------------------------------------- |
| Black screen under Cage     | Run from the physical console, not SSH. Check `loginctl list-sessions`. |
| missing `libwebkit2gtk`     | GUI only: re-run bootstrap with `--ui gui --build-deps`. |
| `cage: command not found`   | Run `sudo ./scripts/bootstrap-pi.sh --ui gui`.          |
| TUI garbage/boxes           | Expected on the kernel console font; `TERM=linux`.      |
| GL errors in GUI            | Confirm the Pi is using the V3D driver:
                              `sudo raspi-config` → Advanced → GL Driver. |

## Next steps

Phase 2 automates startup: a systemd unit owns the session (GUI on `tty1` via
Cage, TUI on `tty1` without a compositor), restarts on crash, and stays
recoverable over SSH. Do not proceed until the app works reliably by hand.