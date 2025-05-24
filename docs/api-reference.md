# API Reference

このドキュメントでは、Rust WebAPI の各エンドポイントの詳細な仕様を提供します。

## 目次

- [ヘルスチェック](#ヘルスチェック)
- [メトリクス](#メトリクス)
- [アイテム管理](#アイテム管理)
- [ユーザー管理](#ユーザー管理)
- [認証・認可](#認証認可)

## ヘルスチェック

### GET /

サーバーの基本的な稼働確認を行います。

**curl例**:
```bash
curl http://localhost:8080/
```

**レスポンス例**:

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### GET /api/health

アプリケーションとその依存サービスの詳細なヘルスチェックを行います。

**curl例**:
```bash
curl http://localhost:8080/api/health
```

**レスポンス例**:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "dependencies": {
    "database": "ok",
    "keycloak": "ok"
  },
  "uptime": 3600
}
```

## メトリクス

### GET /api/metrics

Prometheus 形式のメトリクスを提供します。

**curl例**:
```bash
curl http://localhost:8080/api/metrics
```

**レスポンス例**:

```
# HELP api_request_duration_seconds HTTP request duration in seconds
# TYPE api_request_duration_seconds histogram
api_request_duration_seconds_bucket{service="rust_webapi",endpoint="/api/items",le="0.005"} 2
api_request_duration_seconds_bucket{service="rust_webapi",endpoint="/api/items",le="0.01"} 5
...
```

## アイテム管理

### GET /api/items

登録されているアイテムの一覧を取得します。

**クエリパラメータ**:

| パラメータ | 説明 | デフォルト値 | 例 |
|----------|------|------------|-----|
| limit    | 返却するアイテム数の上限 | 100 | limit=50 |
| offset   | 取得開始位置 | 0 | offset=10 |
| sort     | ソートフィールド | id | sort=name |
| order    | ソート順 (asc, desc) | asc | order=desc |

**curl例**:
```bash
# 基本的な取得
curl http://localhost:8080/api/items

# パラメータ付きの取得
curl "http://localhost:8080/api/items?limit=10&offset=0&sort=name&order=desc"
```

**レスポンス例**:

```json
{
  "items": [
    {
      "id": 1,
      "name": "アイテム名",
      "description": "アイテムの説明"
    },
    {
      "id": 2,
      "name": "別のアイテム",
      "description": "別の説明"
    }
  ],
  "total": 2,
  "limit": 100,
  "offset": 0
}
```

## ユーザー管理

### GET /api/users

登録されているユーザーの一覧を取得します。

**認証要件**: JWT トークンが必要

**クエリパラメータ**:

| パラメータ | 説明 | デフォルト値 | 例 |
|----------|------|------------|-----|
| limit    | 返却するユーザー数の上限 | 100 | limit=50 |
| offset   | 取得開始位置 | 0 | offset=10 |

**curl例**:
```bash
# 環境変数にトークンを設定
export ACCESS_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."

# 認証ありでユーザー一覧を取得
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8080/api/users

# パラメータ付きで取得
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  "http://localhost:8080/api/users?limit=5&offset=0"
```

**レスポンス例**:

```json
{
  "users": [
    {
      "id": 1,
      "username": "user1",
      "email": "user1@example.com"
    },
    {
      "id": 2,
      "username": "user2",
      "email": "user2@example.com"
    }
  ],
  "total": 2,
  "limit": 100,
  "offset": 0
}
```

## 認証・認可

このAPIはKeycloakと連携したOAuth2/OpenID Connectベースの認証を実装しています。

### 認証フロー

1. クライアントはKeycloakから認証トークン（JWT）を取得します
2. 取得したJWTトークンを Authorization ヘッダにBearerトークンとして付与してAPIにリクエストを送信します
3. APIはトークンの有効性を検証し、権限に基づいてリクエストを処理します

### トークン取得の例

**password grant type を使用した認証**:
```bash
# トークンを取得
curl -X POST "http://localhost:8081/realms/rust-webapi/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=api-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "username=testuser" \
  -d "password=testpass123" \
  -d "grant_type=password"
```

**レスポンス例**:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 300,
  "refresh_expires_in": 1800,
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer"
}
```

### 認証済みAPIの使用

取得したトークンを使用してAPIにアクセス：

```bash
# 1. トークンを取得
TOKEN_RESPONSE=$(curl -s -X POST "http://localhost:8081/realms/rust-webapi/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=api-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "username=testuser" \
  -d "password=testpass123" \
  -d "grant_type=password")

# 2. アクセストークンを抽出
ACCESS_TOKEN=$(echo $TOKEN_RESPONSE | jq -r '.access_token')

# 3. 認証が必要なエンドポイントにアクセス
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8080/api/users
```

### エラーレスポンス

認証エラー時のレスポンス例：

```json
{
  "error": "unauthorized",
  "message": "Invalid or missing authentication token"
}
```

### 詳細な認証設定

Keycloakの詳細な設定方法については、[Keycloakセットアップガイド](keycloak-setup.md) を参照してください。
