use chrono::Utc;
use domain::model::item::Item;
use mockall::predicate::*;
use rust_webapi::app_domain::model::category::{Category, CategoryError};
use rust_webapi::app_domain::repository::category_repository::MockCategoryRepository;
use rust_webapi::app_domain::repository::item_repository::MockItemRepository;
use rust_webapi::infrastructure::error::{AppError, AppResult};

/// MockRepositoryBuilderパターンを提供するトレイト
///
/// このトレイトにより、各リポジトリのモック設定を統一的に行える
#[allow(dead_code)]
pub trait MockBuilder<T> {
    /// 新しいモックビルダーを作成
    fn new() -> Self;

    /// モックの設定を完了し、実際のモックインスタンスを返す
    fn build(self) -> T;
}

/// ItemRepository用のMockBuilder
///
/// # 使用例
/// ```
/// let mock_repo = ItemMockBuilder::new()
///     .with_find_all_returning(vec![item1, item2])
///     .with_find_by_id_returning(1, Some(item1))
///     .with_create_success()
///     .build();
/// ```
pub struct ItemMockBuilder {
    mock: MockItemRepository,
}

impl MockBuilder<MockItemRepository> for ItemMockBuilder {
    fn new() -> Self {
        Self {
            mock: MockItemRepository::new(),
        }
    }

    fn build(self) -> MockItemRepository {
        self.mock
    }
}

#[allow(dead_code)]
impl ItemMockBuilder {
    /// find_all()の戻り値を設定
    pub fn with_find_all_returning(mut self, items: Vec<Item>) -> Self {
        self.mock
            .expect_find_all()
            .times(1)
            .returning(move || Ok(items.clone()));
        self
    }

    /// find_all()が空の結果を返すよう設定
    pub fn with_find_all_empty(mut self) -> Self {
        self.mock
            .expect_find_all()
            .times(1)
            .returning(|| Ok(vec![]));
        self
    }

    /// find_by_id()の戻り値を設定
    pub fn with_find_by_id_returning(mut self, id: u64, item: Option<Item>) -> Self {
        self.mock
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(item.clone()));
        self
    }

    /// find_by_id()が見つからない場合を設定
    pub fn with_find_by_id_not_found(mut self, id: u64) -> Self {
        self.mock
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));
        self
    }

    /// create()が成功する場合を設定
    pub fn with_create_success(mut self) -> Self {
        self.mock.expect_create().times(1).returning(Ok);
        self
    }

    /// create()で特定の条件をチェックして成功する場合を設定
    pub fn with_create_success_when<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&Item) -> bool + Send + Sync + 'static,
    {
        self.mock
            .expect_create()
            .with(function(predicate))
            .times(1)
            .returning(Ok);
        self
    }

    /// update()が成功する場合を設定
    pub fn with_update_success(mut self) -> Self {
        self.mock.expect_update().times(1).returning(Ok);
        self
    }

    /// update()が失敗する場合を設定
    pub fn with_update_not_found(mut self, id: u64) -> Self {
        self.mock
            .expect_update()
            .with(function(move |item: &Item| item.id == id))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Item not found".to_string())));
        self
    }

    /// logical_delete()が成功する場合を設定
    pub fn with_logical_delete_success(mut self, id: u64) -> Self {
        self.mock
            .expect_logical_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(()));
        self
    }

    /// logical_delete()が失敗する場合を設定
    pub fn with_logical_delete_not_found(mut self, id: u64) -> Self {
        self.mock
            .expect_logical_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Err(AppError::NotFound("Item not found".to_string())));
        self
    }

    /// 複数回の呼び出しを期待する設定
    pub fn with_find_all_times(mut self, times: usize, items: Vec<Item>) -> Self {
        self.mock
            .expect_find_all()
            .times(times)
            .returning(move || Ok(items.clone()));
        self
    }

    /// 任意の条件でfind_by_id()を設定
    pub fn with_find_by_id_any_returning(mut self, item: Option<Item>) -> Self {
        self.mock
            .expect_find_by_id()
            .with(always())
            .times(1)
            .returning(move |_| Ok(item.clone()));
        self
    }
}

