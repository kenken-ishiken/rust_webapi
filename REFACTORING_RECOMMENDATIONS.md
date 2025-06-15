# rust_webapi リファクタリング提案

## 概要
rust_webapiプロジェクトの詳細な調査を行い、コードの品質向上とメンテナンス性向上のためのリファクタリング提案をまとめました。現在のプロジェクトはClean Architectureを採用していますが、いくつかの改善点が見つかりました。

## 主要な問題点

### 1. 巨大なファイル・関数問題
- `main.rs`: 203行の巨大な main 関数
- `item_handler.rs`: 592行、多くの似た削除メソッド
- `item_service.rs`: 586行、重複するビジネスロジック
- `item_repository.rs`: 593行、InMemoryとPostgresの実装が混在

### 2. コードの重複
- 削除関連メソッドの重複（delete, logical_delete, physical_delete）
- モック設定の重複（テストコード）
- エラーハンドリングの散在
- メトリクス記録の重複

### 3. 責任の混在
- 設定管理が main 関数に集中
- DTOとドメインモデルの変換ロジックの散在
- 認証・認可の処理が散在

### 4. テストコードの問題
- 長いテスト関数（100行超）
- 同じパターンの重複
- モック設定の重複

## リファクタリング提案

### 1. 依存性注入とApplication Composition Root

#### 問題
現在の `main.rs` はすべての依存関係を手動で組み立てており、203行の巨大な関数になっている。

#### 解決策
```rust
// src/infrastructure/di/container.rs
use std::sync::Arc;
use sqlx::PgPool;
use crate::infrastructure::error::AppResult;

#[derive(Clone)]
pub struct AppContainer {
    pub item_service: Arc<ItemService>,
    pub user_service: Arc<UserService>,
    pub category_service: Arc<CategoryService>,
    pub product_service: Arc<ProductService>,
}

impl AppContainer {
    pub async fn new(pool: PgPool) -> AppResult<Self> {
        // リポジトリの作成
        let item_repository = Arc::new(PostgresItemRepository::new(pool.clone()));
        let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
        // ...

        // サービスの作成
        let item_service = Arc::new(ItemService::new(item_repository));
        let user_service = Arc::new(UserService::new(user_repository));
        // ...

        Ok(Self {
            item_service,
            user_service,
            // ...
        })
    }
}
```

### 2. 削除操作の統一化

#### 問題
削除関連のメソッドが複数あり、コードが重複している。

#### 解決策
```rust
// src/app_domain/model/deletion.rs
#[derive(Debug, Clone)]
pub enum DeletionStrategy {
    Logical,
    Physical,
    Restore,
}

// src/application/service/deletion_service.rs
pub struct DeletionService<T> {
    repository: Arc<dyn DeletionRepository<T>>,
}

impl<T> DeletionService<T> {
    pub async fn delete(&self, id: u64, strategy: DeletionStrategy) -> AppResult<()> {
        match strategy {
            DeletionStrategy::Logical => self.repository.logical_delete(id).await,
            DeletionStrategy::Physical => self.repository.physical_delete(id).await,
            DeletionStrategy::Restore => self.repository.restore(id).await,
        }
    }
}
```

### 3. 設定管理の改善

#### 問題
設定が環境変数から直接読み込まれ、main 関数に散在している。

#### 解決策
```rust
// src/infrastructure/config/mod.rs
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub keycloak: KeycloakConfig,
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            database: DatabaseConfig::from_env()?,
            keycloak: KeycloakConfig::from_env()?,
            server: ServerConfig::from_env()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub http_port: u16,
    pub grpc_port: u16,
    pub host: String,
}
```

### 4. ハンドラーの簡素化

#### 問題
ハンドラーが長すぎて、似たようなメソッドが多い。

#### 解決策
```rust
// src/presentation/api/handlers/base_handler.rs
pub trait CrudHandler<T, CreateReq, UpdateReq, Response> {
    async fn get_all(&self) -> ActixResult<Vec<Response>>;
    async fn get_by_id(&self, id: u64) -> ActixResult<Response>;
    async fn create(&self, req: CreateReq) -> ActixResult<Response>;
    async fn update(&self, id: u64, req: UpdateReq) -> ActixResult<Response>;
    async fn delete(&self, id: u64) -> ActixResult<()>;
}

// src/presentation/api/handlers/item_handler.rs
impl CrudHandler<Item, CreateItemRequest, UpdateItemRequest, ItemResponse> for ItemHandler {
    // 実装...
}
```

### 5. テストコードの改善

#### 問題
テストが長すぎて、モック設定が重複している。

