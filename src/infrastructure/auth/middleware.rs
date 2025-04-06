use actix_web::{dev::Payload, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse};
use actix_web::web::Data;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures::future::{ready, Ready};
use log::{error, info};
use std::future::Future;
use std::pin::Pin;

use super::keycloak::{KeycloakAuth, KeycloakClaims, KeycloakError};

pub struct KeycloakUser {
    pub claims: KeycloakClaims,
}

impl FromRequest for KeycloakUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();
        
        Box::pin(async move {
            // BearerAuthからトークンを抽出
            let auth = BearerAuth::extract(&req).await
                .map_err(|_| {
                    error!("認証トークンが見つかりません");
                    HttpResponse::Unauthorized()
                        .json("認証トークンが必要です")
                        .into_error()
                })?;
            
            // Keycloak認証サービスを取得
            let auth_service = req.app_data::<Data<KeycloakAuth>>()
                .ok_or_else(|| {
                    error!("KeycloakAuthサービスが設定されていません");
                    HttpResponse::InternalServerError()
                        .json("サーバー設定エラー")
                        .into_error()
                })?;
            
            // トークンを検証
            let token = auth.token();
            let token_data = auth_service.verify_token(token).await
                .map_err(|e| {
                    let error_message = match e {
                        KeycloakError::TokenExpired => "トークンの有効期限が切れています",
                        KeycloakError::InvalidToken => "無効なトークンです",
                        _ => "認証に失敗しました",
                    };
                    error!("認証エラー: {}", e);
                    HttpResponse::Unauthorized()
                        .json(error_message)
                        .into_error()
                })?;
            
            // 検証成功：ユーザー情報を返す
            let claims = token_data.claims;
            
            Ok(KeycloakUser { claims })
        })
    }
}
