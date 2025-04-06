# Rust WebAPI 開発ガイドライン

このドキュメントでは、クリーンアーキテクチャを採用したRust WebAPIプロジェクトの開発方法について説明します。

## 目次

1. [開発環境のセットアップ](#開発環境のセットアップ)
2. [アーキテクチャ概要](#アーキテクチャ概要)
3. [新機能の追加手順](#新機能の追加手順)
4. [テスト方法](#テスト方法)
5. [デプロイ方法](#デプロイ方法)
6. [コーディング規約](#コーディング規約)

## 開発環境のセットアップ

### 必要な環境

- Rust (Edition 2024)
- Docker と Docker Compose
- PostgreSQL 15

### 環境構築手順

1. **Rustのインストール**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **プロジェクトのクローン**

```bash
git clone <リポジトリURL>
cd rust_webapi
```

3. **データベースの起動**

```bash
docker-compose up -d
```

4. **環境変数の設定**

`.env`ファイルを作成し、必要な環境変数を設定します：

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_webapi
SERVER_ADDR=127.0.0.1:8080
RUST_LOG=debug
```

5. **アプリケーションの起動**

```bash
cargo run
```

## アーキテクチャ概要

このプロジェクトはクリーンアーキテクチャに基づいて設計されており、4つの層で構成されています：

### 1. プレゼンテーション層 (`src/presentation`)

- API エンドポイントの定義
- リクエスト/レスポンスの処理
- バリデーション
- エラーハンドリング

#### ファイル構造

- `api/`: APIハンドラー
  - `item_handler.rs`: アイテム関連のエンドポイント
  - `user_handler.rs`: ユーザー関連のエンドポイント

### 2. アプリケーション層 (`src/application`)

- ビジネスロジックの実装
- ユースケースの定義
- DTOの変換

#### ファイル構造

- `dto/`: データ転送オブジェクト
  - `item_dto.rs`: アイテム関連のDTO
  - `user_dto.rs`: ユーザー関連のDTO
- `service/`: サービス実装
  - `item_service.rs`: アイテム関連のビジネスロジック
  - `user_service.rs`: ユーザー関連のビジネスロジック

### 3. ドメイン層 (`src/domain`)

- ビジネスエンティティの定義
- リポジトリインターフェースの定義
- ドメインロジック

#### ファイル構造

- `model/`: ドメインモデル
  - `item.rs`: アイテムエンティティ
  - `user.rs`: ユーザーエンティティ
- `repository/`: リポジトリインターフェース
  - `item_repository.rs`: アイテムリポジトリのトレイト定義
  - `user_repository.rs`: ユーザーリポジトリのトレイト定義

### 4. インフラストラクチャ層 (`src/infrastructure`)

- リポジトリの実装
- データベース接続
- 外部APIとの連携

#### ファイル構造

- `repository/`: リポジトリ実装
  - `item_repository.rs`: アイテムリポジトリの実装
  - `user_repository.rs`: ユーザーリポジトリの実装

## 新機能の追加手順

新しい機能（例：新しいエンティティやAPIエンドポイント）を追加する場合の手順は以下の通りです。

### 1. ドメインモデルの定義

`src/domain/model/`に新しいエンティティを定義します。

```rust
// src/domain/model/product.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: Option<i32>,
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
}
```

モジュール定義も忘れずに更新します：

```rust
// src/domain/model/mod.rs
pub mod item;
pub mod user;
pub mod product; // 追加
```

### 2. リポジトリインターフェースの定義

`src/domain/repository/`に新しいリポジトリトレイトを定義します。

```rust
// src/domain/repository/product_repository.rs
use async_trait::async_trait;
use crate::domain::model::product::Product;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create(&self, product: &Product) -> Result<Product, anyhow::Error>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Product>, anyhow::Error>;
    async fn find_all(&self) -> Result<Vec<Product>, anyhow::Error>;
    async fn delete(&self, id: i32) -> Result<bool, anyhow::Error>;
}
```

モジュール定義も更新します：

```rust
// src/domain/repository/mod.rs
pub mod item_repository;
pub mod user_repository;
pub mod product_repository; // 追加
```

### 3. DTOの定義

`src/application/dto/`にDTOを定義します。

```rust
// src/application/dto/product_dto.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductDto {
    pub id: Option<i32>,
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductDto {
    pub name: String,
    pub price: f64,
    pub description: Option<String>,
}
```

モジュール定義も更新します：

```rust
// src/application/dto/mod.rs
pub mod item_dto;
pub mod user_dto;
pub mod product_dto; // 追加
```

### 4. サービスの実装

`src/application/service/`にサービスを実装します。

```rust
// src/application/service/product_service.rs
use crate::application::dto::product_dto::{ProductDto, CreateProductDto};
use crate::domain::model::product::Product;
use crate::domain::repository::product_repository::ProductRepository;

pub struct ProductService<R: ProductRepository> {
    repository: R,
}

impl<R: ProductRepository> ProductService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_product(&self, dto: CreateProductDto) -> Result<ProductDto, anyhow::Error> {
        let product = Product {
            id: None,
            name: dto.name,
            price: dto.price,
            description: dto.description,
        };

        let created = self.repository.create(&product).await?;
        
        Ok(ProductDto {
            id: created.id,
            name: created.name,
            price: created.price,
            description: created.description,
        })
    }

    // 他のメソッドも同様に実装
}
```

モジュール定義も更新します：

```rust
// src/application/service/mod.rs
pub mod item_service;
pub mod user_service;
pub mod product_service; // 追加
```

### 5. リポジトリの実装

`src/infrastructure/repository/`にリポジトリを実装します。

```rust
// src/infrastructure/repository/product_repository.rs
use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::model::product::Product;
use crate::domain::repository::product_repository::ProductRepository;

pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn create(&self, product: &Product) -> Result<Product, anyhow::Error> {
        let result = sqlx::query_as!(
            Product,
            r#"
            INSERT INTO products (name, price, description)
            VALUES ($1, $2, $3)
            RETURNING id, name, price, description
            "#,
            product.name,
            product.price,
            product.description
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    // 他のメソッドも同様に実装
}
```

モジュール定義も更新します：

```rust
// src/infrastructure/repository/mod.rs
pub mod item_repository;
pub mod user_repository;
pub mod product_repository; // 追加
```

### 6. APIハンドラーの実装

`src/presentation/api/`にAPIハンドラーを実装します。

```rust
// src/presentation/api/product_handler.rs
use actix_web::{web, HttpResponse, Responder, get, post, delete};
use crate::application::dto::product_dto::CreateProductDto;
use crate::application::service::product_service::ProductService;
use crate::domain::repository::product_repository::ProductRepository;

pub fn configure<R: ProductRepository + 'static>(
    cfg: &mut web::ServiceConfig,
    service: ProductService<R>,
) {
    cfg.app_data(web::Data::new(service))
        .service(
            web::scope("/api/products")
                .service(create_product)
                .service(get_product)
                .service(get_all_products)
                .service(delete_product)
        );
}

#[post("")]
async fn create_product<R: ProductRepository>(
    service: web::Data<ProductService<R>>,
    dto: web::Json<CreateProductDto>,
) -> impl Responder {
    match service.create_product(dto.into_inner()).await {
        Ok(product) => HttpResponse::Created().json(product),
        Err(e) => {
            log::error!("Failed to create product: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// 他のエンドポイントも同様に実装
```

モジュール定義も更新します：

```rust
// src/presentation/api/mod.rs
pub mod item_handler;
pub mod user_handler;
pub mod product_handler; // 追加
```

### 7. メインアプリケーションの更新

`src/main.rs`を更新して、新しいAPIエンドポイントを登録します。

```rust
// src/main.rs の関連部分を更新
use crate::application::service::product_service::ProductService;
use crate::infrastructure::repository::product_repository::PgProductRepository;
use crate::presentation::api::product_handler;

// 他のインポートと設定...

async fn main() -> std::io::Result<()> {
    // 既存の設定...

    // Productsリポジトリとサービス
    let product_repository = PgProductRepository::new(pool.clone());
    let product_service = ProductService::new(product_repository);

    HttpServer::new(move || {
        App::new()
            // 既存の設定...
            .configure(|cfg| item_handler::configure(cfg, item_service.clone()))
            .configure(|cfg| user_handler::configure(cfg, user_service.clone()))
            .configure(|cfg| product_handler::configure(cfg, product_service.clone())) // 追加
            // その他の設定...
    })
    .bind(server_addr)?
    .run()
    .await
}
```

### 8. データベースマイグレーションの作成

新しいテーブルを作成するためのSQLを追加します：

```sql
-- initdb/02_create_products_table.sql
CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    description TEXT
);
```

## テスト方法

### 単体テスト

```bash
cargo test
```

### 統合テスト

統合テストを実行するには、テストデータベースを用意する必要があります：

```bash
# テスト用のデータベースを作成
docker exec -it postgres psql -U postgres -c "CREATE DATABASE rust_webapi_test;"

# テスト実行
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rust_webapi_test cargo test -- --ignored
```

### リポジトリテストの例

```rust
// tests/item_repository_test.rs
use rust_webapi::domain::model::item::Item;
use rust_webapi::domain::repository::item_repository::ItemRepository;
use rust_webapi::infrastructure::repository::item_repository::PgItemRepository;
use sqlx::PgPool;

#[tokio::test]
async fn test_create_item() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/rust_webapi_test")
        .await
        .unwrap();
    
    let repository = PgItemRepository::new(pool);
    
    let item = Item {
        id: None,
        name: "Test Item".to_string(),
        description: Some("This is a test item".to_string()),
    };
    
    let result = repository.create(&item).await.unwrap();
    
    assert!(result.id.is_some());
    assert_eq!(result.name, "Test Item");
    assert_eq!(result.description, Some("This is a test item".to_string()));
}
```

## デプロイ方法

### Docker を使用したデプロイ

1. イメージのビルド

```bash
docker build -t rust-webapi:latest .
```

2. イメージの実行

```bash
docker run -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@db:5432/rust_webapi \
  -e SERVER_ADDR=0.0.0.0:8080 \
  --network your-network \
  rust-webapi:latest
```

### Docker Compose を使用したデプロイ

```bash
docker-compose up -d
```

## コーディング規約

1. **命名規則**
   - 変数・関数: `snake_case`
   - 型・トレイト: `PascalCase`
   - 定数: `SCREAMING_SNAKE_CASE`

2. **コメント**
   - 公開API・関数には必ずドキュメンテーションコメント (`///`) を付ける
   - 複雑なロジックには説明コメントを付ける

3. **エラーハンドリング**
   - `anyhow::Error` を使用してエラーをラップする
   - ユーザーに公開されるエラーは適切にマッピングする

4. **テスト**
   - 新機能には必ずテストを書く
   - テストカバレッジは80%以上を目指す

5. **コードフォーマット**
   - `rustfmt` を使用してフォーマットする
   - `cargo fmt` コマンドでフォーマットを適用

6. **リンター**
   - `clippy` を使用してコードの品質をチェックする
   - `cargo clippy` コマンドで実行

## 関連リソース

- [Rust 公式ドキュメント](https://www.rust-lang.org/ja/learn)
- [Actix Web ドキュメント](https://actix.rs/docs)
- [SQLx ドキュメント](https://github.com/launchbadge/sqlx)
- [Tokio ドキュメント](https://tokio.rs/docs/overview)