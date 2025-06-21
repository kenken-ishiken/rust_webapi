use crate::application::dto::item_dto::{
    BatchDeleteRequest, BatchDeleteResponse, CreateItemRequest, DeletionLogResponse,
    DeletionValidationResponse, ItemResponse, UpdateItemRequest,
};
use crate::infrastructure::error::{AppError, AppResult};
use crate::infrastructure::metrics::{increment_error_counter, increment_success_counter};
use domain::model::item::{DeletionType, Item};
use std::sync::Mutex;

pub struct ItemService {
    repository: std::sync::Arc<
        dyn crate::app_domain::repository::item_repository::ItemRepository + Send + Sync,
    >,
    counter: Mutex<u64>,
}

impl ItemService {
    pub fn new(
        repository: std::sync::Arc<
            dyn crate::app_domain::repository::item_repository::ItemRepository + Send + Sync,
        >,
    ) -> Self {
        Self {
            repository,
            counter: Mutex::new(0),
        }
    }

    pub async fn find_all(&self) -> AppResult<Vec<ItemResponse>> {
        let items = self.repository.find_all().await?;
        increment_success_counter("item", "find_all");
        Ok(items
            .into_iter()
            .map(|item| self.to_response(item))
            .collect())
    }

    pub async fn find_by_id(&self, id: u64) -> AppResult<ItemResponse> {
        let item = self.repository.find_by_id(id).await?;
        match item {
            Some(item) => {
                increment_success_counter("item", "find_by_id");
                Ok(self.to_response(item))
            }
            None => {
                increment_error_counter("item", "find_by_id");
                Err(AppError::NotFound(format!("Item with id {} not found", id)))
            }
        }
    }

    pub async fn create(&self, req: CreateItemRequest) -> AppResult<ItemResponse> {
        let id = {
            let mut counter = self
                .counter
                .lock()
                .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
            let id = *counter;
            *counter += 1;
            id
        };

        let item = Item {
            id,
            name: req.name,
            description: req.description,
            deleted: false,
            deleted_at: None,
        };

        let created_item = self.repository.create(item).await?;
        increment_success_counter("item", "create");
        Ok(self.to_response(created_item))
    }

    pub async fn update(&self, id: u64, req: UpdateItemRequest) -> AppResult<ItemResponse> {
        let item_opt = self.repository.find_by_id(id).await?;
        match item_opt {
            Some(mut item) => {
                if let Some(name) = req.name {
                    item.name = name;
                }
                if let Some(description) = req.description {
                    item.description = Some(description);
                }
                let updated = self.repository.update(item).await?;
                increment_success_counter("item", "update");
                Ok(self.to_response(updated))
            }
            None => {
                increment_error_counter("item", "update");
                Err(AppError::NotFound(format!("Item with id {} not found", id)))
            }
        }
    }

    // New methods for product deletion API

    pub async fn find_deleted(&self) -> AppResult<Vec<ItemResponse>> {
        let items = self.repository.find_deleted().await?;
        increment_success_counter("item", "find_deleted");
        Ok(items
            .into_iter()
            .map(|item| self.to_response(item))
            .collect())
    }

    pub async fn validate_deletion(&self, id: u64) -> AppResult<DeletionValidationResponse> {
        // First check if the item exists
        let item_opt = self.repository.find_by_id(id).await?;
        if item_opt.is_none() {
            increment_error_counter("item", "validate_deletion");
            return Err(AppError::NotFound(format!("Item with id {} not found", id)));
        }

        let validation = self.repository.validate_deletion(id).await?;
        increment_success_counter("item", "validate_deletion");

        Ok(DeletionValidationResponse {
            can_delete: validation.can_delete,
            related_orders: validation.related_data.related_orders,
            related_reviews: validation.related_data.related_reviews,
            related_categories: validation.related_data.related_categories,
        })
    }

