# Rust WebAPI サーバー

クラウドネイティブな実運用を想定した **Rust 製 REST API サーバー** です。  
学習用途から本番環境まで、Rust の高速な非同期処理と型安全性を活かした API 開発が可能です。

## 目次
- [特徴](#特徴)
- [ディレクトリ構成](#ディレクトリ構成)
- [クイックスタート](#クイックスタート)
- [API リファレンス](#api-リファレンス)
- [Observability（可観測性）](#observability可観測性)
- [開発](#開発)
- [Kubernetes デプロイ](#kubernetes-デプロイ)
- [コントリビューション](#コントリビューション)
- [ライセンス](#ライセンス)

## 特徴
- **RESTful API**：CRUD 操作を HTTP/JSON で提供  
- **ドメイン駆動設計**：多層アーキテクチャ（domain, application, infrastructure, presentation）
- **高速**：`tokio` と `actix-web` による非同期 I/O  
- **型安全**：Rust の型システムでリクエスト／レスポンスを保証  
- **永続化**：`sqlx` による PostgreSQL 連携
- **認証**：JWT / Keycloak 連携
- **可観測性**：Prometheus / OpenTelemetry / Tracing 対応
- **コンテナ化**：Docker / Kubernetes / Istio 対応

## ディレクトリ構成

```
.
├── src/                # メインアプリケーション
│   ├── main.rs         # エントリポイント
│   ├── application/    # アプリケーション層（DTO、サービス）
│   ├── domain/         # ドメイン層（モデル、リポジトリインターフェース）
│   ├── infrastructure/ # インフラ層（DB、認証、ロギング）
│   └── presentation/   # プレゼンテーション層（API ハンドラ）
├── crates/domain/      # ドメイン層サブクレート
├── k8s/                # Kubernetes マニフェスト
│   ├── base/           # 共通設定
│   └── overlays/       # 環境別設定（dev, staging, prod）
├── initdb/             # DB 初期化 SQL
├── scripts/            # 補助スクリプト
├── o11y.md             # 可観測性ガイド
├── Dockerfile          # コンテナイメージ定義
├── docker-compose.yml  # ローカル開発環境
└── README.md           # 本ドキュメント
```

## クイックスタート

### 前提
- **Rust** 1.77 以上（`rustup` 推奨）
- `cargo`（Rust 同梱）
- `docker` と `docker-compose`（コンテナ実行時）

### ローカル実行（Rust のみ）
```bash
# 依存関係を取得してサーバーを起動
cargo run

# 別ターミナルで動作確認
curl http://127.0.0.1:8080/
```

### Docker Compose 実行（PostgreSQL 含む）
```bash
# 環境変数ファイル作成
cat > .env << EOF
DATABASE_URL=postgres://postgres:password@postgres:5432/rustwebapi
KEYCLOAK_URL=http://localhost:8081
KEYCLOAK_REALM=rust-webapi
KEYCLOAK_CLIENT_ID=api-client
EOF

# コンテナ起動
docker-compose up -d

# 動作確認
curl http://localhost:8080/
```

デフォルトで **http://127.0.0.1:8080** で待ち受けます。  
ポートを変更したい場合は環境変数 `PORT` を設定してください。

## API リファレンス

| メソッド  | パス                        | 説明                  |
|-----------|-----------------------------|-----------------------|
| GET       | `/`                         | サーバー稼働確認      |
| GET       | `/api/health`               | ヘルスチェック        |
| GET       | `/api/metrics`              | メトリクス取得        |
| GET       | `/api/items`                | アイテム一覧取得      |
| GET       | `/api/items/{id}`           | 特定アイテム取得      |
| POST      | `/api/items`                | アイテム作成          |
| PUT       | `/api/items/{id}`           | 特定アイテム更新      |
| DELETE    | `/api/items/{id}`           | アイテム削除          |
| GET       | `/api/users`                | ユーザー一覧取得      |
| GET       | `/api/users/{id}`           | 特定ユーザー取得      |
| POST      | `/api/users`                | ユーザー作成          |
| PUT       | `/api/users/{id}`           | 特定ユーザー更新      |
| DELETE    | `/api/users/{id}`           | ユーザー削除          |

### 例：アイテム作成
```bash
curl -X POST http://127.0.0.1:8080/api/items \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {JWT_TOKEN}" \
  -d '{"name":"テスト","description":"説明"}'
```

### 例：レスポンス
```json
{
  "id": 0,
  "name": "テスト",
  "description": "説明"
}
```

## Observability（可観測性）

本プロジェクトは包括的な可観測性を実現するため、以下の機能を提供しています：

- **ログ**：`tracing` / `slog` による JSON 構造化ログ
- **メトリクス**：Prometheus エクスポート（`/api/metrics` エンドポイント）
- **トレーシング**：OpenTelemetry 対応（分散トレーシング）

詳細な可観測性の設計と実装ガイドは [o11y.md](o11y.md) を参照してください。

## 開発

### プロジェクト構成
プロジェクトは DDD（ドメイン駆動設計）の考え方に基づいた多層アーキテクチャを採用しています：

- **ドメイン層**：ビジネスロジックとエンティティ
- **アプリケーション層**：ユースケースとサービス
- **インフラストラクチャ層**：DB、認証、ロギングなどの外部連携
- **プレゼンテーション層**：API エンドポイントとリクエスト/レスポンス処理

### テスト
```bash
# 単体テスト・統合テスト実行
cargo test

# カバレッジ計測（要 llvm-tools-preview）
./scripts/coverage.sh
```

### フォーマット & Lint
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### 依存追加
`Cargo.toml` を編集後、`cargo build` を実行してください。

## Kubernetes デプロイ

本プロジェクトは Kubernetes / Istio 環境での本番運用を想定した設定を提供しています：

- **Namespace 分離**：API と Database の分離
- **Istio / Gateway API**：トラフィック制御
- **ConfigMap / Secret**：設定と機密情報の管理
- **HPA / PDB**：スケーリングと可用性確保
- **Kustomize**：環境別設定（dev, staging, prod）

詳細なデプロイ手順は [k8s/README.md](k8s/README.md) を参照してください。

## コントリビューション
Issue や Pull Request は歓迎です。詳細な手順は [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。以下の点にご協力ください：

- コードスタイルは `rustfmt` と `clippy` に準拠
- 新機能には単体テストを追加
- コミットメッセージは具体的かつ簡潔に
- 大きな変更は事前に Issue で相談

## ライセンス
MIT License © 2025 Your Name
