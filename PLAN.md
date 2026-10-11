# Rust Kiosk aka Rusk

## Project Plan: A Reusable Rust-Based Kiosk Runtime for Raspberry Pi

**Project type:** Open-source developer tool / embedded Linux platform
**Primary language:** Rust
**Reference hardware:** Raspberry Pi 4
**Reference OS:** Raspberry Pi OS Lite 64-bit
**Graphics stack:** Wayland + Cage
**Initial application:** Tauri 2 with a local Rust backend
**Deployment target:** ARM64 Linux
**Primary objective:** Turn a Raspberry Pi into a reliable, single-purpose device that boots directly into a graphical application.

---

# 1. Project Vision

Rust Kiosk is a reusable toolkit for building single-purpose Linux devices without maintaining a full desktop environment.

The project provides a standard way to configure, install, launch, supervise, update, diagnose, and recover graphical applications on Raspberry Pi hardware.

Instead of manually configuring Linux, Wayland, Cage, application dependencies, systemd services, and startup scripts for every project, developers should be able to reuse the same infrastructure.

The long-term vision is to make deploying a graphical Rust application to a Raspberry Pi almost as straightforward as running a local development command.

For example:

```bash
kiosk init
kiosk doctor
kiosk install
kiosk deploy
kiosk status
kiosk logs
kiosk rollback
```

The toolkit should remain small, understandable, and easy to maintain. It should automate Linux configuration rather than reimplement functionality already provided by Linux.

## 1.1 Core principles

1. **Minimalism:** Install only what is necessary to run the selected application.
2. **Reliability:** Recover automatically from application crashes and failed updates.
3. **Reusability:** Support different applications without changing the core runtime.
4. **Local-first operation:** The device must work without an internet connection unless the application itself requires one.
5. **Remote administration:** Keep SSH available for development, maintenance, and recovery.
6. **Reproducibility:** A documented procedure should turn a clean OS installation into a working kiosk.
7. **Explicit configuration:** Prefer a version-controlled configuration file over undocumented system modifications.
8. **Separation of concerns:** Keep application code, kiosk management, graphics infrastructure, and OS provisioning independent.
9. **Security:** Run applications as an unprivileged user and minimize unnecessary services.
10. **Incremental complexity:** Start with scripts and systemd; introduce more sophisticated Rust components only when they solve a demonstrated problem.

---

# 2. Scope and Non-Goals

## 2.1 What the project will provide

The initial project will provide:

- A reference installation for Raspberry Pi OS Lite 64-bit.
- A minimal graphical environment using Cage and Wayland, for GUI applications.
- A terminal (TUI) runtime that needs no desktop environment or compositor.
- A Rust CLI for configuration and diagnostics.
- A standardized application configuration format.
- A systemd-managed application session (graphical or terminal).
- Automatic application startup on boot.
- Application crash recovery.
- Remote deployment over SSH.
- Structured logs and diagnostic commands.
- Versioned releases and rollback.
- Documentation and example applications.
- Automated tests for configuration, lifecycle logic, and deployment behavior.

## 2.2 What the project will not initially provide

The project will not initially:

- Implement its own Wayland compositor.
- Replace systemd with a custom process supervisor.
- Build a complete Linux distribution.
- Support every Linux distribution or every Raspberry Pi model.
- Manage arbitrary desktop environments.
- Require a cloud account or remote management server.
- Require Docker or Kubernetes.
- Include a graphical administration interface.
- Guarantee automatic recovery from hardware failure or filesystem corruption.
- Attempt to abstract every Linux graphics backend behind a complex plugin architecture.

These exclusions keep the initial system small enough for one developer to build and maintain.

---

# 3. Target Users and Use Cases

Rust Kiosk should be useful for developers building dedicated hardware interfaces, installations, and embedded applications.

Potential use cases include:

| Use case              | Example application                      |
| --------------------- | ---------------------------------------- |
| E-reader              | EPUB reader with a touchscreen           |
| Musical instrument    | Rust synthesizer or MIDI controller      |
| Embedded interface    | Physical controls for a local device     |
| Digital signage       | Fullscreen information display           |
| Creative installation | Interactive visual or audio installation |
| Home automation       | Local dashboard                          |
| Laboratory instrument | Data acquisition and visualization UI    |
| Network appliance     | Local configuration interface            |

The common requirement is that the device should start directly into one application and require little or no routine interaction with the operating system.

---

# 4. Technical Architecture

## 4.1 High-level architecture

