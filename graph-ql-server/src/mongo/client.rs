use std::convert::TryFrom;

use crate::{AppError, CONFIG};

use super::languages::*;
use super::users_graph::*;
use async_graphql::futures_util::TryStreamExt;
use bongo_mong::dao::Query;
use bongo_mong::PoolManager;
use config::ConfigError;
use mongodb::bson::doc;
use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct Mongod<'a> {
    pub collection_users: UserGraphs<'a>,
    pub collection_languages: Languages<'a>,
}

static POOLS_USERS: Lazy<Result<PoolManager, AppError>> = Lazy::new(|| {
    let config_1 = Lazy::force(&CONFIG)
        .as_ref()
        .map_err(|e| ConfigError::Message(e.to_string()))?;

    Ok(PoolManager::try_from(config_1.users.bongo.clone())?)
});

static POOLS_LANGUAGES: Lazy<Result<PoolManager, AppError>> = Lazy::new(|| {
    let config_1 = Lazy::force(&CONFIG)
        .as_ref()
        .map_err(|e| ConfigError::Message(e.to_string()))?;

    Ok(PoolManager::try_from(config_1.languages.bongo.clone())?)
});
impl<'a> Mongod<'a> {


    pub fn new() -> Result<Self, AppError> {
        let config_1 = Lazy::force(&CONFIG)
            .as_ref()
            .map_err(|e| ConfigError::Message(e.to_string()))?;
        let pool_manager_users = Lazy::force(&POOLS_USERS)
            .as_ref()
            .map_err(|err| ConfigError::Message(err.to_string()))?;

        let pool_manager_languages = Lazy::force(&POOLS_LANGUAGES)
            .as_ref()
            .map_err(|err| ConfigError::Message(err.to_string()))?;

        Ok(Self {
            collection_users: UserGraphs::new(
                config_1.collection_users.as_str(),
                pool_manager_users,
            ),
            collection_languages: Languages::new(
                config_1.collection_languages.as_str(),
                pool_manager_languages,
            ),
        })
    }

    //User
    pub async fn get_users_from_base(&self) -> Result<Vec<UserGraph>, AppError> {
        Ok(self
            .collection_users
            .find(None, None, None)
            .await?
            .try_collect()
            .await?)
    }

    pub async fn find_user_in_base(&self, id: String) -> Result<Option<UserGraph>, AppError> {
        Ok(self
            .collection_users
            .find_one(
                doc! {
                    "id": id
                },
                None,
                None,
            )
            .await?)
    }

    pub async fn add_user_in_base(
        &self,
        id: String,
        name: String,
        age: u8,
        language_id: String,
    ) -> Result<UserGraph, AppError> {
        let new_user = UserGraph {
            id: id.to_string(),
            name,
            age,
            language_id,
        };

        let user = self
            .collection_users
            .find_one(doc! {"id": id}, None, None)
            .await?;

        match user {
            Some(user_in) => Err(AppError::User(user_in.id)),
            None => {
                self.collection_users
                    .insert_one(new_user.clone(), None, None)
                    .await?;
                Ok(new_user)
            }
        }
    }

    pub async fn delete_user_from_base(&self, id: String) -> Result<Option<UserGraph>, AppError> {
        let user = self
            .collection_users
            .find_one(doc! {"id": id.as_str()}, None, None)
            .await?;

        self.collection_users
            .delete_one(doc! {"id": id.as_str()}, None, None)
            .await?;

        Ok(user)
    }

    pub async fn update_user_in_base(
        &self,
        id: String,
        name: Option<String>,
        age: Option<u8>,
        language_id: Option<String>,
    ) -> Result<Option<UserGraph>, AppError> {
        let user = self
            .collection_users
            .find_one(doc! {"id": id.as_str()}, None, None)
            .await?
            .ok_or_else(|| AppError::User(id.to_string()))?;

        let name_old = user.name;

        let name_new = name.unwrap_or_else(|| name_old.to_string());

        let age_old = user.age;

        let age_new = age.unwrap_or(age_old);

        let language_old = user.language_id;

        let language_new = language_id.unwrap_or_else(|| language_old.to_string());

        let user_new = UserGraph {
            id: id.to_string(),
            name: name_new.to_string(),
            age: age_new,
            language_id: language_new.to_string(),
        };

        //impl from docu
        let _update_res =
                self.collection_users
                .update_one(
                    doc!{ "id": id.as_str()},
                    doc! {
                        "$set": {"name": name_new.as_str(), "age": age_new as i32, "language_id": language_new.as_str()} 
                    },
                    None,
                    None
                )
            .await?;

        Ok(Some(user_new))
    }

    pub async fn find_lang_for_use_in_base(
        &self,
        language_id: String,
    ) -> Result<Vec<UserGraph>, AppError> {
        Ok(self
            .collection_users
            .find(doc! {"language_id": language_id}, None, None)
            .await?
            .try_collect()
            .await?)
    }

    //Language
    pub async fn get_languages_from_base(&self) -> Result<Vec<Language>, AppError> {
        let mut lang = self.collection_languages.find(None, None, None).await?;
        let mut vec = Vec::new();
        while let Some(language) = lang.try_next().await? {
            let id = language.id;
            let users = self.find_lang_for_use_in_base(id.to_string()).await?;

            let language_new = Language {
                id,
                name: language.name,
                users,
            };
            vec.push(language_new);
        }
        Ok(vec)
    }

    pub async fn find_language_in_base(&self, id: String) -> Result<Option<Language>, AppError> {
        let lang = self
            .collection_languages
            .find_one(doc! {"id": id.as_str()}, None, None)
            .await?
            .ok_or_else(|| AppError::User(id))?;

        let id_ = lang.id;

        let name_ = lang.name;

        let users = self.find_lang_for_use_in_base(id_.clone()).await?;

        let language = Language {
            id: id_,
            name: name_.to_string(),
            users,
        };

        Ok(Some(language))
    }
}
