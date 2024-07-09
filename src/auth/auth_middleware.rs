use std::sync::Arc;

use crate::auth::jwks::{read_or_fetch_jwks, Jwks};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, Response},
    Json,
};
use jsonwebtoken::{DecodingKey, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::auth_state::JwkState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth(pub Claims);


#[async_trait]
impl<S> FromRequestParts<S> for Auth
where
    Arc<JwkState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response<String>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwk_state = Arc::<JwkState>::from_ref(state);
        let audience: String = std::env::var("AUDIENCE").unwrap(); // resource app registration

        // get token from header
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| unauthorized_response("No token found"))?;

        // get jwks
        let jwks: Jwks = read_or_fetch_jwks(&jwk_state).await.unwrap();

        // get key id from token header
        let decoded_token_header = jsonwebtoken::decode_header(&token)
            .map_err(|_| unauthorized_response("Invalid token"))?;
        println!("\ndecoded_token_header:\n {:?}", decoded_token_header);

        let kid = decoded_token_header
            .kid
            .ok_or_else(|| unauthorized_response("Invalid token"))?;

        // get the key
        let jwk: &crate::auth::Jwk = jwks
            .find_key(&kid)
            .ok_or_else(|| unauthorized_response("Invalid token"))?;

        // verification
        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap();
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[audience]);

        let token_data =
            jsonwebtoken::decode::<serde_json::Value>(&token, &decoding_key, &validation)
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

        Ok(Auth(Claims { roles }))
    }
}

fn unauthorized_response(message: &str) -> Response<String> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-type", "application/json")
        .body(Json(json!({"message": message})).to_string())
        .unwrap()
}

