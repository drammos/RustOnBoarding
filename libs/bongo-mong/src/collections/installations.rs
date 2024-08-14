//! The interface for the "installations" collection.
use crate::dao::{self, Query};
use crate::error::Result;
use crate::PoolManager;
use mongodb::{bson, options};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Installation {
    #[serde(rename = "wappierId")]
    pub wappier_id: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub sessions: bson::Document,
}

#[derive(Clone, Debug)]
pub struct Installations<'a> {
    name: String,
    pool_manager: &'a PoolManager,
}

impl Installations<'_> {
    const NAME: &'static str = "installations";
}

impl<'a> Installations<'a> {
    pub fn new(pool_manager: &'a PoolManager) -> Self {
        Self {
            name: Installations::NAME.into(),
            pool_manager,
        }
    }

    pub async fn assert_session_active(
        &self,
        wappier_id: &str,
        api_key: &str,
        session_id: &str,
    ) -> Result<bool> {
        let query = bson::doc! {
            "wappierId": wappier_id,
            "apiKey": api_key,
            format!("sessions.{}.active", session_id): true
        };
        let projection = bson::doc! {
            "_id": 0_i32,
            "wappierId": 1_i32,
            "apiKey": 1_i32,
            "sessions": 1_i32
        };
        let opts = options::FindOneOptions::builder()
            .projection(projection)
            .build();
        Ok(self.find_one(query, opts, Some(api_key)).await?.is_some())
    }
}

impl<'a> dao::Collection for Installations<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> dao::DbConnect for Installations<'a> {
    fn pool_manager(&self) -> &PoolManager {
        self.pool_manager
    }
}

impl<'a> dao::Query<Installation> for Installations<'a> {}
