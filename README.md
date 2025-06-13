# Rust WebAPI サーバー

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.77+-blue.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-blue.svg)](https://kubernetes.io)

クラウドネイティブな実運用を想定した **Rust 製 REST API サーバー** です。  
学習用途から本番環境まで、Rust の高速な非同期処理と型安全性を活かした API 開発が可能です。

## 目次
- [特徴](#特徴)
- [技術スタック](#技術スタック)
- [パフォーマンス](#パフォーマンス)
- [クイックスタート](#クイックスタート)
- [環境設定](#環境設定)
- [ドキュメント](#ドキュメント)
- [プロジェクト構造](#プロジェクト構造)
- [可観測性](#可観測性)
- [コントリビューション](#コントリビューション)
- [ライセンス](#ライセンス)

## 特徴
- **RESTful API**：CRUD 操作を HTTP/JSON で提供  
- **gRPC API**：高性能なバイナリプロトコルでのAPI提供（ポート50051）
- **ドメイン駆動設計**：多層アーキテクチャ（domain, application, infrastructure, presentation）
- **高速**：`tokio` と `actix-web` による非同期 I/O  
- **型安全**：Rust の型システムでリクエスト／レスポンスを保証  
- **永続化**：`sqlx` による PostgreSQL 連携
- **認証**：JWT / Keycloak 連携
- **可観測性**：Prometheus / OpenTelemetry / Tracing 対応
- **コンテナ化**：Docker / Kubernetes / Istio 対応
- **テスト**：Testcontainers による統合テスト
- **CI/CD**：GitHub Actions 対応

## 技術スタック

### Core Framework
- **Rust 1.77+** - システムプログラミング言語
- **Actix Web 4.4** - 高速 Web フレームワーク
- **Tokio** - 非同期ランタイム
- **SQLx 0.8** - 型安全な SQL クエリビルダー

### データベース・認証
- **PostgreSQL** - メインデータベース
- **JWT** - トークンベース認証
- **Keycloak** - 認証・認可サーバー

### 可観測性
- **Tracing** - 分散トレーシング
- **OpenTelemetry** - 標準的な可観測性フレームワーク
- **Prometheus** - メトリクス収集
- **Structured Logging** - JSON形式のログ出力

### 開発・運用
- **Docker & Docker Compose** - コンテナ化
- **Kubernetes** - オーケストレーション
- **Testcontainers** - 統合テスト
- **GitHub Actions** - CI/CD パイプライン

## パフォーマンス

本プロジェクトは高性能を重視して設計されています：

- **非同期処理**: Tokio による効率的な I/O 処理
- **ゼロコピー**: 可能な限りメモリアロケーションを最小化
- **型安全**: コンパイル時の最適化によるランタイムオーバーヘッド削減
- **コネクションプーリング**: SQLx による効率的なDB接続管理
- **gRPC**: バイナリプロトコルによる高速通信

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
cp .env.example .env
# .env ファイルを編集して実際の値を設定

# コンテナ起動
docker-compose up -d

# 動作確認
curl http://localhost:8080/
```

デフォルトで **http://127.0.0.1:8080** で待ち受けます。  
ポートを変更したい場合は環境変数 `PORT` を設定してください。

## 環境設定

### 必要な環境変数

アプリケーションの実行には以下の環境変数が必要です：

```bash
# データベース設定
DATABASE_URL=postgresql://user:password@localhost:5432/dbname

# サーバー設定
PORT=8080
HOST=127.0.0.1

# 認証設定
JWT_SECRET=your-secret-key
KEYCLOAK_URL=http://localhost:8080/auth
KEYCLOAK_REALM=your-realm

# 可観測性設定
RUST_LOG=info
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
PROMETHEUS_PORT=9090
```

### 開発環境のセットアップ

```bash
# Rust ツールチェーンのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 必要なコンポーネントのインストール
rustup component add rustfmt clippy

# 開発用ツールのインストール
cargo install cargo-watch sqlx-cli

# データベースのセットアップ
sqlx database create
sqlx migrate run
```

## ドキュメント

プロジェクトに関する詳細なドキュメントは以下を参照してください：

### 基本ドキュメント
- [API リファレンス](docs/api-reference.md) - エンドポイントの詳細仕様、リクエスト・レスポンス例、curl使用例
- [API 仕様書](docs/api-documentation.md) - REST API の包括的な仕様とエンドポイント一覧
- [gRPC API 仕様書](docs/grpc-api.md) - gRPC API の仕様とエンドポイント一覧
- [アーキテクチャガイド](docs/architecture-guide.md) - システム設計、データフロー、コンポーネント構成
- [開発ガイド](docs/development-guide.md) - 開発環境のセットアップ、テスト、デバッグ
- [開発ワークフロー](docs/development-testing.md) - テスト戦略、コーディング規約、CI/CD

### 運用・デプロイ
- [運用ガイド](docs/operations-guide.md) - デプロイ、監視、バックアップ、スケーリング
- [デプロイ・運用ガイド](docs/deployment-operations.md) - 本番環境でのデプロイと運用
- [Keycloakセットアップガイド](docs/keycloak-setup.md) - Keycloak認証サーバーの設定と連携方法

### システム設計・データベース
- [データベーススキーマ](docs/database-schema.md) - データベース設計と関係性
- [詳細アーキテクチャ](docs/architecture-detailed.md) - 深掘りしたアーキテクチャ解説

### プロジェクト管理
- [要件管理ガイド](docs/requirement-management-guide.md) - 要件定義の管理方法
- [要件セットアップ例](docs/requirement-setup-examples.md) - 実践的な要件管理の例

その他の重要なドキュメント：
- [可観測性ガイド](o11y.md) - ログ、メトリクス、トレーシングの実装と運用
- [Kubernetesデプロイガイド](k8s/README.md) - Kubernetes環境へのデプロイ手順
- [統合テストガイド](tests/README.md) - Testcontainersを使用した統合テスト
- [スクリプトガイド](scripts/README.md) - 開発・運用で使用する補助スクリプト

## プロジェクト構造

```
.
├── src/                # メインアプリケーション
│   ├── main.rs         # エントリポイント
│   ├── application/    # アプリケーション層（DTO、サービス）
│   ├── app_domain/     # ドメイン層（モデル、リポジトリインターフェース）
│   ├── infrastructure/ # インフラ層（DB、認証、ロギング）
│   └── presentation/   # プレゼンテーション層（API ハンドラ）
├── crates/             # サブクレート
│   └── domain/         # ドメイン層サブクレート
├── docs/               # 詳細ドキュメント
├── k8s/                # Kubernetes マニフェスト
├── initdb/             # DB 初期化 SQL
├── scripts/            # 補助スクリプト
├── tests/              # 統合テスト
├── proto/              # gRPC プロトコル定義
├── datadog/            # Datadog 設定
└── .github/            # GitHub Actions ワークフロー
```

詳細な構造については [.github/directorystructure.md](.github/directorystructure.md) を参照してください。

## 可観測性

本プロジェクトは包括的な可観測性機能を提供します：

### ログ
- **構造化ログ**: JSON形式での出力
- **ログレベル**: TRACE, DEBUG, INFO, WARN, ERROR
- **コンテキスト**: リクエストID、ユーザーID等の自動付与

### メトリクス
- **Prometheus**: カスタムメトリクスの収集
- **システムメトリクス**: CPU、メモリ、ディスク使用量
- **アプリケーションメトリクス**: リクエスト数、レスポンス時間、エラー率

### トレーシング
- **OpenTelemetry**: 標準的な分散トレーシング
- **Jaeger**: トレースの可視化
- **自動計装**: HTTP リクエスト、データベースクエリの自動追跡

詳細は [可観測性ガイド](o11y.md) を参照してください。

## コントリビューション

Issue や Pull Request は歓迎です。詳細な手順は [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。以下の点にご協力ください：

- **コードスタイル**: `rustfmt` と `clippy` に準拠
- **テスト**: 新機能には単体テストと統合テストを追加
- **ドキュメント**: 公開APIには適切なドキュメントコメントを記述
- **コミットメッセージ**: 具体的かつ簡潔に
- **大きな変更**: 事前に Issue で相談

### 開発フロー
1. Fork してブランチを作成
2. 機能を実装・テストを追加
3. `cargo fmt` と `cargo clippy` でコードを整形
4. `cargo test` でテストを実行
5. Pull Request を作成

## ライセンス

MIT License © 2025 Your Name

---

**🚀 高性能で安全な Rust WebAPI サーバーをお楽しみください！**
