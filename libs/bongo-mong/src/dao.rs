//! Module implementing the data-access object pattern.

use super::config::options::PoolPermissionType;
use super::error::{BongoError::DaoError, Result};
use super::{Pool, PoolManager};
use async_trait::async_trait;
use mongodb::{
    bson::{self, Document},
    options, results, Cursor, Database,
};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Borrow;

/// An interface to Mongo db collections.
pub trait Collection {
    /// Get the name of the collection
    fn name(&self) -> &str;
}

/// An interface to handle connections to a Mongo Db
#[async_trait]
pub trait DbConnect: Collection {
    /// Get reference to a connection pool manager.
    fn pool_manager(&self) -> &PoolManager;

    /// Get connection pool with read permissions.
    async fn read_pool(&self, api_key: Option<&str>) -> Result<Pool> {
        self.pool_manager()
            .collection_pool(PoolPermissionType::Read, self.name(), api_key)
            .await
    }

    /// Get connection pool with write permissions.
    async fn write_pool(&self, api_key: Option<&str>) -> Result<Pool> {
        self.pool_manager()
            .collection_pool(PoolPermissionType::Write, self.name(), api_key)
            .await
    }

    /// Get a connection to the read database.
    async fn read_database(&self, api_key: Option<&str>) -> Result<Database> {
        self.read_pool(api_key)
            .await?
            .client()
            .default_database()
            .ok_or_else(|| DaoError("No read database configured".into()))
    }

    /// Get a connection to the read database.
    async fn write_database(&self, api_key: Option<&str>) -> Result<Database> {
        self.write_pool(api_key)
            .await?
            .client()
            .default_database()
            .ok_or_else(|| DaoError("No read database configured".into()))
    }
}

#[async_trait]
pub trait Query<D: DeserializeOwned + Serialize + Send + Sync + Unpin>: DbConnect {
    /// Get a connection with read permissions on the collection.
    async fn read_collection(&self, api_key: Option<&str>) -> Result<mongodb::Collection<D>> {
        self.read_pool(api_key).await?.collection(self.name())
    }

    /// Get a connection with write permissions on the collection.
    async fn write_collection(&self, api_key: Option<&str>) -> Result<mongodb::Collection<D>> {
        self.write_pool(api_key).await?.collection(self.name())
    }

    /// Find all documents matching the given filter.
    async fn find<'a, F, O>(
        &self,
        filter: F,
        options: O,
        api_key: Option<&str>,
    ) -> Result<Cursor<D>>
    where
        F: Into<Option<bson::document::Document>> + Send + 'a,
        O: Into<Option<options::FindOptions>> + Send + 'a,
    {
        Ok(self
            .read_collection(api_key)
            .await?
            .find(filter, options)
            .await?)
    }

    /// Find one document.
    async fn find_one<'a, F, O>(
        &self,
        filter: F,
        options: O,
        api_key: Option<&str>,
    ) -> Result<Option<D>>
    where
        F: Into<Option<bson::document::Document>> + Send + 'a,
        O: Into<Option<options::FindOneOptions>> + Send + 'a,
    {
        Ok(self
            .read_collection(api_key)
            .await?
            .find_one(filter, options)
            .await?)
    }

    ///Delete one document.
    async fn delete_one<'a, O>(
        &self,
        query: Document,
        options: O,
        api_key: Option<&str>,
    ) -> Result<()>
    where
        O: Into<Option<options::DeleteOptions>> + Send + 'a,
    {
        self.write_collection(api_key)
            .await?
            .delete_one(query, options)
            .await?;
        Ok(())
    }

    /// Update up to one document matching `query` in the collection.
    async fn update_one<'a>(
        &self,
        query: Document,
        update: impl Into<options::UpdateModifications> + Send + 'a,
        options: impl Into<Option<options::UpdateOptions>> + Send + 'a,
        api_key: Option<&str>,
    ) -> Result<results::UpdateResult> {
        Ok(self
            .write_collection(api_key)
            .await?
            .update_one(query, update, options)
            .await?)
    }

    /// Insert a single document.
    async fn insert_one<'a, B, O>(
        &self,
        doc: B,
        options: O,
        api_key: Option<&str>,
    ) -> Result<results::InsertOneResult>
    where
        B: Borrow<D> + Sync + Send + 'a,
        O: Into<Option<options::InsertOneOptions>> + Send + 'a,
    {
        Ok(self
            .write_collection(api_key)
            .await?
            .insert_one(doc, options)
            .await?)
    }

    /// Runs an aggregation operation.
    ///
    /// See the documentation [here](https://docs.mongodb.com/manual/aggregation/) for more
    /// information on aggregations.
    async fn aggregate<'a, P, O>(
        &self,
        pipeline: P,
        options: O,
        api_key: Option<&str>,
    ) -> Result<Cursor<Document>>
    where
        P: IntoIterator<Item = Document> + Send + 'a,
        O: Into<Option<options::AggregateOptions>> + Send + 'a,
    {
        Ok(self
            .read_collection(api_key)
            .await?
            .aggregate(pipeline, options)
            .await?)
    }
}
