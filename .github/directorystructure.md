# ディレクトリ構造

このドキュメントは、「rust_webapi」プロジェクトのディレクトリ構造を説明しています。

## プロジェクト概要

このプロジェクトはRustで実装されたWebAPIで、クリーンアーキテクチャのパターンに従って構築されています。

## ルートディレクトリ

```
rust_webapi/
├── Cargo.lock             # 依存関係のロックファイル
├── Cargo.toml             # Rustプロジェクト設定と依存関係
├── docker-compose.yml     # Dockerコンテナ設定
├── Dockerfile             # Dockerイメージ構築設定
├── README.md              # プロジェクト説明
├── initdb/                # データベース初期化スクリプト
│   └── 01_create_tables.sql   # テーブル作成SQL
├── src/                   # ソースコード
└── tests/                 # テストコード
```

## ソースコード構造 (src/)

ソースコードは、クリーンアーキテクチャに基づき、4つの主要な層に分けられています：

```
src/
├── main.rs               # アプリケーションのエントリーポイント
├── application/          # アプリケーション層
│   ├── mod.rs
│   ├── dto/              # データ転送オブジェクト
│   │   ├── item_dto.rs   # アイテムのDTO定義
│   │   └── mod.rs
│   └── service/          # ビジネスロジックサービス
│       ├── item_service.rs   # アイテム関連のサービス実装
│       └── mod.rs
├── app_domain/           # ドメイン層（ビジネスルール）
│   ├── mod.rs
│   ├── model/            # ドメインモデル
│   │   ├── item.rs       # アイテムエンティティ
│   │   └── mod.rs
│   └── repository/       # リポジトリインターフェース
│       ├── item_repository.rs   # アイテムリポジトリのトレイト定義
│       └── mod.rs
├── infrastructure/       # インフラストラクチャ層
│   ├── mod.rs
│   └── repository/       # リポジトリの実装
│       ├── item_repository.rs   # アイテムリポジトリの実装
│       └── mod.rs
└── presentation/         # プレゼンテーション層
    ├── mod.rs
    └── api/              # API実装
        ├── item_handler.rs   # アイテム関連のエンドポイントハンドラー
        └── mod.rs
```

## テスト構造 (tests/)

```
tests/
├── item_repository_test.rs   # アイテムリポジトリのテスト
└── infrastructure/           # インフラストラクチャテスト
    └── repository/           # リポジトリ実装のテスト
```

## 各層の責任

### プレゼンテーション層 (`presentation/`)
- HTTPリクエスト/レスポンスの処理
- APIエンドポイントの定義と実装
- リクエストのバリデーション
- 認証・認可の処理

### アプリケーション層 (`application/`)
- ユースケースの実装（サービス）
- データ転送オブジェクト（DTO）の定義
- ドメインオブジェクトとDTOの変換
- トランザクション管理

### ドメイン層 (`app_domain/`)
- ビジネスエンティティの定義
- ビジネスルールの実装
- リポジトリインターフェースの定義
- ドメインサービスの定義

### インフラストラクチャ層 (`infrastructure/`)
- リポジトリインターフェースの実装
- データベース接続の管理
- 外部APIとの連携
- 永続化ロジックの実装

## ビルドとデプロイ

- `target/` - ビルド成果物が格納されるディレクトリ（バージョン管理されない）
- `Dockerfile` - アプリケーションのコンテナ化
- `docker-compose.yml` - 開発環境のセットアップ

---

最終更新日: 2025年4月6日