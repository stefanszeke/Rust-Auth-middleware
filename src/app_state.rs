use std::sync::Arc;
use crate::auth::auth_state::JwkState;

#[derive(Clone)]
pub struct AppState {
    pub jwks_state: Arc<JwkState>,
}
