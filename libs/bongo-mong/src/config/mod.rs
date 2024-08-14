//! Utilities to handle Mongo DB configurations.
//!
//! Relies on the `config` crate, and thus the configuration format may be in any form supported by
//! the crate.
pub mod options;
pub mod path;

use super::error::Result;
use config::{Config, ConfigError, Value};
use options::{BongoClientOptions, BongoOptions, PoolPermissionType};
use path::{AppPath, CollectionPath, ConfigPath, PermissionPath};
use std::collections::HashMap;

type LeafConfig = HashMap<String, Value>;
type PermissionConfig = HashMap<PoolPermissionType, LeafConfig>;
type CollectionConfig = HashMap<String, PermissionConfig>;
type AppConfig = HashMap<String, CollectionConfig>;

/// Configuration repository for the pool manager.
#[derive(Clone, Debug, Default)]
pub struct BongoConfig {
    pub src: Config,
    global: Option<PermissionConfig>,
    collections: Option<CollectionConfig>,
    app: Option<AppConfig>,
}

impl BongoConfig {
    /// Create a new configuration.
    ///
    /// # Errors
    ///
    /// The construction performs some type validations according to the
    /// prescribed schema, that is of the following form:
    ///
    /// ```json
    /// {
    ///     "read" : {
    ///         "maxPoolSize": "15"
    ///     },
    ///     "redemptions": {
    ///         "read" : {
    ///             "baseUri": "mongodb://wat.com/",
    ///             "maxPoolSize": "10",
    ///             "connectTimeoutMS": "15"
    ///         }
    ///     },
    ///     "mongodbPerApp": {
    ///         "api-key": {
    ///             "installations": {
    ///                 "read" : {
    ///                     "maxPoolSize": "100",
    ///                     "connectTimeoutMS": "150"
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn new(config: Config) -> Result<Self> {
        let mut config = Self {
            src: config,
            ..Default::default()
        };
        // Cache configurations as maps
        let mut cfg_map: LeafConfig = config.src.clone().try_deserialize()?;

        // Cache global configuration
        let mut global: PermissionConfig = Default::default();
        for permission in [PoolPermissionType::Read, PoolPermissionType::Write] {
            if let Some(cfg) = cfg_map.remove(&permission.to_lowercase_string()) {
                global.insert(permission, cfg.try_deserialize()?);
            }
        }
        if !global.is_empty() {
            config.global = Some(global);
        }
        // Cache per-app configuration
        if let Some(cfg) = cfg_map.remove("mongodbPerApp") {
            config.app = Some(cfg.try_deserialize()?);
        }
        // Cache collections
        if !cfg_map.is_empty() {
            let mut collections: CollectionConfig = Default::default();
            for (k, v) in cfg_map {
                collections.insert(k, v.try_deserialize()?);
            }
            config.collections = Some(collections);
        }
        Ok(config)
    }

    /// Resolve configuration path based on set of given parameters.
    ///
    /// The method checks if the application path exists, othewise it checks
    /// if the collection path exists.
    ///
    /// If none of the above is found it falls back to the global path.
    pub fn resolve_path(
        &self,
        api_key: Option<impl AsRef<str>>,
        collection: impl AsRef<str>,
        permission: PoolPermissionType,
    ) -> ConfigPath {
        let path = if let Some(api_key) = api_key {
            ConfigPath::from(AppPath::new(api_key, collection, permission))
        } else {
            ConfigPath::from(CollectionPath::new(collection, permission))
        };
        match path {
            ConfigPath::Collection(ref p) => {
                if self.contains_collection(p) {
                    path
                } else {
                    ConfigPath::from(PermissionPath::new(permission))
                }
            }
            ConfigPath::App(ref p) => {
                if self.contains_app(p) {
                    path
                } else if self.contains_collection(p.collection_path()) {
                    ConfigPath::from(p.collection_path().clone())
                } else {
                    ConfigPath::from(PermissionPath::new(permission))
                }
            }
            _ => unreachable!(),
        }
    }

    /// Check if the configuration contains the global options
    /// for the given path.
    pub fn contains_global(&self, path: &PermissionPath) -> bool {
        if let Some(cfg) = self.global() {
            return cfg.contains_key(&path.permission());
        }
        false
    }

    /// Check if the configuration contains the collection options
    /// for the given path.
    pub fn contains_collection(&self, path: &CollectionPath) -> bool {
        if let Some(cfg) = self.collections() {
            let collection = path.collection().to_string();
            return cfg.contains_key(&collection)
                && cfg
                    .get(&collection)
                    .unwrap()
                    .contains_key(&path.permission_path().permission());
        }
        false
    }

    /// Check if the configuration contains the application options
    /// for the given path.
    pub fn contains_app(&self, path: &AppPath) -> bool {
        if let Some(cfg) = self.app() {
            let api_key = path.api_key().to_string();
            if cfg.contains_key(&api_key) {
                let path = path.collection_path();
                let collection = path.collection().to_string();
                let cfg = cfg.get(&api_key).unwrap();
                if cfg.contains_key(&collection) {
                    let path = path.permission_path();
                    return cfg
                        .get(&collection)
                        .unwrap()
                        .contains_key(&path.permission());
                }
            }
            return false;
        }
        false
    }

