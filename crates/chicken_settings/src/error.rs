use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in the settings system.
#[derive(Error, Debug)]
pub enum SettingsError {
    /// I/O error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error during serialization.
    #[error("Failed to serialize settings: {0}")]
    Serialization(String),

    /// Error during deserialization.
    #[error("Failed to deserialize settings: {0}")]
    Deserialization(String),

    /// Path resolution failed due to missing placeholder value.
    #[error("Path resolution failed: missing placeholder '{0}'")]
    PathResolution(String),

    /// Settings file not found at the specified path.
    #[error("Settings file not found: {0}")]
    NotFound(PathBuf),

    /// Unsupported file format.
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    /// Invalid file extension.
    #[error("Could not determine format from file extension: {0}")]
    InvalidExtension(String),

    /// Settings type not registered.
    #[error("Settings type not registered: {0}")]
    NotRegistered(String),

    /// Delta storage error.
    #[error("Delta storage error: {0}")]
    DeltaError(String),

    /// Runtime error.
    #[error("Runtime error: {0}")]
    Runtime(String),
}

/// Result type alias for settings operations.
pub type SettingsResult<T> = Result<T, SettingsError>;
