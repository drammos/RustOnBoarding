// use super::{ UserGraph};
use crate::{
    mongo::{client::Mongod, languages::Language, users_graph::UserGraph},
    AppError,
};
use async_graphql::{Context, Object};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, id: String) -> Result<Option<UserGraph>, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.find_user_in_base(id).await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }

    pub async fn users(
        &self,
        ctx: &Context<'_>,
        language_id: Option<String>,
    ) -> Result<Vec<UserGraph>, AppError> {
        match language_id {
            Some(x) => match ctx.data::<Mongod>() {
                Ok(it) => it.find_lang_for_use_in_base(x).await,
                Err(_err) => Err(AppError::ContextData(
                    "in data context with Mongod".to_string(),
                )),
            },
            None => match ctx.data::<Mongod>() {
                Ok(it) => it.get_users_from_base().await,
                Err(_err) => Err(AppError::ContextData(
                    "in data context with Mongod".to_string(),
                )),
            },
        }
    }

    async fn language(&self, ctx: &Context<'_>, id: String) -> Result<Option<Language>, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.find_language_in_base(id).await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }

    async fn languages(&self, ctx: &Context<'_>) -> Result<Vec<Language>, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.get_languages_from_base().await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn add_user(
        &self,
        ctx: &Context<'_>,
        id: String,
        name: String,
        age: u8,
        language_id: String,
    ) -> Result<UserGraph, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.add_user_in_base(id, name, age, language_id).await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }

    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<Option<UserGraph>, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.delete_user_from_base(id).await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: String,
        name: Option<String>,
        age: Option<u8>,
        language_id: Option<String>,
    ) -> Result<Option<UserGraph>, AppError> {
        match ctx.data::<Mongod>() {
            Ok(it) => it.update_user_in_base(id, name, age, language_id).await,
            Err(_err) => Err(AppError::ContextData(
                "in data context with Mongod".to_string(),
            )),
        }
    }
}
