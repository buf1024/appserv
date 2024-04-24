use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Recently {
    pub id: Option<i64>,
    pub user_id: i64,
    pub stationuuid: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
}
