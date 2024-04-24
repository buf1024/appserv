use serde::{Deserialize, Serialize};

pub const PRODUCT_STATUS_NORMAL: &'static str = "00";
pub const PRODUCT_STATUS_CANCEL: &'static str = "99";

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Option<i64>,
    pub product: String,
    pub desc: String,
    pub update_time: i64,
}
