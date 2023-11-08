// create table if not exists user (
//     `id` integer not null primary key auto_increment,
//     `user_name` varchar(64) not null,
//     `email` varchar(64) not null,
//     `passwd` varchar(32) not null,
//     -- 00 正常 -- 01 待激活 -- 99 已注销
//     `status` char(2) not null,
//     `avatar` varchar(128) null,
//     -- 其他资料待补充
//     `active_time` datetime not null,
//     `update_time` datetime not null
// );

use serde::{Deserialize, Serialize};

pub const PRODUCT_STATUS_NORMAL: &'static str = "00";
pub const PRODUCT_STATUS_CANCEL: &'static str = "99";

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Option<i64>,
    pub product_no: String,
    pub product_name: String,
    pub desc: String,
    pub status: String,
    pub update_time: Option<chrono::NaiveDateTime>,
}
