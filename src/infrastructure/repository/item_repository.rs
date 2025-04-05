use std::collections::HashMap;
use std::sync::Mutex;
use crate::domain::model::item::Item;
use crate::domain::repository::item_repository::ItemRepository;

pub struct InMemoryItemRepository {
    items: Mutex<HashMap<u64, Item>>,
}

impl InMemoryItemRepository {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }
}

impl ItemRepository for InMemoryItemRepository {
    fn find_all(&self) -> Vec<Item> {
        let items = self.items.lock().unwrap();
        items.values().cloned().collect()
    }

    fn find_by_id(&self, id: u64) -> Option<Item> {
        let items = self.items.lock().unwrap();
        items.get(&id).cloned()
    }

    fn create(&self, item: Item) -> Item {
        let mut items = self.items.lock().unwrap();
        items.insert(item.id, item.clone());
        item
    }

    fn update(&self, item: Item) -> Option<Item> {
        let mut items = self.items.lock().unwrap();
        if items.contains_key(&item.id) {
            items.insert(item.id, item.clone());
            Some(item)
        } else {
            None
        }
    }

    fn delete(&self, id: u64) -> bool {
        let mut items = self.items.lock().unwrap();
        items.remove(&id).is_some()
    }
}