    pub async fn batch_delete(&self, req: BatchDeleteRequest) -> AppResult<BatchDeleteResponse> {
        let is_physical = req.is_physical.unwrap_or(false);
        let all_ids = req.ids.clone();

        let successful_ids = self.repository.batch_delete(req.ids, is_physical).await?;
        let failed_ids: Vec<u64> = all_ids
            .into_iter()
            .filter(|id| !successful_ids.contains(id))
            .collect();

        if !successful_ids.is_empty() {
            increment_success_counter("item", "batch_delete");
        }
        if !failed_ids.is_empty() {
            increment_error_counter("item", "batch_delete");
        }

        Ok(BatchDeleteResponse {
            successful_ids,
            failed_ids,
        })
    }

    pub async fn get_deletion_logs(
        &self,
        item_id: Option<u64>,
    ) -> AppResult<Vec<DeletionLogResponse>> {
        let logs = self.repository.get_deletion_logs(item_id).await?;
        increment_success_counter("item", "get_deletion_logs");

        Ok(logs
            .into_iter()
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
            .collect())
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
    use crate::app_domain::repository::item_repository::MockItemRepository;
    use chrono::Utc;
    use domain::model::item::{DeletionLog, DeletionValidation, RelatedDataCount};
    use mockall::predicate::*;
    use std::sync::Arc;

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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_all()
            .return_once(move || Ok(items.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_all().await.unwrap();

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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Ok(Some(item.clone())));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_by_id(1).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "Item 1");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| Ok(None));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_by_id(999).await;

        assert!(result.is_err());
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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_create()
            .with(function(|item: &Item| {
                item.name == "New Item"
                    && item.description == Some("New Description".to_string())
                    && !item.deleted
            }))
            .return_once(move |_| Ok(created_item.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.create(req).await.unwrap();

        assert_eq!(result.id, 0);
        assert_eq!(result.name, "New Item");
        assert_eq!(result.description, Some("New Description".to_string()));
        assert!(!result.deleted);
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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Ok(Some(existing_item)));

        mock_repo
            .expect_update()
            .with(function(|item: &Item| {
                item.id == 1
                    && item.name == "Updated Item"
                    && item.description == Some("Updated Description".to_string())
                    && !item.deleted
            }))
            .return_once(move |_| Ok(updated_item));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.update(1, req).await.unwrap();

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "Updated Item");
        assert_eq!(result.description, Some("Updated Description".to_string()));
        assert!(!result.deleted);
        assert!(result.deleted_at.is_none());
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let req = UpdateItemRequest {
            name: Some("Updated Item".to_string()),
            description: None,
        };

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| Ok(None));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.update(999, req).await;

        assert!(result.is_err());
    }

    // New tests for product deletion API

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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_deleted()
            .return_once(move || Ok(deleted_items.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.find_deleted().await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].name, "Deleted Item 1");
        assert!(result[0].deleted);
        assert!(result[0].deleted_at.is_some());
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].name, "Deleted Item 2");
        assert!(result[1].deleted);
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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(|_| {
                Ok(Some(Item {
                    id: 1,
                    name: "Test Item".to_string(),
                    description: None,
                    deleted: false,
                    deleted_at: None,
                }))
            });

        mock_repo
            .expect_validate_deletion()
            .with(eq(1u64))
            .return_once(move |_| Ok(validation));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.validate_deletion(1).await;

        assert!(result.is_ok());
        let validation_response = result.unwrap();
        assert!(validation_response.can_delete);
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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_batch_delete()
            .with(eq(vec![1, 2, 3]), eq(false))
            .return_once(move |_, _| Ok(successful_ids.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.batch_delete(req).await.unwrap();

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

        let mut mock_repo = MockItemRepository::new();
        mock_repo
            .expect_get_deletion_logs()
            .with(eq(None))
            .return_once(move |_| Ok(logs.clone()));

        let service = ItemService::new(Arc::new(mock_repo));
        let result = service.get_deletion_logs(None).await.unwrap();

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
