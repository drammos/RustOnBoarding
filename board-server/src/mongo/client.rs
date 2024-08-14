use std::convert::TryFrom;

use mongodb::results::InsertOneResult;

use mongodb::bson::doc;

use super::users::{User, Users};
use crate::{AppError, CONFIG};
use bongo_mong::dao::Query;
use bongo_mong::PoolManager;
use config::ConfigError;
use once_cell::sync::Lazy;

static POOLS: Lazy<Result<PoolManager, AppError>> = Lazy::new(|| {
    let config_1 = Lazy::force(&CONFIG)
        .as_ref()
        .map_err(|e| ConfigError::Message(e.to_string()))?;

    Ok(PoolManager::try_from(config_1.users.bongo.clone())?)
});

#[derive(Clone)]
pub struct Mongod<'a> {
    pub collection: Users<'a>,
}

impl<'a> Mongod<'a> {
    pub fn new() -> Result<Self, AppError> {
        let config_1 = Lazy::force(&CONFIG)
            .as_ref()
            .map_err(|e| ConfigError::Message(e.to_string()))?;
        let pool_manager = Lazy::force(&POOLS)
            .as_ref()
            .map_err(|err| ConfigError::Message(err.to_string()))?;

        Ok(Self {
            collection: Users::new(config_1.collection.as_str(), pool_manager),
        })
    }

    pub async fn insert_user_in_base(
        &self,
        id: String,
        name: String,
        age: u8,
    ) -> Result<InsertOneResult, AppError> {
        let new_user = User { id, name, age };

        Ok(self.collection.insert_one(new_user, None, None).await?)
    }

    pub async fn find_use_in_base(&self, id: String) -> Result<User, AppError> {
        self.collection
            .find_one(
                doc! {
                    "id": id.as_str()
                },
                None,
                None,
            )
            .await?
            .ok_or(AppError::User(id))
    }
}
