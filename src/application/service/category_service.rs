use std::sync::Arc;
use tracing::{info, error};

use crate::app_domain::model::category::{Category, CategoryPath, CategoryTree, CategoryError};
use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::application::dto::category_dto::{
    CreateCategoryRequest, UpdateCategoryRequest, MoveCategoryRequest,
    CategoryResponse, CategoryListResponse, CategoryTreeResponse, CategoryPathResponse,
    CategoriesResponse, CategoryTreesResponse, CategoryPathItem
};
use crate::infrastructure::metrics::{increment_success_counter, increment_error_counter};

pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repository: Arc<dyn CategoryRepository>) -> Self {
        Self { repository }
    }

    pub async fn find_all(&self, include_inactive: bool) -> CategoriesResponse {
        let categories = self.repository.find_all(include_inactive).await;
        
        let mut category_list = Vec::new();
        for category in &categories {
            let children_count = self.repository.count_children(&category.id).await;
            category_list.push(CategoryListResponse {
                id: category.id.clone(),
                name: category.name.clone(),
                description: category.description.clone(),
                parent_id: category.parent_id.clone(),
                sort_order: category.sort_order,
                is_active: category.is_active,
                children_count,
                created_at: category.created_at,
                updated_at: category.updated_at,
            });
        }

        increment_success_counter("category", "find_all");
        info!("Fetched {} categories", categories.len());
        
        CategoriesResponse {
            categories: category_list,
            total: categories.len(),
        }
    }

    pub async fn find_by_id(&self, id: &str) -> Result<CategoryResponse, CategoryError> {
        match self.repository.find_by_id(id).await {
            Some(category) => {
                increment_success_counter("category", "find_by_id");
                info!("Fetched category {}", id);
                Ok(category.into())
            }
            None => {
                increment_error_counter("category", "find_by_id");
                error!("Category {} not found", id);
                Err(CategoryError::NotFound("カテゴリが見つかりません".to_string()))
            }
        }
    }

    pub async fn find_by_parent_id(&self, parent_id: Option<&str>, include_inactive: bool) -> CategoriesResponse {
        let categories = self.repository.find_by_parent_id(parent_id, include_inactive).await;
        
        let mut category_list = Vec::new();
        for category in &categories {
            let children_count = self.repository.count_children(&category.id).await;
            category_list.push(CategoryListResponse {
                id: category.id.clone(),
                name: category.name.clone(),
                description: category.description.clone(),
                parent_id: category.parent_id.clone(),
                sort_order: category.sort_order,
                is_active: category.is_active,
                children_count,
                created_at: category.created_at,
                updated_at: category.updated_at,
            });
        }

        increment_success_counter("category", "find_by_parent");
        info!("Fetched {} categories for parent {:?}", categories.len(), parent_id);
        
        CategoriesResponse {
            categories: category_list,
            total: categories.len(),
        }
    }

    pub async fn find_children(&self, id: &str, include_inactive: bool) -> CategoriesResponse {
        self.find_by_parent_id(Some(id), include_inactive).await
    }

    pub async fn find_path(&self, id: &str) -> Result<CategoryPathResponse, CategoryError> {
        let path = self.repository.find_path(id).await?;
        
        // Enrich path with category names
        let mut path_items = Vec::new();
        for category_id in &path.path {
            if let Some(category) = self.repository.find_by_id(category_id).await {
                path_items.push(CategoryPathItem {
                    id: category.id,
                    name: category.name,
                });
            }
        }

        increment_success_counter("category", "find_path");
        info!("Fetched path for category {}, depth: {}", id, path.depth);
        
        Ok(CategoryPathResponse {
            path: path_items,
            depth: path.depth,
        })
    }

    pub async fn find_tree(&self, include_inactive: bool) -> CategoryTreesResponse {
        let trees = self.repository.find_tree(include_inactive).await;
        
        increment_success_counter("category", "find_tree");
        info!("Fetched category tree with {} root categories", trees.len());
        
        CategoryTreesResponse {
            tree: trees.into_iter().map(|t| t.into()).collect(),
        }
    }

    pub async fn create(&self, req: CreateCategoryRequest) -> Result<CategoryResponse, CategoryError> {
        // Generate unique ID (in real application, you might want to use UUID or database sequence)
        let id = format!("cat_{}", chrono::Utc::now().timestamp_millis());
        
        let category = Category::new(
            id,
            req.name,
            req.description,
            req.parent_id,
            req.sort_order,
        );

        match self.repository.create(category).await {
            Ok(created_category) => {
                increment_success_counter("category", "create");
                info!("Created category with id {}", created_category.id);
                Ok(created_category.into())
            }
            Err(e) => {
                increment_error_counter("category", "create");
                error!("Failed to create category: {}", e);
                Err(e)
            }
        }
    }

    pub async fn update(&self, id: &str, req: UpdateCategoryRequest) -> Result<CategoryResponse, CategoryError> {
        let mut category = self.repository.find_by_id(id).await
            .ok_or_else(|| CategoryError::NotFound("カテゴリが見つかりません".to_string()))?;

        // Update fields if provided
        if let Some(name) = req.name {
            category.update_name(name)?;
        }
        
        if let Some(description) = req.description {
            category.update_description(Some(description));
        }
        
        if let Some(sort_order) = req.sort_order {
            category.update_sort_order(sort_order)?;
        }
        
        if let Some(is_active) = req.is_active {
            if is_active {
                category.activate();
            } else {
                category.deactivate();
            }
        }

        match self.repository.update(category).await {
            Ok(updated_category) => {
                increment_success_counter("category", "update");
                info!("Updated category {}", id);
                Ok(updated_category.into())
            }
            Err(e) => {
                increment_error_counter("category", "update");
                error!("Failed to update category {}: {}", id, e);
                Err(e)
            }
        }
    }

    pub async fn delete(&self, id: &str) -> Result<bool, CategoryError> {
        match self.repository.delete(id).await {
            Ok(deleted) => {
                if deleted {
                    increment_success_counter("category", "delete");
                    info!("Deleted category {}", id);
                } else {
                    increment_error_counter("category", "delete");
                    error!("Category {} not found for deletion", id);
                    return Err(CategoryError::NotFound("カテゴリが見つかりません".to_string()));
                }
                Ok(deleted)
            }
            Err(e) => {
                increment_error_counter("category", "delete");
                error!("Failed to delete category {}: {}", id, e);
                Err(e)
            }
        }
    }

    pub async fn move_category(&self, id: &str, req: MoveCategoryRequest) -> Result<CategoryResponse, CategoryError> {
        match self.repository.move_category(id, req.parent_id.as_deref(), req.sort_order).await {
            Ok(moved_category) => {
                increment_success_counter("category", "move");
                info!("Moved category {} to parent {:?} with sort order {}", id, req.parent_id, req.sort_order);
                Ok(moved_category.into())
            }
            Err(e) => {
                increment_error_counter("category", "move");
                error!("Failed to move category {}: {}", id, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_domain::repository::category_repository::MockCategoryRepository;
    use mockall::predicate::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_find_by_id_success() {
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
            .with(eq("cat_123"))
            .return_once(move |_| Some(category.clone()));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.find_by_id("cat_123").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.id, "cat_123");
        assert_eq!(response.name, "Electronics");
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_find_by_id()
            .with(eq("cat_999"))
            .return_once(|_| None);

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.find_by_id("cat_999").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CategoryError::NotFound(_) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_create_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let request = CreateCategoryRequest {
            name: "Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: None,
            sort_order: 1,
        };

        let expected_category = Category {
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
            .expect_create()
            .withf(|cat| cat.name == "Electronics")
            .return_once(move |_| Ok(expected_category));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.create(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.name, "Electronics");
        assert_eq!(response.description, Some("Electronic devices".to_string()));
    }

    #[tokio::test]
    async fn test_create_validation_error() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let request = CreateCategoryRequest {
            name: "".to_string(), // Invalid empty name
            description: None,
            parent_id: None,
            sort_order: 1,
        };

        mock_repo
            .expect_create()
            .return_once(|_| Err(CategoryError::InvalidName("カテゴリ名は必須です".to_string())));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.create(request).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CategoryError::InvalidName(_) => (),
            _ => panic!("Expected InvalidName error"),
        }
    }

    #[tokio::test]
    async fn test_update_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let existing_category = Category {
            id: "cat_123".to_string(),
            name: "Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let updated_category = Category {
            id: "cat_123".to_string(),
            name: "Updated Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        mock_repo
            .expect_find_by_id()
            .with(eq("cat_123"))
            .return_once(move |_| Some(existing_category));

        mock_repo
            .expect_update()
            .withf(|cat| cat.name == "Updated Electronics")
            .return_once(move |_| Ok(updated_category));

        let request = UpdateCategoryRequest {
            name: Some("Updated Electronics".to_string()),
            description: None,
            sort_order: None,
            is_active: None,
        };

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.update("cat_123", request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.name, "Updated Electronics");
    }

    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_delete()
            .with(eq("cat_123"))
            .return_once(|_| Ok(true));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.delete("cat_123").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_delete_has_children() {
        let mut mock_repo = MockCategoryRepository::new();
        
        mock_repo
            .expect_delete()
            .with(eq("cat_123"))
            .return_once(|_| Err(CategoryError::HasChildren("子カテゴリが存在するため削除できません".to_string())));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.delete("cat_123").await;

        assert!(result.is_err());
        match result.unwrap_err() {
            CategoryError::HasChildren(_) => (),
            _ => panic!("Expected HasChildren error"),
        }
    }

    #[tokio::test]
    async fn test_move_category_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let moved_category = Category {
            id: "cat_123".to_string(),
            name: "Electronics".to_string(),
            description: Some("Electronic devices".to_string()),
            parent_id: Some("cat_parent".to_string()),
            sort_order: 2,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        mock_repo
            .expect_move_category()
            .with(eq("cat_123"), eq(Some("cat_parent")), eq(2))
            .return_once(move |_, _, _| Ok(moved_category));

        let request = MoveCategoryRequest {
            parent_id: Some("cat_parent".to_string()),
            sort_order: 2,
        };

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.move_category("cat_123", request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.parent_id, Some("cat_parent".to_string()));
        assert_eq!(response.sort_order, 2);
    }

    #[tokio::test]
    async fn test_find_path_success() {
        let mut mock_repo = MockCategoryRepository::new();
        
        let path = CategoryPath::new(vec![
            "cat_root".to_string(),
            "cat_child".to_string(),
            "cat_grandchild".to_string(),
        ]);

        // Mock the path finding
        mock_repo
            .expect_find_path()
            .with(eq("cat_grandchild"))
            .return_once(move |_| Ok(path));

        // Mock the category finding for enriching path
        let root_category = Category {
            id: "cat_root".to_string(),
            name: "Root".to_string(),
            description: None,
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let child_category = Category {
            id: "cat_child".to_string(),
            name: "Child".to_string(),
            description: None,
            parent_id: Some("cat_root".to_string()),
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let grandchild_category = Category {
            id: "cat_grandchild".to_string(),
            name: "Grandchild".to_string(),
            description: None,
            parent_id: Some("cat_child".to_string()),
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        mock_repo
            .expect_find_by_id()
            .with(eq("cat_root"))
            .return_once(move |_| Some(root_category));

        mock_repo
            .expect_find_by_id()
            .with(eq("cat_child"))
            .return_once(move |_| Some(child_category));

        mock_repo
            .expect_find_by_id()
            .with(eq("cat_grandchild"))
            .return_once(move |_| Some(grandchild_category));

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.find_path("cat_grandchild").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.depth, 3);
        assert_eq!(response.path.len(), 3);
        assert_eq!(response.path[0].name, "Root");
        assert_eq!(response.path[1].name, "Child");
        assert_eq!(response.path[2].name, "Grandchild");
    }
}