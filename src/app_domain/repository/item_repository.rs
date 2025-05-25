use async_trait::async_trait;
use mockall::automock;
use domain::model::item::{Item, DeletionValidation, DeletionLog};

pub use mockall::predicate;

#[automock]
#[async_trait]
pub trait ItemRepository {
    async fn find_all(&self) -> Vec<Item>;
    async fn find_by_id(&self, id: u64) -> Option<Item>;
    async fn create(&self, item: Item) -> Item;
    async fn update(&self, item: Item) -> Option<Item>;
    async fn delete(&self, id: u64) -> bool;
    
    // New methods for product deletion API
    async fn logical_delete(&self, id: u64) -> bool;
    async fn physical_delete(&self, id: u64) -> bool;
    async fn restore(&self, id: u64) -> bool;
    async fn find_deleted(&self) -> Vec<Item>;
    async fn validate_deletion(&self, id: u64) -> DeletionValidation;
    async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> Vec<u64>;
    async fn get_deletion_logs(&self, item_id: Option<u64>) -> Vec<DeletionLog>;
}
