use async_trait::async_trait;
use mockall::automock;
use domain::model::item::Item;

pub use mockall::predicate;

#[automock]
#[async_trait]
pub trait ItemRepository {
    async fn find_all(&self) -> Vec<Item>;
    async fn find_by_id(&self, id: u64) -> Option<Item>;
    async fn create(&self, item: Item) -> Item;
    async fn update(&self, item: Item) -> Option<Item>;
    async fn delete(&self, id: u64) -> bool;
}
