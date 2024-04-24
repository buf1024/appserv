use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FavGroup {
    pub id: Option<i64>,
    pub user_id: i64,
    pub create_time: i64,
    pub name: String,
    pub desc: String,
    pub is_def: i64,
}
