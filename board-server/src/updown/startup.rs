use axum::Extension;
use std::net::SocketAddr;

use crate::mongo::client;
use {
    axum::{middleware, routing::IntoMakeService, Router, Server},
    hyper::server::conn::AddrIncoming,
    metrics_exporter_prometheus::PrometheusBuilder,
    sentry_wrapper::{NewSentryLayer, SentryHttpLayer},
    tower_http::trace::TraceLayer,
};

use axum::body::BoxBody;
use http::Response;
use std::time::Duration;
use tracing::Span;

use hyper::{Body, Request};
use sentry_wrapper::sentry::configure_scope;
use tower_http::trace::DefaultOnRequest;
use tower_request_id::{RequestId, RequestIdLayer};
use tracing_wrapper::tracing::{self, Level};

use crate::{
    error::{AppError, Result},
    handlers, metrics,
};

pub fn run(
    listener: std::net::TcpListener, // db_con: &'static mongo::client::Mongod,
) -> Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let db_con = client::Mongod::new()?;
    let router = handlers::routes();
    let app = router
        .layer(NewSentryLayer::new_from_top())
        .layer(SentryHttpLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "unknown".into());
                    let uri = request.uri().to_string();
                    configure_scope(|scope| {
                        scope.set_tag("url", uri);
                    });
                    tracing::info_span!(
                        "request",
                        id = %request_id,
                        method = %request.method(),
                        uri = %request.uri()
                    )
                })
                .on_request(DefaultOnRequest::new().level(Level::INFO)), // .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO))
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|_request: &Request<Body>| tracing::info_span!("response",))
                .on_response(
                    |_response: &Response<BoxBody>, _latency: Duration, _span: &Span| {
                        tracing::info!("response generated")
                    },
                ),
        )
        .layer(RequestIdLayer)
        .layer(Extension(db_con))
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
