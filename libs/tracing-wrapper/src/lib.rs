#![doc = include_str!("../README.md")]

use sentry::integrations::tracing::EventFilter;
pub use tracing;
#[allow(unused_imports)] // Used when compiling as release
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    fmt,
    prelude::__tracing_subscriber_SubscriberExt,
    util::{SubscriberInitExt, TryInitError},
    EnvFilter, Registry,
};

/// This is used for configuring [`tracing_subscriber`]
#[derive(Debug)]
pub struct Logger {
    #[allow(dead_code)] // Used when compiling as release
    /// This is used with bunyan json layer
    app_name: String,
    /// Minimum log level to use
    log_level: String,
    /// Whether or not to report breadcrumbs and events to sentry
    enable_sentry: bool,
    /// How to report tracing errors to sentry (Event or Breadcrumb)
    tracing_error_filter: EventFilter,
}

impl Logger {
    /// Returns a new Logger with defaults
    ///
    /// `app_name` is used with bunyan json layer
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            log_level: std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("trace")),
            enable_sentry: false,
            tracing_error_filter: EventFilter::Breadcrumb,
        }
    }

    /// Initialise Logger as global default
    pub fn init(self) -> Result<(), TryInitError> {
        let subscriber = Registry::default().with(EnvFilter::new(&self.log_level));
        #[cfg(debug_assertions)]
        let subscriber = subscriber.with(fmt::layer().with_target(true));

        #[cfg(not(debug_assertions))]
        let subscriber = subscriber
            .with(JsonStorageLayer)
            .with(BunyanFormattingLayer::new(self.app_name, std::io::stdout));

        #[cfg(feature = "enable_sentry")]
        if self.enable_sentry {
            subscriber
                .with(
                    sentry::integrations::tracing::layer().event_filter(move |md| {
                        match *md.level() {
                            tracing::Level::ERROR => self.tracing_error_filter,
                            _ => EventFilter::Breadcrumb,
                        }
                    }),
                )
                .try_init()
        } else {
            subscriber.try_init()
        }
        #[cfg(not(feature = "enable_sentry"))]
        subscriber.try_init()
    }

    /// Set the logger's env filter var.
    /// Defaults to the env variable `RUST_LOG`, or `"trace"`
    pub fn set_log_level(mut self, log_level: String) -> Self {
        self.log_level = log_level;
        self
    }

    /// Enable/Disable sentry tracing integration.
    /// Defaults to disabled
    #[cfg(feature = "enable_sentry")]
    pub fn with_sentry(mut self, enable_sentry: bool) -> Self {
        self.enable_sentry = enable_sentry;
        self
    }

    ///  How to report tracing errors to sentry
    ///
    /// When `true` report them as Events, otherwise Breadcrumbs.
    /// Defaults to Breadcrumbs
    #[cfg(feature = "enable_sentry")]
    pub fn report_with_tracing(mut self, report_with_tracing: bool) -> Self {
        self.tracing_error_filter = if report_with_tracing {
            EventFilter::Event
        } else {
            EventFilter::Breadcrumb
        };
        self
    }
}