```text
┌────────────────────────────────────────────┐
│              Raspberry Pi 4               │
├────────────────────────────────────────────┤
│           Raspberry Pi OS Lite             │
│                                            │
│  Linux kernel / device drivers             │
│  systemd / networking / SSH                │
│                                            │
├────────────────────────────────────────────┤
│             Rust Kiosk CLI                 │
│                                            │
│  init / doctor / install / deploy           │
│  status / logs / restart / rollback         │
├────────────────────────────────────────────┤
│           Kiosk systemd service            │
│                                            │
│  User session / TTY / environment           │
├────────────────────────────────────────────┤
│               Cage / Wayland               │
│                                            │
│  Display / fullscreen / input               │
├────────────────────────────────────────────┤
│              Application                   │
│                                            │
│  Tauri / native Rust / SDL / other GUI      │
│                                            │
└────────────────────────────────────────────┘
```

The CLI manages configuration and installation. Systemd manages the long-running session. Cage manages the graphical environment. The application remains independent.

The **Cage / Wayland** layer is used only by GUI applications. A TUI
application is a plain full-screen terminal program (for example built with
`ratatui` and `crossterm`) that draws to a Linux virtual console directly, with
no compositor and no webview:

```text
GUI application                    TUI application
───────────────                    ───────────────
Linux kernel / device drivers      Linux kernel / device drivers
systemd / networking / SSH         systemd / networking / SSH
Kiosk session (systemd)            Kiosk session (systemd, owns a TTY)
Cage / Wayland                     (no compositor)
Webview / GPU client               Terminal graphics (ANSI/Ratatui)
```

Both paths share the same configuration format, session supervision, logging,
deployment, and rollback machinery; only the runtime adapter differs.


## 4.2 Responsibilities by layer

### Operating system

Responsible for:

- Hardware drivers.
- Filesystem and storage.
- Network interfaces.
- SSH access.
- User accounts.
- System service management.
- Kernel and boot configuration.

### Kiosk management layer

Responsible for:

- Reading and validating configuration.
- Installing kiosk services.
- Checking runtime dependencies.
- Deploying application releases.
- Reporting application and session status.
- Managing rollback and recovery.

### Graphical runtime

Responsible for:

- Creating the Wayland session.
- Managing fullscreen presentation.
- Providing input and display access.
- Launching the selected graphical application.

### Application

Responsible for:

- Its user interface.
- Business logic.
- Application-specific state.
- Hardware access required by the application.
- Application-specific background work.

The kiosk runtime should not need to know whether the application is an EPUB reader, a synthesizer, or a dashboard.

---

# 5. Repository Structure

Begin with a compact workspace. Avoid creating a separate crate for every conceptual component before there is a practical need.

```text
rust-kiosk/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── LICENSE
├── CONTRIBUTING.md
│
├── crates/
│   └── kiosk/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── cli.rs
│           ├── config.rs
│           ├── error.rs
│           ├── commands/
│           │   ├── init.rs
│           │   ├── doctor.rs
│           │   ├── install.rs
│           │   ├── uninstall.rs
│           │   ├── start.rs
│           │   ├── stop.rs
│           │   ├── restart.rs
│           │   ├── status.rs
│           │   ├── logs.rs
│           │   ├── deploy.rs
│           │   └── rollback.rs
│           └── system/
│               ├── packages.rs
│               ├── services.rs
│               ├── display.rs
│               ├── processes.rs
│               └── ssh.rs
│
├── config/
│   └── kiosk.example.toml
│
├── templates/
│   ├── kiosk-session.service
│   ├── kiosk-app.service
│   └── kiosk-session.sh
│
├── scripts/
│   ├── bootstrap-pi.sh
│   ├── build-arm64.sh
│   └── deploy-dev.sh
│
├── examples/
│   ├── tauri-app/
│   └── native-rust-app/
│
├── tests/
│   ├── config_tests.rs
│   ├── command_tests.rs
│   └── fixtures/
│
└── docs/
    ├── architecture.md
    ├── installation.md
    ├── configuration.md
    ├── development.md
    ├── deployment.md
    ├── troubleshooting.md
    └── security.md
```

### Architectural evolution

For the first release, a single `kiosk` crate is sufficient.

Split it into `kiosk-core`, `kiosk-cli`, and potentially `kiosk-agent` only when the boundaries become useful. A local daemon is not necessary for the MVP because systemd already provides process supervision.

---

# 6. Configuration Design

Configuration should be the main interface between the developer and the kiosk runtime.

Use TOML because it is human-readable, supports nested settings, and works well with Rust's Serde ecosystem.

## 6.1 Initial configuration format

