use std::error::Error;

use {
    axum::{
        async_trait,
        extract::{rejection::JsonRejection, FromRequest, RequestParts},
        http::StatusCode,
        response::{IntoResponse, Response},
        BoxError,
    },
    serde::{de::DeserializeOwned, Serialize},
    serde_json::{json, Value},
};

/// A custom `Json` Extractor / Response that allows as to customize the error message if it fails
pub struct Json<T>(pub T);

#[async_trait]
impl<B, T> FromRequest<B> for Json<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::Json`
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, err_message) = match rejection {
                    JsonRejection::JsonDataError(err) => {
                        let msg = if let Some(inner) = err.source() {
                            format!("Invalid JSON request: {}, {}", err, inner)
                        } else {
                            format!("Invalid JSON request: {}", err)
                        };
                        (StatusCode::BAD_REQUEST, msg)
                    }
                    JsonRejection::MissingJsonContentType(err) => {
                        (StatusCode::BAD_REQUEST, err.to_string())
                    }
                    err => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unknown internal error: {}", err),
                    ),
                };

                let body = axum::Json(json!({ "error": err_message }));
                Err((status, body))
            }
        }
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
