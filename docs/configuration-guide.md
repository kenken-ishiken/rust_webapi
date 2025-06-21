# Configuration Guide / 設定ガイド

このドキュメントでは、Rust WebAPIの設定管理システムについて説明します。

## 目次

- [概要](#概要)
- [設定構造](#設定構造)
- [環境変数](#環境変数)
- [デフォルト値](#デフォルト値)
- [検証](#検証)
- [カスタマイズ](#カスタマイズ)

## 概要

Rust WebAPIは環境変数ベースの設定管理システムを採用しています。すべての設定は`AppConfig`構造体に集約され、起動時に検証されます。

## 設定構造

```rust
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub telemetry: TelemetryConfig,
}
```

### DatabaseConfig

データベース接続に関する設定：

| 環境変数 | 説明 | 必須 | デフォルト値 |
|----------|------|------|--------------|
| `DATABASE_URL` | PostgreSQL接続URL | ✅ | なし |
| `DATABASE_MAX_CONNECTIONS` | 最大接続数 | ❌ | 5 |

例：
```bash
DATABASE_URL=postgres://user:password@localhost/dbname
DATABASE_MAX_CONNECTIONS=10
```

### ServerConfig

HTTPおよびgRPCサーバーの設定：

| 環境変数 | 説明 | 必須 | デフォルト値 |
|----------|------|------|--------------|
| `HTTP_HOST` | HTTPサーバーのホスト | ❌ | 127.0.0.1 |
| `HTTP_PORT` | HTTPサーバーのポート | ❌ | 8080 |
| `GRPC_HOST` | gRPCサーバーのホスト | ❌ | 127.0.0.1 |
| `GRPC_PORT` | gRPCサーバーのポート | ❌ | 50051 |

例：
```bash
HTTP_HOST=0.0.0.0
HTTP_PORT=3000
GRPC_HOST=0.0.0.0
GRPC_PORT=50052
```

### AuthConfig

Keycloak認証の設定：

| 環境変数 | 説明 | 必須 | デフォルト値 |
|----------|------|------|--------------|
| `KEYCLOAK_REALM` | Keycloakレルム | ✅ | なし |
| `KEYCLOAK_AUTH_SERVER_URL` | Keycloak認証サーバーURL | ✅ | なし |
| `KEYCLOAK_CLIENT_ID` | KeycloakクライアントID | ✅ | なし |

例：
```bash
KEYCLOAK_REALM=my-realm
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8080/auth
KEYCLOAK_CLIENT_ID=my-client
```

### TelemetryConfig

ロギングとトレーシングの設定：

| 環境変数 | 説明 | 必須 | デフォルト値 |
|----------|------|------|--------------|
| `SERVICE_NAME` | サービス名（ログ、メトリクスで使用） | ❌ | rust_webapi |
| `LOG_LEVEL` | ログレベル | ❌ | info |

例：
```bash
SERVICE_NAME=my-api-service
LOG_LEVEL=debug
```

## 検証

起動時に以下の検証が行われます：

1. **必須フィールドの存在確認**
   - DATABASE_URL、KEYCLOAK関連の環境変数

2. **値の妥当性チェック**
   - ポート番号が0より大きい
   - 最大接続数が0より大きい
   - URLが空でない

3. **型の検証**
   - 数値型の環境変数が正しくパースできる

検証エラーの場合、わかりやすいエラーメッセージと共に起動を中止します。

## カスタマイズ

### 新しい設定項目の追加

1. 設定構造体に新しいフィールドを追加：

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CustomConfig {
    pub my_setting: String,
    pub my_number: u32,
}
```

2. `AppConfig`に追加：

```rust
pub struct AppConfig {
    // ... existing fields ...
    pub custom: CustomConfig,
}
```

3. 環境変数からの読み込みを実装：

```rust
impl CustomConfig {
    fn from_env() -> StartupResult<Self> {
        Ok(Self {
            my_setting: env::var("MY_SETTING")
                .unwrap_or_else(|_| "default".to_string()),
            my_number: env::var("MY_NUMBER")
                .unwrap_or_else(|_| "42".to_string())
                .parse()
                .map_err(|_| StartupError::Configuration("Invalid MY_NUMBER".to_string()))?,
        })
    }
}
```

### 設定ファイルのサポート

将来的に設定ファイル（YAML、TOML）をサポートする場合：

```rust
impl AppConfig {
    pub fn from_file(path: &str) -> StartupResult<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| StartupError::Configuration(format!("Failed to read config file: {}", e)))?;
        
        toml::from_str(&contents)
            .map_err(|e| StartupError::Configuration(format!("Failed to parse config: {}", e)))
    }
}
```

## ベストプラクティス

1. **環境変数のプレフィックス**
   - アプリケーション固有のプレフィックスを使用（例：`RUST_WEBAPI_`）

2. **シークレットの管理**
   - パスワードなどの機密情報は環境変数で管理
   - `.env`ファイルは絶対にコミットしない

3. **デフォルト値**
   - 開発を容易にするため、適切なデフォルト値を設定
   - 本番環境では明示的に設定することを推奨

4. **ドキュメント**
   - 新しい設定項目は必ずドキュメント化
   - 例を含める

## トラブルシューティング

### 環境変数が読み込まれない

```bash
# .envファイルの確認
cat .env

# 環境変数の確認
echo $DATABASE_URL

# 実行時に環境変数を設定
DATABASE_URL=postgres://... cargo run
```

### 設定エラーのデバッグ

起動時のエラーメッセージを確認：

```
Error: Configuration error: DATABASE_URL cannot be empty
```

詳細なログを有効化：

```bash
RUST_LOG=debug cargo run
``` 