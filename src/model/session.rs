use crate::config::CONFIG;
use chrono::Local;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: Option<i64>,
    pub token: String,
    pub user_id: i64,
    pub product_id: i64,
    pub expire: i64,
}

impl Session {
    pub fn token(product_id: i64, user_id: i64) -> Self {
        let expire = Local::now().timestamp() + CONFIG.token_expire;
        let alphabet: [char; 16] = [
            '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
        ];
        let token = nanoid!(32, &alphabet);

        Self {
            id: None,
            token,
            user_id,
            product_id,
            expire,
        }
    }
}
