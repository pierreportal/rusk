//! Kiosk configuration model.
//!
//! This mirrors `config/kiosk.example.toml`. Unknown fields are rejected so a
//! typo is an error rather than a silently ignored setting.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{KioskError, Result};

/// Only schema version understood by this build.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Runtime adapter selected by the `ui` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    /// Wayland via Cage plus a webview/GPU client.
    #[default]
    Gui,
    /// Full-screen terminal program on a Linux virtual console.
    Tui,
    /// No display at all.
    Headless,
}

impl Runtime {
    /// Human-readable name used in diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Runtime::Gui => "gui",
            Runtime::Tui => "tui",
            Runtime::Headless => "headless",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RestartPolicy {
    #[serde(rename = "always")]
    #[default]
    Always,
    #[serde(rename = "on-failure")]
    OnFailure,
    #[serde(rename = "never")]
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub kiosk: Kiosk,
    pub application: Application,
    #[serde(default)]
    pub display: Option<Display>,
    #[serde(default)]
    pub terminal: Option<Terminal>,
    #[serde(default)]
    pub network: Network,
    #[serde(default)]
    pub deployment: Option<Deployment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kiosk {
    pub name: String,
    #[serde(default)]
    pub ui: Runtime,
    /// Compositor backend; only meaningful when `ui = "gui"`.
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default = "default_true")]
    pub fullscreen: bool,
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    #[serde(default = "default_restart_delay")]
    pub restart_delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub name: String,
    pub command: PathBuf,
    #[serde(default)]
    pub working_directory: Option<PathBuf>,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Display {
    #[serde(default = "default_true")]
    pub disable_screen_blanking: bool,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terminal {
    #[serde(default = "default_tty")]
    pub tty: String,
    #[serde(default = "default_term")]
    pub term: String,
    #[serde(default = "default_true")]
    pub disable_screen_blanking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(default = "default_true")]
    pub ssh_enabled: bool,
}

impl Default for Network {
    fn default() -> Self {
        Network { ssh_enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deployment {
    pub release_directory: PathBuf,
    #[serde(default = "default_keep_releases")]
    pub keep_releases: u32,
}

fn default_true() -> bool {
    true
}
fn default_restart_delay() -> u64 {
    2
}
fn default_keep_releases() -> u32 {
    3
}
fn default_tty() -> String {
    "tty1".to_string()
}
fn default_term() -> String {
    "linux".to_string()
}

impl Config {
    /// Load and parse a configuration file, without validating its contents.
    pub fn load(path: &Path) -> Result<Config> {
        if !path.exists() {
            return Err(KioskError::ConfigNotFound(path.to_path_buf()));
        }
        let text = std::fs::read_to_string(path).map_err(|source| KioskError::ConfigRead {
            path: path.to_path_buf(),
            source,
        })?;
        match Self::parse(&text) {
            Err(KioskError::ConfigParse { source, .. }) => Err(KioskError::ConfigParse {
                path: path.to_path_buf(),
                source,
            }),
            other => other,
        }
    }

    /// Parse a configuration from a TOML string.
    pub fn parse(text: &str) -> Result<Config> {
        toml::from_str(text).map_err(|source| KioskError::ConfigParse {
            path: PathBuf::from("<memory>"),
            source,
        })
    }

    /// The selected runtime adapter.
    pub fn runtime(&self) -> Runtime {
        self.kiosk.ui
    }

    /// Validate the configuration, returning every problem found.
    pub fn validate(&self) -> std::result::Result<(), Vec<String>> {
        let mut problems = Vec::new();

        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            problems.push(format!(
                "unsupported schema_version {} (this build supports {})",
                self.schema_version, SUPPORTED_SCHEMA_VERSION
            ));
        }

        if self.kiosk.name.trim().is_empty() {
            problems.push("kiosk.name must not be empty".to_string());
        }
        if self.application.name.trim().is_empty() {
            problems.push("application.name must not be empty".to_string());
        }
        if self.application.command.as_os_str().is_empty() {
            problems.push("application.command must not be empty".to_string());
        }

        match self.kiosk.ui {
            Runtime::Gui => {
                if let Some(backend) = &self.kiosk.backend {
                    if backend != "cage" {
                        problems.push(format!(
                            "unsupported kiosk.backend '{backend}' (only \"cage\" is supported)"
                        ));
                    }
                }
            }
            Runtime::Tui => match &self.terminal {
                Some(terminal) => {
                    if terminal.tty.trim().is_empty() {
                        problems.push("terminal.tty must not be empty".to_string());
                    }
                    if terminal.term.trim().is_empty() {
                        problems.push("terminal.term must not be empty".to_string());
                    }
                }
                None => {
                    problems.push("ui = \"tui\" requires a [terminal] section".to_string());
                }
            },
            Runtime::Headless => {}
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUI_EXAMPLE: &str = include_str!("../../../config/kiosk.example.toml");
    const TUI_EXAMPLE: &str = include_str!("../../../config/kiosk.tui.example.toml");

    #[test]
    fn example_gui_config_is_valid() {
        let config = Config::parse(GUI_EXAMPLE).unwrap();
        assert_eq!(config.schema_version, 1);
        assert_eq!(config.runtime(), Runtime::Gui);
        assert!(config.kiosk.fullscreen);
        assert_eq!(config.kiosk.restart_policy, RestartPolicy::Always);
        assert_eq!(config.application.name, "my-app");
        config.validate().unwrap();
    }

    #[test]
    fn example_tui_config_is_valid() {
        let config = Config::parse(TUI_EXAMPLE).unwrap();
        assert_eq!(config.runtime(), Runtime::Tui);
        config.validate().unwrap();
    }

    #[test]
    fn ui_defaults_to_gui() {
        let config = Config::parse(
            r#"
            schema_version = 1
            [kiosk]
            name = "k"
            [application]
            name = "a"
            command = "/bin/true"
            "#,
        )
        .unwrap();
        assert_eq!(config.runtime(), Runtime::Gui);
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let config = Config::parse(
            r#"
            schema_version = 99
            [kiosk]
            name = "k"
            [application]
            name = "a"
            command = "/bin/true"
            "#,
        )
        .unwrap();
        let err = config.validate().unwrap_err();
        assert!(err.iter().any(|p| p.contains("schema_version")));
    }

    #[test]
    fn tui_requires_terminal_section() {
        let config = Config::parse(
            r#"
            schema_version = 1
            [kiosk]
            name = "k"
            ui = "tui"
            [application]
            name = "a"
            command = "/bin/true"
            "#,
        )
        .unwrap();
        let err = config.validate().unwrap_err();
        assert!(err.iter().any(|p| p.contains("[terminal]")));
    }

    #[test]
    fn unsupported_backend_is_rejected_for_gui() {
        let config = Config::parse(
            r#"
            schema_version = 1
            [kiosk]
            name = "k"
            ui = "gui"
            backend = "weston"
            [application]
            name = "a"
            command = "/bin/true"
            "#,
        )
        .unwrap();
        let err = config.validate().unwrap_err();
        assert!(err.iter().any(|p| p.contains("backend")));
    }

    #[test]
    fn unknown_field_is_a_parse_error() {
        let err = Config::parse(
            r#"
            schema_version = 1
            [kiosk]
            name = "k"
            typo = true
            [application]
            name = "a"
            command = "/bin/true"
            "#,
        )
        .unwrap_err();
        assert!(matches!(err, KioskError::ConfigParse { .. }));
    }
}
