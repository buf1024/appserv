use async_trait::async_trait;
use chrono::Local;
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

use crate::{
    errors::Error,
    model::{
        product::Product, product_user::{ProductUser, USER_PRODUCT_STATUS_NORMAL}, user::{User, USER_STATUS_NORMAL}
    },
    proto::{SignInReq, SignUpReq},
    Result,
};

use super::AppServRepo;

#[derive(Debug, Clone)]
pub struct SqliteRepo {
    pool: Pool<Sqlite>,
}

impl SqliteRepo {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .map_err(|e| {
                Error::Custom(format!("connecting to sqlite: path={} error={}", url, e))
            })?;
        Ok(Self { pool })
    }

    fn user_from_signup(&self, signup: &SignUpReq, passwd: String) -> User {
        let user_name = signup
            .email
            .split("@")
            .collect::<Vec<_>>()
            .get(0)
            .unwrap()
            .to_string();
        User {
            id: None,
            user_name,
            email: signup.email.clone(),
            passwd,
            status: String::from(USER_STATUS_NORMAL),
            update_time: Local::now().timestamp(),
        }
    }
}

#[async_trait]
impl AppServRepo for SqliteRepo {
    async fn create_user(&self, signup: &SignUpReq) -> Result<User> {
        if let Some(user) = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&signup.email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("query user error: {}", e);
            Error::DatabaseException
        })? {
            return Err(Error::UserExists(format!(
                "email \"{}\" exists",
                &user.email
            )));
        }

        let product = sqlx::query_as::<_, Product>(
            r#"select id, product, desc, status, update_time
            from user where product = ?"#,
        )
        .bind(&signup.product)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("query product error: {}", e);
            Error::DatabaseException
        })?
        .ok_or(Error::ProductNotExists)?;

        let mut txn = self.pool.begin().await.map_err(|e| {
            tracing::error!("start transaction error: {}", e);
            Error::DatabaseException
        })?;
        let passwd = {
            let mut context = Context::new(&SHA256);
            let mut data = String::new();
            data.push_str(&signup.email);
            data.push_str(&signup.passwd);
            context.update(data.as_bytes());
            let digest = context.finish();
            HEXLOWER.encode(digest.as_ref())
        };
        let user = self.user_from_signup(signup, passwd);

        sqlx::query(
            "insert into user(`user_name`, `email`, `passwd`, `status`, `update_time`) values (?, ?, ?, ?, ?)",
        )
        .bind(&user.user_name)
        .bind(&user.email)
        .bind(user.passwd)
        .bind(user.status)
        .bind(user.update_time)
        .execute(&mut *txn).await.map_err(|e| {
            tracing::error!("insert user error: {}", e);
            Error::DatabaseException
        })?;

        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&user.email)
        .fetch_one(&mut *txn)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::DatabaseException
        })?;

        sqlx::query(
            "insert into user_product(`user_id`, `product_id`, `status`, unixepoch(current_timestamp)) values (?, ?, ?)",
        )
        .bind(user.id.unwrap())
        .bind(product.id.unwrap())
        .bind(USER_PRODUCT_STATUS_NORMAL)
        .execute(&mut *txn).await.map_err(|e| {
            tracing::error!("insert user_product error: {}", e);
            Error::DatabaseException
        })?;

        txn.commit().await.map_err(|e| {
            tracing::error!("create user commit error: {}", e);
            Error::DatabaseException
        })?;

        Ok(user)
    }

    async fn signin_user(&self, signin: &SignInReq) -> Result<(User, Product)> {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&signin.email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::UserNotExists
        })?;

        let passwd = {
            let mut context = Context::new(&SHA256);
            let mut data = String::new();
            data.push_str(&signin.email);
            data.push_str(&signin.passwd);
            context.update(data.as_bytes());
            let digest = context.finish();
            HEXLOWER.encode(digest.as_ref())
        };
        if passwd != user.passwd {
            return Err(Error::UserPasswdError);
        }

        let products = self.query_products(user.id.unwrap()).await?;

        let open_products: Vec<_> = products
            .into_iter()
            .filter(|product| product.product == signin.product)
            .collect();

        if open_products.is_empty() {
            return Err(Error::ProductNotOpen);
        }
        let product = (*open_products.get(0).unwrap()).clone();

        Ok((user, product))
    }

    async fn query_products(&self, user_id: i64) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"select a.id as id, a.product as product, a.desc as desc, a.update_time as update_time
            from product a, user_product b 
            where a.id = b.product_id and a.status = '00' and b.status = '00' and b.user_id = ?"#,
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::ProductNotExists
        })?;

        Ok(products)
    }

    async fn query_product_user(&self, product_id: i64, user_id: i64) -> Result<ProductUser> {
        let user = sqlx::query_as::<_, ProductUser>(
            // r#"select a.id as user_id, a.user_name as user_name, a.email as email, 
            // b.avatar as avatar, b.status as status, b.update_time as update_time
            // from user a, user_product b 
            // where a.id = b.user_id and a.status = '00' and b.status = '00' and b.user_id = ? and b.product_id = ?"#,
            // pub product_id: i64,
            // pub user_id: i64,
            // pub user_name: String,
            // pub email: String,
            // pub avatar: String,
            // pub status: String,
            // pub update_time: i64,
            r#"select a.id as user_id, a.user_name, a.email, 
            b.product_id, b.avatar, b.status, b.update_time 
            from user a, user_product b 
            where a.id = b.user_id and a.status = '00' and b.status = '00' and b.user_id = ? and b.product_id = ?"#,
        )
        .bind(user_id)
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::UserNotExists
        })?;

        Ok(user)
    }
}
