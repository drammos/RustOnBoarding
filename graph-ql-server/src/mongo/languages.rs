//! The interface for the "Languages" collection.
use bongo_mong::dao;
use bongo_mong::PoolManager;

use super::users_graph::UserGraph;
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Clone, Debug, Deserialize, Serialize)]
pub struct Language {
    pub id: String,
    pub name: String,
    pub users: Vec<UserGraph>,
}

#[derive( Clone, Debug, Deserialize, Serialize)]
pub struct LanguageForBase {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Languages<'a> {
    name: String,
    pool_manager: &'a PoolManager,
}

impl<'a> Languages<'a> {
    pub fn new(name: &'a str, pool_manager: &'a PoolManager) -> Self {
        Self {
            name: name.into(),
            pool_manager,
        }
    }
}

impl<'a> dao::Collection for Languages<'a> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> dao::DbConnect for Languages<'a> {
    fn pool_manager(&self) -> &PoolManager {
        self.pool_manager
    }
}

impl<'a> dao::Query<LanguageForBase> for Languages<'a> {}
