# Tracing Wrapper

## Usage

```rust
// In main
tracing_wrapper::Logger::new(concat!(env!("CARGO_PKG_NAME"), "_", env!("CARGO_PKG_VERSION")))
     .with_sentry(true)
     .report_with_tracing(true)
     .init()
     .unwrap();

// Anywhere
use tracing_wrapper::tracing;

tracing::info!("info log"); // Used as breadcrumb in sentry
tracing::error!("error log"); // Used as event in sentry
```

## With axum
When integrating with axum it is suggested you add a few layers to improve logging and error reporting:
```rust,ignore
use {
    sentry_tower::{NewSentryLayer, SentryHttpLayer},
    tower_http::trace::TraceLayer,
};

let app = Router::new()
    // Your routes
    .layer(NewSentryLayer::new_from_top()) // Bind a new sentry hub for each request
    .layer(SentryHttpLayer::new()) // Log http headers
    .layer(TraceLayer::new_for_http());
```