Create `config/kiosk.example.toml`:

```toml
schema_version = 1

[kiosk]
name = "my-kiosk"
# Runtime kind: "gui" (Wayland/Cage + webview), "tui" (terminal), or
# "headless" (no display at all). Defaults to "gui".
ui = "gui"
# Compositor backend. Only meaningful when ui = "gui".
backend = "cage"
fullscreen = true
restart_policy = "always"
restart_delay_seconds = 2

[application]
name = "my-app"
command = "/opt/kiosk/releases/initial/bin/my-app"
working_directory = "/opt/kiosk"
arguments = []
environment = {}

# GUI only.
[display]
disable_screen_blanking = true
width = 800
height = 480

# TUI only. The application runs directly on a Linux virtual console with no
# compositor; `term` selects the TERM value the console is described as.
[terminal]
tty = "tty1"
term = "linux"
disable_screen_blanking = true

[network]
ssh_enabled = true

[deployment]
release_directory = "/opt/kiosk/releases"
keep_releases = 3
```

This is an initial schema, not a promise that every field is implemented in version one.

The `ui` field selects the runtime adapter and therefore which system
dependencies are required: a GUI kiosk needs Cage and WebKitGTK, whereas a TUI
kiosk needs neither. `kiosk doctor` must check only the dependencies relevant
to the configured `ui`.

For example, `width` and `height` should eventually be optional hints or enforced modes rather than mandatory values. Display capabilities should be inspected before applying a mode.

## 6.2 Configuration requirements

The configuration system must:

- Validate required fields.
- Reject unsupported schema versions.
- Check that the application path exists when appropriate.
- Validate restart policies.
- Validate argument and environment variable formats.
- Provide clear error messages.
- Preserve defaults in one place.
- Avoid silently ignoring invalid settings.
- Support a dry-run mode before modifying the OS.
- Avoid shell interpolation when launching applications.

## 6.3 Configuration precedence

Define a predictable precedence model:

1. Explicit CLI arguments.
2. The selected configuration file.
3. Documented built-in defaults.

Environment variables can override selected operational settings, but their use should be explicit and documented.

---

# 7. Command-Line Interface

The CLI is the main product interface.

Use `clap` for argument parsing, `serde` for configuration serialization, and `anyhow` or `thiserror` for error handling where appropriate.

## 7.1 `kiosk init`

Creates a configuration file.

```bash
kiosk init
kiosk init --name music-controller
kiosk init --app /opt/kiosk/bin/my-app
```

Responsibilities:

- Create the project configuration.
- Apply safe defaults.
- Validate supplied values.
- Avoid overwriting an existing configuration without confirmation.
- Print the next steps.

Acceptance criteria:

- A valid configuration can be generated from scratch.
- The generated file passes `kiosk config validate`.
- Existing files are protected from accidental replacement.

## 7.2 `kiosk doctor`

Diagnoses whether the current device can run the configured application.

```bash
kiosk doctor
kiosk doctor --verbose
kiosk doctor --json
```

Checks should include:

- OS and architecture.
- Available memory and storage.
- Required package presence.
- Cage availability.
- WebKitGTK dependencies for Tauri applications.
- systemd availability.
- User and TTY configuration.
- Display and seat availability where detectable.
- Application executable permissions.
- Configuration validity.
- SSH status.

The command should distinguish errors from warnings.

For example, missing WebKitGTK should be an error for a Tauri application but irrelevant to a native application that does not use it.

## 7.3 `kiosk install`

Configures the device to launch the application on boot.

```bash
kiosk install
kiosk install --dry-run
```

Responsibilities:

1. Validate the configuration.
2. Check runtime dependencies.
3. Confirm the selected application.
4. Install the session script and systemd unit.
5. Apply necessary permissions.
6. Enable the service.
7. Preserve remote administration.
8. Report how to start, stop, or recover the kiosk.

Installation must be idempotent: repeating the command should not create duplicate services or damage existing configuration.

Privileged operations should use a narrow, explicit mechanism such as `sudo` for specific system changes, not run the whole application as root.

## 7.4 Lifecycle commands

Implement:

```bash
kiosk start
kiosk stop
kiosk restart
kiosk status
kiosk logs
```

Use systemd to manage the lifecycle.

`status` should report:

- Service state.
- Active release.
- Last exit status when available.
- Recent restart activity.
- Application configuration.
- Relevant diagnostics.

`logs` should expose recent service output using the journal rather than inventing a separate log management system.

## 7.5 `kiosk uninstall`

