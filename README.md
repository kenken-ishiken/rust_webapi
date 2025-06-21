# Rust WebAPI サーバー

クラウドネイティブな実運用を想定した **Rust 製 REST API サーバー** です。  
学習用途から本番環境まで、Rust の高速な非同期処理と型安全性を活かした API 開発が可能です。

## 目次
- [特徴](#特徴)
- [クイックスタート](#クイックスタート)
- [ドキュメント](#ドキュメント)
- [プロジェクト構造](#プロジェクト構造)
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
# 環境変数ファイル作成（.env.example をコピーして編集）
cp .env.example .env
# .env ファイルを編集して実際の値を設定

# コンテナ起動
docker-compose up -d

# 動作確認
curl http://localhost:8080/
```

デフォルトで **http://127.0.0.1:8080** で待ち受けます。  
ポートを変更したい場合は環境変数 `PORT` を設定してください。

## ドキュメント

プロジェクトに関する詳細なドキュメントは以下を参照してください：

### 基本ドキュメント
- [API リファレンス](docs/api-reference.md) - エンドポイントの詳細仕様、リクエスト・レスポンス例、curl使用例
- [API 仕様書](docs/api-documentation.md) - REST API の包括的な仕様とエンドポイント一覧
- [gRPC API 仕様書](docs/grpc-api.md) - gRPC API の仕様とエンドポイント一覧
- [アーキテクチャガイド](docs/architecture-guide.md) - システム設計、データフロー、コンポーネント構成
- [設定ガイド](docs/configuration-guide.md) - 環境変数と設定管理の詳細
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
├── crates/domain/      # ドメイン層サブクレート
├── docs/               # 詳細ドキュメント
├── k8s/                # Kubernetes マニフェスト
├── initdb/             # DB 初期化 SQL
├── scripts/            # 補助スクリプト
├── tests/              # 統合テスト
└── ...                 # その他の設定ファイル
```

詳細な構造については [.github/directorystructure.md](.github/directorystructure.md) を参照してください。

## コントリビューション

Issue や Pull Request は歓迎です。詳細な手順は [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。以下の点にご協力ください：

- コードスタイルは `rustfmt` と `clippy` に準拠
- 新機能には単体テストを追加
- コミットメッセージは具体的かつ簡潔に
- 大きな変更は事前に Issue で相談

## ライセンス

MIT License © 2025 Your Name
