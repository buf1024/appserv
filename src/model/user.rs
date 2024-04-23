// `id` integer not null primary key autoincrement,
// `user_name` varchar(64) null,
// `email` varchar(64) not null,
// `passwd` char(64) not null,
// -- 00 正常 -- 99 已注销
// `status` char(2) not null,
// `update_time` integer null

use serde::{Deserialize, Serialize};

pub const USER_STATUS_NORMAL: &'static str = "00";
pub const USER_STATUS_CANCEL: &'static str = "99";

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub user_name: String,
    pub email: String,
    pub passwd: String,
    pub status: String,
    pub update_time: i64,
}
