# Rust WebAPI プロジェクトのカスタムインストラクション

## 最優先事項
- **エラー処理**: 本番コードでは`unwrap()`や`expect()`を避け、`Result`と`Option`を適切に処理してください。
- **非同期処理**: すべてのI/O操作には`async/await`パターンを使用してください。
- **セキュリティ**: すべてのユーザー入力を検証し、OWASPのガイドラインに従ってください。

## コードスタイルと構造
- 4スペースインデント、スネークケース（`snake_case`）の命名規則を使用してください。
- モジュール構造は以下の例に従ってください：

```rust
// src/main.rs または src/lib.rs
mod api;
mod models;
mod services;
mod repositories;
mod config;
mod errors;

// 標準ライブラリのインポートを先に
use std::sync::Arc;
// 次に外部クレート
use axum::{Router, Server};
use tokio::net::TcpListener;
// 最後に内部モジュール
use crate::config::AppConfig;
```

## APIエンドポイント実装例

```rust
// src/api/users.rs
use axum::{extract::State, Json, Router};
use axum::routing::{get, post};

use crate::models::User;
use crate::services::UserService;
use crate::errors::AppError;

pub fn router() -> Router {
    Router::new()
        .route("/users", get(get_users).post(create_user))
        .route("/users/:id", get(get_user_by_id))
}

async fn get_users(
    State(service): State<Arc<UserService>>
) -> Result<Json<Vec<User>>, AppError> {
    let users = service.get_all().await?;
    Ok(Json(users))
}
```

## エラー処理パターン

```rust
// src/errors.rs
use axum::{response::{IntoResponse, Response}, http::StatusCode, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("データベースエラー: {0}")]
    Database(#[from] sqlx::Error),

    #[error("認証エラー: {0}")]
    Auth(String),

    #[error("リソースが見つかりません")]
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
        };

        tracing::error!(?self);

        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
```

## 依存性注入パターン

```rust
// src/main.rs
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 設定の読み込み
    let config = AppConfig::from_env()?;

    // データベース接続
    let db_pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?

    // サービスの初期化
    let user_repo = Arc::new(UserRepository::new(db_pool.clone()));
    let user_service = Arc::new(UserService::new(user_repo));

    // アプリケーションの状態を構築
    let app_state = Arc::new(AppState {
        user_service,
        // 他のサービス
    });

    // ルーターの設定
    let app = Router::new()
        .merge(api::users::router())
        .with_state(app_state);

    // サーバーの起動
    let listener = TcpListener::bind(&config.server_addr).await?;
    Server::from_tcp(listener)?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

## テスト例

```rust
// src/services/users_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use crate::repositories::MockUserRepository;

    #[tokio::test]
    async fn test_get_user_by_id_success() {
        // モックの設定
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1))
            .times(1)
            .returning(|_| Ok(User {
                id: 1,
                name: "テストユーザー".to_string(),
                email: "test@example.com".to_string(),
            }));

        let service = UserService::new(Arc::new(mock_repo));

        // テスト実行
        let result = service.get_by_id(1).await;

        // 検証
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "テストユーザー");
    }
}
```

## 技術スタック
- **Webフレームワーク**: axumを優先的に使用（Actix-Webも可）
- **データベース**: sqlx（PostgreSQL）を使用
- **認証**: JWTベースの認証（jsonwebtoken）
- **シリアライゼーション**: serde
- **ロギング**: tracing
- **テスト**: tokio::test、mockall
- **エラー処理**: thiserror、anyhow

## セキュリティのベストプラクティス
- ユーザーパスワードは必ずargon2でハッシュ化してください。
- 認証エンドポイントには必ずレート制限を実装してください。
- 機密情報は環境変数または.envファイル（dotenv）で管理し、ソースコードにハードコードしないでください。

## パフォーマンス最適化
- データベースクエリには常にインデックスを適切に設定してください。
- 頻繁にアクセスされるデータにはRedisなどを使用したキャッシュ層を検討してください。
- 重いCPU処理は`tokio::spawn`を使用して別スレッドで実行してください。
