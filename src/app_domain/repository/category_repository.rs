use async_trait::async_trait;
use mockall::automock;
use crate::app_domain::model::category::{Category, CategoryPath, CategoryTree, CategoryError};

#[cfg(test)]
pub use mockall::predicate;

#[automock]
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_all(&self, include_inactive: bool) -> Vec<Category>;
    async fn find_by_id(&self, id: &str) -> Option<Category>;
    async fn find_by_parent_id(&self, parent_id: Option<String>, include_inactive: bool) -> Vec<Category>;
    // async fn find_children(&self, id: &str, include_inactive: bool) -> Vec<Category>;
    async fn find_path(&self, id: &str) -> Result<CategoryPath, CategoryError>;
    async fn find_tree(&self, include_inactive: bool) -> Vec<CategoryTree>;
    async fn exists_by_name_and_parent(&self, name: &str, parent_id: Option<String>, exclude_id: Option<String>) -> bool;
    async fn create(&self, category: Category) -> Result<Category, CategoryError>;
    async fn update(&self, category: Category) -> Result<Category, CategoryError>;
    async fn delete(&self, id: &str) -> Result<bool, CategoryError>;
    async fn move_category(&self, id: &str, new_parent_id: Option<String>, new_sort_order: i32) -> Result<Category, CategoryError>;
    async fn count_children(&self, id: &str) -> i64;
    // async fn count_products(&self, id: &str) -> i64;
    async fn validate_depth(&self, parent_id: Option<String>) -> Result<(), CategoryError>;
    async fn validate_circular_reference(&self, id: &str, new_parent_id: Option<String>) -> Result<(), CategoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_mock_category_repository() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let category = Category {
            id: "cat_123".to_string(),
            name: "Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        mock_repo
            .expect_find_by_id()
            .with(predicate::eq("cat_123"))
            .return_once(move |_| Some(category.clone()));

        let result = mock_repo.find_by_id("cat_123").await;
        assert!(result.is_some());
        
        let found_category = result.unwrap();
        assert_eq!(found_category.id, "cat_123");
        assert_eq!(found_category.name, "Electronics");
    }

    #[tokio::test]
    async fn test_mock_find_all() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let categories = vec![
            Category {
                id: "cat_1".to_string(),
                name: "Category 1".to_string(),
                description: None,
                parent_id: None,
                sort_order: 1,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            Category {
                id: "cat_2".to_string(),
                name: "Category 2".to_string(),
                description: None,
                parent_id: None,
                sort_order: 2,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        ];

        mock_repo
            .expect_find_all()
            .with(predicate::eq(true))
            .return_once(move |_| categories.clone());

        let result = mock_repo.find_all(true).await;
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "cat_1");
        assert_eq!(result[1].id, "cat_2");
    }

    #[tokio::test]
    async fn test_mock_create() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let new_category = Category {
            id: "cat_new".to_string(),
            name: "New Category".to_string(),
            description: None,
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let expected_category = new_category.clone();

        mock_repo
            .expect_create()
            .withf(|cat| cat.name == "New Category")
            .return_once(move |_| Ok(expected_category));

        let result = mock_repo.create(new_category).await;
        assert!(result.is_ok());
        
        let created_category = result.unwrap();
        assert_eq!(created_category.id, "cat_new");
        assert_eq!(created_category.name, "New Category");
    }

    #[tokio::test]
    async fn test_mock_find_path() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let path = CategoryPath::new(vec![
            "cat_root".to_string(),
            "cat_child".to_string(),
            "cat_grandchild".to_string(),
        ]);

        mock_repo
            .expect_find_path()
            .with(predicate::eq("cat_grandchild"))
            .return_once(move |_| Ok(path));

        let result = mock_repo.find_path("cat_grandchild").await;
        assert!(result.is_ok());
        
        let category_path = result.unwrap();
        assert_eq!(category_path.depth, 3);
        assert!(category_path.contains("cat_child"));
    }

    #[tokio::test]
    async fn test_mock_validate_circular_reference() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_validate_circular_reference()
            .with(predicate::eq("cat_parent"), predicate::eq(Some("cat_child".to_string())))
            .return_once(|_, _| Err(CategoryError::CircularReference("循環参照が検出されました".to_string())));

        let result = mock_repo.validate_circular_reference("cat_parent", Some("cat_child".to_string())).await;
        assert!(result.is_err());
        
        match result {
            Err(CategoryError::CircularReference(msg)) => assert_eq!(msg, "循環参照が検出されました"),
            _ => panic!("Expected CircularReference error"),
        }
    }
}