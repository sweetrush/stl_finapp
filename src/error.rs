use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CLI error: {0}")]
    Cli(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Client error: {0}")]
    Client(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Io(_) => 1,
            AppError::Cli(_) => 2,
            AppError::Crypto(_) => 3,
            AppError::Auth(_) => 4,
            AppError::Protocol(_) => 5,
            AppError::Server(_) => 6,
            AppError::Client(_) => 7,
            AppError::Config(_) => 8,
            AppError::Serialization(_) => 9,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
