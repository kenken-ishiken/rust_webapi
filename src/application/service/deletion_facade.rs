use std::sync::Arc;

use crate::app_domain::service::deletion_service::{DeleteKind, DeletionError, DeletionStrategy, ItemDeletionStrategy};
use crate::infrastructure::error::{AppError, AppResult};
use crate::app_domain::repository::item_repository::ItemRepository;

/// ドメイン層 DeletionStrategy をアプリケーション層に公開するファサード
///
/// 現時点では Item エンティティのみをラップしているが、今後 Category / Product などもここに追加していく。
/// HTTP / gRPC ハンドラはこの Facade を経由して削除操作を呼び出し、
/// AppError 型に変換された結果を取得することでプレゼンテーション層との依存を切り離す。
pub struct DeletionFacade {
    item_strategy: Arc<dyn DeletionStrategy<Id = u64> + Send + Sync>,
}

impl DeletionFacade {
    /// Factory メソッド。DI コンテナから呼び出されることを想定。
    pub fn new<R>(item_repository: Arc<R>) -> Self
    where
        R: ItemRepository + Send + Sync + 'static + ?Sized,
    {
        let strategy = ItemDeletionStrategy::new(item_repository);
        Self {
            item_strategy: Arc::new(strategy),
        }
    }

    /// Item を削除する共通メソッド
    pub async fn delete_item(&self, id: u64, kind: DeleteKind) -> AppResult<()> {
        self.map_error(self.item_strategy.delete(id, kind).await)
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