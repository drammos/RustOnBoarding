//! Demonstrate the usage of [`PoolManager`](bongo_mong::PoolManager).
use bongo_mong::config::options::PoolPermissionType;
use bongo_mong::error::Result;
use bongo_mong::PoolManager;
use config::{Config, File};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::builder()
        .add_source(File::with_name("examples/simple/mongo_pools.json"))
        .build()?;
    let pool_manager = PoolManager::new(config)?;
    let redemptions_read = pool_manager
        .collection_pool(PoolPermissionType::Read, "redemptions", None)
        .await?;
    assert_eq!(
        redemptions_read.client().default_database().unwrap().name(),
        "redemptions"
    );
    let redemptions_write = pool_manager
        .collection_pool(PoolPermissionType::Write, "redemptions", None)
        .await?;
    assert_eq!(
        redemptions_write
            .client()
            .default_database()
            .unwrap()
            .name(),
        "redemptions"
    );
    assert_eq!(pool_manager.cache_size(), 2);
    Ok(())
}
