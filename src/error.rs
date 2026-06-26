use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Screen capture failed: {0}")]
    Capture(String),
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
