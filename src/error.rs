use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Claude CLI error: {0}")]
    Claude(String),

    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid flag combination: {0}")]
    InvalidFlags(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
