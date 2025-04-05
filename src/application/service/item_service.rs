use crate::domain::model::item::Item;
use crate::domain::repository::item_repository::ItemRepositoryImpl;
use crate::application::dto::item_dto::{CreateItemRequest, UpdateItemRequest};

pub struct ItemService {
    repository: ItemRepositoryImpl,
    counter: std::sync::Mutex<u64>,
}

impl ItemService {
    pub fn new(repository: ItemRepositoryImpl) -> Self {
        Self {
            repository,
            counter: std::sync::Mutex::new(0),
        }
    }

    pub fn find_all(&self) -> Vec<Item> {
        self.repository.find_all()
    }

    pub fn find_by_id(&self, id: u64) -> Option<Item> {
        self.repository.find_by_id(id)
    }

    pub fn create(&self, req: CreateItemRequest) -> Item {
        let mut counter = self.counter.lock().unwrap();
        let id = *counter;
        *counter += 1;

        let item = Item {
            id,
            name: req.name,
            description: req.description,
        };

        self.repository.create(item)
    }

    pub fn update(&self, id: u64, req: UpdateItemRequest) -> Option<Item> {
        if let Some(mut item) = self.repository.find_by_id(id) {
            if let Some(name) = req.name {
                item.name = name;
            }
            if let Some(description) = req.description {
                item.description = Some(description);
            }
            self.repository.update(item)
        } else {
            None
        }
    }

    pub fn delete(&self, id: u64) -> bool {
        self.repository.delete(id)
    }
}