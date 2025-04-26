use crate::model::item::Item;
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

#[cfg(test)]
pub use mockall::predicate::*;
#[cfg(test)]
pub use mockall::mock;

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_all_success() {
        let item1 = Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
        };
        let item2 = Item {
            id: 2,
            name: "Item 2".to_string(),
            description: None,
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo.expect_find_all()
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
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo.expect_find_by_id()
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
        mock_repo.expect_find_by_id()
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
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo.expect_create()
            .with(function(move |i: &Item| {
                i.id == 1 && i.name == "New Item" && i.description == Some("New Description".to_string())
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
        };

        let mut mock_repo = MockItemRepo::new();
        mock_repo.expect_update()
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
        mock_repo.expect_delete()
            .with(eq(1u64))
            .return_once(|_| true);

        let result = mock_repo.delete(1).await;

        assert!(result);
    }
}