Remove the kiosk service without automatically deleting application data or user configuration.

Offer explicit flags for destructive cleanup.

```bash
kiosk uninstall
kiosk uninstall --remove-releases
```

Never silently delete the only known-good application release or user data.

---

# 8. Phase 1 — Establish the Reference Platform

**Goal:** Prove that the basic kiosk stack works before building the management tool.

Estimated effort: 1–3 days.

## Tasks

1. Install Raspberry Pi OS Lite 64-bit on a Raspberry Pi 4.
2. Configure SSH and a non-root user.
3. Install Cage and the required runtime packages.
4. Install the Tauri application's Linux ARM64 dependencies.
5. Build a minimal Tauri application.
6. Run the application manually under Cage.
7. Verify keyboard, mouse, touchscreen, and display behavior as applicable.
8. Verify that the Rust backend functions correctly.
9. Test the application without a network connection.
10. Record the exact OS release, packages, and commands used.

## Deliverables

- A clean Pi installation.
- A working fullscreen Tauri application.
- A reproducible installation guide.
- A list of required runtime dependencies.
- A known-good baseline for later testing.

## Acceptance criteria

- The application launches from a terminal.
- Its Rust backend works on the Pi.
- The interface renders correctly on the connected display.
- SSH remains accessible.
- The application does not require a full desktop environment.

**Exit condition:** Do not automate startup until the app works reliably when launched manually.

---

# 9. Phase 2 — Reliable Boot and Session Management

**Goal:** Make the application start automatically and recover from ordinary application failures.

Estimated effort: 2–4 days.

## Tasks

1. Define the kiosk service lifecycle.
2. Configure a dedicated application user.
3. Create the session launch script.
4. Create a systemd unit for the graphical session.
5. Configure the correct TTY and seat permissions.
6. Launch Cage with the selected application.
7. Configure restart behavior.
8. Capture stdout and stderr in the system journal.
9. Prevent accidental startup of multiple kiosk sessions.
10. Verify that remote SSH access remains functional.
11. Provide an equivalent systemd unit for TUI applications that claims a TTY
    and sets `TERM` without starting a compositor.

## Lifecycle requirements

The runtime should handle:

- Normal startup.
- Application crashes.
- Application exit.
- Missing executable.
- Missing graphical dependencies.
- Invalid configuration.
- Service restart.
- Reboot.
- Manual service stop.

Avoid an infinite, uncontrolled crash loop. Introduce restart limits, a backoff policy, or a failed state for repeated failures. Recovery should be automatic for ordinary failures but should not conceal a permanently broken application.

## Acceptance criteria

- The Pi boots directly into the application.
- No desktop shell is required.
- The application starts without manual login.
- A crashed application can be restarted.
- Rebooting the Pi reproduces the same behavior.
- An operator can stop the kiosk remotely.
- Failure details can be inspected through systemd and journalctl.

---

# 10. Phase 3 — Rust CLI MVP

**Goal:** Replace project-specific startup scripts with reusable, tested commands.

Estimated effort: 4–7 days.

## Tasks

1. Create the Cargo workspace or single-crate project.
2. Add `clap`, `serde`, `toml`, and error-handling dependencies.
3. Implement configuration loading.
4. Implement configuration validation.
5. Implement `kiosk init`.
6. Implement `kiosk doctor`.
7. Implement `kiosk install`.
8. Implement `kiosk status`.
9. Implement `kiosk logs`.
10. Implement `kiosk start`, `stop`, and `restart`.
11. Add dry-run support for operations that modify the OS.
12. Add unit tests and integration tests.

## Engineering requirements

- Separate command parsing from business logic.
- Avoid shell commands when a direct API or safely constructed process invocation is sufficient.
- Never interpolate untrusted configuration into shell scripts.
- Use structured error types for recoverable failures.
- Print actionable diagnostics.
- Make installation idempotent.
- Use stable exit codes for automation.
- Provide human-readable output and machine-readable JSON where useful.

## Acceptance criteria

A new developer can configure and inspect the kiosk without manually editing systemd files.

The CLI must explain what is missing and how to fix it when the device is not ready.

---

# 11. Phase 4 — Application Runtime Abstraction

**Goal:** Make the toolkit reusable across application types.

Estimated effort: 3–5 days.

## Initial runtime types

Start with three application types, selected by the `ui` configuration field:

1. **GUI application** (`ui = "gui"`): launched inside Cage, which provides a
   Wayland session for a webview or native GPU client (Tauri, winit, SDL).
