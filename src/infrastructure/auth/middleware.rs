use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpResponse, ResponseError};
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use super::keycloak::KeycloakClaims;

#[cfg(feature = "test-support")]
use crate::infrastructure::auth::keycloak::{RealmAccess, ResourceAccess, Account};

pub struct KeycloakUser {
    pub claims: KeycloakClaims,
}

#[derive(Debug)]
pub enum AuthError {
    #[allow(dead_code)]
    Unauthorized(String),
    #[allow(dead_code)]
    InternalError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AuthError::InternalError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::Unauthorized(msg) => HttpResponse::Unauthorized().json(msg),
            AuthError::InternalError(msg) => HttpResponse::InternalServerError().json(msg),
        }
    }
}

#[cfg(not(feature = "test-support"))]
impl FromRequest for KeycloakUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            // BearerAuthからトークンを抽出
            let auth = BearerAuth::extract(&req).await.map_err(|_| {
                error!("認証トークンが見つかりません");
                AuthError::Unauthorized("認証トークンが必要です".to_string())
            })?;

            // Keycloak認証サービスを取得
            let auth_service = req.app_data::<Data<KeycloakAuth>>().ok_or_else(|| {
                error!("KeycloakAuthサービスが設定されていません");
                AuthError::InternalError("サーバー設定エラー".to_string())
            })?;

            // トークンを検証
            let token = auth.token();
            let token_data = auth_service.verify_token(token).await.map_err(|e| {
                let error_message = match e {
                    KeycloakError::TokenExpired => "トークンの有効期限が切れています",
                    KeycloakError::InvalidToken => "無効なトークンです",
                    _ => "認証に失敗しました",
                };
                error!("認証エラー: {}", e);
                AuthError::Unauthorized(error_message.to_string())
            })?;

            // 検証成功：ユーザー情報を返す
            let claims = token_data.claims;

            Ok(KeycloakUser { claims })
        })
    }
}

#[cfg(feature = "test-support")]
impl FromRequest for KeycloakUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(_req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        Box::pin(async move {
            Ok(KeycloakUser {
                claims: KeycloakClaims {
                    exp: 9999999999,
                    iat: 0,
                    auth_time: 0,
                    jti: "dummy-jti".to_string(),
                    iss: "dummy-iss".to_string(),
                    aud: "dummy-aud".to_string(),
                    sub: "dummy-sub".to_string(),
                    typ: "Bearer".to_string(),
                    azp: "dummy-azp".to_string(),
                    session_state: "dummy-session".to_string(),
                    acr: "dummy-acr".to_string(),
                    realm_access: RealmAccess { roles: vec!["user".to_string()] },
                    resource_access: ResourceAccess { account: Account { roles: vec!["user".to_string()] } },
                    scope: "openid profile email".to_string(),
                    sid: "dummy-sid".to_string(),
                    email_verified: true,
                    name: "Test User".to_string(),
                    preferred_username: "testuser".to_string(),
                    given_name: "Test".to_string(),
                    family_name: "User".to_string(),
                    email: "test@example.com".to_string(),
                }
            })
        })
    }
}
