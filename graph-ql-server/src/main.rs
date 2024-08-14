use std::net::TcpListener;

use once_cell::sync::Lazy;
use sentry_wrapper::sentry;
use tracing_wrapper::tracing;

use async_graphql::{EmptySubscription, Schema};
use graph_ql_server::{
    error::{AppError, Result},
    metrics,
    mongo::client::{self},
    updown::{shutdown, startup},
    user_schema::{Mutation, QueryRoot},
    CONFIG,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_wrapper::Logger::new(concat!(
        env!("CARGO_PKG_NAME"),
        "_",
        env!("CARGO_PKG_VERSION")
    ))
    .with_sentry(true)
    .report_with_tracing(true)
    .init()
    .unwrap();

    let config = Lazy::force(&CONFIG)
        .as_ref()
        .map_err(|err| AppError::Startup(err.to_string()))?;

    let _sentry_guard = sentry_wrapper::init(
        config.sentry_key.as_deref(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            attach_stacktrace: true,
            ..Default::default()
        },
    );

    startup::start_metrics_server(([0, 0, 0, 0], config.metrics_port))?;
    metrics::track_system_metrics();
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = TcpListener::bind(&addr).or(Err(AppError::TcpBind))?;
    //
    //connection with database
    let db_con = client::Mongod::new()?;

    let schema = Schema::build(QueryRoot, Mutation, EmptySubscription)
        .data(db_con)
        .finish();

    //build schema

    startup::run(listener, schema)?
        .with_graceful_shutdown(shutdown::signal())
        .await
        .map_err(|e| AppError::Startup(e.to_string()))?;

    Ok(())
}
