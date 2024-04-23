pub mod mysql;
pub mod sqlite;

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors,
    model::{product::Product, product_user::ProductUser, user::User},
    proto::{SignInReq, SignUpReq},
    Result,
};

// use self::mysql::MySQLRepo;
use self::sqlite::SqliteRepo;

#[async_trait]
pub trait AppServRepo {
    async fn create_user(&self, signup: &SignUpReq) -> Result<User>;
    async fn signin_user(&self, signin: &SignInReq) -> Result<(User, Product)>;
    // async fn activate_user(&self, user_id: i64) -> Result;

    async fn query_products(&self, user_id: i64) -> Result<Vec<Product>>;
    async fn query_product_user(&self, product_id: i64, user_id: i64) -> Result<ProductUser>;
}

pub type DynAppServRepo = Arc<dyn AppServRepo + Send + Sync>;

pub async fn new(url: &str) -> Result<DynAppServRepo> {
    // if url.starts_with("mysql") {
    //     let repo = MySQLRepo::new(url).await?;
    //     return Ok(Arc::new(repo));
    // }
    if url.starts_with("sqlite") {
        let url = url.strip_prefix("sqlite://").unwrap();
        let repo = SqliteRepo::new(url).await?;
        return Ok(Arc::new(repo));
    }
    Err(errors::Error::Custom(format!(
        "url schema repo \"{}\" not implement",
        url
    )))
}
