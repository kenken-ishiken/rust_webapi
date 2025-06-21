use async_trait::async_trait;
use std::fmt::Debug;

use crate::app_domain::repository::item_repository::ItemRepository;

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