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

**レスポンス例**:

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### GET /api/health

アプリケーションとその依存サービスの詳細なヘルスチェックを行います。

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

1. クライアントはKeycloakから認証トークン（JWT）を取得します
2. 取得したJWTトークンを Authorization ヘッダにBearerトークンとして付与してAPIにリクエストを送信します
3. APIはトークンの有効性を検証し、権限に基づいてリクエストを処理します
