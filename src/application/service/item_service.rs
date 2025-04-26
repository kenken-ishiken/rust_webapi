use domain::model::item::Item;
use crate::application::dto::item_dto::{CreateItemRequest, UpdateItemRequest};
use std::sync::Mutex;
use crate::infrastructure::metrics::{increment_success_counter, increment_error_counter};

pub struct ItemService {
    repository: domain::repository::item_repository::ItemRepositoryImpl,
    counter: Mutex<u64>,
}

impl ItemService {
    pub fn new(repository: domain::repository::item_repository::ItemRepositoryImpl) -> Self {
        Self {
            repository,
            counter: Mutex::new(0),
        }
    }

    pub async fn find_all(&self) -> Vec<Item> {
        let items = self.repository.find_all().await;
        increment_success_counter("item", "find_all");
        items
    }

    pub async fn find_by_id(&self, id: u64) -> Option<Item> {
        let item = self.repository.find_by_id(id).await;
        if item.is_some() {
            increment_success_counter("item", "find_by_id");
        } else {
            increment_error_counter("item", "find_by_id");
        }
        item
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

        let _created_item = self.repository.create(item).await;
        increment_success_counter("item", "create");
        _created_item
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
            increment_error_counter("item", "update");
            None
        }
    }

    pub async fn delete(&self, id: u64) -> bool {
        let result = self.repository.delete(id).await;
        if result {
            increment_success_counter("item", "delete");
        } else {
            increment_error_counter("item", "delete");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;
    use domain::repository::item_repository::ItemRepository;

    mock! {
        ItemRep {}
        #[async_trait::async_trait]
        impl ItemRepository for ItemRep {
            async fn find_all(&self) -> Vec<Item>;
            async fn find_by_id(&self, id: u64) -> Option<Item>;
            async fn create(&self, item: Item) -> Item;
            async fn update(&self, item: Item) -> Option<Item>;
            async fn delete(&self, id: u64) -> bool;
        }
    }

    #[tokio::test]
    async fn test_find_all() {
        let items = vec![
            Item {
                id: 1,
                name: "Item 1".to_string(),
                description: Some("Description 1".to_string()),
            },
            Item {
                id: 2,
                name: "Item 2".to_string(),
                description: None,
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_all()
            .return_once(move || items.clone());

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_all().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].name, "Item 1");
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].name, "Item 2");
    }

    #[tokio::test]
    async fn test_find_by_id_found() {
        let item = Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(item.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_by_id(1).await;

        assert!(result.is_some());
        let found_item = result.unwrap();
        assert_eq!(found_item.id, 1);
        assert_eq!(found_item.name, "Item 1");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_by_id(999).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create() {
        let req = CreateItemRequest {
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
        };

        let _created_item = Item {
            id: 0,
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_create()
            .with(function(|item: &Item| {
                item.name == "New Item" && 
                item.description == Some("New Description".to_string())
            }))
            .return_once(|item| item);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.create(req).await;

        assert_eq!(result.id, 0);
        assert_eq!(result.name, "New Item");
        assert_eq!(result.description, Some("New Description".to_string()));
    }

    #[tokio::test]
    async fn test_update_found() {
        let req = UpdateItemRequest {
            name: Some("Updated Item".to_string()),
            description: Some("Updated Description".to_string()),
        };

        let existing_item = Item {
            id: 1,
            name: "Original Item".to_string(),
            description: None,
        };

        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(existing_item));

        mock_repo.expect_update()
            .with(function(|item: &Item| {
                item.id == 1 && 
                item.name == "Updated Item" && 
                item.description == Some("Updated Description".to_string())
            }))
            .return_once(move |_| Some(updated_item));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.update(1, req).await;

        assert!(result.is_some());
        let updated = result.unwrap();
        assert_eq!(updated.id, 1);
        assert_eq!(updated.name, "Updated Item");
        assert_eq!(updated.description, Some("Updated Description".to_string()));
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let req = UpdateItemRequest {
            name: Some("Updated Item".to_string()),
            description: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.update(999, req).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_delete()
            .with(eq(999u64))
            .return_once(|_| false);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.delete(999).await;

        assert!(!result);
    }
}
