# Installation — Reference Platform (Phase 1)

This procedure turns a clean Raspberry Pi OS Lite (64-bit) installation into a
device that can run a Rust kiosk application, either **GUI** (Wayland via Cage
+ WebKitGTK) or **TUI** (terminal graphics on a virtual console, no desktop
stack).

It is the acceptance test for the automation that arrives in later phases:
scripts in Phase 1, then `kiosk install` in Phase 3.

---

## 1. What you need

- Raspberry Pi 4 (2 GB RAM or more).
- A microSD card, 32 GB or larger (class A1/A2 recommended).
- A card reader and a computer to flash from (macOS, Linux, or Windows).
- Network access: an Ethernet cable, or Wi-Fi credentials.
- A display with HDMI plus keyboard and mouse — needed for GUI kiosks. A **TUI
  kiosk can be set up entirely headless**.
- Your SSH public key (optional but recommended). See §2.2.

## 2. Flash Raspberry Pi OS Lite (64-bit)

### 2.1 Raspberry Pi Imager settings

1. Install the Raspberry Pi Imager:
   ```bash
   # macOS (Homebrew)
   brew install --cask raspberry-pi-imager
   ```
   Otherwise download it from
   <https://www.raspberrypi.com/software/>.
2. Insert the microSD card.
3. In Imager, choose:
   - **Choose Device** → Raspberry Pi 4.
   - **Choose OS** → *Raspberry Pi OS (other)* →
     **Raspberry Pi OS Lite (64-bit)**.
   - **Choose Storage** → your microSD card. Double-check you selected the
     card and not an internal disk.
4. Open **Edit Settings** (the gear icon, or `Ctrl+Shift+X`) and set:
   - **Hostname**: `kiosk` — reachable later as `kiosk.local`.
   - **Enable SSH** → *Allow public-key authentication only*, and paste the
     contents of your public key (§2.2). If you do not have a key yet, use
     password authentication and tighten it later.
   - **Username and password**: create your maintenance account, e.g. username
     `admin`. Current images have **no default `pi` user**; this account is
     your remote-administration login. Do not use `root`.
   - **Configure wireless LAN**: SSID, password, and country — skip when using
     Ethernet.
   - **Set locale settings**: timezone and keyboard layout.
5. **Write**, then eject the card once it finishes.

> The `admin` account is for maintenance and SSH. The unprivileged `kiosk`
> user that runs the application is created later by `bootstrap-pi.sh` (§4).
> Keeping the two separate is part of the security model.

### 2.2 Generating an SSH key (if you don't have one)

```bash
ssh-keygen -t ed25519 -C "rusk@$(hostname)"
# Press Enter for the default path (~/.ssh/id_ed25519); set a passphrase.

# Print the public key to paste into Raspberry Pi Imager:
cat ~/.ssh/id_ed25519.pub
```

Paste the **public** key (`.pub`) into Imager. Never share the private key.

### 2.3 Changing the user later (optional)

If you already imaged a card with an unwanted username, re-flashing with the
wanted name is simpler and safer than renaming in place. To rename anyway:

```bash
sudo usermod --login admin --move-home --home /home/admin oldname
sudo groupmod --new-name admin oldname
```

Renaming touches group memberships, sudoers entries, cron jobs, and file
ownership, so re-imaging is usually the better choice.

## 3. Headless first boot and SSH

1. Insert the card into the Pi, connect Ethernet (or rely on Wi-Fi), and power
   on.
2. **First boot takes 1–3 minutes** while the filesystem expands and the
   hostname is applied. Wait before assuming it failed.
3. Find the device on the network. mDNS usually just works:
   ```bash
   ping kiosk.local
   ```
   If `.local` does not resolve, check your router's DHCP client list, or scan
   the subnet:
   ```bash
   # macOS / Linux
   arp -a
   nmap -sn 192.168.1.0/24     # if nmap is installed
   ```
   On Windows, install Bonjour or use the IP address directly.
4. Connect (key-based if you installed a key in §2.2):
   ```bash
   ssh admin@kiosk.local
   # or select a specific key explicitly:
   ssh -i ~/.ssh/id_ed25519 admin@kiosk.local
   ```
5. Update the system and reboot:
   ```bash
   sudo apt update && sudo apt full-upgrade -y
   sudo reboot
   ```

If you chose public-key-only authentication, password login is already
disabled. Verify with:

```bash
grep -E '^(PasswordAuthentication|PubkeyAuthentication)' /etc/ssh/sshd_config
```

A DHCP reservation on your router keeps `kiosk.local` and its IP stable across
reboots. Then continue with §4 to bootstrap the reference platform.

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

### Optional: SSH access for the `kiosk` user

`bootstrap-pi.sh` creates `kiosk` with a locked password and an empty
`~/.ssh`. If you want to log in as `kiosk` directly for maintenance, authorize
your admin key for it (run as your `admin` user):

```bash
ssh admin@kiosk.local '
  sudo install -d -m 700 -o kiosk -g kiosk /home/kiosk/.ssh
  sudo tee /home/kiosk/.ssh/authorized_keys >/dev/null
  sudo chown kiosk:kiosk /home/kiosk/.ssh/authorized_keys
  sudo chmod 600 /home/kiosk/.ssh/authorized_keys
' < ~/.ssh/id_ed25519.pub
```

Log in with `ssh kiosk@kiosk.local`. This is optional — `admin` remains the
recovery path — but it is convenient for inspecting the application user's
environment.

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
| GL errors in GUI            | Confirm the Pi uses the V3D driver: `sudo raspi-config` → Advanced → GL Driver. |
| `kiosk.local` won't resolve | First boot may still be running; wait 1–3 min, then use the router's DHCP list or the IP. |
| SSH `Permission denied`     | Wrong user or key; check `ssh -v admin@kiosk.local`. Re-check the key pasted into Imager. |
| SSH host key warning        | Re-imaging changes the host key; clear the old entry with `ssh-keygen -R kiosk.local`. |

## Next steps

Phase 2 automates startup: a systemd unit owns the session (GUI on `tty1` via
Cage, TUI on `tty1` without a compositor), restarts on crash, and stays
recoverable over SSH. Do not proceed until the app works reliably by hand.