2. **TUI application** (`ui = "tui"`): a full-screen terminal program (for
   example built with `ratatui` and `crossterm`) run directly on a Linux virtual
   console. No compositor, no webview, and no GUI libraries are installed.
3. **Headless application** (`ui = "headless"`): launched directly as a systemd
   service with no display of any kind.

None of the three require application-specific knowledge. Each only needs an
executable, arguments, environment, working directory, and restart policy.

The GUI type installs and starts Cage and the WebKitGTK/GTK runtime. The TUI
type claims a TTY, sets `TERM`, and starts the program with no compositor; it
must not install Cage. The headless type starts no graphical or terminal
session at all.

A TUI is chosen when the device should not carry a desktop or GPU stack: it
boots faster, uses less memory, and avoids WebKitGTK entirely. The console
rendering is limited by the kernel console font, so a more capable console
(such as `kmscon`) may be offered later as an optional enhancement rather than
a requirement.

## Future runtime adapters

Consider supporting, within the GUI or TUI runtime types:

- Tauri + WebKitGTK.
- Native Rust applications using winit.
- SDL applications.
- Ratatui/crossterm TUI applications.
- Browser-based kiosk applications.
- Additional compositor backends if a real use case requires them.

Do not build a large trait hierarchy prematurely. A small enum and a validated configuration model may be enough.

## Acceptance criteria

- Two different graphical applications can use the same kiosk runtime.
- Switching applications requires configuration changes, not modifications to the core CLI.
- A headless service does not install Cage unnecessarily.
- Missing application-specific dependencies produce relevant diagnostics.

---

# 12. Phase 5 — ARM64 Build and Deployment Workflow

**Goal:** Deploy application updates to the Pi without manually copying files and editing paths.

Estimated effort: 3–6 days.

## Build workflow

Support a developer workflow on a Linux ARM64 environment first.

```text
Source repository
       |
       v
Build application
       |
       v
Validate package
       |
       v
Transfer release over SSH
       |
       v
Verify files and metadata
       |
       v
Activate release
       |
       v
Restart kiosk
       |
       v
Check startup
```

macOS can remain the main development machine, but Linux ARM64 compilation and runtime dependencies must be verified. Native compilation on the Pi is a useful initial path; cross-compilation or building inside an ARM64 Linux container can be introduced later.

## `kiosk deploy`

Example:

```bash
kiosk deploy \
  --host kiosk.local \
  --release ./dist/my-app
```

The actual CLI syntax can evolve, but the deployment mechanism should:

1. Verify that the target is reachable.
2. Check OS architecture and compatibility.
3. Transfer files using SSH-based tooling.
4. Stage the release in a new directory.
5. Verify the transferred files.
6. Preserve executable permissions.
7. Record release metadata.
8. Activate the release atomically where the filesystem supports it.
9. Restart the service.
10. Report the outcome.

Use a release directory layout such as:

```text
/opt/kiosk/
├── releases/
│   ├── 2026-10-11T120000Z/
│   ├── 2026-10-11T130000Z/
│   └── 2026-10-11T140000Z/
├── current -> releases/2026-10-11T140000Z/
└── shared/
    ├── config/
    └── data/
```

Release names should be unique and filesystem-safe. A Git commit hash or build identifier is preferable to relying exclusively on timestamps.

Application data should live outside release directories so deploying a new version does not overwrite user data.

## Acceptance criteria

- An updated application can be deployed remotely.
- An interrupted transfer does not replace the active release.
- The active release can be identified.
- Deployment failure does not silently destroy the previous release.
- Application data survives an update.

---

# 13. Phase 6 — Health Checks and Rollback

**Goal:** Protect a working kiosk from a broken update.

Estimated effort: 3–5 days.

## Health checks

A process being alive does not necessarily mean the application is healthy.

Implement a staged health-check strategy:

1. Confirm that the service started.
2. Confirm that the application process remains alive for a configurable interval.
3. Detect repeated early crashes.
4. Where supported, run an application-specific health check.
5. Report the result to the deployment command.

For graphical applications, generic health checking is difficult. A running process cannot prove that the display is working or that the UI is responsive. Avoid claiming stronger guarantees than the checks actually provide.

## Rollback strategy

Maintain a configurable number of previous releases.

```bash
kiosk rollback
kiosk rollback --to RELEASE_ID
```

Rollback should:

1. Identify the requested previous release.
2. Verify that the release still exists.
3. Switch the active release.
4. Restart the service.
5. Verify startup using the configured health checks.
6. Report success or failure.

Do not automatically delete previous releases until the new one has been activated and basic validation has succeeded.

