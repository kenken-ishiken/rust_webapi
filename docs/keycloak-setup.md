# Keycloak セットアップガイド

このドキュメントでは、Rust WebAPI と連携するための Keycloak の設定方法を説明します。

## 目次

- [概要](#概要)
- [Keycloak のインストール](#keycloak-のインストール)
- [レルムの作成](#レルムの作成)
- [クライアントの設定](#クライアントの設定)
- [ユーザーとロールの設定](#ユーザーとロールの設定)
- [API での認証テスト](#api-での認証テスト)
- [トラブルシューティング](#トラブルシューティング)

## 概要

Rust WebAPI は Keycloak を使用して JWT ベースの認証を実装しています。この設定により、以下が可能になります：

- ユーザー認証とセッション管理
- ロールベースのアクセス制御
- シングルサインオン (SSO)
- トークンの自動更新

## Keycloak のインストール

### Docker を使用したインストール

最も簡単な方法は Docker を使用することです：

```bash
# Keycloak コンテナを起動
docker run -d \
  --name keycloak \
  -p 8081:8080 \
  -e KEYCLOAK_ADMIN=admin \
  -e KEYCLOAK_ADMIN_PASSWORD=admin123 \
  quay.io/keycloak/keycloak:latest \
  start-dev
```

### Docker Compose での設定

`docker-compose.yml` に以下を追加：

```yaml
version: '3.8'
services:
  keycloak:
    image: quay.io/keycloak/keycloak:latest
    command: start-dev
    environment:
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin123
      KC_HOSTNAME_STRICT: false
      KC_HOSTNAME_STRICT_HTTPS: false
    ports:
      - "8081:8080"
    volumes:
      - keycloak_data:/opt/keycloak/data

volumes:
  keycloak_data:
```

## レルムの作成

1. ブラウザで http://localhost:8081 にアクセス
2. 管理者アカウント（admin/admin123）でログイン
3. 左上の「Master」ドロップダウンから「Create Realm」を選択
4. 以下の設定でレルムを作成：

```json
{
  "realm": "rust-webapi",
  "displayName": "Rust WebAPI",
  "enabled": true,
  "registrationAllowed": true,
  "loginWithEmailAllowed": true,
  "duplicateEmailsAllowed": false
}
```

## クライアントの設定

### API クライアントの作成

1. 「Clients」セクションに移動
2. 「Create client」をクリック
3. 以下の設定を使用：

**Basic Settings:**
```
Client ID: api-client
Name: Rust WebAPI Client
Description: API access client for Rust WebAPI
```

**Capability config:**
```
Client authentication: ON
Authorization: OFF
Authentication flow:
  ☑ Standard flow
  ☑ Direct access grants
  ☐ Implicit flow
  ☐ Service accounts roles
```

**Login settings:**
```
Root URL: http://localhost:8080
Home URL: http://localhost:8080
Valid redirect URIs: 
  - http://localhost:8080/*
  - http://127.0.0.1:8080/*
Valid post logout redirect URIs:
  - http://localhost:8080/*
  - http://127.0.0.1:8080/*
Web origins:
  - http://localhost:8080
  - http://127.0.0.1:8080
```

### クライアントシークレットの取得

1. 作成したクライアントの「Credentials」タブに移動
2. 「Client secret」をコピーして保存（.env ファイルで使用）

## ユーザーとロールの設定

### ロールの作成

1. 「Realm roles」セクションに移動
2. 以下のロールを作成：

```
- api-user: 基本的なAPI アクセス権限
- api-admin: 管理者権限
```

### テストユーザーの作成

1. 「Users」セクションに移動
2. 「Add user」をクリック
3. 以下の設定でユーザーを作成：

**User info:**
```
Username: testuser
Email: testuser@example.com
First name: Test
Last name: User
Email verified: ON
```

**Credentials:**
```
Password: testpass123
Temporary: OFF
```

**Role mapping:**
1. 「Role mapping」タブに移動
2. 「Assign role」をクリック
3. 「api-user」ロールを割り当て

## API での認証テスト

### トークンの取得

```bash
# アクセストークンを取得
curl -X POST "http://localhost:8081/realms/rust-webapi/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=api-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "username=testuser" \
  -d "password=testpass123" \
  -d "grant_type=password"
```

レスポンス例：
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 300,
  "refresh_expires_in": 1800,
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer"
}
```

### API での認証

取得したトークンを使用してAPIにアクセス：

```bash
# トークンを環境変数に設定
export ACCESS_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."

# 認証が必要なエンドポイントにアクセス
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  http://localhost:8080/api/users
```

## 環境変数の設定

`.env` ファイルに以下を設定：

```bash
KEYCLOAK_AUTH_SERVER_URL=http://localhost:8081
KEYCLOAK_REALM=rust-webapi
KEYCLOAK_CLIENT_ID=api-client
```

## トラブルシューティング

### よくある問題と解決策

**問題: "Invalid token" エラー**
- トークンの有効期限が切れていないか確認
- クライアント設定が正しいか確認
- レルム名とクライアントIDが一致しているか確認

**問題: CORS エラー**
- Keycloak のクライアント設定で「Web origins」が正しく設定されているか確認
- `*` を設定して一時的にすべてのオリジンを許可してテスト

**問題: 接続エラー**
- Keycloak が起動しているか確認：`docker ps`
- ネットワーク設定が正しいか確認
- ファイアウォールの設定を確認

### デバッグ用のエンドポイント

Keycloak の設定を確認するためのエンドポイント：

```bash
# OpenID Connect 設定を確認
curl "http://localhost:8081/realms/rust-webapi/.well-known/openid_connect_configuration"

# 公開鍵を確認
curl "http://localhost:8081/realms/rust-webapi/protocol/openid-connect/certs"
```

### ログの確認

Keycloak のログを確認：

```bash
# Docker コンテナのログを確認
docker logs keycloak

# リアルタイムでログを監視
docker logs -f keycloak
```

## 本番環境での考慮事項

1. **HTTPS の使用**: 本番環境では必ずHTTPSを使用
2. **強力なパスワード**: 管理者パスワードを強力なものに変更
3. **データベースバックアップ**: Keycloak データの定期バックアップ
4. **監視とアラート**: Keycloak の稼働状況を監視
5. **セキュリティ設定**: 適切なセキュリティヘッダーの設定

詳細な本番環境設定については、[運用ガイド](operations-guide.md) を参照してください。