use crate::domain::model::item::Item;
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn find_all(&self) -> Vec<Item>;
    async fn find_by_id(&self, id: u64) -> Option<Item>;
    async fn create(&self, item: Item) -> Item;
    async fn update(&self, item: Item) -> Option<Item>;
    async fn delete(&self, id: u64) -> bool;
}

pub type ItemRepositoryImpl = Arc<dyn ItemRepository>;