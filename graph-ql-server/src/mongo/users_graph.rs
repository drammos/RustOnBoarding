//! The interface for the "UserGraphs" collection.
use bongo_mong::dao;
use bongo_mong::PoolManager;

use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Clone, Debug, Deserialize, Serialize)]
pub struct UserGraph {
    pub id: String,
    pub name: String,
    pub age: u8,
    pub language_id: String,
}

#[derive(Clone, Debug)]
pub struct UserGraphs<'a> {
    name: String,
    pool_manager: &'a PoolManager,
}

impl<'a> UserGraphs<'a> {
    pub fn new(name: &'a str, pool_manager: &'a PoolManager) -> Self {
        Self {
            name: name.into(),
            pool_manager,
        }
    }
}

impl<'a> dao::Collection for UserGraphs<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> dao::DbConnect for UserGraphs<'a> {
    fn pool_manager(&self) -> &PoolManager {
        self.pool_manager
    }
}

impl<'a> dao::Query<UserGraph> for UserGraphs<'a> {}
