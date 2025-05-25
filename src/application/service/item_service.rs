use domain::model::item::{Item, DeletionLog, DeletionType, DeletionValidation};
use crate::application::dto::item_dto::{
    CreateItemRequest, UpdateItemRequest, ItemResponse, 
    BatchDeleteRequest, BatchDeleteResponse, DeletionValidationResponse, DeletionLogResponse
};
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

    pub async fn find_all(&self) -> Vec<ItemResponse> {
        let items = self.repository.find_all().await;
        increment_success_counter("item", "find_all");
        items.into_iter().map(|item| self.to_response(item)).collect()
    }

    pub async fn find_by_id(&self, id: u64) -> Option<ItemResponse> {
        let item = self.repository.find_by_id(id).await;
        if item.is_some() {
            increment_success_counter("item", "find_by_id");
        } else {
            increment_error_counter("item", "find_by_id");
        }
        item.map(|item| self.to_response(item))
    }

    pub async fn create(&self, req: CreateItemRequest) -> ItemResponse {
        let mut counter = self.counter.lock().unwrap();
        let id = *counter;
        *counter += 1;

        let item = Item {
            id,
            name: req.name,
            description: req.description,
            deleted: false,
            deleted_at: None,
        };

        let created_item = self.repository.create(item).await;
        increment_success_counter("item", "create");
        self.to_response(created_item)
    }

    pub async fn update(&self, id: u64, req: UpdateItemRequest) -> Option<ItemResponse> {
        if let Some(mut item) = self.repository.find_by_id(id).await {
            if let Some(name) = req.name {
                item.name = name;
            }
            if let Some(description) = req.description {
                item.description = Some(description);
            }
            let updated = self.repository.update(item).await;
            if updated.is_some() {
                increment_success_counter("item", "update");
            } else {
                increment_error_counter("item", "update");
            }
            updated.map(|item| self.to_response(item))
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
    
    // New methods for product deletion API
    
    pub async fn logical_delete(&self, id: u64) -> bool {
        let result = self.repository.logical_delete(id).await;
        if result {
            increment_success_counter("item", "logical_delete");
        } else {
            increment_error_counter("item", "logical_delete");
        }
        result
    }
    
    pub async fn physical_delete(&self, id: u64) -> bool {
        let result = self.repository.physical_delete(id).await;
        if result {
            increment_success_counter("item", "physical_delete");
        } else {
            increment_error_counter("item", "physical_delete");
        }
        result
    }
    
    pub async fn restore(&self, id: u64) -> bool {
        let result = self.repository.restore(id).await;
        if result {
            increment_success_counter("item", "restore");
        } else {
            increment_error_counter("item", "restore");
        }
        result
    }
    
    pub async fn find_deleted(&self) -> Vec<ItemResponse> {
        let items = self.repository.find_deleted().await;
        increment_success_counter("item", "find_deleted");
        items.into_iter().map(|item| self.to_response(item)).collect()
    }
    
    pub async fn validate_deletion(&self, id: u64) -> Option<DeletionValidationResponse> {
        // First check if the item exists
        if self.repository.find_by_id(id).await.is_none() {
            increment_error_counter("item", "validate_deletion");
            return None;
        }
        
        let validation = self.repository.validate_deletion(id).await;
        increment_success_counter("item", "validate_deletion");
        
        Some(DeletionValidationResponse {
            can_delete: validation.can_delete,
            related_orders: validation.related_data.related_orders,
            related_reviews: validation.related_data.related_reviews,
            related_categories: validation.related_data.related_categories,
        })
    }
    
    pub async fn batch_delete(&self, req: BatchDeleteRequest) -> BatchDeleteResponse {
        let is_physical = req.is_physical.unwrap_or(false);
        let all_ids = req.ids.clone();
        
        let successful_ids = self.repository.batch_delete(req.ids, is_physical).await;
        let failed_ids: Vec<u64> = all_ids.into_iter()
            .filter(|id| !successful_ids.contains(id))
            .collect();
        
        if !successful_ids.is_empty() {
            increment_success_counter("item", "batch_delete");
        }
        if !failed_ids.is_empty() {
            increment_error_counter("item", "batch_delete");
        }
        
        BatchDeleteResponse {
            successful_ids,
            failed_ids,
        }
    }
    
    pub async fn get_deletion_logs(&self, item_id: Option<u64>) -> Vec<DeletionLogResponse> {
        let logs = self.repository.get_deletion_logs(item_id).await;
        increment_success_counter("item", "get_deletion_logs");
        
        logs.into_iter()
            .map(|log| DeletionLogResponse {
                id: log.id,
                item_id: log.item_id,
                item_name: log.item_name,
                deletion_type: match log.deletion_type {
                    DeletionType::Logical => "Logical".to_string(),
                    DeletionType::Physical => "Physical".to_string(),
                    DeletionType::Restore => "Restore".to_string(),
                },
                deleted_at: log.deleted_at,
                deleted_by: log.deleted_by,
            })
            .collect()
    }
    
    // Helper method to convert domain Item to ItemResponse DTO
    fn to_response(&self, item: Item) -> ItemResponse {
        ItemResponse {
            id: item.id,
            name: item.name,
            description: item.description,
            deleted: item.deleted,
            deleted_at: item.deleted_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;
    use domain::repository::item_repository::ItemRepository;
    use chrono::Utc;

    mock! {
        ItemRep {}
        #[async_trait::async_trait]
        impl ItemRepository for ItemRep {
            async fn find_all(&self) -> Vec<Item>;
            async fn find_by_id(&self, id: u64) -> Option<Item>;
            async fn create(&self, item: Item) -> Item;
            async fn update(&self, item: Item) -> Option<Item>;
            async fn delete(&self, id: u64) -> bool;
            async fn logical_delete(&self, id: u64) -> bool;
            async fn physical_delete(&self, id: u64) -> bool;
            async fn restore(&self, id: u64) -> bool;
            async fn find_deleted(&self) -> Vec<Item>;
            async fn validate_deletion(&self, id: u64) -> DeletionValidation;
            async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> Vec<u64>;
            async fn get_deletion_logs(&self, item_id: Option<u64>) -> Vec<DeletionLog>;
        }
    }

    #[tokio::test]
    async fn test_find_all() {
        let items = vec![
            Item {
                id: 1,
                name: "Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: false,
                deleted_at: None,
            },
            Item {
                id: 2,
                name: "Item 2".to_string(),
                description: None,
                deleted: false,
                deleted_at: None,
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
            deleted: false,
            deleted_at: None,
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

        let created_item = Item {
            id: 0,
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_create()
            .with(function(|item: &Item| {
                item.name == "New Item" && 
                item.description == Some("New Description".to_string()) &&
                !item.deleted
            }))
            .return_once(|_| created_item.clone());

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.create(req).await;

        assert_eq!(result.id, 0);
        assert_eq!(result.name, "New Item");
        assert_eq!(result.description, Some("New Description".to_string()));
        assert_eq!(result.deleted, false);
        assert!(result.deleted_at.is_none());
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
            deleted: false,
            deleted_at: None,
        };

        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(existing_item));

        mock_repo.expect_update()
            .with(function(|item: &Item| {
                item.id == 1 && 
                item.name == "Updated Item" && 
                item.description == Some("Updated Description".to_string()) &&
                !item.deleted
            }))
            .return_once(move |_| Some(updated_item));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.update(1, req).await;

        assert!(result.is_some());
        let updated = result.unwrap();
        assert_eq!(updated.id, 1);
        assert_eq!(updated.name, "Updated Item");
        assert_eq!(updated.description, Some("Updated Description".to_string()));
        assert_eq!(updated.deleted, false);
        assert!(updated.deleted_at.is_none());
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
    
    // New tests for product deletion API
    
    #[tokio::test]
    async fn test_logical_delete_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_logical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.logical_delete(1).await;

        assert!(result);
    }
    
    #[tokio::test]
    async fn test_logical_delete_not_found() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_logical_delete()
            .with(eq(999u64))
            .return_once(|_| false);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.logical_delete(999).await;

        assert!(!result);
    }
    
    #[tokio::test]
    async fn test_physical_delete_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_physical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.physical_delete(1).await;

        assert!(result);
    }
    
    #[tokio::test]
    async fn test_restore_success() {
        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_restore()
            .with(eq(1u64))
            .return_once(|_| true);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.restore(1).await;

        assert!(result);
    }
    
    #[tokio::test]
    async fn test_find_deleted() {
        let deleted_items = vec![
            Item {
                id: 1,
                name: "Deleted Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: true,
                deleted_at: Some(Utc::now()),
            },
            Item {
                id: 2,
                name: "Deleted Item 2".to_string(),
                description: None,
                deleted: true,
                deleted_at: Some(Utc::now()),
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_deleted()
            .return_once(move || deleted_items.clone());

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_deleted().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].name, "Deleted Item 1");
        assert_eq!(result[0].deleted, true);
        assert!(result[0].deleted_at.is_some());
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].name, "Deleted Item 2");
        assert_eq!(result[1].deleted, true);
        assert!(result[1].deleted_at.is_some());
    }
    
    #[tokio::test]
    async fn test_validate_deletion() {
        let validation = DeletionValidation {
            can_delete: true,
            related_data: RelatedDataCount {
                related_orders: 0,
                related_reviews: 0,
                related_categories: 0,
            },
        };

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| Some(Item {
                id: 1,
                name: "Test Item".to_string(),
                description: None,
                deleted: false,
                deleted_at: None,
            }));
            
        mock_repo.expect_validate_deletion()
            .with(eq(1u64))
            .return_once(move |_| validation);

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.validate_deletion(1).await;

        assert!(result.is_some());
        let validation_response = result.unwrap();
        assert_eq!(validation_response.can_delete, true);
        assert_eq!(validation_response.related_orders, 0);
        assert_eq!(validation_response.related_reviews, 0);
        assert_eq!(validation_response.related_categories, 0);
    }
    
    #[tokio::test]
    async fn test_batch_delete() {
        let req = BatchDeleteRequest {
            ids: vec![1, 2, 3],
            is_physical: Some(false),
        };

        let successful_ids = vec![1, 3];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_batch_delete()
            .with(eq(vec![1, 2, 3]), eq(false))
            .return_once(move |_, _| successful_ids.clone());

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.batch_delete(req).await;

        assert_eq!(result.successful_ids, vec![1, 3]);
        assert_eq!(result.failed_ids, vec![2]);
    }
    
    #[tokio::test]
    async fn test_get_deletion_logs() {
        let now = Utc::now();
        let logs = vec![
            DeletionLog {
                id: 1,
                item_id: 1,
                item_name: "Item 1".to_string(),
                deletion_type: DeletionType::Logical,
                deleted_at: now,
                deleted_by: "test_user".to_string(),
            },
            DeletionLog {
                id: 2,
                item_id: 2,
                item_name: "Item 2".to_string(),
                deletion_type: DeletionType::Physical,
                deleted_at: now,
                deleted_by: "test_user".to_string(),
            },
        ];

        let mut mock_repo = MockItemRep::new();
        mock_repo.expect_get_deletion_logs()
            .with(eq(None))
            .return_once(move |_| logs.clone());

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.get_deletion_logs(None).await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].item_id, 1);
        assert_eq!(result[0].item_name, "Item 1");
        assert_eq!(result[0].deletion_type, "Logical");
        assert_eq!(result[0].deleted_by, "test_user");
        
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].item_id, 2);
        assert_eq!(result[1].item_name, "Item 2");
        assert_eq!(result[1].deletion_type, "Physical");
        assert_eq!(result[1].deleted_by, "test_user");
    }
}
