use async_session::MemoryStore;
use axum::extract::FromRef;

use crate::{
    config::CONFIG, repo::{self, DynAppServRepo}, Result
};

#[derive(Clone)]
pub struct AppState {
    pub store: MemoryStore,
    pub repo: DynAppServRepo,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let store = MemoryStore::new();
        let repo = repo::new(&CONFIG.db_url).await?;
        Ok(Self { store, repo })
    }
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(input: &AppState) -> Self {
        input.store.clone()
    }
}
