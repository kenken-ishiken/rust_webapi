use crate::domain::model::item::Item;
use std::sync::Arc;

pub trait ItemRepository: Send + Sync {
    fn find_all(&self) -> Vec<Item>;
    fn find_by_id(&self, id: u64) -> Option<Item>;
    fn create(&self, item: Item) -> Item;
    fn update(&self, item: Item) -> Option<Item>;
    fn delete(&self, id: u64) -> bool;
}

pub type ItemRepositoryImpl = Arc<dyn ItemRepository>;