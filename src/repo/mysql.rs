use async_trait::async_trait;
use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::{
    errors::Error,
    model::user::{User, USER_STATUS_NORMAL, USER_STATUS_WAIT_ACTIVATE},
    proto::{SignInReq, SignUpReq},
    Result,
};

use super::UserServRepo;

#[derive(Debug, Clone)]
pub struct MySQLRepo {
    pool: Pool<MySql>,
}

impl MySQLRepo {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .map_err(|e| Error::Custom(format!("connecting to mysql error: {}", e)))?;
        Ok(Self { pool })
    }

    fn user_from_signup(&self, signup: SignUpReq, passwd: String) -> User {
        User {
            id: None,
            user_name: signup.user_name,
            email: signup.email,
            passwd,
            status: String::from(USER_STATUS_WAIT_ACTIVATE),
            avatar: None,
            active_time: None,
            update_time: None,
        }
    }
}

#[async_trait]
impl UserServRepo for MySQLRepo {
    async fn create_user(&self, signup: SignUpReq) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, avatar, active_time, update_time 
            from user where user_name = ?"#,
        )
        .bind(&signup.user_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("query user error: {}", e);
            Error::DatabaseException
        })?;
        if user.is_some() {
            return Err(Error::UserExists(format!(
                "user \"{}\" exists",
                &signup.user_name
            )));
        }

        let mut txn = self.pool.begin().await.map_err(|e| {
            tracing::error!("start transaction error: {}", e);
            Error::DatabaseException
        })?;
        let passwd = {
            let mut context = Context::new(&SHA256);
            let mut data = String::new();
            data.push_str(&signup.user_name);
            data.push_str(&signup.passwd);
            context.update(data.as_bytes());
            let digest = context.finish();
            HEXLOWER.encode(digest.as_ref())
        };
        let user = self.user_from_signup(signup, passwd);

        let last_insert_id = sqlx::query(
            "insert into user(`user_name`, `email`, `passwd`, `status`, `update_time`) values (?, ?, ?, ?, now())",
        )
        .bind(user.user_name)
        .bind(user.email)
        .bind(user.passwd)
        .bind(user.status)
        .execute(&mut *txn).await.map_err(|e| {
            tracing::error!("insert user error: {}", e);
            Error::DatabaseException
        })?.last_insert_id();

        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, avatar, active_time, update_time 
            from user where id = ?"#,
        )
        .bind(last_insert_id)
        .fetch_one(&mut *txn)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::DatabaseException
        })?;

        txn.commit().await.map_err(|e| {
            tracing::error!("insert user commit error: {}", e);
            Error::DatabaseException
        })?;

        Ok(user)
    }

    async fn activate_user(&self, user_id: i64) -> Result {
        let mut txn = self.pool.begin().await.map_err(|e| {
            tracing::error!("start transaction error: {}", e);
            Error::DatabaseException
        })?;

        sqlx::query(
            "update user set status = ?, active_time = now(), update_time = now() where id = ?",
        )
        .bind(USER_STATUS_NORMAL)
        .bind(user_id)
        .execute(&mut *txn)
        .await
        .map_err(|e| {
            tracing::error!("update user error: {}", e);
            Error::DatabaseException
        })?;

        txn.commit().await.map_err(|e| {
            tracing::error!("update user commit error: {}", e);
            Error::DatabaseException
        })?;

        Ok(())
    }
    async fn signin_user(&self, signin: SignInReq) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, avatar, active_time, update_time 
            from user where user_name = ?"#,
        )
        .bind(&signin.user_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(?e);
            Error::UserNoExists
        })?;

        let passwd = {
            let mut context = Context::new(&SHA256);
            let mut data = String::new();
            data.push_str(&signin.user_name);
            data.push_str(&signin.passwd);
            context.update(data.as_bytes());
            let digest = context.finish();
            HEXLOWER.encode(digest.as_ref())
        };
        if passwd != user.passwd {
            return Err(Error::UserPasswdError);
        }

        Ok(user)
    }
}
