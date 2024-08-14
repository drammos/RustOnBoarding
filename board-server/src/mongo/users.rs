//! The interface for the "Users" collection.
use bongo_mong::dao;
use bongo_mong::PoolManager;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: u8,
}

#[derive(Clone, Debug)]
pub struct Users<'a> {
    name: String,
    pool_manager: &'a PoolManager,
}

impl<'a> Users<'a> {
    pub fn new(name: &'a str, pool_manager: &'a PoolManager) -> Self {
        Self {
            name: name.into(),
            pool_manager,
        }
    }
}

impl<'a> dao::Collection for Users<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> dao::DbConnect for Users<'a> {
    fn pool_manager(&self) -> &PoolManager {
        self.pool_manager
    }
}

impl<'a> dao::Query<User> for Users<'a> {}
