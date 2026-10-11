use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Rusk — manage a single-purpose Raspberry Pi kiosk.
#[derive(Debug, Parser)]
#[command(name = "kiosk", version, about, long_about = None)]
pub struct Cli {
    /// Path to the kiosk configuration file.
    #[arg(long, global = true, default_value = "kiosk.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a kiosk configuration file.
    Init(InitArgs),
    /// Diagnose whether this device can run the configured application.
    Doctor(DoctorArgs),
    /// Install the kiosk service so it starts on boot.
    Install(InstallArgs),
    /// Remove the kiosk service without deleting application data.
    Uninstall(UninstallArgs),
    /// Start the kiosk service.
    Start,
    /// Stop the kiosk service.
    Stop,
    /// Restart the kiosk service.
    Restart,
    /// Report kiosk service and release status.
    Status(StatusArgs),
    /// Show recent kiosk logs from the system journal.
    Logs(LogsArgs),
    /// Deploy an application release to a target device.
    Deploy(DeployArgs),
    /// Activate a previous release.
    Rollback(RollbackArgs),
    /// Configuration utilities.
    Config {
        #[command(subcommand)]
        command: ConfigCmd,
    },
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Kiosk name.
    #[arg(long)]
    pub name: Option<String>,
    /// Path to the application executable.
    #[arg(long)]
    pub app: Option<PathBuf>,
    /// Overwrite an existing configuration file.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Include detailed checks.
    #[arg(long)]
    pub verbose: bool,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Show what would change without modifying the system.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Also delete deployed releases (never deletes application data).
    #[arg(long)]
    pub remove_releases: bool,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Number of recent lines to show.
    #[arg(short = 'n', long, default_value_t = 100)]
    pub lines: u32,
    /// Follow the journal.
    #[arg(long)]
    pub follow: bool,
}

#[derive(Debug, Args)]
pub struct DeployArgs {
    /// Target device, e.g. kiosk.local.
    #[arg(long)]
    pub host: String,
    /// Path to the built release directory.
    #[arg(long)]
    pub release: PathBuf,
}

#[derive(Debug, Args)]
pub struct RollbackArgs {
    /// Release identifier to activate (defaults to the previous release).
    #[arg(long)]
    pub to: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Validate a configuration file.
    Validate,
}
