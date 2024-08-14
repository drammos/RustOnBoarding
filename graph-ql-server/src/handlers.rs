use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Request, Response,
};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Json, Router,
};

// use crate::user_schema;
use crate::user_schema::{Mutation, QueryRoot};

use async_graphql::{EmptySubscription, Schema};
pub fn routes() -> Router {
    Router::new()
        .route("/health-check", get(health_check))
        .route(
            "/api/graphql",
            get(graphql_playground).post(graphql_handler),
        )
}

#[utoipa::path(
    get,
    path = "/health-check", 
    responses(
        (status = 200, description = "The service is online", body = String)
    )
)]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Service is healthy")
}

pub async fn graphql_handler(
    Extension(schema): Extension<Schema<QueryRoot, Mutation, EmptySubscription>>,
    Json(request): Json<Request>,
) -> Json<Response> {
    schema.execute(request).await.into()
}
pub async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new(
        "/api/graphql",
    )))
}