    /// Get configurations corresponding to collections.
    pub fn collections(&self) -> Option<&CollectionConfig> {
        self.collections.as_ref()
    }

    /// Get the configuration per application.
    pub fn app(&self) -> Option<&AppConfig> {
        self.app.as_ref()
    }

    /// Get the global configuration.
    pub fn global(&self) -> Option<&PermissionConfig> {
        self.global.as_ref()
    }

    /// Get the cached configuration.
    pub fn cache(&self) -> &config::Value {
        &self.src.cache
    }

    /// Get global configuration for the given permission path.
    fn global_config(&self, path: &PermissionPath) -> Result<&LeafConfig> {
        Ok(self
            .global()
            .ok_or_else(|| ConfigError::NotFound("global configuration missing".into()))?
            .get(&path.permission())
            .ok_or_else(|| {
                ConfigError::NotFound(format!(
                    "missing global configuration for {}",
                    path.permission()
                ))
            })?)
    }

    /// Get collections configuration for the given collection path.
    fn collection_config(&self, path: &CollectionPath) -> Result<&LeafConfig> {
        Ok(self
            .collections()
            .ok_or_else(|| {
                ConfigError::NotFound("collection-specific configuration missing".into())
            })?
            .get(path.collection())
            .ok_or_else(|| {
                ConfigError::NotFound(format!(
                    "missing configuration for {} in collections",
                    path.collection()
                ))
            })?
            .get(&path.permission_path().permission())
            .ok_or_else(|| {
                ConfigError::NotFound(format!(
                    "missing configuration for {} permission in {}",
                    path.permission_path().permission(),
                    path.collection()
                ))
            })?)
    }

    /// Get collection configuration for the given applicatin path.
    fn app_config(&self, path: &AppPath) -> Result<&LeafConfig> {
        let api_key = path.api_key();
        let collection = path.collection_path().collection();
        let permission = path.collection_path().permission_path().permission();
        Ok(self
            .app()
            .ok_or_else(|| ConfigError::NotFound("per-app configuration missing".into()))?
            .get(api_key)
            .ok_or_else(|| ConfigError::NotFound(format!("missing configuration for {}", api_key)))?
            .get(collection)
            .ok_or_else(|| {
                ConfigError::NotFound(format!(
                    "missing configuration for {} in {}",
                    collection, api_key
                ))
            })?
            .get(&permission)
            .ok_or_else(|| {
                ConfigError::NotFound(format!(
                    "missing configuration for {} permission in {}-{}",
                    permission, api_key, collection
                ))
            })?)
    }

    /// Merge base values onto a configuration
    fn merge_configs(base: Option<LeafConfig>, config: LeafConfig) -> LeafConfig {
        if let Some(mut base) = base {
            for (k, v) in config {
                base.insert(k, v);
            }
            base
        } else {
            config
        }
    }

    /// Get the global pool options for the given permission path.
    pub async fn to_global_opts(&self, path: &PermissionPath) -> Result<BongoClientOptions> {
        let opts: BongoOptions = self.global_config(path)?.clone().try_into()?;
        BongoClientOptions::try_from_bongo_options(opts).await
    }

    /// Get the pool options for the given collection path.
    pub async fn to_collection_opts(&self, path: &CollectionPath) -> Result<BongoClientOptions> {
        let global_config = self.global_config(path.permission_path()).ok().cloned();
        let key_value = Self::merge_configs(global_config, self.collection_config(path)?.clone());
        let opts: BongoOptions = key_value.try_into()?;
        BongoClientOptions::try_from_bongo_options(opts).await
    }

    /// Get the pool options per `permission` type, `collection`, and `api_key`.
    ///
    /// # Errors
    ///
    /// Fails if there is not configuration given for the combination of parameters
    /// given.
    pub async fn to_app_opts(&self, path: &AppPath) -> Result<BongoClientOptions> {
        let config = self.app_config(path)?.clone();
        let base = self
            .global_config(path.collection_path().permission_path())
            .ok()
            .cloned();
        let base = if let Ok(cfg) = self.collection_config(path.collection_path()) {
            Some(Self::merge_configs(base, cfg.clone()))
        } else {
            base
        };
        let opts: BongoOptions = Self::merge_configs(base, config).try_into()?;
        BongoClientOptions::try_from_bongo_options(opts).await
    }

