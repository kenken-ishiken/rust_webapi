# API Reference

このドキュメントでは、Rust WebAPI の各エンドポイントの詳細な仕様を提供します。

## 目次

- [ヘルスチェック](#ヘルスチェック)
- [メトリクス](#メトリクス)
- [商品管理](#商品管理)
- [カテゴリ管理](#カテゴリ管理)
- [アイテム管理](#アイテム管理)
- [ユーザー管理](#ユーザー管理)
- [削除管理](#削除管理)
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

## 商品管理

### GET /api/products

商品の検索・一覧取得を行います。高度なフィルタリングとソート機能をサポートしています。

**クエリパラメータ**:

| パラメータ | 説明 | デフォルト値 | 例 |
|----------|------|------------|-----|
| q | 検索キーワード（名前、説明で検索） | - | q=ノート |
| category_id | カテゴリIDでフィルタ | - | category_id=1 |
| min_price | 最小価格 | - | min_price=100 |
| max_price | 最大価格 | - | max_price=1000 |
| is_active | アクティブ状態 | true | is_active=false |
| sort | ソートフィールド | id | sort=price |
| order | ソート順 | asc | order=desc |
| limit | 返却数の上限 | 50 | limit=20 |
| offset | 取得開始位置 | 0 | offset=10 |

**curl例**:
```bash
# 基本的な商品一覧取得
curl http://localhost:8080/api/products

# 検索とフィルタリング
curl "http://localhost:8080/api/products?q=ノート&category_id=1&min_price=100&max_price=1000"

# ソートと制限
curl "http://localhost:8080/api/products?sort=price&order=desc&limit=10"
```

**レスポンス例**:
```json
{
  "products": [
    {
      "id": "prod_001",
      "sku": "NB001",
      "name": "ノートブック",
      "description": "A4サイズのノートブック",
      "price": "350.00",
      "category_id": "cat_001",
      "stock_quantity": 100,
      "is_active": true,
      "created_at": "2024-01-15T09:00:00Z",
      "updated_at": "2024-01-15T09:30:00Z"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

### GET /api/products/{id}

指定されたIDの商品詳細を取得します。

**curl例**:
```bash
curl http://localhost:8080/api/products/prod_001
```

**レスポンス例**:
```json
{
  "id": "prod_001",
  "sku": "NB001",
  "name": "ノートブック",
  "description": "A4サイズのノートブック",
  "price": "350.00",
  "category_id": "cat_001",
  "stock_quantity": 100,
  "is_active": true,
  "created_at": "2024-01-15T09:00:00Z",
  "updated_at": "2024-01-15T09:30:00Z"
}
```

### GET /api/products/sku/{sku}

指定されたSKUの商品詳細を取得します。

**curl例**:
```bash
curl http://localhost:8080/api/products/sku/NB001
```

### POST /api/products

新しい商品を作成します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "sku": "NB002",
  "name": "新しいノートブック",
  "description": "B5サイズのノートブック",
  "price": "280.00",
  "category_id": "cat_001",
  "stock_quantity": 50,
  "is_active": true
}
```

**curl例**:
```bash
curl -X POST http://localhost:8080/api/products \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "sku": "NB002",
    "name": "新しいノートブック",
    "description": "B5サイズのノートブック",
    "price": "280.00",
    "category_id": "cat_001",
    "stock_quantity": 50,
    "is_active": true
  }'
```

### PUT /api/products/{id}

商品情報を更新します（全フィールド置換）。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X PUT http://localhost:8080/api/products/prod_001 \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "sku": "NB001",
    "name": "更新されたノートブック",
    "description": "A4サイズの高品質ノートブック",
    "price": "400.00",
    "category_id": "cat_001",
    "stock_quantity": 80,
    "is_active": true
  }'
```

### PATCH /api/products/{id}

商品情報を部分更新します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X PATCH http://localhost:8080/api/products/prod_001 \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "price": "450.00",
    "stock_quantity": 120
  }'
```

### GET /api/products/{id}/history

商品の変更履歴を取得します。

**クエリパラメータ**:
| パラメータ | 説明 | デフォルト値 |
|----------|------|------------|
| limit | 返却数の上限 | 10 |

**curl例**:
```bash
curl "http://localhost:8080/api/products/prod_001/history?limit=5"
```

### GET /api/products/reports/low-stock

在庫が少ない商品のレポートを取得します。

**クエリパラメータ**:
| パラメータ | 説明 | デフォルト値 |
|----------|------|------------|
| threshold | 在庫閾値 | 10 |

**curl例**:
```bash
curl "http://localhost:8080/api/products/reports/low-stock?threshold=50"
```

### GET /api/products/reports/out-of-stock

在庫切れ商品のレポートを取得します。

**curl例**:
```bash
curl http://localhost:8080/api/products/reports/out-of-stock
```

## カテゴリ管理

### GET /api/categories

カテゴリ一覧を取得します。階層構造をサポートしています。

**クエリパラメータ**:

| パラメータ | 説明 | デフォルト値 | 例 |
|----------|------|------------|-----|
| parent_id | 親カテゴリID | - | parent_id=cat_001 |
| include_inactive | 非アクティブカテゴリを含める | false | include_inactive=true |

**curl例**:
```bash
# 全カテゴリ取得
curl http://localhost:8080/api/categories

# 特定の親カテゴリの子カテゴリ取得
curl "http://localhost:8080/api/categories?parent_id=cat_001"

# 非アクティブカテゴリも含める
curl "http://localhost:8080/api/categories?include_inactive=true"
```

**レスポンス例**:
```json
{
  "categories": [
    {
      "id": "cat_001",
      "name": "文房具",
      "description": "筆記用具やノートなど",
      "parent_id": null,
      "is_active": true,
      "created_at": "2024-01-15T09:00:00Z",
      "updated_at": "2024-01-15T09:00:00Z"
    }
  ],
  "total": 1
}
```

### GET /api/categories/{id}

指定されたIDのカテゴリ詳細を取得します。

**curl例**:
```bash
curl http://localhost:8080/api/categories/cat_001
```

### POST /api/categories

新しいカテゴリを作成します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "name": "電子機器",
  "description": "電子製品カテゴリ",
  "parent_id": null,
  "is_active": true
}
```

**curl例**:
```bash
curl -X POST http://localhost:8080/api/categories \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "電子機器",
    "description": "電子製品カテゴリ",
    "parent_id": null,
    "is_active": true
  }'
```

### PUT /api/categories/{id}

カテゴリ情報を更新します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X PUT http://localhost:8080/api/categories/cat_002 \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "電子機器・家電",
    "description": "電子製品と家電製品",
    "parent_id": null,
    "is_active": true
  }'
```

### DELETE /api/categories/{id}

カテゴリを削除します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/categories/cat_002 \
  -H "Authorization: Bearer $ACCESS_TOKEN"
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

### POST /api/items

新しいアイテムを作成します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "name": "新しいアイテム",
  "description": "アイテムの詳細説明"
}
```

**curl例**:
```bash
curl -X POST http://localhost:8080/api/items \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新しいアイテム",
    "description": "アイテムの詳細説明"
  }'
```

### GET /api/items/{id}

指定されたIDのアイテム詳細を取得します。

**curl例**:
```bash
curl http://localhost:8080/api/items/1
```

### PUT /api/items/{id}

アイテム情報を更新します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X PUT http://localhost:8080/api/items/1 \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "更新されたアイテム",
    "description": "更新された説明"
  }'
```

### DELETE /api/items/{id}

アイテムを削除します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/items/1 \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

## 削除管理

本システムでは、論理削除と物理削除の両方をサポートしています。

### DELETE /api/products/{id}

商品を論理削除します（復旧可能）。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/products/prod_001 \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### DELETE /api/products/{id}/permanent

商品を物理削除します（復旧不可能）。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/products/prod_001/permanent \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### POST /api/products/{id}/restore

論理削除された商品を復旧します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X POST http://localhost:8080/api/products/prod_001/restore \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### GET /api/products/{id}/deletion-check

商品が削除可能かどうかをチェックします。

**curl例**:
```bash
curl http://localhost:8080/api/products/prod_001/deletion-check
```

**レスポンス例**:
```json
{
  "can_delete": true,
  "dependencies": [],
  "warnings": []
}
```

### DELETE /api/products/batch

複数の商品を一括削除します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "ids": ["prod_001", "prod_002", "prod_003"],
  "is_physical": false
}
```

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/products/batch \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": ["prod_001", "prod_002", "prod_003"],
    "is_physical": false
  }'
```

### GET /api/products/deleted

削除された商品の一覧を取得します。

**curl例**:
```bash
curl http://localhost:8080/api/products/deleted
```

### GET /api/products/{id}/deletion-log

特定商品の削除ログを取得します。

**curl例**:
```bash
curl http://localhost:8080/api/products/prod_001/deletion-log
```

### GET /api/deletion-logs

全ての削除ログを取得します。

**curl例**:
```bash
curl http://localhost:8080/api/deletion-logs
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

### POST /api/users

新しいユーザーを作成します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "username": "newuser",
  "email": "newuser@example.com",
  "password": "securepassword123"
}
```

**curl例**:
```bash
curl -X POST http://localhost:8080/api/users \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "email": "newuser@example.com",
    "password": "securepassword123"
  }'
```

### GET /api/users/{id}

指定されたIDのユーザー詳細を取得します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8080/api/users/1
```

**レスポンス例**:
```json
{
  "id": 1,
  "username": "user1",
  "email": "user1@example.com",
  "created_at": "2024-01-15T09:00:00Z",
  "updated_at": "2024-01-15T09:30:00Z"
}
```

### PUT /api/users/{id}

ユーザー情報を更新します。

**認証要件**: JWT トークンが必要

**リクエストボディ**:
```json
{
  "username": "updateduser",
  "email": "updated@example.com"
}
```

**curl例**:
```bash
curl -X PUT http://localhost:8080/api/users/1 \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "updateduser",
    "email": "updated@example.com"
  }'
```

### DELETE /api/users/{id}

ユーザーを削除します。

**認証要件**: JWT トークンが必要

**curl例**:
```bash
curl -X DELETE http://localhost:8080/api/users/1 \
  -H "Authorization: Bearer $ACCESS_TOKEN"
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
