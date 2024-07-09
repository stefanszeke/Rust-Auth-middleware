use std::sync::Arc;
use crate::auth::jwks::{read_or_fetch_jwks, Jwks};
use axum::middleware::Next;
use axum::extract::{Request, State};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    Json,
    response::Response,
};
use jsonwebtoken::{DecodingKey, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::auth_state::JwkState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth(pub Claims);


pub async fn log_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    println!("log middleware: {} {}", req.method(), req.uri());

    Ok(next.run(req).await)
}

pub async fn auth_middleware(
    State(jwk_state): State<Arc<JwkState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, Response<String>> {
    dotenv::dotenv().ok();
    let audience: String = std::env::var("AUDIENCE").unwrap(); // resource app registration
    // get token from header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| unauthorized_response("No token found"))?;

    // get jwks
    let jwks = read_or_fetch_jwks(&jwk_state).await.unwrap();
    println!("auth middleware jwk_keys: {:?}", jwks.keys.len());

    // get key id from token header
    let decoded_token_header = jsonwebtoken::decode_header(&token)
        .map_err(|_| unauthorized_response("Invalid token"))?;
    println!("\ndecoded_token_header:\n {:?}", decoded_token_header);

    let kid = decoded_token_header
        .kid
        .ok_or_else(|| unauthorized_response("Invalid token"))
        .unwrap();

    // get the key
    let jwk: &crate::auth::Jwk = jwks
        .find_key(&kid)
        .ok_or_else(|| unauthorized_response("Invalid token"))?;

    // verification
    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap();
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[audience]);

    let token_data = jsonwebtoken::decode::<serde_json::Value>(&token, &decoding_key, &validation)
        .map_err(|e| unauthorized_response(format!("Invalid token: {}", e).as_str()))?;

    println!("\ntoken_data:\n{:?}", token_data);

    // get roles
    let roles: Vec<String> = token_data
        .claims
        .get("roles")
        .ok_or_else(|| unauthorized_response("No roles found"))?
        .as_array()
        .unwrap()
        .iter()
        .map(|role| role.as_str().unwrap().to_string())
        .collect();

    req.extensions_mut().insert(Auth(Claims { roles }));

    Ok(next.run(req).await)
}

fn unauthorized_response(message: &str) -> Response<String> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-type", "application/json")
        .body(Json(json!({"message": message})).to_string())
        .unwrap()
}
