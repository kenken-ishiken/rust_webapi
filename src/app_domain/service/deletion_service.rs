use async_trait::async_trait;
use std::fmt::Debug;

use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::app_domain::repository::item_repository::ItemRepository;
use crate::app_domain::repository::product_repository::ProductRepository;

/// 削除方法を示す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteKind {
    /// 論理削除 (ソフトデリート)
    Logical,
    /// 物理削除 (ハードデリート)
    Physical,
    /// 削除状態からの復元
    Restore,
}

/// Domain 層で使用する汎用的な削除エラー
#[derive(thiserror::Error, Debug)]
pub enum DeletionError {
    /// 対象エンティティが見つからない場合
    #[error("Entity not found: {0}")]
    NotFound(String),
    /// 削除前バリデーションに失敗した場合
    #[error("Deletion validation failed: {0}")]
    Validation(String),
    /// 上記以外のその他のエラー
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// エンティティ別に論理/物理削除を実装するための戦略トレイト
///
/// DeleteKind に応じて適切な削除処理を行う共通インターフェースを提供する。
#[async_trait]
pub trait DeletionStrategy: Send + Sync {
    /// エンティティの識別子型
    type Id: Send + Sync + Debug;

    /// 指定された `id` に対して削除処理を実行する
    async fn delete(&self, id: Self::Id, kind: DeleteKind) -> Result<(), DeletionError>;
}

/// Item エンティティ用の DeletionStrategy 実装
pub struct ItemDeletionStrategy<R>
where
    R: ItemRepository + Send + Sync + ?Sized,
{
    repository: std::sync::Arc<R>,
}

impl<R> ItemDeletionStrategy<R>
where
    R: ItemRepository + Send + Sync + ?Sized,
{
    /// 新しい ItemDeletionStrategy を生成する
    pub fn new(repository: std::sync::Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> DeletionStrategy for ItemDeletionStrategy<R>
where
    R: ItemRepository + Send + Sync + ?Sized,
{
    type Id = u64;

    async fn delete(&self, id: Self::Id, kind: DeleteKind) -> Result<(), DeletionError> {
        use crate::infrastructure::error::AppError;

        let res = match kind {
            DeleteKind::Logical => self.repository.logical_delete(id).await,
            DeleteKind::Physical => self.repository.physical_delete(id).await,
            DeleteKind::Restore => self.repository.restore(id).await,
        };

        // AppError → DeletionError へ変換
        match res {
            Ok(_) => Ok(()),
            Err(err) => match err {
                AppError::NotFound(msg) => Err(DeletionError::NotFound(msg)),
                _ => Err(DeletionError::Other(anyhow::Error::new(err))),
            },
        }
    }
}

/// Category エンティティ用の DeletionStrategy 実装
pub struct CategoryDeletionStrategy<R>
where
    R: CategoryRepository + Send + Sync + ?Sized,
{
    repository: std::sync::Arc<R>,
}

impl<R> CategoryDeletionStrategy<R>
where
    R: CategoryRepository + Send + Sync + ?Sized,
{
    /// 新しい CategoryDeletionStrategy を生成する
    pub fn new(repository: std::sync::Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> DeletionStrategy for CategoryDeletionStrategy<R>
where
    R: CategoryRepository + Send + Sync + ?Sized,
{
    type Id = String;

    async fn delete(&self, id: Self::Id, kind: DeleteKind) -> Result<(), DeletionError> {
        use crate::app_domain::model::category::CategoryError;

        let res = match kind {
            DeleteKind::Logical => {
                // Categoryは論理削除として非アクティブ化
                match self.repository.find_by_id(&id).await {
                    Some(mut category) => {
                        category.deactivate();
                        self.repository
                            .update(category)
                            .await
                            .map(|_| true)
                            .map_err(|e| CategoryError::NotFound(format!("Update failed: {}", e)))
                    }
                    None => Err(CategoryError::NotFound(format!(
                        "Category {} not found",
                        id
                    ))),
                }
            }
            DeleteKind::Physical => self.repository.delete(&id).await,
            DeleteKind::Restore => {
                // Categoryの復元として再アクティブ化
                match self.repository.find_by_id(&id).await {
                    Some(mut category) => {
                        category.activate();
                        self.repository
                            .update(category)
                            .await
                            .map(|_| true)
                            .map_err(|e| CategoryError::NotFound(format!("Update failed: {}", e)))
                    }
                    None => Err(CategoryError::NotFound(format!(
                        "Category {} not found",
                        id
                    ))),
                }
            }
        };

        // CategoryError → DeletionError へ変換
        match res {
            Ok(_) => Ok(()),
            Err(err) => match err {
                CategoryError::NotFound(msg) => Err(DeletionError::NotFound(msg)),
                CategoryError::HasChildren(msg) => Err(DeletionError::Validation(msg)),
                _ => Err(DeletionError::Other(anyhow::Error::new(err))),
            },
        }
    }
}

/// Product エンティティ用の DeletionStrategy 実装
pub struct ProductDeletionStrategy<R>
where
    R: ProductRepository + Send + Sync + ?Sized,
{
    repository: std::sync::Arc<R>,
}

impl<R> ProductDeletionStrategy<R>
where
    R: ProductRepository + Send + Sync + ?Sized,
{
    /// 新しい ProductDeletionStrategy を生成する
    pub fn new(repository: std::sync::Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R> DeletionStrategy for ProductDeletionStrategy<R>
where
    R: ProductRepository + Send + Sync + ?Sized,
{
    type Id = String;

    async fn delete(&self, id: Self::Id, kind: DeleteKind) -> Result<(), DeletionError> {
        use crate::app_domain::model::product::{ProductError, ProductStatus};

        let res = match kind {
            DeleteKind::Logical => {
                // Productは論理削除としてステータスを非アクティブに変更
                match self.repository.find_by_id(&id).await {
                    Some(mut product) => {
                        product.update_status(ProductStatus::Discontinued);
                        self.repository
                            .update(product)
                            .await
                            .map(|_| ())
                            .map_err(|_e| ProductError::ProductNotFound)
                    }
                    None => Err(ProductError::ProductNotFound),
                }
            }
            DeleteKind::Physical => self.repository.delete(&id).await,
            DeleteKind::Restore => {
                // Productの復元としてステータスをアクティブに変更
                match self.repository.find_by_id(&id).await {
                    Some(mut product) => {
                        product.update_status(ProductStatus::Active);
                        self.repository
                            .update(product)
                            .await
                            .map(|_| ())
                            .map_err(|_e| ProductError::ProductNotFound)
                    }
                    None => Err(ProductError::ProductNotFound),
                }
            }
        };

        // ProductError → DeletionError へ変換
        match res {
            Ok(_) => Ok(()),
            Err(err) => match err {
                ProductError::ProductNotFound => {
                    Err(DeletionError::NotFound(format!("Product {} not found", id)))
                }
                _ => Err(DeletionError::Other(anyhow::Error::new(err))),
            },
        }
    }
}
