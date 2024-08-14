//! Utilities to manage Mongo Db pools.
use crate::config::{
    options::{LooseOption, LooseOptions, PoolPermissionType},
    path, BongoConfig,
};
use crate::error::{BongoError, Result};
use config::{Config, Value};
use mongodb::Client;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Represent a pool of connections to MongoDb.
///
/// The value wraps additional ad-hoc options supported by the library.
#[derive(Clone, Debug)]
pub struct Pool {
    client: Client,
    options: LooseOptions,
}

impl Pool {
    /// Create a new pool.
    pub fn new(client: Client, options: LooseOptions) -> Self {
        Self { client, options }
    }

    /// Get the client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get loose options.
    pub fn options(&self) -> &LooseOptions {
        &self.options
    }

    /// Get a connection to a collection.
    ///
    /// The method looks for a collection name in `options()`, and uses the given name as fallback.
    pub fn collection<D>(&self, name: impl AsRef<str>) -> Result<mongodb::Collection<D>>
    where
        D: DeserializeOwned + Serialize + Send + Sync + Unpin,
    {
        let collection =
            if let Some(collection) = self.options().get(&LooseOption::Collection).cloned() {
                collection.into_string()?
            } else {
                name.as_ref().to_string()
            };
        Ok(self
            .client()
            .default_database()
            .ok_or_else(|| config::ConfigError::Message("No read database configured".into()))?
            .collection(&collection))
    }
}

/// Manage mongodb pools.
///
/// Uses `Arc` internally, so it can be used between threads safely.
#[derive(Clone, Debug, Default)]
pub struct PoolManager {
    pools: Arc<RwLock<HashMap<String, Pool>>>,
    config: Arc<BongoConfig>,
}

impl PoolManager {
    /// Create a new pool manager.
    pub fn new(config: Config) -> Result<Self> {
        let config = Arc::new(BongoConfig::new(config)?);
        Ok(Self {
            config,
            ..Default::default()
        })
    }

    /// The current size of the cache.
    ///
    /// Used for introspection.
    pub fn cache_size(&self) -> usize {
        let cache = self.pools.read();
        cache.len()
    }

    fn read_from_cache(&self, key: &str) -> Option<Pool> {
        let cache = self.pools.read();
        cache.get(key).cloned()
    }

    fn write_to_cache(&self, key: &str, pool: Pool) {
        let mut cache = self.pools.write();
        cache.insert(key.to_string(), pool);
    }

    /// Get the global pool for the given permission type.
    ///
    /// First we check the inner cache for an existing pool, otherwise
    /// we create a new pool and store into the cache.
    pub async fn global_pool(&self, permission: PoolPermissionType) -> Result<Pool> {
        let path = path::PermissionPath::new(permission);
        let key = path.to_string();

        if let Some(hit) = self.read_from_cache(&key) {
            Ok(hit)
        } else {
            let opts = self.config.to_global_opts(&path).await?;
            let client = Client::with_options(opts.connection)?;
            self.write_to_cache(&key, Pool::new(client, opts.other));
            Ok(self.read_from_cache(&key).unwrap())
        }
    }

    /// Get the collection pool for the given permission type.
    ///
    /// First we check the inner cache for an existing pool, otherwise
    /// we create a new pool and store into the cache.
    pub async fn collection_pool(
        &self,
        permission: PoolPermissionType,
        collection: impl AsRef<str>,
        api_key: Option<&str>,
    ) -> Result<Pool> {
        let path = self.config.resolve_path(api_key, collection, permission);
        let key = path.to_string();

        if let Some(hit) = self.read_from_cache(&key) {
            Ok(hit)
        } else {
            let opts = self.config.to_opts(&path).await?;
            let client = Client::with_options(opts.connection)?;
            self.write_to_cache(&key, Pool::new(client, opts.other));
            Ok(self.read_from_cache(&key).unwrap())
        }
    }
}

impl TryFrom<Value> for PoolManager {
    type Error = BongoError;

    /// Create a `PoolManager` value from a `Value`.
    fn try_from(value: Value) -> Result<Self> {
        // Infallible because we have no registered sources
        let mut config = Config::builder().build().unwrap();
        config.cache = value;
        PoolManager::new(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::config::{File, FileFormat};
    use tokio;

    #[tokio::test]
    async fn pool_manager_global_pool_from_value() {
        let source = r#"{
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/redemptions",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                },
                "write" : {
                    "baseUri": "mongodb://wat.com/redemptions",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let value: Value = config.get("redemptions").unwrap();
        let pool_manager = PoolManager::try_from(value).unwrap();
        for permission in [PoolPermissionType::Read, PoolPermissionType::Write] {
            let manager = pool_manager.clone();
            let task = tokio::task::spawn(async move {
                let pool = manager.global_pool(permission).await.unwrap();
                assert_eq!(
                    pool.client().default_database().unwrap().name(),
                    "redemptions"
                );
            });
            task.await.unwrap();
        }
        assert_eq!(pool_manager.cache_size(), 2);
    }
    #[tokio::test]
    async fn pool_manager() {
        let source = r#"{
            "read" : {
                "maxPoolSize": "15"
            },
            "write" : {
                "maxPoolSize": "15"
            },
            "redemptions": {
                "read" : {
                    "baseUri": "mongodb://wat.com/redemptions",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                },
                "write" : {
                    "baseUri": "mongodb://wat.com/redemptions",
                    "maxPoolSize": "10",
                    "connectTimeoutMS": "15"
                }
            }
        }
        "#;

        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Json))
            .build()
            .unwrap();
        let pool_manager = PoolManager::new(config).unwrap();
        for permission in [PoolPermissionType::Read, PoolPermissionType::Write] {
            let manager = pool_manager.clone();
            let task = tokio::task::spawn(async move {
                let pool = manager
                    .collection_pool(permission, "redemptions", None)
                    .await
                    .unwrap();
                assert_eq!(
                    pool.client().default_database().unwrap().name(),
                    "redemptions"
                );
            });
            task.await.unwrap();
        }
        assert_eq!(pool_manager.cache_size(), 2);
    }
}
