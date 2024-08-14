use sentry::integrations::tracing as sentry_tracing;
use sentry::integrations::tracing::EventFilter;
use std::{env, io};
use tracing::{dispatcher::SetGlobalDefaultError, Metadata, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    fmt::{self, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    util::{SubscriberInitExt, TryInitError},
    EnvFilter, Registry,
};

pub fn init() {
    let asd = |md: &Metadata| match *md.level() {
        tracing::Level::ERROR => EventFilter::Event,
        tracing::Level::WARN | tracing::Level::INFO => EventFilter::Breadcrumb,
        tracing::Level::DEBUG | tracing::Level::TRACE => EventFilter::Ignore,
    };
    let app_name = concat!(env!("CARGO_PKG_NAME"), "_", env!("CARGO_PKG_VERSION"));
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| String::from("DEBUG"));
    let asd = fmt::layer().with_target(true);
    let subscriber = Registry::default()
        .with(asd)
        .with(
            sentry_tracing::layer().event_filter(|md| match *md.level() {
                tracing::Level::ERROR => EventFilter::Event,
                tracing::Level::WARN | tracing::Level::INFO => EventFilter::Breadcrumb,
                tracing::Level::DEBUG | tracing::Level::TRACE => EventFilter::Ignore,
            }),
        )
        .with(EnvFilter::new(rust_log))
        .with(JsonStorageLayer)
        .with(BunyanFormattingLayer::new(app_name.to_owned(), io::stdout));
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub struct Logger {
    app_name: String,
    env_filter_var: String,
    json_layer: Option<JsonStorageLayer>,
    plain_layer: Option<Layer<Registry>>,
    bunyan_layer: Option<BunyanFormattingLayer<fn() -> io::Stdout>>,
    env_filter_layer: Option<EnvFilter>,
    sentry_layer: Option<sentry_tracing::SentryLayer<Registry>>,
}

impl std::fmt::Debug for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Logger")
            .field("app_name", &self.app_name)
            .field("env_filter_var", &self.env_filter_var)
            .field("json_layer", &self.json_layer)
            .field("plain_layer", &self.plain_layer)
            .field("bunyan_layer", &"&self.bunyan_layer")
            .field("env_filter_layer", &self.env_filter_layer)
            .field("sentry_layer", &"&self.sentry_layer")
            .finish()
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            app_name: concat!(env!("CARGO_PKG_NAME"), "_", env!("CARGO_PKG_VERSION")).to_string(),
            env_filter_var: env::var("RUST_LOG").unwrap_or_else(|_| String::from("trace")),
            bunyan_layer: None,
            json_layer: None,
            env_filter_layer: None,
            sentry_layer: None,
            plain_layer: None,
        }
    }
}

impl Logger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialise Tracing
    pub fn init(self) -> Result<(), TryInitError> {
        let subscriber = Registry::default();

        #[cfg(feature = "enable_sentry")]
        {
            let ad = fmt::layer().with_target(true);
            subscriber
                .with(self.sentry_layer)
                .with(ad)
                .with(self.env_filter_layer)
                .with(self.json_layer)
                .with(self.bunyan_layer)
                .try_init()
        }
        #[cfg(not(feature = "enable_sentry"))]
        {
            subscriber
                .with(self.env_filter_layer)
                .with(self.json_layer)
                .with(self.bunyan_layer)
                .try_init()
        }
        // tracing::subscriber::set_global_default(subscriber)
    }

    pub fn enable_all(self) -> Result<(), TryInitError> {
        // #[cfg(feature = "enable_sentry")]
        // {
        //     self.enable_env_filter()
        //         .enable_sentry()
        //         .enable_plain()
        //         // .enable_json()
        //         .init()
        // }

        // #[cfg(not(feature = "enable_sentry"))]
        {
            self.enable_plain().enable_env_filter().init()
        }
    }

    #[cfg(feature = "enable_sentry")]
    pub fn enable_sentry(mut self) -> Self {
        self.sentry_layer = Some(
            sentry_tracing::layer().event_filter(|md| match *md.level() {
                tracing::Level::ERROR => EventFilter::Event,
                tracing::Level::WARN | tracing::Level::INFO => EventFilter::Breadcrumb,
                tracing::Level::DEBUG | tracing::Level::TRACE => EventFilter::Ignore,
            }),
        );
        self
    }

    pub fn enable_plain(mut self) -> Self {
        self.plain_layer = Some(fmt::layer().with_target(true));
        self
    }

    pub fn enable_json(mut self) -> Self {
        self.json_layer = Some(JsonStorageLayer);
        self.bunyan_layer = Some(BunyanFormattingLayer::new(
            self.app_name.clone(),
            io::stdout,
        ));
        self
    }

    pub fn enable_env_filter(mut self) -> Self {
        self.env_filter_layer = Some(EnvFilter::new(&self.env_filter_var));
        self
    }

    /// Set the logger's app name.
    pub fn set_app_name(mut self, app_name: String) -> Self {
        self.app_name = app_name;
        self
    }

    /// Set the logger's env filter var.
    pub fn set_env_filter_var(mut self, env_filter_var: String) -> Self {
        self.env_filter_var = env_filter_var;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        println!("\n\n");
        let result = 2 + 2;
        println!("asdasd");
        tracing::info!("INFO");
        Logger::new().enable_all().unwrap();
        tracing::debug!("DEBUG");
        tracing::info!("INFO");
        assert_eq!(result, 4);
        println!("\n\n");
    }
}
