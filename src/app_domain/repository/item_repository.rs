use crate::infrastructure::error::AppResult;
use async_trait::async_trait;
use domain::model::item::{DeletionLog, DeletionValidation, Item};
use mockall::automock;

#[automock]
#[async_trait]
pub trait ItemRepository {
    async fn find_all(&self) -> AppResult<Vec<Item>>;
    async fn find_by_id(&self, id: u64) -> AppResult<Option<Item>>;
    async fn create(&self, item: Item) -> AppResult<Item>;
    async fn update(&self, item: Item) -> AppResult<Item>;

    // New methods for product deletion API
    async fn logical_delete(&self, id: u64) -> AppResult<()>;
    async fn physical_delete(&self, id: u64) -> AppResult<()>;
    async fn restore(&self, id: u64) -> AppResult<()>;
    async fn find_deleted(&self) -> AppResult<Vec<Item>>;
    async fn validate_deletion(&self, id: u64) -> AppResult<DeletionValidation>;
    async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> AppResult<Vec<u64>>;
    async fn get_deletion_logs(&self, item_id: Option<u64>) -> AppResult<Vec<DeletionLog>>;
}
