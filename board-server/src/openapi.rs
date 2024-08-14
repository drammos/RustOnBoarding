use utoipa::{openapi, OpenApi};

use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    handlers(
        handlers::health_check
    ),
    tags(
        (name = "server")
    )
)]
struct ApiDoc;

pub fn gen_openapi() -> openapi::OpenApi {
    ApiDoc::openapi()
}
