use std::sync::Arc;

use rust_jwks::auth::auth_state::JwkState;
use rust_jwks::http;
use tokio::sync::RwLock;

use tokio::net::TcpListener;

use rust_jwks::app_state::AppState;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client = reqwest::Client::new();
    let jwks = Arc::new(RwLock::new(None));

    let state = AppState {
        jwks_state: Arc::new(JwkState { client: client.clone(), jwks: jwks.clone() }),
    };

    let app = http::create_router(state);

    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();
    println!("\n✅ Server started at http://localhost:8081\n");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

