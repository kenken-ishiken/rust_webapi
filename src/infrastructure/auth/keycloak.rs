use base64::Engine;
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum KeycloakError {
    InvalidToken,
    #[allow(dead_code)]
    TokenExpired,
    JwtError(jsonwebtoken::errors::Error),
    ReqwestError(reqwest::Error),
    Other(String),
}

impl fmt::Display for KeycloakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeycloakError::InvalidToken => write!(f, "Invalid token"),
            KeycloakError::TokenExpired => write!(f, "Token expired"),
            KeycloakError::JwtError(e) => write!(f, "JWT error: {}", e),
            KeycloakError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            KeycloakError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl Error for KeycloakError {}

impl From<jsonwebtoken::errors::Error> for KeycloakError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        KeycloakError::JwtError(error)
    }
}

impl From<reqwest::Error> for KeycloakError {
    fn from(error: reqwest::Error) -> Self {
        KeycloakError::ReqwestError(error)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakClaims {
    pub exp: usize,
    pub iat: usize,
    pub auth_time: usize,
    pub jti: String,
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub typ: String,
    pub azp: String,
    pub session_state: String,
    pub acr: String,
    pub realm_access: RealmAccess,
    pub resource_access: ResourceAccess,
    pub scope: String,
    pub sid: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAccess {
    #[serde(rename = "account")]
    pub account: Account,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    pub realm: String,
    pub auth_server_url: String,
    pub client_id: String,
}

impl KeycloakConfig {
    pub fn new(realm: String, auth_server_url: String, client_id: String) -> Self {
        Self {
            realm,
            auth_server_url,
            client_id,
        }
    }

    pub fn from_auth_config(config: &crate::infrastructure::config::AuthConfig) -> Self {
        Self::new(
            config.keycloak_realm.clone(),
            config.keycloak_auth_server_url.clone(),
            config.keycloak_client_id.clone(),
        )
    }

    pub fn get_jwks_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/certs",
            self.auth_server_url, self.realm
        )
    }
}

#[derive(Debug, Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    #[allow(dead_code)]
    kty: String,
    #[allow(dead_code)]
    #[serde(rename = "use")]
    usage: String,
    n: String,
    e: String,
    #[allow(dead_code)]
    alg: Option<String>,
}

pub struct KeycloakAuth {
    config: KeycloakConfig,
    http_client: Client,
}

impl KeycloakAuth {
    pub fn new(config: KeycloakConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    pub async fn verify_token(
        &self,
        token: &str,
    ) -> Result<TokenData<KeycloakClaims>, KeycloakError> {
        // JWKSエンドポイントからキーを取得
        let jwks_url = self.config.get_jwks_url();
        let jwks = self
            .http_client
            .get(&jwks_url)
            .send()
            .await?
            .json::<JwkSet>()
            .await?;

        // トークンのヘッダーを解析してkidを取得
        let header = jsonwebtoken::decode_header(token)?;
        let kid = header.kid.ok_or(KeycloakError::InvalidToken)?;

        // 対応するJWKを検索
        let jwk = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or(KeycloakError::InvalidToken)?;

        // JWKからRSA公開鍵を作成
        // トークンを検証
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        // audienceとして"account"と client_id の両方を許可
        validation.set_audience(&["account", &self.config.client_id]);
        validation.set_issuer(&[format!(
            "{}/realms/{}",
            self.config.auth_server_url, self.config.realm
        )]);

        let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
        let token_data = decode::<KeycloakClaims>(token, &key, &validation)?;

        Ok(token_data)
    }
}

#[allow(dead_code)]
fn base64_url_decode(input: &str) -> Result<Vec<u8>, KeycloakError> {
    let input = input.replace('-', "+").replace('_', "/");
    let padding = match input.len() % 4 {
        0 => "",
        1 => "===",
        2 => "==",
        3 => "=",
        _ => unreachable!(),
    };
    let input = format!("{}{}", input, padding);

    base64::engine::general_purpose::STANDARD
        .decode(&input)
        .map_err(|e| KeycloakError::Other(format!("Base64 decode error: {}", e)))
}
