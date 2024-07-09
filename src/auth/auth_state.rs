use std::sync::Arc;

use crate::app_state::AppState;
use axum::extract::FromRef;
use tokio::sync::RwLock;
use super::jwks::Jwks;

#[derive(Clone)]
pub struct JwkState {
    pub client: reqwest::Client,
    pub jwks: Arc<RwLock<Option<Jwks>>>,
}

impl FromRef<AppState> for Arc<JwkState> {
    fn from_ref(state: &AppState) -> Arc<JwkState> {
        state.jwks_state.clone()
    }
}
