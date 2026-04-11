use thiserror::Error;

pub type Result<T> = std::result::Result<T, CinemaError>;

#[derive(Error, Debug)]
pub enum CinemaError {
    #[error("State error: {0}")]
    State(String),
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
