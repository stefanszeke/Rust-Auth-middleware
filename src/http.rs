use axum::{
    http::StatusCode, middleware, response::IntoResponse, routing::get, Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::{
    app_state::AppState,
    auth::auth_middleware::{auth_middleware, log_middleware, Auth},
};

pub fn create_router(state: AppState) -> Router {
    let public_router = Router::new()
        .route("/public", get(public))
        .route("/public2", get(public2));

    let secure_router = Router::new()
        .route("/secure", get(secure))
        .route_layer(middleware::from_fn_with_state(state, auth_middleware));

    Router::new()
        .merge(public_router)
        .merge(secure_router)
        .route_layer(middleware::from_fn(log_middleware))
}

#[utoipa::path(
    get,
    path = "/public",
    responses(
        (status = OK, description = "Public route")
    )
)]
async fn public() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "Public route"})))
}

#[utoipa::path(
    get,
    path = "/secure",
    responses(
        (status = OK, description = "Secure route", body = MyResponse ),
        (status = UNAUTHORIZED, description = "Unauthorized", body = MyErrorResponse)
    )
)]

async fn secure(Extension(auth): Extension<Auth>) -> impl IntoResponse {
    let response = MyResponse {
        message: "Secure route".into(),
        roles: auth.0.roles,
    };

    (StatusCode::OK, Json(response))
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"message": "Secure route", "roles": ["admin", "user"]}))]
pub struct MyResponse {
    message: String,
    roles: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"message": "No token found"}))]
pub struct MyErrorResponse {
    message: String,
}

async fn public2() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "Public route 2",})))
}
