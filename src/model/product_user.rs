// create table if not exists user_product (
//     `id` integer not null primary key autoincrement,
//     `product_id` varchar(64) not null,
//     `user_id` varchar(64) not null,
//     `avatar` varchar(256) null,
//     -- 00 正常 -- 99 已注销
//     `status` char(2) not null,
//     `update_time` integer not null
// );

use serde::{Deserialize, Serialize};

pub const USER_PRODUCT_STATUS_NORMAL: &'static str = "00";
pub const USER_PRODUCT_STATUS_CANCEL: &'static str = "99";

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProductUser {
    pub user_id: i64,
    pub user_name: String,
    pub email: String,
    pub product_id: i64,
    pub avatar: String,
    pub status: String,
    pub update_time: i64,
}
