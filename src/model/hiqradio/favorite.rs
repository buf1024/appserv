use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Favorite {
    pub id: Option<i64>,
    pub user_id: i64,
    pub stationuuid: String,
    pub group_id: i64,
}
