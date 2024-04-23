use std::fmt::Display;

use chrono::Local;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{config::CONFIG, errors::Error, Result};

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
pub static JWT_KEY: Lazy<Keys> = Lazy::new(|| Keys::new(CONFIG.jwt_key.as_bytes()));

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub product_id: i64,
    pub user_id: i64,
    pub exp: i64,
}

impl Claims {
    pub fn token(product_id: i64, user_id: i64) -> Result<String> {
        let exp = Local::now().timestamp() + CONFIG.jwt_exp;
        let claims = Claims {
            product_id,
            user_id,
            exp,
        };
        // Create the authorization token
        let token = encode(&Header::default(), &claims, &JWT_KEY.encoding)
            .map_err(|_| Error::TokenCreation)?;

        Ok(token)
    }
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "product_id: {}\nuser_id: {}\nexpire: {}",
            self.product_id, self.user_id, self.exp
        )
    }
}
