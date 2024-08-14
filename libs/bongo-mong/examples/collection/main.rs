//! We demonstrate how to use a specific collection that implements the `bongo_mong::dao` traits.
//!
//! The example requires a running mongo server. For example, to instantiate with `docker` run:
//!
//! ```
//! $ docker run --name mongodb -p 27017:27017 -d mongo:4.0.5
//! ```
use bongo_mong::collections::redemptions::{Redemption, Redemptions};
use bongo_mong::dao::Query;
use bongo_mong::error::Result;
use bongo_mong::PoolManager;
use config::{Config, File};
use mongodb::bson::doc;
use once_cell::sync::Lazy;

static POOLS: Lazy<PoolManager> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(File::with_name("examples/collection/mongo_pools.json"))
        .build()
        .unwrap();
    PoolManager::new(config).unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    let collection = Redemptions::new("redemptions", &POOLS);

    // Insert documents
    let mut insertions = vec![];
    for i in 1..5 {
        let resource = collection.clone();
        insertions.push(tokio::spawn(async move {
            let result = resource
                .insert_one(
                    Redemption {
                        id: i,
                        price: 30 + i,
                    },
                    None,
                    None,
                )
                .await;
            assert!(result.is_ok());
        }));
    }
    for insertion in insertions {
        insertion.await.unwrap();
    }

    // Find the documents
    let mut queries = vec![];
    for i in 1..5 {
        let resource = collection.clone();
        queries.push(tokio::spawn(async move {
            let redemption = resource
                .find_one(doc! { "id": i as u32 }, None, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(redemption.price, 30 + i);
        }));
    }
    for query in queries {
        query.await.unwrap();
    }

    Ok(())
}
