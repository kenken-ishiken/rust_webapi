use std::sync::Arc;

use crate::app_domain::service::deletion_service::{DeleteKind, DeletionError, DeletionStrategy, ItemDeletionStrategy, CategoryDeletionStrategy, ProductDeletionStrategy};
use crate::infrastructure::error::{AppError, AppResult};
use crate::app_domain::repository::item_repository::ItemRepository;
use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::app_domain::repository::product_repository::ProductRepository;

/// ドメイン層 DeletionStrategy をアプリケーション層に公開するファサード
///
/// Item / Category / Product エンティティの削除操作を統一インターフェースで提供。
/// HTTP / gRPC ハンドラはこの Facade を経由して削除操作を呼び出し、
/// AppError 型に変換された結果を取得することでプレゼンテーション層との依存を切り離す。
pub struct DeletionFacade {
    item_strategy: Arc<dyn DeletionStrategy<Id = u64> + Send + Sync>,
    category_strategy: Arc<dyn DeletionStrategy<Id = String> + Send + Sync>,
    product_strategy: Arc<dyn DeletionStrategy<Id = String> + Send + Sync>,
}

impl DeletionFacade {
    /// Factory メソッド。DI コンテナから呼び出されることを想定。
    pub fn new<IR, CR, PR>(
        item_repository: Arc<IR>,
        category_repository: Arc<CR>,
        product_repository: Arc<PR>,
    ) -> Self
    where
        IR: ItemRepository + Send + Sync + 'static + ?Sized,
        CR: CategoryRepository + Send + Sync + 'static + ?Sized,
        PR: ProductRepository + Send + Sync + 'static + ?Sized,
    {
        let item_strategy = ItemDeletionStrategy::new(item_repository);
        let category_strategy = CategoryDeletionStrategy::new(category_repository);
        let product_strategy = ProductDeletionStrategy::new(product_repository);
        
        Self {
            item_strategy: Arc::new(item_strategy),
            category_strategy: Arc::new(category_strategy),
            product_strategy: Arc::new(product_strategy),
        }
    }

    /// Item を削除する共通メソッド
    pub async fn delete_item(&self, id: u64, kind: DeleteKind) -> AppResult<()> {
        self.map_error(self.item_strategy.delete(id, kind).await)
    }

    /// Category を削除する共通メソッド
    pub async fn delete_category(&self, id: String, kind: DeleteKind) -> AppResult<()> {
        self.map_error(self.category_strategy.delete(id, kind).await)
    }

    /// Product を削除する共通メソッド
    pub async fn delete_product(&self, id: String, kind: DeleteKind) -> AppResult<()> {
        self.map_error(self.product_strategy.delete(id, kind).await)
    }

    /// Domain エラーをアプリケーション層の AppError にマッピング
    fn map_error<T>(&self, result: Result<T, DeletionError>) -> AppResult<T> {
        result.map_err(|e| match e {
            DeletionError::NotFound(msg) => AppError::NotFound(msg),
            DeletionError::Validation(msg) => AppError::InternalServerError(msg),
            DeletionError::Other(err) => {
                AppError::InternalServerError(format!("Deletion error: {}", err))
            }
        })
    }
} 