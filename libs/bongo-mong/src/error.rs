//! Library specific error types.

#[derive(thiserror::Error, Debug)]
pub enum BongoError {
    #[error("{0}")]
    MongoDbUriCreate(String),
    #[error("{0}")]
    MongoDbError(#[from] mongodb::error::Error),
    #[error("{0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("{0}")]
    DaoError(String),
    #[error("option {0} not supported")]
    UnsupportedOption(String),
    #[error("unknown permission type {0}")]
    UnknownPermission(String),
}

pub type Result<T> = std::result::Result<T, BongoError>;
