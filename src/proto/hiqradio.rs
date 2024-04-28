use serde::{Deserialize, Serialize};

use crate::model::hiqradio::{FavGroup, Recently, StationGroup};

/// 最近播放
#[derive(Debug, Serialize)]
pub struct RecentlyRsp {
    pub error: usize,
    pub message: String,
    pub recently: Vec<Recently>,
}

/// 新增记录
#[derive(Debug, Deserialize)]
pub struct RecentlyNew {
    pub stationuuid: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct RecentlyNewReq {
    pub new_recently: Vec<RecentlyNew>,
}

#[derive(Debug, Deserialize)]
pub struct GroupsReq {
    pub groups: Option<Vec<String>>,
}
/// 最近播放
#[derive(Debug, Serialize)]
pub struct GroupsRsp {
    pub error: usize,
    pub message: String,
    pub groups: Vec<FavGroup>,
}

#[derive(Debug, Deserialize)]
pub struct GroupDeleteReq {
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GroupNew {
    pub create_time: i64,
    pub name: String,
    pub desc: String,
    pub is_def: i64,
}

#[derive(Debug, Deserialize)]
pub struct GroupNewReq {
    pub new_group: Vec<GroupNew>,
}

#[derive(Debug, Deserialize)]
pub struct GroupModifyReq {
    pub old_name: String,
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Serialize)]
pub struct FavoritesRsp {
    pub error: usize,
    pub message: String,
    pub favorites: Vec<StationGroup>,
}

#[derive(Debug, Deserialize)]
pub struct FavoriteNewReq {
    pub new_favorite: Vec<StationGroup>,
}

#[derive(Debug, Deserialize)]
pub struct FavoriteDeleteReq {
    pub favorites: Option<Vec<String>>,
    pub group_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FavoriteModifyReq {
    pub stationuuid: String,
    pub group_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncReq {
    pub start_time: i64,
}

#[derive(Debug, Serialize)]
pub struct SyncRsp {
    pub error: usize,
    pub message: String,
    pub groups: Vec<FavGroup>,
    pub recently: Vec<Recently>,
    pub favorites: Vec<StationGroup>,
}
