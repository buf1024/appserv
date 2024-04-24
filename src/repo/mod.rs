pub mod mysql;
pub mod sqlite;

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors,
    model::{
        hiqradio::{FavGroup, Recently, StationGroup},
        product::Product,
        session::Session,
        user::User,
        user_product::UserProduct,
    },
    proto::{GroupNew, RecentlyNew, SignInReq, SignUpReq},
    Result,
};

// use self::mysql::MySQLRepo;
use self::sqlite::SqliteRepo;

#[async_trait]
pub trait AppServRepo {
    // service
    async fn create_user(&self, signup: &SignUpReq) -> Result<User>;
    async fn signin_user(&self, signin: &SignInReq) -> Result<(User, Product, Session)>;
    async fn get_session(&self, token: &str) -> Result<Session>;
    async fn update_user_info(
        &self,
        user_id: i64,
        product_id: i64,
        user_name: Option<String>,
        passwd: Option<String>,
        avatar: Option<String>,
    ) -> Result;

    // dao
    async fn query_user_products(&self, user_id: i64) -> Result<Vec<Product>>;
    async fn query_products(&self) -> Result<Vec<Product>>;
    async fn query_product(&self, product_id: i64) -> Result<Product>;
    async fn query_user(&self, user_id: i64) -> Result<User>;
    async fn query_user_product(&self, user_id: i64, product_id: i64) -> Result<UserProduct>;
    async fn delete_session(&self, token: &str) -> Result;

    // hiqradio dao
    async fn query_recently(&self, user_id: i64) -> Result<Vec<Recently>>;
    async fn delete_recently(&self, user_id: i64) -> Result;
    async fn new_recently(&self, user_id: i64, recently: &Vec<RecentlyNew>) -> Result;

    async fn query_groups(&self, user_id: i64) -> Result<Vec<FavGroup>>;
    async fn delete_groups(&self, user_id: i64, groups: &Vec<String>) -> Result;
    async fn new_groups(&self, user_id: i64, groups: &Vec<GroupNew>) -> Result;
    async fn modify_group(&self, user_id: i64, old_name: &str, name: &str, desc: &str) -> Result;

    async fn query_favorites(&self, user_id: i64) -> Result<Vec<StationGroup>>;
    async fn new_favorite(&self, user_id: i64, stations: &Vec<StationGroup>) -> Result;
    async fn delete_favorite(
        &self,
        user_id: i64,
        favorites: &Option<Vec<String>>,
        group_names: &Option<Vec<String>>,
    ) -> Result;
    async fn modify_favorite(
        &self,
        user_id: i64,
        stationuuid: &str,
        groups: &Vec<String>,
    ) -> Result;
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
