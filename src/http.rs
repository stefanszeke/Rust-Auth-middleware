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

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, sync::Arc};

    use crate::auth::auth_state::JwkState;

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use http_body_util::BodyExt;
    use mockito::ServerGuard;
    use serde_json::Value;
    use tokio::{net::TcpListener, sync::RwLock};
    use tower::ServiceExt;

    async fn setup_mock_server(
        proxy_status_code: StatusCode,
    ) -> (Router, ServerGuard) {
        let mut server = mockito::Server::new_async().await;

        #[allow(unused)]
        let public_mock = server
            .mock("GET", "/public")
            .with_status(proxy_status_code.as_u16() as usize)
            .with_body(r#"{"message": "Public route"}"#)
            .create_async()
            .await;


        let client = reqwest::Client::new();
        let jwks = Arc::new(RwLock::new(None));

        let state = AppState {
            jwks_state: Arc::new(JwkState {
                client: client.clone(),
                jwks: jwks.clone(),
            }),
        };

        let router: Router = crate::http::create_router(state);
        (router, server)
    }

    #[tokio::test]
    async fn mock_public_should_return_200_with_body() {
        let (router, _server) = setup_mock_server(StatusCode::OK).await;

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/public")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body_bytes.into_iter().collect()).unwrap();
        assert_eq!(&body_str, r#"{"message":"Public route"}"#);
    }

    async fn start_real_server() -> SocketAddr {
        let client = reqwest::Client::new();
        let jwks = Arc::new(RwLock::new(None));

        let state = AppState {
            jwks_state: Arc::new(JwkState {
                client: client.clone(),
                jwks: jwks.clone(),
            }),
        };

        let app = crate::http::create_router(state);

        let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
            println!("Server started at http://localhost:8081");
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        addr
    }

    #[tokio::test]
    async fn public_endpoint_should_return_200_with_body() {
        let addr = start_real_server().await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{}/public", addr))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body: Value = response.json().await.unwrap();
        assert_eq!(body, json!({"message": "Public route"}));
    }
}