/// Category用のMockBuilder
pub struct CategoryMockBuilder {
    mock: MockCategoryRepository,
}

impl MockBuilder<MockCategoryRepository> for CategoryMockBuilder {
    fn new() -> Self {
        Self {
            mock: MockCategoryRepository::new(),
        }
    }

    fn build(self) -> MockCategoryRepository {
        self.mock
    }
}

#[allow(dead_code)]
impl CategoryMockBuilder {
    /// find_all()の戻り値を設定
    pub fn with_find_all_returning(
        mut self,
        include_inactive: bool,
        categories: Vec<Category>,
    ) -> Self {
        self.mock
            .expect_find_all()
            .with(eq(include_inactive))
            .times(1)
            .returning(move |_| categories.clone());
        self
    }

    /// find_by_id()の戻り値を設定
    pub fn with_find_by_id_returning(mut self, id: String, category: Option<Category>) -> Self {
        self.mock
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| category.clone());
        self
    }

    /// create()が成功する場合を設定
    pub fn with_create_success(mut self, expected_category: Category) -> Self {
        self.mock
            .expect_create()
            .times(1)
            .returning(move |_| Ok(expected_category.clone()));
        self
    }

    /// create()が失敗する場合を設定
    pub fn with_create_error(mut self, error: CategoryError) -> Self {
        self.mock
            .expect_create()
            .times(1)
            .returning(move |_| Err(error.clone()));
        self
    }
}

/// テストデータファクトリー
#[allow(dead_code)]
pub struct TestDataFactory;

#[allow(dead_code)]
impl TestDataFactory {
    /// 標準的なItemを作成
    pub fn create_item(id: u64, name: &str) -> Item {
        Item {
            id,
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            deleted: false,
            deleted_at: None,
        }
    }

    /// 削除済みのItemを作成
    pub fn create_deleted_item(id: u64, name: &str) -> Item {
        Item {
            id,
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            deleted: true,
            deleted_at: Some(Utc::now()),
        }
    }

    /// 標準的なCategoryを作成
    pub fn create_category(id: &str, name: &str) -> Category {
        Category {
            id: id.to_string(),
            name: name.to_string(),
            description: Some(format!("Description for {}", name)),
            parent_id: None,
            sort_order: 1,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 複数のItemを作成
    pub fn create_items(count: usize) -> Vec<Item> {
        (1..=count)
            .map(|i| Self::create_item(i as u64, &format!("Item {}", i)))
            .collect()
    }

    /// 複数のCategoryを作成
    pub fn create_categories(count: usize) -> Vec<Category> {
        (1..=count)
            .map(|i| Self::create_category(&format!("cat_{}", i), &format!("Category {}", i)))
            .collect()
    }
}

/// テストアサーション用のヘルパー
#[allow(dead_code)]
pub struct TestAssertions;

#[allow(dead_code)]
impl TestAssertions {
    /// Item同士の等価性をチェック（deleted_atを除く）
    pub fn assert_item_eq(actual: &Item, expected: &Item) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.description, expected.description);
        assert_eq!(actual.deleted, expected.deleted);
    }

    /// Category同士の等価性をチェック（created_at/updated_atを除く）
    pub fn assert_category_eq(actual: &Category, expected: &Category) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.description, expected.description);
        assert_eq!(actual.parent_id, expected.parent_id);
        assert_eq!(actual.sort_order, expected.sort_order);
        assert_eq!(actual.is_active, expected.is_active);
    }

    /// AppErrorの種類をチェック
    pub fn assert_app_error_not_found(result: AppResult<()>) {
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound(_) => (),
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    /// CategoryErrorの種類をチェック
    pub fn assert_category_error_invalid_name(result: Result<Category, CategoryError>) {
        assert!(result.is_err());
        match result.unwrap_err() {
            CategoryError::InvalidName(_) => (),
            other => panic!("Expected InvalidName error, got: {:?}", other),
        }
    }
}
