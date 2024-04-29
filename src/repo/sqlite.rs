use std::fs;

use async_trait::async_trait;
use chrono::Local;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Transaction};

use crate::{
    config::CONFIG,
    errors::Error,
    model::{
        hiqradio::{FavGroup, Favorite, Recently, StationGroup},
        product::Product,
        session::Session,
        user::{User, USER_STATUS_NORMAL},
        user_product::UserProduct,
    },
    proto::{GroupNew, RecentlyNew, ResetPasswdReq, SignInReq, SignUpReq},
    util::gen_passwd,
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
                Error::DatabaseException(format!("connecting to sqlite: path={} error={}", url, e))
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
            update_time: Local::now().timestamp_millis(),
        }
    }

    fn build_in_param<T>(&self, param: &Vec<T>) -> String {
        let len = param.len();
        match len {
            0 => String::from(""),
            1 => String::from("?"),
            _ => format!("?{}", ", ?".repeat(param.len() - 1)),
        }
    }

    async fn begin(&self) -> Result<Transaction<'static, Sqlite>> {
        let txn = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(txn)
    }
    async fn rollback(&self, txn: Transaction<'static, Sqlite>) -> Result {
        txn.rollback()
            .await
            .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(())
    }
    async fn commit(&self, txn: Transaction<'static, Sqlite>) -> Result {
        txn.commit()
            .await
            .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl AppServRepo for SqliteRepo {
    async fn clean_avatar_path(&self, path: &str) -> Result {
        if let Err(e) = sqlx::query_as::<_, UserProduct>(
            r#"select id, product_id, user_id, avatar, status, update_time
            from user_product 
            where avatar = ?"#,
        )
        .bind(path)
        .fetch_one(&self.pool)
        .await
        {
            match e {
                sqlx::Error::RowNotFound => {
                    let path = format!("{}/{}", &CONFIG.avatar_path, path);
                    tracing::info!("remove unused avatar: {}", &path);
                    fs::remove_file(path)
                        .map_err(|e| Error::Internal(format!("remove file error: {}", e)))?;
                }

                _ => (),
            };
        }
        Ok(())
    }
    async fn clean_session(&self) -> Result {
        let now = Local::now().timestamp_millis();
        let session: Vec<_> = sqlx::query_as::<_, Session>(
            r#"select id, token, user_id, product_id, expire 
            from session 
            where expire <= ?"#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?
        .iter()
        .map(|e| e.id.unwrap())
        .collect();

        if session.len() > 0 {
            let mut txn = self.begin().await?;

            let query_str = format!(
                r#"delete from session  
                where id in ({})"#,
                self.build_in_param(&session)
            );

            let mut query = sqlx::query(&query_str);

            for param in session {
                query = query.bind(param);
            }
            if let Err(e) = query.execute(&mut *txn).await {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }

            self.commit(txn).await?;
        }

        Ok(())
    }
    async fn create_user(&self, signup: &SignUpReq) -> Result<User> {
        if let Some(user) = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&signup.email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?
        {
            return Err(Error::UserExists(format!(
                "email \"{}\" exists",
                &user.email
            )));
        }

        let product = sqlx::query_as::<_, Product>(
            r#"select id, product, desc, status, update_time
            from product where product = ?"#,
        )
        .bind(&signup.product)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?
        .ok_or(Error::ProductNotExists)?;

        let mut txn = self.begin().await?;
        let passwd = gen_passwd(&signup.email, &signup.passwd);
        let user = self.user_from_signup(signup, passwd);

        if let Err(e) = sqlx::query(
            "insert into user(user_name, email, passwd, status, update_time) values (?, ?, ?, ?, ?)",
        )
        .bind(&user.user_name)
        .bind(&user.email)
        .bind(user.passwd)
        .bind(user.status)
        .bind(user.update_time)
        .execute(&mut *txn)
        .await{
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&user.email)
        .fetch_one(&mut *txn)
        .await;
        if let Err(e) = &user {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }
        let user = user.unwrap();

        if let Err(e) = sqlx::query(
            "insert into user_product(user_id, product_id, status, update_time) values (?, ?, '00', unixepoch(current_timestamp))",
        )
        .bind(user.id.unwrap())
        .bind(product.id.unwrap())
        .execute(&mut *txn)
        .await{
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        self.commit(txn).await?;

        Ok(user)
    }

    async fn signin_user(&self, signin: &SignInReq) -> Result<(User, Product, Session)> {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&signin.email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::UserNotExists,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        let passwd = gen_passwd(&user.email, &signin.passwd);
        if passwd != user.passwd {
            return Err(Error::UserPasswdError);
        }

        let products = self.query_user_products(user.id.unwrap()).await?;

        let open_products: Vec<_> = products
            .into_iter()
            .filter(|product| product.product == signin.product)
            .collect();

        if open_products.is_empty() {
            if signin.product_open_flag {
                let mut txn = self.begin().await?;

                let product = sqlx::query_as::<_, Product>(
                    r#"select id, product, desc, status, update_time
                    from product 
                    where status = '00' and product = ?"#,
                )
                .bind(&signin.product)
                .fetch_one(&mut *txn)
                .await
                .map_err(|e| {
                    return match e {
                        sqlx::Error::RowNotFound => Error::ProductNotExists,

                        _ => Error::DatabaseException(e.to_string()),
                    };
                })?;

                sqlx::query(
                    "insert into user_product(`user_id`, `product_id`, `status`, `update_time`) values (?, ?, '00', unixepoch(current_timestamp))",
                )
                .bind(user.id.unwrap())
                .bind(product.id.unwrap())
                .execute(&mut *txn).await.map_err(|e| {
                    Error::DatabaseException(e.to_string())
                })?;

                self.commit(txn).await?;
            } else {
                return Err(Error::ProductNotOpen);
            }
        }
        let product = (*open_products.get(0).unwrap()).clone();

        let token = {
            let mut txn = self.begin().await?;
            let token = Session::token(product.id.unwrap(), user.id.unwrap());

            if let Err(e) = sqlx::query(
                "insert into session(token, user_id, product_id, expire) values (?, ?, ?, ?)",
            )
            .bind(&token.token)
            .bind(token.user_id)
            .bind(token.product_id)
            .bind(token.expire)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }

            self.commit(txn).await?;

            token
        };

        Ok((user, product, token))
    }

    async fn reset_user_passwd(&self, reset: &ResetPasswdReq) -> Result {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user where email = ?"#,
        )
        .bind(&reset.email)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::UserNotExists,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        let passwd = gen_passwd(&user.email, &reset.passwd);

        let mut txn = self.begin().await?;

        sqlx::query(
            "update user set passwd = ?, update_time = unixepoch(current_timestamp) where id = ?",
        )
        .bind(&passwd)
        .bind(user.id.unwrap())
        .execute(&mut *txn)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        self.commit(txn).await?;

        Ok(())
    }
    async fn open_product(&self, user_id: i64, product: &str) -> Result {
        let products = self.query_user_products(user_id).await?;

        let open_products: Vec<_> = products
            .into_iter()
            .filter(|p| p.product == product)
            .collect();

        if open_products.is_empty() {
            let mut txn = self.begin().await?;

            let product = sqlx::query_as::<_, Product>(
                r#"select id, product, desc, status, update_time
                    from product 
                    where status = '00' and product = ?"#,
            )
            .bind(product)
            .fetch_one(&mut *txn)
            .await
            .map_err(|e| {
                return match e {
                    sqlx::Error::RowNotFound => Error::ProductNotExists,

                    _ => Error::DatabaseException(e.to_string()),
                };
            })?;

            sqlx::query(
                    "insert into user_product(`user_id`, `product_id`, `status`, `update_time`) values (?, ?, '00', unixepoch(current_timestamp))",
                )
                .bind(user_id)
                .bind(product.id.unwrap())
                .execute(&mut *txn).await.map_err(|e| {
                    Error::DatabaseException(e.to_string())
                })?;

            self.commit(txn).await?;
        }
        Ok(())
    }

    async fn get_session(&self, token: &str) -> Result<Session> {
        let mut session = sqlx::query_as::<_, Session>(
            r#"select id, token, user_id, product_id, expire 
            from session 
            where token = ?"#,
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::TokenInvalid,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        let res = {
            let mut res = Err(Error::TokenInvalid);

            let mut txn = self.begin().await?;

            let now = Local::now().timestamp_millis();

            if session.expire < now {
                if let Err(e) = sqlx::query("delete from session where token = ?")
                    .bind(&session.token)
                    .execute(&mut *txn)
                    .await
                {
                    self.rollback(txn).await?;
                    return Err(Error::DatabaseException(e.to_string()));
                }
            } else if session.expire - now < CONFIG.token_refresh * 1000 {
                let expire = now + CONFIG.token_expire * 1000;
                if let Err(e) = sqlx::query("update session set expire = ? where token = ?")
                    .bind(expire)
                    .bind(&session.token)
                    .execute(&mut *txn)
                    .await
                {
                    self.rollback(txn).await?;
                    return Err(Error::DatabaseException(e.to_string()));
                }

                session.expire = now;
                res = Ok(session)
            } else {
                res = Ok(session)
            }

            self.commit(txn).await?;
            res
        };

        res
    }

    async fn update_user_info(
        &self,
        user_id: i64,
        product_id: i64,
        user_name: Option<String>,
        new_passwd: Option<String>,
        avatar: Option<String>,
    ) -> Result {
        let mut txn = self.begin().await?;

        if let Some(new_user_name) = user_name {
            if let Err(e) = sqlx::query("update user set user_name = ? where id = ?")
                .bind(&new_user_name)
                .bind(user_id)
                .execute(&mut *txn)
                .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }
        if let Some(new_passwd) = new_passwd {
            if let Err(e) = sqlx::query("update user set passwd = ? where id = ?")
                .bind(&new_passwd)
                .bind(user_id)
                .execute(&mut *txn)
                .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }
        if let Some(new_avatar) = avatar {
            if let Err(e) = sqlx::query(
                "update user_product set avatar = ? where user_id = ? and product_id = ?",
            )
            .bind(&new_avatar)
            .bind(user_id)
            .bind(product_id)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn query_user_products(&self, user_id: i64) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"select a.id as id, a.product as product, a.desc as desc, a.update_time as update_time
            from product a, user_product b 
            where a.id = b.product_id and a.status = '00' and b.status = '00' and b.user_id = ?"#,
        )
        .bind(&user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(products)
    }

    async fn query_products(&self) -> Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"select a.id as id, a.product as product, a.desc as desc, a.update_time as update_time
            from product a
            where a.status = '00' and 1 = ? "#,
        )
        .bind(1)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(products)
    }

    async fn query_product(&self, product_id: i64) -> Result<Product> {
        let product = sqlx::query_as::<_, Product>(
            r#"select id, product, desc, status, update_time
            from product 
            where status = '00' and id = ?"#,
        )
        .bind(product_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::ProductNotExists,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        Ok(product)
    }
    async fn query_user(&self, user_id: i64) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"select id, user_name, email, passwd, status, update_time
            from user
            where status = '00' and id = ?"#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::UserNotExists,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        Ok(user)
    }
    async fn query_user_product(&self, user_id: i64, product_id: i64) -> Result<UserProduct> {
        let user_product = sqlx::query_as::<_, UserProduct>(
            r#"select id, product_id, user_id, avatar, status, update_time
            from user_product 
            where status = '00' and product_id = ? and user_id = ?"#,
        )
        .bind(product_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            return match e {
                sqlx::Error::RowNotFound => Error::ProductNotOpen,

                _ => Error::DatabaseException(e.to_string()),
            };
        })?;

        Ok(user_product)
    }
    async fn delete_session(&self, token: &str) -> Result {
        let mut txn = self.begin().await?;
        if let Err(e) = sqlx::query("delete from session where token = ?")
            .bind(token)
            .execute(&mut *txn)
            .await
        {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn query_recently(&self, user_id: i64) -> Result<Vec<Recently>> {
        let recently = sqlx::query_as::<_, Recently>(
            r#"select id, user_id, stationuuid, start_time, end_time 
            from hiqradio_recently
            where user_id = ? order by start_time desc"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(recently)
    }

    async fn delete_recently(&self, user_id: i64) -> Result {
        let mut txn = self.begin().await?;
        if let Err(e) = sqlx::query("delete from hiqradio_recently where user_id = ?")
            .bind(user_id)
            .execute(&mut *txn)
            .await
        {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn new_recently(&self, user_id: i64, recently: &Vec<RecentlyNew>) -> Result {
        let mut txn = self.begin().await?;
        let mut count: usize = 0;
        for (_, e) in recently.into_iter().enumerate() {
            if let Err(e) = sqlx::query(
                r#"insert into hiqradio_recently(user_id, stationuuid, start_time, end_time) 
                values (?, ?, ?, ?)"#,
            )
            .bind(user_id)
            .bind(&e.stationuuid)
            .bind(e.start_time)
            .bind(e.end_time)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
            count += 1;

            if count >= 50 {
                self.commit(txn).await?;

                txn = self.begin().await?;
            }
        }
        self.commit(txn).await?;
        Ok(())
    }

    async fn modify_recently(
        &self,
        user_id: i64,
        stationuuid: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result {
        let mut txn = self.begin().await?;
        if let Err(e) = sqlx::query(
                r#"update hiqradio_recently set end_time = ? where stationuuid = ? and start_time = ? and user_id = ?"#,
            )
            .bind(end_time)
            .bind(stationuuid)
            .bind(start_time)
            .bind(user_id)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }

        self.commit(txn).await?;
        Ok(())
    }

    async fn query_groups(&self, user_id: i64) -> Result<Vec<FavGroup>> {
        let groups = sqlx::query_as::<_, FavGroup>(
            r#"select id, user_id, create_time, name, desc, is_def 
            from hiqradio_fav_group
            where user_id = ?"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(groups)
    }

    async fn delete_groups(&self, user_id: i64, groups: &Vec<String>) -> Result {
        let mut txn = self.begin().await?;

        let mut count: usize = 0;
        for (_, e) in groups.into_iter().enumerate() {
            let group = sqlx::query_as::<_, FavGroup>(
                r#"select id, user_id, create_time, name, desc, is_def 
                from hiqradio_fav_group
                where user_id = ? and name = ?"#,
            )
            .bind(user_id)
            .bind(e)
            .fetch_one(&mut *txn)
            .await;
            if let Err(e) = group {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
            let group = group.unwrap();

            if let Err(e) = sqlx::query(r#"delete from hiqradio_favorite where group_id = ?"#)
                .bind(group.id.unwrap())
                .execute(&mut *txn)
                .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }

            if let Err(e) =
                sqlx::query(r#"delete from hiqradio_fav_group where name = ? and user_id = ?"#)
                    .bind(e)
                    .bind(user_id)
                    .execute(&mut *txn)
                    .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }

            count += 1;

            if count >= 50 {
                self.commit(txn).await?;

                txn = self.begin().await?;
            }
        }
        self.commit(txn).await?;
        Ok(())
    }

    async fn new_groups(&self, user_id: i64, groups: &Vec<GroupNew>) -> Result {
        let mut txn = self.begin().await?;

        let mut count: usize = 0;
        for (_, e) in groups.into_iter().enumerate() {
            if let Ok(_) = sqlx::query_as::<_, FavGroup>(
                r#"select id, user_id, create_time, name, desc, is_def 
                from hiqradio_fav_group
                where user_id = ? and name = ?"#,
            )
            .bind(user_id)
            .bind(&e.name)
            .fetch_one(&mut *txn)
            .await
            {
                continue;
            }

            if e.is_def > 0 {
                if let Ok(fg) = sqlx::query_as::<_, FavGroup>(
                    r#"select id, user_id, create_time, name, desc, is_def 
                    from hiqradio_fav_group
                    where user_id = ? and is_def = 1"#,
                )
                .bind(user_id)
                .fetch_one(&mut *txn)
                .await
                {
                    if fg.create_time > e.create_time {
                        continue;
                    }

                    if let Err(e) = sqlx::query(
                        r#"delete from hiqradio_fav_group where user_id = ? and is_def = 1"#,
                    )
                    .bind(user_id)
                    .execute(&mut *txn)
                    .await
                    {
                        self.rollback(txn).await?;
                        return Err(Error::DatabaseException(e.to_string()));
                    }
                }
            }

            if let Err(e) = sqlx::query(
                r#"insert into hiqradio_fav_group(user_id, create_time, name, desc, is_def) 
                values(?, ?, ?, ?, ?)"#,
            )
            .bind(user_id)
            .bind(e.create_time)
            .bind(&e.name)
            .bind(&e.desc)
            .bind(e.is_def)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
            count += 1;

            if count >= 50 {
                self.commit(txn).await?;

                txn = self.begin().await?;
            }
        }
        self.commit(txn).await?;
        Ok(())
    }
    async fn modify_group(&self, user_id: i64, old_name: &str, name: &str, desc: &str) -> Result {
        let mut txn = self.begin().await?;

        if let Err(e) = sqlx::query(
            r#"update hiqradio_fav_group set name = ?, desc = ? where name = ? and user_id = ?"#,
        )
        .bind(name)
        .bind(desc)
        .bind(old_name)
        .bind(user_id)
        .execute(&mut *txn)
        .await
        {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn query_favorites(&self, user_id: i64) -> Result<Vec<StationGroup>> {
        let groups = sqlx::query_as::<_, StationGroup>(
            r#"select a.name as group_name,  b.stationuuid, b.create_time
            from hiqradio_fav_group a, hiqradio_favorite b
            where a.id = b.group_id and a.user_id = b.user_id and a.user_id = ?"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok(groups)
    }
    async fn new_favorite(&self, user_id: i64, stations: &Vec<StationGroup>) -> Result {
        let mut txn = self.begin().await?;

        let mut count: usize = 0;
        for (_, elem) in stations.into_iter().enumerate() {
            let group = sqlx::query_as::<_, FavGroup>(
                r#"select id, user_id, create_time, name, desc, is_def 
                from hiqradio_fav_group
                where user_id = ? and name = ?"#,
            )
            .bind(user_id)
            .bind(&elem.group_name)
            .fetch_one(&mut *txn)
            .await;

            if let Err(e) = &group {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
            let group = group.unwrap();

            if let Ok(_) = sqlx::query_as::<_, Favorite>(
                r#"select id, user_id, stationuuid, group_id, create_time
                from hiqradio_favorite
                where user_id = ? and group_id = ? and stationuuid = ?"#,
            )
            .bind(user_id)
            .bind(group.id.unwrap())
            .bind(&elem.stationuuid)
            .fetch_one(&mut *txn)
            .await
            {
                continue;
            }

            if let Err(e) = sqlx::query(
                r#"insert into hiqradio_favorite(user_id, stationuuid, group_id, create_time) 
                values(?, ?, ?, ?)"#,
            )
            .bind(user_id)
            .bind(&elem.stationuuid)
            .bind(group.id.unwrap())
            .bind(elem.create_time)
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
            count += 1;

            if count >= 50 {
                self.commit(txn).await?;

                txn = self.begin().await?;
            }
        }
        self.commit(txn).await?;
        Ok(())
    }

    async fn delete_favorite(
        &self,
        user_id: i64,
        favorites: &Option<Vec<String>>,
        group_names: &Option<Vec<String>>,
    ) -> Result {
        let mut txn = self.begin().await?;

        if let Some(favorites) = favorites {
            let query_str = format!(
                r#"delete from hiqradio_favorite  
                where user_id = ? and stationuuid in ({})"#,
                self.build_in_param(favorites)
            );

            let mut query = sqlx::query(&query_str);
            query = query.bind(user_id);

            for param in favorites {
                query = query.bind(param);
            }
            if let Err(e) = query.execute(&mut *txn).await {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }
        if let Some(group_names) = group_names {
            let query_str = format!(
                r#"delete from hiqradio_favorite  
                where user_id = ? and group_id in (
                    select id from hiqradio_fav_group where name in ({})
                )"#,
                self.build_in_param(group_names)
            );

            let mut query = sqlx::query(&query_str);
            query = query.bind(user_id);

            for param in group_names {
                query = query.bind(param);
            }

            if let Err(e) = query.execute(&mut *txn).await {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn modify_favorite(
        &self,
        user_id: i64,
        stationuuid: &str,
        groups: &Vec<String>,
    ) -> Result {
        if let Err(e) = sqlx::query_as::<_, Favorite>(
            r#"select id, user_id, stationuuid, group_id, create_time
            from hiqradio_favorite 
            where user_id = ? and stationuuid = ?"#,
        )
        .bind(user_id)
        .bind(stationuuid)
        .fetch_one(&self.pool)
        .await
        {
            return match e {
                sqlx::Error::RowNotFound => Err(Error::DatabaseException(format!("station not found",))),

                _ => Err(Error::DatabaseException(e.to_string())),
            };
        }

        let mut txn = self.begin().await?;
        if let Err(e) = sqlx::query(
            r#"delete from hiqradio_favorite  
            where user_id = ? and stationuuid = ?"#,
        )
        .bind(user_id)
        .bind(stationuuid)
        .execute(&mut *txn)
        .await
        {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }

        let query_str = format!(
            r#"select id, user_id, create_time, name, desc, is_def 
        from hiqradio_fav_group
        where user_id = ? and name in ({})"#,
            self.build_in_param(groups)
        );

        let mut query = sqlx::query_as::<_, FavGroup>(&query_str);

        query = query.bind(user_id);
        for param in groups {
            query = query.bind(param);
        }
        let groups = query.fetch_all(&mut *txn).await;
        if let Err(e) = &groups {
            self.rollback(txn).await?;
            return Err(Error::DatabaseException(e.to_string()));
        }
        let groups = groups.unwrap();

        for e in groups {
            if let Err(e) = sqlx::query(
                r#"insert into hiqradio_favorite(user_id, stationuuid, group_id, create_time) 
                values(?, ?, ?, unixepoch(current_timestamp))"#,
            )
            .bind(user_id)
            .bind(stationuuid)
            .bind(e.id.unwrap())
            .execute(&mut *txn)
            .await
            {
                self.rollback(txn).await?;
                return Err(Error::DatabaseException(e.to_string()));
            }
        }

        self.commit(txn).await?;
        Ok(())
    }

    async fn query_sync(
        &self,
        user_id: i64,
        start_time: i64,
    ) -> Result<(Vec<FavGroup>, Vec<Recently>, Vec<StationGroup>)> {
        let fav_groups = sqlx::query_as::<_, FavGroup>(
            r#"select id, user_id, create_time, name, desc, is_def 
            from hiqradio_fav_group
            where user_id = ? and create_time >= ?"#,
        )
        .bind(user_id)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        let recently = sqlx::query_as::<_, Recently>(
            r#"select id, user_id, stationuuid, start_time, end_time 
            from hiqradio_recently
            where user_id = ? and start_time >= ?  order by start_time desc"#,
        )
        .bind(user_id)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        let stations = sqlx::query_as::<_, StationGroup>(
            r#"select a.name as group_name,  b.stationuuid, b.create_time
            from hiqradio_fav_group a, hiqradio_favorite b
            where a.id = b.group_id and a.user_id = b.user_id and a.user_id = ? and b.create_time >= ?"#,
        )
        .bind(user_id)
        .bind(start_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseException(e.to_string()))?;

        Ok((fav_groups, recently, stations))
    }
}
