use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KioskError {
    #[error("configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("failed to read {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("invalid configuration:\n  - {}", .0.join("\n  - "))]
    ConfigInvalid(Vec<String>),

    #[error("{0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, KioskError>;