## Acceptance criteria

- A failed deployment can restore the previous release.
- Release history is inspectable.
- Rollback does not delete application data.
- The system clearly distinguishes process startup from application health.

---

# 14. Phase 7 — Security and System Hardening

**Goal:** Make the system safe to leave running unattended.

Estimated effort: 3–6 days for initial hardening, followed by ongoing testing.

## User and permission model

- Run graphical applications as a dedicated unprivileged user.
- Use the minimum permissions required for display, audio, USB, serial ports, and other peripherals.
- Avoid running the CLI or application as root by default.
- Keep system administration separate from application execution.
- Restrict write access to release and configuration directories.
- Store persistent application data separately.

## Network and remote access

- Retain SSH for maintenance.
- Prefer SSH keys over password-only administration where appropriate.
- Avoid exposing administrative services to the public internet.
- Do not require a cloud management service.
- Make network access requirements configurable.
- Keep the kiosk usable offline when the application permits it.

## Process management

- Use systemd restart policies.
- Limit uncontrolled restart loops.
- Capture logs in the journal.
- Ensure service stop and restart operations are predictable.
- Avoid hidden background processes.

## Filesystem resilience

Eventually evaluate:

- Read-only root filesystem.
- Separate writable data partition or directory.
- Bounded logs.
- Safe application-data writes.
- Power-loss testing.
- Filesystem integrity checks.
- Recovery from interrupted deployment.

A read-only root filesystem is a later hardening task, not an MVP requirement. It changes how logs, updates, and persistent application data must be handled.

## Acceptance criteria

- No application runs as root without a documented, justified requirement.
- SSH provides a recovery path.
- Application crashes do not cause uncontrolled resource consumption.
- Updates do not overwrite persistent user data.
- Configuration changes are auditable.

---

# 15. Phase 8 — Testing Strategy

**Goal:** Make the toolkit reliable across repeated installations and upgrades.

## Unit tests

Test:

- Configuration parsing.
- Default values.
- Invalid configurations.
- Unsupported schema versions.
- Application argument handling.
- Path validation.
- Release identifier validation.
- Restart policy validation.
- Deployment manifest parsing.

These tests should run on the development machine without Raspberry Pi hardware.

## Integration tests

Test:

- Generated systemd unit files.
- Service installation and removal.
- CLI exit codes.
- Dry-run behavior.
- Release staging.
- Activation and rollback.
- Handling of missing packages.
- Handling of inaccessible paths.
- Interrupted deployment.

Use temporary directories and mocked system operations wherever practical.

## Hardware acceptance tests

Run on the reference Pi 4:

- Clean installation.
- Cold boot.
- Warm reboot.
- Application crash.
- Repeated crash.
- Invalid configuration.
- Missing executable.
- Network unavailable at startup.
- Network disconnection during normal use.
- Remote stop and restart.
- Interrupted update.
- Rollback.
- Power interruption.
- Recovery after a failed startup.

## Performance measurements

Record:

- Time from power-on to application visible.
- Idle RAM usage.
- CPU utilization.
- Application startup time.
- Storage consumption.
- Restart recovery time.
- Temperature under sustained load.

Establish baseline measurements before optimizing. Avoid imposing arbitrary performance targets before measuring the actual application and hardware.

---

# 16. Phase 9 — Documentation and Developer Experience

**Goal:** Make the project understandable to someone who has never configured a Raspberry Pi kiosk manually.

## Required documentation

### README

Include:

- Project purpose.
- Supported hardware and OS.
- Feature overview.
- Quick-start instructions.
- Installation requirements.
- Example configuration.
- Current limitations.
- Link to detailed documentation.

### Installation guide

Explain:

- OS imaging.
- Initial user setup.
- SSH configuration.
- Package installation.
- Application deployment.
- Kiosk installation.
- First boot.
- Recovery procedure.

### Development guide

Explain:

- Rust toolchain.
- Repository layout.
- Local testing.
- ARM64 build options.
- How to add CLI commands.
- How to change service templates.
- How to test deployment and rollback.

### Troubleshooting guide

Include solutions for:

- Black screen.
- Missing WebKitGTK libraries.
- Cage startup failure.
- Incorrect display mode.
- Missing Wayland socket or permissions.
- TTY and seat configuration errors.
- Repeated application restarts.
- Full disk.
- SSH connection failure.
- Failed application update.

### Security guide

Document:

- Privilege boundaries.
- SSH configuration.
- File permissions.
- Network exposure.
- Update integrity.
- Recovery access.
- Known security limitations.

---

