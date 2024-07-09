use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use crate::{app_state::AppState, auth::auth_middleware::Auth};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/public", get(public))
        .route("/secure", get(secure))
        .with_state(state)
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

async fn secure(Auth(claims): Auth) -> impl IntoResponse {
    let response = MyResponse {
        message: "Secure route".into(),
        roles: claims.roles,
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