use crate::model::item::{DeletionLog, DeletionValidation, Item};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ItemRepository: Send + Sync {
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

pub type ItemRepositoryImpl = Arc<dyn ItemRepository>;

#[cfg(test)]
pub use mockall::mock;
#[cfg(test)]
pub use mockall::predicate::*;

#[cfg(test)]
mock! {
    pub ItemRepo {}
    #[async_trait]
    impl ItemRepository for ItemRepo {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::item::{DeletionType, RelatedDataCount};
    use chrono::Utc;

    #[tokio::test]
    async fn test_find_all_success() {
        let item1 = Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
            deleted: false,
            deleted_at: None,
        };
        let item2 = Item {
            id: 2,
            name: "Item 2".to_string(),
            description: None,
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_find_all()
            .return_once(move || vec![item1.clone(), item2.clone()]);

        let result = mock_repo.find_all().await;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].name, "Item 1");
        assert_eq!(result[0].description, Some("Description 1".to_string()));
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].name, "Item 2");
        assert_eq!(result[1].description, None);
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

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(1u64))
            .return_once(move |_| Some(item.clone()));

        let result = mock_repo.find_by_id(1).await;

        assert!(result.is_some());
        let found_item = result.unwrap();
        assert_eq!(found_item.id, 1);
        assert_eq!(found_item.name, "Item 1");
        assert_eq!(found_item.description, Some("Description 1".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_find_by_id()
            .with(eq(999u64))
            .return_once(|_| None);

        let result = mock_repo.find_by_id(999).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_success() {
        let item = Item {
            id: 1,
            name: "New Item".to_string(),
            description: Some("New Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_create()
            .with(function(move |i: &Item| {
                i.id == 1
                    && i.name == "New Item"
                    && i.description == Some("New Description".to_string())
            }))
            .return_once(move |item| item);

        let result = mock_repo.create(item.clone()).await;

        assert_eq!(result.id, 1);
        assert_eq!(result.name, "New Item");
        assert_eq!(result.description, Some("New Description".to_string()));
    }

    #[tokio::test]
    async fn test_update_success() {
        let item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_update()
            .with(function(move |i: &Item| {
                i.id == 1 && i.name == "Updated Item"
            }))
            .return_once(move |item| Some(item));

        let result = mock_repo.update(item.clone()).await;

        assert!(result.is_some());
        let updated = result.unwrap();
        assert_eq!(updated.id, 1);
        assert_eq!(updated.name, "Updated Item");
        assert_eq!(updated.description, Some("Updated Description".to_string()));
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_logical_delete_success() {
        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_logical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.logical_delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_physical_delete_success() {
        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_physical_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.physical_delete(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_restore_success() {
        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_restore()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.restore(1).await;

        assert!(result);
    }

    #[tokio::test]
    async fn test_find_deleted_success() {
        let now = Utc::now();
        let items = vec![
            Item {
                id: 1,
                name: "Deleted Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: true,
                deleted_at: Some(now),
            },
            Item {
                id: 2,
                name: "Deleted Item 2".to_string(),
                description: None,
                deleted: true,
                deleted_at: Some(now),
            },
        ];

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_find_deleted()
            .return_once(move || items.clone());

        let result = mock_repo.find_deleted().await;

        assert_eq!(result.len(), 2);
        assert!(result[0].deleted);
        assert!(result[0].deleted_at.is_some());
        assert!(result[1].deleted);
        assert!(result[1].deleted_at.is_some());
    }

    #[tokio::test]
    async fn test_batch_delete_success() {
        let ids = vec![1, 2, 3];
        let successful_ids = vec![1, 3];

        let mut mock_repo = MockItemRepo::new();
        mock_repo
            .expect_batch_delete()
            .with(eq(ids.clone()), eq(false))
            .return_once(move |_, _| successful_ids.clone());

        let result = mock_repo.batch_delete(ids, false).await;

        assert_eq!(result.len(), 2);
        assert_eq!(result, vec![1, 3]);
    }
}
