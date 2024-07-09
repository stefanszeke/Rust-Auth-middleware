use axum::http::Response;
use serde::{Deserialize, Serialize};
use tokio::time::{self, Duration};

use super::auth_state::JwkState;



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwk {
    pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub kid: String,
    pub n: String,
    pub e: String,
    pub x5c: Option<Vec<String>>,
    pub x5t: Option<String>,
    pub alg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Jwks {
    pub fn find_key(&self, kid: &str) -> Option<&Jwk> {
        self.keys.iter().find(|key| key.kid == kid)
    }
}


pub async fn fetch_jwks(client: &reqwest::Client) -> Result<Jwks, reqwest::Error> {
    let tenant_id = std::env::var("TENANT_ID").unwrap();
    let jwks_url = format!(
        "https://login.microsoftonline.com/{}/discovery/v2.0/keys",
        tenant_id
    );

    let response = client.get(&jwks_url).send().await?;
    let jwks = response.json().await?;
    Ok(jwks)
}

pub async fn read_or_fetch_jwks(state: &JwkState) -> Result<Jwks, Response<String>> {
    let jwks_option = {
        let read_guard = state.jwks.read().await;
        read_guard.clone()
    };

    println!("getting token keys");
    if let Some(jwks) = jwks_option {
        println!("getting token keys from cache");
        Ok(jwks)
    } else {
        match fetch_jwks(&state.client).await {
            Ok(jwks) => {
                println!("getting token keys from fetch");
                let mut write_guard = state.jwks.write().await;
                *write_guard = Some(jwks.clone());
                Ok(jwks)
            }
            Err(e) => {
                eprintln!("Failed to fetch JWKS: {:?}", e);
                Err(Response::new("Failed to fetch JWKS".to_string()))
            }
        }
    }
}


#[allow(dead_code)]
pub async fn refresh_jwks(state: &JwkState) {
    loop {
        match fetch_jwks(&state.client).await {
            Ok(jwks) => {
                let mut write_guard = state.jwks.write().await;
                *write_guard = Some(jwks);
            }
            Err(e) => {
                eprintln!("Failed to fetch JWKS: {:?}", e);
            }
        }
        time::sleep(Duration::from_secs(3600)).await;
    }
}