#### 解決策
```rust
// tests/helpers/mock_builder.rs
pub struct MockItemRepositoryBuilder {
    mock: MockItemRepository,
}

impl MockItemRepositoryBuilder {
    pub fn new() -> Self {
        Self { mock: MockItemRepository::new() }
    }

    pub fn with_find_all(mut self, items: Vec<Item>) -> Self {
        self.mock.expect_find_all()
            .return_once(move || Ok(items));
        self
    }

    pub fn with_find_by_id(mut self, id: u64, item: Option<Item>) -> Self {
        self.mock.expect_find_by_id()
            .with(eq(id))
            .return_once(move |_| Ok(item));
        self
    }

    pub fn build(self) -> MockItemRepository {
        self.mock
    }
}

// テストでの使用
#[tokio::test]
async fn test_get_items() {
    let mock_repo = MockItemRepositoryBuilder::new()
        .with_find_all(vec![sample_item()])
        .build();
    
    // テストの実行...
}
```

### 6. エラーハンドリングの統一

#### 問題
エラーハンドリングが各レイヤーに散在し、コメントアウトされた部分が多い。

#### 解決策
```rust
// src/infrastructure/error/mod.rs
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRule(String),
}

impl AppError {
    pub fn not_found(entity: &str, id: impl std::fmt::Display) -> Self {
        Self::NotFound {
            entity: entity.to_string(),
            id: id.to_string(),
        }
    }
}
```

### 7. ドメインサービスの分離

#### 問題
ビジネスロジックがアプリケーションサービスに集中している。

#### 解決策
```rust
// src/app_domain/service/item_domain_service.rs
pub struct ItemDomainService {
    item_repository: Arc<dyn ItemRepository>,
}

impl ItemDomainService {
    pub async fn can_delete(&self, item_id: u64) -> AppResult<bool> {
        let validation = self.item_repository.validate_deletion(item_id).await?;
        Ok(validation.can_delete)
    }

    pub async fn apply_business_rules(&self, item: &mut Item) -> AppResult<()> {
        // ビジネスルールの適用
        if item.name.is_empty() {
            return Err(AppError::BusinessRule("Item name cannot be empty".to_string()));
        }
        Ok(())
    }
}
```

### 8. リポジトリの分離

#### 問題
InMemoryとPostgresの実装が同じファイルにあり、長すぎる。

#### 解決策
```bash
src/infrastructure/repository/
├── item/
│   ├── mod.rs
│   ├── postgres_repository.rs
│   └── in_memory_repository.rs
├── user/
│   ├── mod.rs
│   ├── postgres_repository.rs
│   └── in_memory_repository.rs
└── traits/
    └── deletion_repository.rs
```

### 9. 共通トレイトの抽出

#### 問題
削除操作が各エンティティで重複している。

#### 解決策
```rust
// src/app_domain/repository/traits/deletion_repository.rs
#[async_trait]
pub trait DeletionRepository<T> {
    async fn logical_delete(&self, id: u64) -> AppResult<()>;
    async fn physical_delete(&self, id: u64) -> AppResult<()>;
    async fn restore(&self, id: u64) -> AppResult<()>;
    async fn find_deleted(&self) -> AppResult<Vec<T>>;
    async fn validate_deletion(&self, id: u64) -> AppResult<DeletionValidation>;
}
```

### 10. マクロによるボイラープレートの削減

#### 問題
CRUDハンドラーのボイラープレートが多い。

#### 解決策
```rust
// src/presentation/macros/crud_handlers.rs
macro_rules! impl_crud_handlers {
    ($handler:ident, $service:ident, $entity:ident, $create_req:ident, $update_req:ident, $response:ident) => {
        impl $handler {
            pub async fn get_all(
                data: web::Data<$handler>,
            ) -> ActixResult<impl Responder> {
                let items = data.service.find_all().await?;
                Ok(HttpResponse::Ok().json(items))
            }
            // 他のCRUDメソッドも同様に生成
        }
    };
}
```

## 実装優先順位

### Phase 1: 基盤整備（1-2週間）
1. 設定管理の分離
2. 依存性注入コンテナの実装
3. エラーハンドリングの統一

### Phase 2: コード分離（2-3週間）
1. リポジトリの分離（ファイル分割）
2. ハンドラーの共通化
3. テストヘルパーの実装

### Phase 3: ドメイン改善（1-2週間）
1. ドメインサービスの分離
2. 削除操作の統一
3. ビジネスルールの明確化

### Phase 4: 最適化（1週間）
1. マクロによるボイラープレート削減
2. パフォーマンスの最適化
3. ドキュメント整備

## 期待される効果

### コード品質の向上
- 関数とファイルサイズの削減（平均50-70%削減）
- 重複コードの除去（30-40%削減）
- テスタビリティの向上

### メンテナンス性の向上
- 責任の明確化
- 変更時の影響範囲の局所化
- 新機能追加の容易さ

### 開発効率の向上
- テスト作成の高速化
- デバッグの容易さ
- コードレビューの効率化

## まとめ

このリファクタリング計画により、rust_webapiプロジェクトはより保守しやすく、拡張しやすい構造になります。特に、巨大な関数・ファイルの分割と責任の明確化により、開発チームの生産性が大幅に向上することが期待されます。

実装時は段階的に進めることで、既存機能への影響を最小限に抑えながら改善を進められます。