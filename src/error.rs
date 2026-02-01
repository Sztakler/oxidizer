use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxidizerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decoding failed: {0}")]
    Decoding(String),
    #[error("Encoding failed {0}")]
    Encoding(String),
    #[error("Symphonia error: {0}")]
    Symphonia(String),
}

pub type Result<T> = std::result::Result<T, OxidizerError>;
