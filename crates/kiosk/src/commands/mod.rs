//! Command dispatch.
//!
//! Phase 3 implements the operational commands. For now only configuration
//! validation is wired up, so the CLI shape is real but the system-changing
//! commands report that they are not implemented rather than doing nothing.

use crate::cli::{Cli, Command, ConfigCmd};
use crate::config::Config;
use crate::error::{KioskError, Result};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Config { command } => match command {
            ConfigCmd::Validate => validate(&cli.config),
        },
        Command::Init(_) => not_implemented("init"),
        Command::Doctor(_) => not_implemented("doctor"),
        Command::Install(_) => not_implemented("install"),
        Command::Uninstall(_) => not_implemented("uninstall"),
        Command::Start => not_implemented("start"),
        Command::Stop => not_implemented("stop"),
        Command::Restart => not_implemented("restart"),
        Command::Status(_) => not_implemented("status"),
        Command::Logs(_) => not_implemented("logs"),
        Command::Deploy(_) => not_implemented("deploy"),
        Command::Rollback(_) => not_implemented("rollback"),
    }
}

fn validate(path: &std::path::Path) -> Result<()> {
    let config = Config::load(path)?;
    config.validate().map_err(KioskError::ConfigInvalid)?;
    println!(
        "{}: valid (schema_version = {}, ui = {})",
        path.display(),
        config.schema_version,
        config.runtime().as_str()
    );
    Ok(())
}

fn not_implemented(command: &str) -> Result<()> {
    Err(KioskError::NotImplemented(format!(
        "`kiosk {command}` is not implemented yet (planned for Phase 3); see PLAN.md"
    )))
}
