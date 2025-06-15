use std::sync::Arc;
use tracing::{info, error};
use futures::stream::{self, StreamExt};
use uuid::Uuid;

use crate::app_domain::model::category::{Category, CategoryError};
#[cfg(test)]
use crate::app_domain::model::category::CategoryPath;
use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::application::dto::category_dto::{
    CreateCategoryRequest, UpdateCategoryRequest, MoveCategoryRequest,
    CategoryResponse, CategoryListResponse, CategoryPathResponse,
    CategoriesResponse, CategoryTreesResponse, CategoryPathItem
};
use crate::infrastructure::metrics::{increment_success_counter, increment_error_counter};

/// カテゴリ関連のユースケースを提供するサービス層。
///
/// Repository へのアクセスを仲介し、DTO との相互変換や
/// 監査・メトリクス・ロギングなどクロスカットな処理を担う。
pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    /// 新しい `CategoryService` インスタンスを生成します。
    ///
    /// # 引数
    /// * `repository` - `CategoryRepository` の実装をラップした `Arc`。
    pub fn new(repository: Arc<dyn CategoryRepository>) -> Self {
        Self { repository }
    }

    /// 全カテゴリを取得します。
    ///
    /// `include_inactive` が `true` の場合、非アクティブなカテゴリも含めて取得します。
    pub async fn find_all(&self, include_inactive: bool) -> CategoriesResponse {
        let categories = self.repository.find_all(include_inactive).await;
        
        let category_list = self.build_category_list_response(&categories).await;

        increment_success_counter("category", "find_all");
        info!("Fetched {} categories", categories.len());
        
        let total = category_list.len();
        CategoriesResponse {
            categories: category_list,
            total,
        }
    }

    /// 複数の `Category` を `CategoryListResponse` に変換します。
    ///
    /// 子カテゴリ数の取得を *並列* に行い、N+1 問題を軽減します。
    /// 同時実行数を制限してDB接続プールの枯渇を防ぎます。
    async fn build_category_list_response(&self, categories: &[Category]) -> Vec<CategoryListResponse> {
        // 同時クエリを最大 8 件に制限してDB接続プール枯渇を防ぐ
        let tasks_stream = stream::iter(categories.iter().cloned()).map(|category_clone| {
            let repo = Arc::clone(&self.repository);
            async move {
                let children_count = repo.count_children(&category_clone.id).await;
                CategoryListResponse {
                    id: category_clone.id,
                    name: category_clone.name,
                    description: category_clone.description,
                    parent_id: category_clone.parent_id,
                    sort_order: category_clone.sort_order,
                    is_active: category_clone.is_active,
                    children_count,
                    created_at: category_clone.created_at,
                    updated_at: category_clone.updated_at,
                }
            }
        })
        .buffer_unordered(8);

        tasks_stream.collect::<Vec<_>>().await
    }

    /// ID でカテゴリを取得します。
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

    /// 指定された親 ID 配下のカテゴリを取得します。
    ///
    /// `parent_id` が `None` の場合はルートカテゴリを取得します。
    pub async fn find_by_parent_id(&self, parent_id: Option<String>, include_inactive: bool) -> CategoriesResponse {
        let categories = self
            .repository
            .find_by_parent_id(parent_id.clone(), include_inactive)
            .await;

        let category_list = self.build_category_list_response(&categories).await;

        increment_success_counter("category", "find_by_parent");
        info!("Fetched {} categories for parent {:?}", categories.len(), parent_id);

        let total = category_list.len();
        CategoriesResponse {
            categories: category_list,
            total,
        }
    }

    /// 指定カテゴリの子カテゴリ一覧を取得します。
    pub async fn find_children(&self, id: &str, include_inactive: bool) -> CategoriesResponse {
        self.find_by_parent_id(Some(id.to_string()), include_inactive).await
    }

    /// 指定カテゴリからルートまでのパス情報を取得します。
    ///
    /// 戻り値の `depth` はルートからの階層深さを示します。
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

    /// 全カテゴリを木構造で取得します。
    pub async fn find_tree(&self, include_inactive: bool) -> CategoryTreesResponse {
        let trees = self.repository.find_tree(include_inactive).await;
        
        increment_success_counter("category", "find_tree");
        info!("Fetched category tree with {} root categories", trees.len());
        
        CategoryTreesResponse {
            tree: trees.into_iter().map(|t| t.into()).collect(),
        }
    }

    /// 新しいカテゴリを作成します。
    ///
    /// # 失敗時
    /// * `CategoryError::InvalidName` - 名前が無効な場合 など
    pub async fn create(&self, req: CreateCategoryRequest) -> Result<CategoryResponse, CategoryError> {
        // 一意な ID を UUID v4 で生成
        let id = format!("cat_{}", Uuid::new_v4());
        
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

    /// 既存カテゴリを更新します。
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

    /// カテゴリを削除します。
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

    /// 親カテゴリ変更および並び順変更を行います。
    pub async fn move_category(&self, id: &str, req: MoveCategoryRequest) -> Result<CategoryResponse, CategoryError> {
        let parent_id = req.parent_id.clone();
        match self.repository.move_category(id, req.parent_id, req.sort_order).await {
            Ok(moved_category) => {
                increment_success_counter("category", "move");
                info!("Moved category {} to parent {:?} with sort order {}", id, parent_id, req.sort_order);
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
            .with(eq("cat_123"), eq(Some("cat_parent".to_string())), eq(2))
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

    #[tokio::test]
    async fn test_build_category_list_response() {
        let mut mock_repo = MockCategoryRepository::new();
        
        // Create test categories
        let category1 = Category {
            id: "cat_1".to_string(),
            name: "Category 1".to_string(),
            description: Some("Description 1".to_string()),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let category2 = Category {
            id: "cat_2".to_string(),
            name: "Category 2".to_string(),
            description: Some("Description 2".to_string()),
            parent_id: None,
            sort_order: 2,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Mock count_children calls - verify parallel execution works
        mock_repo
            .expect_count_children()
            .with(eq("cat_1"))
            .return_once(|_| 2);

        mock_repo
            .expect_count_children()
            .with(eq("cat_2"))
            .return_once(|_| 0);

        let service = CategoryService::new(Arc::new(mock_repo));
        let categories = vec![category1.clone(), category2.clone()];
        
        let result = service.build_category_list_response(&categories).await;

        assert_eq!(result.len(), 2);
        
        // Verify first category
        let cat1_response = result.iter().find(|r| r.id == "cat_1").unwrap();
        assert_eq!(cat1_response.name, "Category 1");
        assert_eq!(cat1_response.children_count, 2);
        
        // Verify second category
        let cat2_response = result.iter().find(|r| r.id == "cat_2").unwrap();
        assert_eq!(cat2_response.name, "Category 2");
        assert_eq!(cat2_response.children_count, 0);
    }

    #[tokio::test]
    async fn test_build_category_list_response_empty() {
        let mock_repo = MockCategoryRepository::new();
        let service = CategoryService::new(Arc::new(mock_repo));
        
        let result = service.build_category_list_response(&[]).await;
        
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_total_count_uses_category_list_length() {
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
            .with(eq(true))
            .return_once(move |_| categories);

        mock_repo
            .expect_count_children()
            .with(eq("cat_1"))
            .return_once(|_| 1);

        mock_repo
            .expect_count_children()
            .with(eq("cat_2"))
            .return_once(|_| 0);

        let service = CategoryService::new(Arc::new(mock_repo));
        let result = service.find_all(true).await;

        // Verify that total matches the category_list length (2)
        assert_eq!(result.total, 2);
        assert_eq!(result.categories.len(), 2);
    }
}