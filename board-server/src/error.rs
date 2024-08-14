use {
    axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
    },
    serde_json::json,
};

use crate::Json;

/// Alias for `std::result::Result` with an error type [`AppError`].
pub type Result<T> = std::result::Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Value not found")]
    NotFound,
    #[error("Configuration Error {0}")]
    Config(#[from] config::ConfigError),
    #[error("Could not bind server to tcp address")]
    TcpBind,
    #[error("Could not start service: {0}")]
    Startup(String),
    #[error("Failed to build the Prometheus metrics exporter")]
    Prometheus(#[from] metrics_exporter_prometheus::BuildError),

    ////
    #[error("Mongo error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[error("Bongo error: {0}")]
    Bongo(#[from] bongo_mong::error::BongoError),

    #[error("Not found user with id: {0}")]
    User(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, err_message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            Self::Mongo(_)
            | Self::Bongo(_)
            | Self::User(_)
            | Self::Config(_)
            | Self::TcpBind
            | Self::Startup(_)
            | Self::Prometheus(_) => {
                unreachable!("This error can only occur during startup")
            }
        };

        let body = Json(json!({ "error": err_message }));
        (status, body).into_response()
    }
}
