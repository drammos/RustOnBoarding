use std::net::SocketAddr;

use crate::user_schema;

use crate::{
    error::{AppError, Result},
    handlers, metrics,
};
use user_schema::{Mutation, QueryRoot};
use {
    axum::{middleware, routing::IntoMakeService, Extension, Router, Server},
    hyper::server::conn::AddrIncoming,
    metrics_exporter_prometheus::PrometheusBuilder,
    sentry_wrapper::{NewSentryLayer, SentryHttpLayer},
    tower_http::trace::TraceLayer,
};

use async_graphql::{EmptySubscription, Schema};

pub fn run(
    listener: std::net::TcpListener,
    schema: Schema<QueryRoot, Mutation, EmptySubscription>,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let router = handlers::routes();
    let app = router
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(schema))
        .route_layer(middleware::from_fn(metrics::track_metrics));

    Ok(axum::Server::from_tcp(listener)
        .or(Err(AppError::TcpBind))?
        .serve(app.into_make_service()))
}

/// Start the metrics server at the given address and explain the known metrics
pub fn start_metrics_server(addr: impl Into<SocketAddr>) -> Result<()> {
    // Start the server for metrics
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()?;

    // Describe generic metrics
    metrics::describe_counter!(
        metrics::REQUESTS_COUNTER,
        "How many requests are done where"
    );
    metrics::describe_histogram!(
        metrics::REQUESTS_DURATION,
        "How long requests take to complete"
    );

    Ok(())
}
