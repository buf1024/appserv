use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StationGroup {
    pub group_name: String,
    pub stationuuid: String,
}
