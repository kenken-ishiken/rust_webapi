# 開発ワークフロー＆テスティングガイド

## 目次

1. [開発環境セットアップ](#開発環境セットアップ)
2. [コーディング規約](#コーディング規約)
3. [テスト戦略](#テスト戦略)
4. [単体テスト](#単体テスト)
5. [統合テスト](#統合テスト)
6. [E2Eテスト](#e2eテスト)
7. [パフォーマンステスト](#パフォーマンステスト)
8. [デバッグ手法](#デバッグ手法)
9. [開発ツール](#開発ツール)

## 開発環境セットアップ

### 必要なツール

```bash
# Rust開発環境
rustup component add rustfmt clippy rust-analyzer

# 開発支援ツール
cargo install cargo-watch      # ファイル変更監視
cargo install cargo-edit       # 依存関係管理
cargo install cargo-outdated   # 依存関係更新チェック
cargo install cargo-audit      # セキュリティ監査
cargo install cargo-expand     # マクロ展開
cargo install cargo-flamegraph # プロファイリング

# テストツール
cargo install cargo-nextest    # 高速テストランナー
cargo install cargo-tarpaulin  # カバレッジ計測
```

### VS Code設定

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.procMacro.enable": true,
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### 推奨拡張機能

- rust-analyzer
- crates
- Even Better TOML
- Error Lens
- GitLens

## コーディング規約

### 命名規則

```rust
// モジュール名: snake_case
mod product_service;

// 構造体・列挙型: PascalCase
struct ProductService;
enum ProductStatus;

// 関数・メソッド: snake_case
fn find_product_by_id() {}

// 定数: SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;

// 型パラメータ: 単一大文字または短い PascalCase
fn process<T, Err>(data: T) -> Result<T, Err> {}
```

### コード構造

```rust
// ファイル構造の推奨順序
use std::collections::HashMap;  // 1. 標準ライブラリ
use actix_web::{web, HttpResponse};  // 2. 外部クレート
use crate::domain::Product;  // 3. 内部モジュール

// 4. 型定義
type ProductMap = HashMap<String, Product>;

// 5. 定数
const DEFAULT_PAGE_SIZE: usize = 20;

// 6. 構造体/列挙型
#[derive(Debug, Clone)]
pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
}

// 7. 実装
impl ProductService {
    // コンストラクタを最初に
    pub fn new(repository: Arc<dyn ProductRepository>) -> Self {
        Self { repository }
    }
    
    // パブリックメソッド
    pub async fn find_all(&self) -> Result<Vec<Product>> {
        // 実装
    }
    
    // プライベートメソッド
    fn validate_product(&self, product: &Product) -> Result<()> {
        // 実装
    }
}

// 8. テスト
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_product_validation() {
        // テストコード
    }
}
```

### エラーハンドリング

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Product not found: {id}")]
    ProductNotFound { id: String },
    
    #[error("Validation failed: {message}")]
    ValidationError { message: String },
    
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}

// Result型のエイリアス
pub type ServiceResult<T> = Result<T, ServiceError>;

// エラー処理の例
pub async fn update_product(
    &self,
    id: &str,
    update: UpdateProductDto,
) -> ServiceResult<Product> {
    let product = self.repository
        .find_by_id(id)
        .await?
        .ok_or_else(|| ServiceError::ProductNotFound { 
            id: id.to_string() 
        })?;
    
    self.validate_update(&product, &update)?;
    
    Ok(self.repository.update(id, update).await?)
}
```

## テスト戦略

### テストピラミッド

```
         /\
        /E2E\      (5%)  - クリティカルなユーザーフロー
       /______\
      /統合テスト\   (25%) - APIエンドポイント、DB操作
     /___________\
    /  単体テスト  \  (70%) - ビジネスロジック、ユーティリティ
   /_____________\
```

### テストカバレッジ目標

| レイヤー | カバレッジ目標 | 重点項目 |
|---------|--------------|----------|
| ドメイン層 | 90%以上 | ビジネスロジック、バリデーション |
| アプリケーション層 | 80%以上 | サービスメソッド、エラーケース |
| インフラ層 | 70%以上 | リポジトリ実装、外部連携 |
| プレゼンテーション層 | 60%以上 | ハンドラー、ミドルウェア |

## 単体テスト

### 基本的な単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[test]
    fn test_product_creation() {
        // Arrange
        let sku = Sku::new("PROD-001".to_string()).unwrap();
        let name = ProductName::new("テスト商品".to_string()).unwrap();
        
        // Act
        let product = Product::new(sku, name);
        
        // Assert
        assert_eq!(product.sku().value(), "PROD-001");
        assert_eq!(product.name().value(), "テスト商品");
        assert_eq!(product.status(), &ProductStatus::Draft);
    }
    
    #[test]
    fn test_invalid_sku() {
        // 短すぎるSKU
        let result = Sku::new("AB".to_string());
        assert!(result.is_err());
        
        // 空のSKU
        let result = Sku::new("".to_string());
        assert!(result.is_err());
    }
}
```

### モックを使用したテスト

```rust
use mockall::*;

#[automock]
#[async_trait]
pub trait ProductRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Product>>;
    async fn save(&self, product: &Product) -> Result<()>;
}

#[tokio::test]
async fn test_product_service_create() {
    // モックの設定
    let mut mock_repo = MockProductRepository::new();
    
    mock_repo
        .expect_save()
        .with(predicate::function(|p: &Product| {
            p.sku().value() == "PROD-001"
        }))
        .times(1)
        .returning(|_| Ok(()));
    
    // サービスのテスト
    let service = ProductService::new(Arc::new(mock_repo));
    let dto = CreateProductDto {
        sku: "PROD-001".to_string(),
        name: "テスト商品".to_string(),
    };
    
    let result = service.create_product(dto).await;
    assert!(result.is_ok());
}
```

### パラメータ化テスト

```rust
use rstest::rstest;

#[rstest]
#[case("PROD-001", true)]
#[case("PR", false)]
#[case("PROD-12345678901234567890123456789012345678901234567890", false)]
#[case("PROD_001", true)]
#[case("prod-001", true)]
fn test_sku_validation(#[case] input: &str, #[case] expected: bool) {
    let result = Sku::new(input.to_string());
    assert_eq!(result.is_ok(), expected);
}
```

## 統合テスト

### データベース統合テスト

```rust
// tests/integration/product_repository_test.rs
use sqlx::PgPool;
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_product_repository_crud() {
    // Testcontainersでデータベースを起動
    let docker = clients::Cli::default();
    let postgres = docker.run(Postgres::default());
    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );
    
    // データベースプールの作成
    let pool = PgPool::connect(&connection_string).await.unwrap();
    
    // マイグレーション実行
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // リポジトリのテスト
    let repo = PostgresProductRepository::new(pool);
    
    // Create
    let product = Product::new(
        Sku::new("TEST-001".to_string()).unwrap(),
        ProductName::new("テスト商品".to_string()).unwrap(),
    );
    repo.save(&product).await.unwrap();
    
    // Read
    let found = repo.find_by_id(product.id()).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().sku().value(), "TEST-001");
    
    // Update
    let mut product = found.unwrap();
    product.update_name(ProductName::new("更新済み商品".to_string()).unwrap());
    repo.save(&product).await.unwrap();
    
    // Delete
    repo.delete(product.id()).await.unwrap();
    let deleted = repo.find_by_id(product.id()).await.unwrap();
    assert!(deleted.is_none());
}
```

### API統合テスト

```rust
// tests/integration/api_test.rs
use actix_web::{test, web, App};

#[actix_web::test]
async fn test_create_product_endpoint() {
    // テスト用のアプリケーション構築
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(create_test_app_state()))
            .configure(configure_routes)
    ).await;
    
    // リクエストの作成
    let req = test::TestRequest::post()
        .uri("/api/products")
        .set_json(&json!({
            "sku": "API-TEST-001",
            "name": "API テスト商品",
            "initial_stock": 100,
            "price": 1980
        }))
        .to_request();
    
    // レスポンスの検証
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    
    let body: ProductDto = test::read_body_json(resp).await;
    assert_eq!(body.sku, "API-TEST-001");
}
```

## E2Eテスト

### Cucumber を使用したE2Eテスト

```gherkin
# features/product_management.feature
Feature: 商品管理
  商品の作成、更新、削除ができること

  Background:
    Given 認証済みのユーザーとしてログインしている
    And テスト用のカテゴリ "電化製品" が存在する

  Scenario: 新商品の作成
    When 以下の商品を作成する:
      | sku      | FEAT-001     |
      | name     | 機能テスト商品 |
      | category | 電化製品      |
      | price    | 2980         |
      | stock    | 50           |
    Then レスポンスステータスは 201 である
    And 商品 "FEAT-001" が作成されている

  Scenario: 在庫切れ商品の注文
    Given 商品 "OUT-001" の在庫が 0 である
    When 商品 "OUT-001" を 1 個注文する
    Then レスポンスステータスは 400 である
    And エラーメッセージに "在庫不足" が含まれる
```

```rust
// tests/e2e/steps/product_steps.rs
use cucumber::{given, when, then, World};

#[derive(Debug, Default, World)]
pub struct ProductWorld {
    response: Option<Response>,
    auth_token: Option<String>,
}

#[given("認証済みのユーザーとしてログインしている")]
async fn given_authenticated(world: &mut ProductWorld) {
    let token = authenticate_test_user().await;
    world.auth_token = Some(token);
}

#[when(regex = r"^以下の商品を作成する:$")]
async fn when_create_product(
    world: &mut ProductWorld,
    step: &Step,
) {
    let table = step.table().unwrap();
    let product_data = parse_product_from_table(table);
    
    let response = create_product(
        &product_data,
        world.auth_token.as_ref().unwrap(),
    ).await;
    
    world.response = Some(response);
}

#[then(expr = "レスポンスステータスは {int} である")]
async fn then_status_code(
    world: &mut ProductWorld,
    expected: u16,
) {
    let response = world.response.as_ref().unwrap();
    assert_eq!(response.status().as_u16(), expected);
}
```

## パフォーマンステスト

### 負荷テスト

```rust
// benches/api_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_product_api(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let app_state = create_test_app_state();
    
    let mut group = c.benchmark_group("product_api");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("list_products", size),
            size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    list_products_with_limit(size).await
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_product_api);
criterion_main!(benches);
```

### プロファイリング

```bash
# フレームグラフの生成
cargo flamegraph --bin rust_webapi

# 特定のテストのプロファイリング
cargo flamegraph --test integration_test -- --test test_heavy_operation

# perf を使用した詳細プロファイリング
perf record -g target/release/rust_webapi
perf report
```

## デバッグ手法

### ログ駆動デバッグ

```rust
use tracing::{debug, error, info, warn, instrument};

#[instrument(skip(repository))]
pub async fn process_order(
    order_id: &str,
    repository: &dyn OrderRepository,
) -> Result<Order> {
    info!("Processing order: {}", order_id);
    
    let order = repository
        .find_by_id(order_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch order {}: {:?}", order_id, e);
            e
        })?;
    
    debug!("Order details: {:?}", order);
    
    if order.items.is_empty() {
        warn!("Order {} has no items", order_id);
        return Err(OrderError::NoItems);
    }
    
    // 処理続行...
}
```

### インタラクティブデバッグ

```rust
// VS Code launch.json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug rust_webapi",
            "cargo": {
                "args": [
                    "build",
                    "--bin=rust_webapi",
                    "--package=rust_webapi"
                ],
                "filter": {
                    "name": "rust_webapi",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug",
                "DATABASE_URL": "postgres://localhost/test"
            }
        }
    ]
}
```

### デバッグマクロ

```rust
// カスタムデバッグマクロ
macro_rules! debug_var {
    ($val:expr) => {
        #[cfg(debug_assertions)]
        {
            eprintln!("[{}:{}] {} = {:?}", 
                file!(), line!(), stringify!($val), $val);
        }
    };
}

// 使用例
let product = fetch_product(id).await?;
debug_var!(product);
debug_var!(product.inventory_status());
```

## 開発ツール

### Make タスク

```makefile
# Makefile
.PHONY: dev test lint format clean

dev:
	cargo watch -x run

test:
	cargo nextest run

test-coverage:
	cargo tarpaulin --out Html

lint:
	cargo clippy --all-targets -- -D warnings

format:
	cargo fmt

clean:
	cargo clean
	rm -rf target/

check-all: format lint test
	@echo "All checks passed!"

docker-dev:
	docker-compose up -d
	cargo watch -x run

benchmark:
	cargo bench

profile:
	cargo flamegraph --bin rust_webapi
```

### Git フック

```bash
#!/bin/sh
# .git/hooks/pre-commit

echo "Running pre-commit checks..."

# フォーマットチェック
cargo fmt -- --check
if [ $? -ne 0 ]; then
    echo "❌ Format check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy チェック
cargo clippy --all-targets -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy check failed."
    exit 1
fi

# テスト実行
cargo test --quiet
if [ $? -ne 0 ]; then
    echo "❌ Tests failed."
    exit 1
fi

echo "✅ All checks passed!"
```

### CI 最適化

```yaml
# .github/workflows/ci.yml の最適化
name: Optimized CI

on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always
  CARGO_INCREMENTAL: 0
  RUSTFLAGS: "-D warnings"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: Swatinem/rust-cache@v2  # キャッシュ
    
    - name: Run tests in parallel
      run: cargo nextest run --all-features
    
    - name: Generate coverage
      run: cargo tarpaulin --out Xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

## ベストプラクティス

### テスト作成のガイドライン

1. **AAA パターン**: Arrange, Act, Assert
2. **1テスト1アサーション**: 各テストは1つの振る舞いのみ検証
3. **独立性**: テスト間の依存関係を排除
4. **再現性**: 同じ条件で常に同じ結果
5. **高速性**: 単体テストは数ミリ秒で完了

### コードレビューチェックリスト

- [ ] 適切なエラーハンドリング
- [ ] 十分なテストカバレッジ
- [ ] ドキュメントコメント
- [ ] パフォーマンスへの配慮
- [ ] セキュリティの考慮
- [ ] 依存関係の最小化
- [ ] 非同期処理の適切な使用