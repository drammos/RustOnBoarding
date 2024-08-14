//! We demonstrate how to use a specific collection that implements the `bongo_mong::dao` traits.
//!
//! The example requires a running mongo server. For example, to instantiate with `docker` run:
//!
//! ```
//! $ docker run --name mongodb -p 27017:27017 -d mongo:4.0.5
//! ```
use bongo_mong::collections::installations::{Installation, Installations};
use bongo_mong::dao::Query;
use bongo_mong::error::Result;
use bongo_mong::PoolManager;
use config::{Config, File};
use mongodb::bson;
use once_cell::sync::Lazy;

mod fixtures;

static POOLS: Lazy<PoolManager> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(File::with_name("examples/installations/mongo_pools.json"))
        .build()
        .unwrap();
    PoolManager::new(config).unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    let collection = Installations::new(&POOLS);
    let api_key = "cff9dc8b-1e74-461e-9f76-0337f77d77ed";

    // Insert documents
    let document: Installation = bson::from_document(fixtures::installations()).unwrap();
    let result = collection.insert_one(document, None, Some(api_key)).await;
    assert!(result.is_ok());

    // Query document
    let wappier_id = "e6031bd6-17bf-542b-a202-82e6edfef394";
    let session_id = "367d950a-84b5-5a0a-bbf7-79bd06f164ba";
    assert!(
        !collection
            .assert_session_active(wappier_id, api_key, session_id)
            .await?
    );
    let session_id = "d799de6d-7925-5ab3-9581-8d0360a4a7d6";
    assert!(
        collection
            .assert_session_active(wappier_id, api_key, session_id)
            .await?
    );
    Ok(())
}
