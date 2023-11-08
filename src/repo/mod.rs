pub mod mysql;
pub mod sqlite;

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors,
    model::user::User,
    proto::{SignInReq, SignUpReq},
    Result,
};

use self::mysql::MySQLRepo;
use self::sqlite::SqliteRepo;

#[async_trait]
pub trait UserServRepo {
    async fn create_user(&self, signup: SignUpReq) -> Result<User>;
    async fn signin_user(&self, signin: SignInReq) -> Result<User>;
    async fn activate_user(&self, user_id: i64) -> Result;
}

pub type DynUserServRepo = Arc<dyn UserServRepo + Send + Sync>;

pub async fn new(url: &str) -> Result<DynUserServRepo> {
    if url.starts_with("mysql") {
        let repo = MySQLRepo::new(url).await?;
        return Ok(Arc::new(repo));
    }
    if url.starts_with("sqlite") {
        let repo = SqliteRepo::new(url).await?;
        return Ok(Arc::new(repo));
    }
    Err(errors::Error::Custom(format!(
        "url schema repo \"{}\" not implement",
        url
    )))
}
