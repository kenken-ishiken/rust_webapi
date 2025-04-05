use crate::domain::model::item::Item;
use crate::application::dto::item_dto::{CreateItemRequest, UpdateItemRequest};
use std::sync::Mutex;

pub struct ItemService {
    repository: crate::domain::repository::item_repository::ItemRepositoryImpl,
    counter: Mutex<u64>,
}

impl ItemService {
    pub fn new(repository: crate::domain::repository::item_repository::ItemRepositoryImpl) -> Self {
        Self {
            repository,
            counter: Mutex::new(0),
        }
    }

    pub async fn find_all(&self) -> Vec<Item> {
        self.repository.find_all().await
    }

    pub async fn find_by_id(&self, id: u64) -> Option<Item> {
        self.repository.find_by_id(id).await
    }

    pub async fn create(&self, req: CreateItemRequest) -> Item {
        let mut counter = self.counter.lock().unwrap();
        let id = *counter;
        *counter += 1;

        let item = Item {
            id,
            name: req.name,
            description: req.description,
        };

        self.repository.create(item).await
    }

    pub async fn update(&self, id: u64, req: UpdateItemRequest) -> Option<Item> {
        if let Some(mut item) = self.repository.find_by_id(id).await {
            if let Some(name) = req.name {
                item.name = name;
            }
            if let Some(description) = req.description {
                item.description = Some(description);
            }
            self.repository.update(item).await
        } else {
            None
        }
    }

    pub async fn delete(&self, id: u64) -> bool {
        self.repository.delete(id).await
    }
}