    /// Get the pool options that correspond to the given path.
    pub async fn to_opts(&self, path: &ConfigPath) -> Result<BongoClientOptions> {
        match path {
            ConfigPath::Global(path) => self.to_global_opts(path).await,
            ConfigPath::Collection(path) => self.to_collection_opts(path).await,
            ConfigPath::App(path) => self.to_app_opts(path).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File, FileFormat};

    #[test]
    fn bongo_config_contains_global() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "redemptions": {
                        "read" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();

        let r = PermissionPath::new(PoolPermissionType::Read);
        assert!(config.contains_global(&r));
        let w = PermissionPath::new(PoolPermissionType::Write);
        assert!(!config.contains_global(&w));
    }

    #[test]
    fn bongo_config_contains_collection() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "write" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "redemptions": {
                        "read" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();

        let r = CollectionPath::new("redemptions", PoolPermissionType::Read);
        assert!(!config.contains_collection(&r));
        let w = CollectionPath::new("redemptions", PoolPermissionType::Write);
        assert!(config.contains_collection(&w));
        let r = CollectionPath::new("installations", PoolPermissionType::Write);
        assert!(!config.contains_collection(&r));
    }

    #[test]
    fn bongo_config_contains_app() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "redemptions": {
                        "write" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();

        let r = AppPath::new("wat", "redemptions", PoolPermissionType::Read);
        assert!(!config.contains_app(&r));
        let w = AppPath::new("wat", "redemptions", PoolPermissionType::Write);
        assert!(config.contains_app(&w));
    }

    #[test]
    fn bongo_config_cached_fields() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "installations": {
                        "read" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        // global
        assert!(config.global().is_some());
        assert!(config
            .global()
            .unwrap()
            .get(&PoolPermissionType::Read)
            .is_some());
        assert!(config
            .global()
            .unwrap()
            .get(&PoolPermissionType::Write)
            .is_none());
        // per app
        assert!(config.app().is_some());
        let app = config.app().unwrap().get("wat");
        assert!(app.is_some());
        let collection = app.unwrap().get("installations");
        assert!(collection.is_some());
        let permission = collection.unwrap().get(&PoolPermissionType::Read);
        assert!(permission.is_some());
        // collections
        assert!(config.collections().is_some());
        let collection = config.collections().unwrap().get("redemptions");
        assert!(collection.is_some());
        let permission = collection.unwrap().get(&PoolPermissionType::Read);
        assert!(permission.is_some());
    }

    #[tokio::test]
    async fn bongo_config_to_app_opts_err() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "installations": {
                        "read" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = AppPath::new("wat", "redemptions", PoolPermissionType::Read);
        let opts = config.to_app_opts(&path).await;
        assert!(opts.is_err());
    }

    #[tokio::test]
    async fn bongo_config_to_app_opts_ok() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            },
            "mongodbPerApp": {
                "wat": {
                    "redemptions": {
                        "read" : {
                            "maxPoolSize": "100",
                            "connectTimeoutMS": "150"
                        }
                    }
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = AppPath::new("wat", "redemptions", PoolPermissionType::Read);
        let opts = config.to_app_opts(&path).await;
        assert!(opts.is_ok());
        let opts = opts.unwrap();
        assert_eq!(opts.connection.max_pool_size, Some(100));
        assert_eq!(opts.connection.connect_timeout.unwrap().as_millis(), 150);

        // Verify persistence
        let opts = config.to_app_opts(&path).await;
        assert!(opts.is_ok());
    }

    #[tokio::test]
    async fn bongo_config_to_collection_opts_ok() {
        let source = r#"{
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "15",
                    "connectTimeoutMS": "15"
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = CollectionPath::new("redemptions", PoolPermissionType::Read);
        let opts = config.to_collection_opts(&path).await;
        assert!(opts.is_ok());
        let opts = opts.unwrap();
        assert_eq!(opts.connection.max_pool_size, Some(15));
        assert_eq!(opts.connection.connect_timeout.unwrap().as_millis(), 15);

        // Verify persistence
        let opts = config.to_collection_opts(&path).await;
        assert!(opts.is_ok());
    }

    #[tokio::test]
    async fn bongo_config_to_collection_opts_err() {
        let source = r#"{
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/",
                    "maxPoolSize": "15",
                    "connectTimeoutMS": "15"
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = CollectionPath::new("redemptions", PoolPermissionType::Write);
        let opts = config.to_collection_opts(&path).await;
        assert!(opts.is_err());
    }

    #[tokio::test]
    async fn bongo_config_to_global_opts_ok() {
        let source = r#"{
            "read" : {
                "baseUri": "mongodb://wat.com/",
                "maxPoolSize": "15",
                "connectTimeoutMS": "15"
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = PermissionPath::new(PoolPermissionType::Read);
        let opts = config.to_global_opts(&path).await;
        assert!(opts.is_ok());
        let opts = opts.unwrap();
        assert_eq!(opts.connection.max_pool_size, Some(15));
        assert_eq!(opts.connection.connect_timeout.unwrap().as_millis(), 15);
    }

    #[tokio::test]
    async fn bongo_config_to_global_opts_err() {
        let source = r#"{
            "read" : {
                "baseUri": "mongodb://wat.com/",
                "maxPoolSize": "15",
                "connectTimeoutMS": "15"
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let config = BongoConfig::new(config).unwrap();
        let path = PermissionPath::new(PoolPermissionType::Write);
        let opts = config.to_global_opts(&path).await;
        assert!(opts.is_err());
    }
}