# 17. Phase 10 — Release Engineering

**Goal:** Produce repeatable releases that other developers can install.

## Initial release

Publish version `0.1.0` when the reference platform works reliably and the CLI can install and inspect the kiosk.

The first release should include:

- Source code.
- Tagged release.
- Installation guide.
- Example configuration.
- Reference systemd unit.
- Tauri example.
- Native Rust example, if available.
- Hardware test report.
- Known limitations.

## Continuous integration

Automate:

- Formatting with `cargo fmt`.
- Linting with `cargo clippy`.
- Unit tests.
- Configuration fixtures.
- CLI help and version checks.
- Documentation checks.
- Release artifact generation.

Use hardware testing or an explicitly maintained reference Pi for tests that cannot be validated in a generic CI runner.

## Versioning

Use semantic versioning once the public CLI and configuration schema become stable.

Treat changes to the configuration schema as compatibility-sensitive. Provide explicit migrations if future versions require incompatible changes.

## Release acceptance criteria

- A tagged version builds reproducibly.
- Installation instructions match the tested OS release.
- Configuration schema changes are documented.
- The release identifies tested hardware and known limitations.

---

# 18. Phase 11 — Optional Custom Linux Image

**Goal:** Reduce setup time and create a more controlled appliance image.

This phase is deliberately optional.

Once the toolkit is stable on Raspberry Pi OS Lite, evaluate whether building a custom image provides meaningful benefits.

Potential approaches:

- Raspberry Pi OS customization.
- A scripted image provisioning process.
- Buildroot for a tightly controlled embedded Linux image.

Do not begin with Buildroot simply because it can produce a smaller image. A custom image creates additional work for package maintenance, kernel updates, security fixes, and hardware compatibility.

## Evaluate before committing

Measure:

- Installed image size.
- Boot time.
- RAM usage.
- Package maintenance effort.
- Update mechanism complexity.
- Recovery options.
- Compatibility with WebKitGTK and the target application.

Move to a custom image only when the benefits outweigh the maintenance burden.

---

# 19. Project Milestones and Completion Criteria

The following schedule is a planning estimate for one developer, not a guarantee.

| Milestone               | Estimated effort | Definition of done                              |
| ----------------------- | ---------------: | ----------------------------------------------- |
| M1: Reference platform  |         1–3 days | Tauri app works under Cage on the Pi            |
| M2: Automatic startup   |         2–4 days | Reliable boot-to-app and remote recovery        |
| M3: CLI MVP             |         4–7 days | Init, doctor, install, status, logs, lifecycle  |
| M4: Runtime abstraction |         3–5 days | Multiple application types supported            |
| M5: Remote deployment   |         3–6 days | Repeatable ARM64 deployment                     |
| M6: Rollback            |         3–5 days | Failed updates can restore a known-good release |
| M7: Hardening           |         3–6 days | Documented security and recovery baseline       |
| M8: Testing and docs    |         3–5 days | Reproducible installation and acceptance tests  |
| M9: First release       |         1–3 days | Tagged, documented, tested release              |

Some phases overlap, so the total elapsed time may be shorter than the sum of these estimates. Reserve additional time for hardware-specific debugging.

## Definition of project success

Rust Kiosk is ready for its first stable release when a developer can:

1. Start with a clean, supported Raspberry Pi OS installation.
2. Install the required runtime dependencies.
3. Configure an arbitrary supported graphical executable.
4. Launch it fullscreen automatically at boot.
5. Access the device remotely.
6. Inspect the kiosk's health and logs.
7. Deploy an updated application.
8. Restore a previous release after a failed update.
9. Reproduce the setup using the documented procedure.

---

# 20. First Two Weeks: Concrete Work Plan

## Week 1 — Make the reference kiosk reliable

### Day 1: Hardware and OS

- Flash Raspberry Pi OS Lite 64-bit.
- Configure SSH and the kiosk user.
- Record OS version and architecture.
- Install Cage and Tauri runtime dependencies.

**Deliverable:** A clean reference device.

### Day 2: Application execution

- Build the Tauri app for Linux ARM64.
- Run it manually.
- Check the frontend and Rust backend.
- Resolve missing libraries and graphics issues.

**Deliverable:** A working fullscreen app launched manually.

### Day 3: Startup service

- Configure the graphical session.
- Create the systemd unit.
- Set the TTY and seat configuration.
- Start the application at boot.

**Deliverable:** Boot-to-app functionality.

### Day 4: Failure recovery

- Test application crashes.
- Test missing executable.
- Test service restart and shutdown.
- Verify SSH access during failures.

