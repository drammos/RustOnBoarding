use {
    std::error::Error,
    std::fmt::{Display, Formatter},
};

pub use sentry;
pub use sentry::release_name;
pub use sentry::Level;
use sentry::{ClientInitGuard, ClientOptions};
pub use sentry_tower::NewSentryLayer;
pub use sentry_tower::SentryHttpLayer;
use {http::HeaderMap, serde_json::Value};

#[derive(Copy, Clone)]
pub enum AlertType {
    Low,
    Medium,
    Critical,
}

impl Display for AlertType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::Low => write!(f, "LOW"),
            AlertType::Medium => write!(f, "MEDIUM"),
            AlertType::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Try to extract common headers that we care about:
/// * x-wappier-id,
/// * user-agent
/// * x-wappier-device
///
/// If their values don't contain valid utf-8, the invalid bytes are replaced with `�`
pub fn extract_common_headers(headers: &HeaderMap) -> Vec<(&'static str, String)> {
    let mut res = Vec::with_capacity(3);

    for key in ["x-wappier-id", "user-agent", "x-wappier-device"] {
        if let Some(val) = headers.get(key) {
            let val = String::from_utf8_lossy(val.as_bytes());
            res.push((key, val.to_string()));
        }
    }

    res
}

pub fn init(dsn: Option<&str>, options: ClientOptions) -> ClientInitGuard {
    sentry::init((dsn, options))
}

#[derive(Clone)]
pub struct ErrorReport<'a, E: Error + ?Sized> {
    error: &'a E,
    level: Option<Level>,
    alert: AlertType,
    tags: Vec<(String, String)>,
    extras: Vec<(String, Value)>,
}

impl<'a, E> ErrorReport<'a, E>
where
    E: Error + ?Sized,
{
    pub fn new(error: &'a E) -> Self {
        Self {
            error,
            level: Some(Level::Warning),
            alert: AlertType::Low,
            tags: vec![],
            extras: vec![],
        }
    }

    /// Defaults to `None`
    pub fn set_level(mut self, level: Option<Level>) -> Self {
        self.level = level;
        self
    }

    /// Defaults to `Warning`
    pub fn set_alert(mut self, alert: AlertType) -> Self {
        self.alert = alert;
        self
    }

    pub fn add_tag<T: ToString, V: ToString>(mut self, tag: T, value: V) -> Self {
        self.tags.push((tag.to_string(), value.to_string()));
        self
    }

    /// Appends tags
    pub fn add_tags<T: ToString, V: ToString, I: IntoIterator<Item = (T, V)>>(
        mut self,
        tags: I,
    ) -> Self {
        let mut tags = tags
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>();
        self.tags.append(&mut tags);
        self
    }

    pub fn add_extra<K: ToString, V: Into<Value>>(mut self, key: K, value: V) -> Self {
        self.extras.push((key.to_string(), value.into()));
        self
    }

    /// Appends extras
    pub fn add_extras<K: ToString, V: Into<Value>, I: IntoIterator<Item = (K, V)>>(
        mut self,
        extras: I,
    ) -> Self {
        let mut extras = extras
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect::<Vec<_>>();
        self.extras.append(&mut extras);
        self
    }

    pub fn send(self) -> uuid::Uuid {
        sentry::with_scope(
            |scope| {
                scope.clear();
                // ToDo: If `level` is `None` should it be selected from a mapping from `AlertType`?
                scope.set_level(self.level);
                scope.set_tag("alert", self.alert);

                for (tag, value) in self.tags {
                    scope.set_tag(&tag, value);
                }
                if !self.extras.is_empty() {
                    let mut map = std::collections::BTreeMap::new();
                    for (extra, value) in self.extras {
                        map.insert(extra, value);
                    }
                    scope.set_context("character", sentry::protocol::Context::Other(map));
                }
            },
            || sentry::capture_error(self.error),
        )
    }
}
