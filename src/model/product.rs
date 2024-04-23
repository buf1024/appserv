// `product` varchar(64) not null,
// `desc` varchar(256) null,
// -- 00 正常 -- 99 已下架
// `status` char(2) not null,
// `update_time` integer null

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