**Deliverable:** A repeatable recovery procedure.

### Day 5: Documentation

- Record the complete installation.
- Capture the final package list.
- Document the service configuration.
- Write the troubleshooting guide.

**Deliverable:** A reproducible manual setup.

## Week 2 — Build the reusable tooling

### Day 6: Rust CLI foundation

- Create the Cargo project.
- Add argument parsing and configuration dependencies.
- Implement `kiosk init`.
- Implement configuration validation.

### Day 7: Diagnostics

- Implement `kiosk doctor`.
- Add package, architecture, and application checks.
- Add useful error messages and exit codes.

### Day 8: Installation and lifecycle

- Generate the systemd configuration.
- Implement `kiosk install`.
- Add start, stop, restart, and status commands.
- Test repeated installations.

### Day 9: Logs and tests

- Implement `kiosk logs`.
- Add unit tests for configuration.
- Add integration tests for command behavior.
- Test dry-run and failure cases.

### Day 10: Clean installation

- Re-image the Pi.
- Follow only the written instructions and CLI.
- Identify undocumented manual steps.
- Fix reproducibility issues.
- Tag the first development release.

**Deliverable:** A working `0.1.0` candidate with automated installation and diagnostics.

Remote deployment and rollback should follow once this baseline is stable.

---

# 21. Technical Decisions to Record

Maintain an architecture decision log for decisions that are likely to affect future compatibility.

The first decisions should cover:

1. Why Raspberry Pi OS Lite is the reference OS.
2. Why Cage is the initial graphical backend.
3. Why systemd owns the service lifecycle.
4. Why TOML is used for configuration.
5. Why the CLI is implemented in Rust.
6. How the kiosk user and permissions are managed.
7. How application releases are staged and activated.
8. How persistent data is separated from application releases.
9. How rollback and health checks work.
10. What is required before supporting another OS or compositor.
11. Why both a GUI (Wayland/Cage) and a TUI (terminal) runtime are supported,
    and how the `ui` field selects the runtime adapter.

These records should explain the problem, the chosen solution, alternatives considered, and conditions that might justify revisiting the decision.

---

# 22. Future Possibilities

Only pursue these after the core runtime works.

## Remote fleet management

A future optional service could manage multiple kiosks, collect health reports, schedule updates, and report failures. The local kiosk should continue to work without it.

## Custom hardware profiles

Support reusable profiles for:

- Touchscreen dimensions.
- Audio devices.
- MIDI interfaces.
- Serial peripherals.
- USB devices.
- Display orientation.
- Input devices.

Hardware-specific configuration should remain separate from application logic.

## Reproducible images

Provide scripts that generate a prepared SD card image for a known hardware profile.

## Application bundles

Define an application bundle format containing:

- Executable or package.
- Metadata.
- Runtime requirements.
- Configuration schema.
- Version information.
- Optional health checks.

Keep this simple initially; a package manager is not needed until there is a clear use case.

## Rust-native applications

Provide a minimal Rust application template that integrates with the kiosk runtime and demonstrates display initialization, configuration, logging, and graceful shutdown.

---

# 23. Recommended Implementation Order

Do not implement every phase in parallel.

The order should be:

1. Make one Tauri application work on the Pi.
2. Make it launch reliably at boot.
3. Turn the working setup into documented scripts.
4. Build the Rust CLI around those scripts and systemd.
5. Add configuration validation and diagnostics.
6. Add remote deployment.
7. Add release management and rollback.
8. Harden the system and test failure scenarios.
9. Publish the first reusable release.
10. Consider custom images, extra backends, and remote fleet management.

The fundamental rule is to **automate a working system, rather than designing an abstract system and hoping it works on the hardware**.

---

# 24. Immediate Next Actions

Start with these tasks before writing the CLI:

- [ ] Create the `rust-kiosk` Git repository.
- [ ] Install Raspberry Pi OS Lite 64-bit on the Pi 4.
- [ ] Build and run one Tauri application under Cage.
- [ ] Verify the Rust backend and graphics dependencies.
- [ ] Configure automatic startup with systemd.
- [ ] Test crash recovery and SSH access.
- [ ] Write down the exact installation procedure.
- [ ] Use that procedure as the acceptance test for `kiosk install`.

Once these are complete, the first meaningful Rust implementation is `kiosk doctor`: a small, testable program that tells you whether the current device is ready to run the configured application.

That gives Rust Kiosk a narrow, useful foundation that can grow into a proper reusable deployment toolkit without becoming an unnecessarily complicated operating system project.
