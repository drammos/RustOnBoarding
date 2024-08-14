//! The interface for the "redemptions" collection.
use crate::dao;
use crate::PoolManager;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Redemption {
    pub id: u32,
    pub price: u32,
}

#[derive(Clone, Debug)]
pub struct Redemptions<'a> {
    name: String,
    pool_manager: &'a PoolManager,
}

impl<'a> Redemptions<'a> {
    pub fn new(name: &'a str, pool_manager: &'a PoolManager) -> Self {
        Self {
            name: name.into(),
            pool_manager,
        }
    }
}

impl<'a> dao::Collection for Redemptions<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> dao::DbConnect for Redemptions<'a> {
    fn pool_manager(&self) -> &PoolManager {
        self.pool_manager
    }
}

impl<'a> dao::Query<Redemption> for Redemptions<'a> {}
