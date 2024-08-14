use std::env;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpListener;

use board_server::{
    error::{AppError, Result},
    metrics,
    updown::startup,
    CONFIG,
};
use once_cell::sync::Lazy;
use sentry_wrapper::sentry;
use std::net::IpAddr;
use tracing_wrapper::tracing;

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
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), config.port);
    tracing::info!("Starting server on {}", addr);
    let listener = TcpListener::bind(&addr).or(Err(AppError::TcpBind))?;

    startup::run(listener)?
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Startup(e.to_string()))?;

    Ok(())
}

pub async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    metrics::set_variable_stop();
    tracing::info!("signal received, starting graceful shutdown");
}
