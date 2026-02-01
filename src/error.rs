use thiserror::Error;

/// Custom error type for the Oxidizer library.
/// It wraps various error sources like I/O, audio decoding, and encoding.
#[derive(Error, Debug)]
pub enum OxidizerError {
    /// Errors related to file system operations (e.g., file not found, permissions).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Used when a parameter or input value is out of range or malformed.
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Errors occurring during the decoding of raw audio packets.
    #[error("Decoding failed: {0}")]
    Decoding(String),

    /// Errors occurring when writing the processed data back to a file.
    #[error("Encoding failed {0}")]
    Encoding(String),

    /// Errors passed through from the Symphonia multimedia framework.
    #[error("Symphonia error: {0}")]
    Symphonia(String),
}

/// A specialized Result type for Oxidizer operations.
pub type Result<T> = std::result::Result<T, OxidizerError>;
