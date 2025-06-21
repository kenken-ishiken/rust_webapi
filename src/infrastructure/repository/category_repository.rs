use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use tracing::error;

use crate::app_domain::model::category::{Category, CategoryError, CategoryPath, CategoryTree};
use crate::app_domain::repository::category_repository::CategoryRepository;

pub struct PostgresCategoryRepository {
    pool: PgPool,
}

impl PostgresCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[cfg(any(test, feature = "testing"))]
    pub async fn init_table(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS categories (
                id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                description TEXT,
                parent_id VARCHAR(255),
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE,
                CONSTRAINT check_sort_order_non_negative CHECK (sort_order >= 0),
                CONSTRAINT check_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
                UNIQUE(name, parent_id)
            )",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }

    fn row_to_category(row: &sqlx::postgres::PgRow) -> Category {
        Category {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            parent_id: row.get("parent_id"),
            sort_order: row.get("sort_order"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn build_category_tree_recursive<'a>(
        &'a self,
        categories: &'a [Category],
        parent_id: Option<&'a str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<CategoryTree>> + Send + 'a>> {
        Box::pin(async move {
            let mut tree = Vec::new();

            for category in categories {
                if category.parent_id.as_deref() == parent_id {
                    let children = self
                        .build_category_tree_recursive(categories, Some(&category.id))
                        .await;

                    tree.push(CategoryTree {
                        id: category.id.clone(),
                        name: category.name.clone(),
                        description: category.description.clone(),
                        sort_order: category.sort_order,
                        is_active: category.is_active,
                        children,
                    });
                }
            }

            tree.sort_by_key(|t| t.sort_order);
            tree
        })
    }

    async fn get_category_ancestors(
        &self,
        category_id: &str,
    ) -> Result<Vec<String>, CategoryError> {
        let mut ancestors = Vec::new();
        let mut current_id = Some(category_id.to_string());

        while let Some(id) = current_id {
            if let Some(category) = self.find_by_id(&id).await {
                ancestors.push(category.id);
                current_id = category.parent_id;
            } else {
                break;
            }
        }

        ancestors.reverse();
        Ok(ancestors)
    }
}

#[async_trait]
impl CategoryRepository for PostgresCategoryRepository {
    async fn find_all(&self, include_inactive: bool) -> Vec<Category> {
        let query = if include_inactive {
            "SELECT id, name, description, parent_id, sort_order, is_active, created_at, updated_at 
             FROM categories 
             ORDER BY sort_order, name"
        } else {
            "SELECT id, name, description, parent_id, sort_order, is_active, created_at, updated_at 
             FROM categories 
             WHERE is_active = true 
             ORDER BY sort_order, name"
        };

        match sqlx::query(query).fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(Self::row_to_category).collect(),
            Err(e) => {
                error!("Error fetching all categories: {}", e);
                vec![]
            }
        }
    }

    async fn find_by_id(&self, id: &str) -> Option<Category> {
        let query = "SELECT id, name, description, parent_id, sort_order, is_active, created_at, updated_at 
                     FROM categories 
                     WHERE id = $1";

        match sqlx::query(query).bind(id).fetch_optional(&self.pool).await {
            Ok(Some(row)) => Some(Self::row_to_category(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error finding category by id {}: {}", id, e);
                None
            }
        }
    }

    async fn find_by_parent_id(
        &self,
        parent_id: Option<String>,
        include_inactive: bool,
    ) -> Vec<Category> {
        let query = if include_inactive {
            "SELECT id, name, description, parent_id, sort_order, is_active, created_at, updated_at 
             FROM categories 
             WHERE ($1::varchar IS NULL AND parent_id IS NULL) OR parent_id = $1
             ORDER BY sort_order, name"
        } else {
            "SELECT id, name, description, parent_id, sort_order, is_active, created_at, updated_at 
             FROM categories 
             WHERE (($1::varchar IS NULL AND parent_id IS NULL) OR parent_id = $1) AND is_active = true
             ORDER BY sort_order, name"
        };

        let parent_id_for_log = parent_id.clone();
        match sqlx::query(query)
            .bind(parent_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(Self::row_to_category).collect(),
            Err(e) => {
                error!(
                    "Error finding categories by parent_id {:?}: {}",
                    parent_id_for_log, e
                );
                vec![]
            }
        }
    }

    // async fn find_children(&self, id: &str, include_inactive: bool) -> Vec<Category> {
    //     self.find_by_parent_id(Some(id.to_string()), include_inactive).await
    // }

    async fn find_path(&self, id: &str) -> Result<CategoryPath, CategoryError> {
        match self.get_category_ancestors(id).await {
            Ok(ancestors) => Ok(CategoryPath::new(ancestors)),
            Err(e) => Err(e),
        }
    }

    async fn find_tree(&self, include_inactive: bool) -> Vec<CategoryTree> {
        let categories = self.find_all(include_inactive).await;
        self.build_category_tree_recursive(&categories, None).await
    }

    async fn exists_by_name_and_parent(
        &self,
        name: &str,
        parent_id: Option<String>,
        exclude_id: Option<String>,
    ) -> bool {
        let query = if exclude_id.is_some() {
            "SELECT COUNT(*) as count 
             FROM categories 
             WHERE name = $1 AND 
                   (($2::varchar IS NULL AND parent_id IS NULL) OR parent_id = $2) AND 
                   id != $3"
        } else {
            "SELECT COUNT(*) as count 
             FROM categories 
             WHERE name = $1 AND 
                   (($2::varchar IS NULL AND parent_id IS NULL) OR parent_id = $2)"
        };

        let result = if let Some(exclude) = exclude_id {
            sqlx::query(query)
                .bind(name)
                .bind(parent_id)
                .bind(exclude)
                .fetch_one(&self.pool)
                .await
        } else {
            sqlx::query(
                "SELECT COUNT(*) as count
                         FROM categories 
                         WHERE name = $1 AND 
                               (($2::varchar IS NULL AND parent_id IS NULL) OR parent_id = $2)",
            )
            .bind(name)
            .bind(parent_id)
            .fetch_one(&self.pool)
            .await
        };

        match result {
            Ok(row) => {
                let count: i64 = row.get("count");
                count > 0
            }
            Err(e) => {
                error!("Error checking name duplicate: {}", e);
                false
            }
        }
    }

    async fn create(&self, category: Category) -> Result<Category, CategoryError> {
        // Validate category
        category.validate()?;

        // Check for duplicate name in same parent
        if self
            .exists_by_name_and_parent(&category.name, category.parent_id.clone(), None)
            .await
        {
            return Err(CategoryError::NameDuplicate(
                "同一階層内に同じ名前のカテゴリが既に存在します".to_string(),
            ));
        }

        // Validate depth if has parent
        if category.parent_id.is_some() {
            self.validate_depth(category.parent_id.clone()).await?;
        }

        let query = "INSERT INTO categories (id, name, description, parent_id, sort_order, is_active, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                     RETURNING id, name, description, parent_id, sort_order, is_active, created_at, updated_at";

        match sqlx::query(query)
            .bind(&category.id)
            .bind(&category.name)
            .bind(&category.description)
            .bind(&category.parent_id)
            .bind(category.sort_order)
            .bind(category.is_active)
            .bind(category.created_at)
            .bind(category.updated_at)
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => Ok(Self::row_to_category(&row)),
            Err(e) => {
                error!("Error creating category: {}", e);
                Err(CategoryError::NotFound(format!(
                    "カテゴリの作成に失敗しました: {}",
                    e
                )))
            }
        }
    }

    async fn update(&self, category: Category) -> Result<Category, CategoryError> {
        // Validate category
        category.validate()?;

        // Check for duplicate name in same parent (excluding current category)
        if self
            .exists_by_name_and_parent(
                &category.name,
                category.parent_id.clone(),
                Some(category.id.clone()),
            )
            .await
        {
            return Err(CategoryError::NameDuplicate(
                "同一階層内に同じ名前のカテゴリが既に存在します".to_string(),
            ));
        }

        let query = "UPDATE categories 
                     SET name = $2, description = $3, parent_id = $4, sort_order = $5, is_active = $6, updated_at = $7
                     WHERE id = $1
                     RETURNING id, name, description, parent_id, sort_order, is_active, created_at, updated_at";

        match sqlx::query(query)
            .bind(&category.id)
            .bind(&category.name)
            .bind(&category.description)
            .bind(&category.parent_id)
            .bind(category.sort_order)
            .bind(category.is_active)
            .bind(Utc::now())
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Ok(Self::row_to_category(&row)),
            Ok(None) => Err(CategoryError::NotFound(
                "カテゴリが見つかりません".to_string(),
            )),
            Err(e) => {
                error!("Error updating category {}: {}", category.id, e);
                Err(CategoryError::NotFound(format!(
                    "カテゴリの更新に失敗しました: {}",
                    e
                )))
            }
        }
    }

    async fn delete(&self, id: &str) -> Result<bool, CategoryError> {
        // Check if category has children
        let children_count = self.count_children(id).await;
        if children_count > 0 {
            return Err(CategoryError::HasChildren(
                "子カテゴリが存在するため削除できません".to_string(),
            ));
        }

        // Check if category has products (for now, we'll skip this check as we don't have products yet)
        // let products_count = self.count_products(id).await;
        // if products_count > 0 {
        //     return Err(CategoryError::HasProducts(
        //         "商品が紐づいているため削除できません".to_string(),
        //     ));
        // }

        let query = "DELETE FROM categories WHERE id = $1";

        match sqlx::query(query).bind(id).execute(&self.pool).await {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                error!("Error deleting category {}: {}", id, e);
                Err(CategoryError::NotFound(format!(
                    "カテゴリの削除に失敗しました: {}",
                    e
                )))
            }
        }
    }

    async fn move_category(
        &self,
        id: &str,
        new_parent_id: Option<String>,
        new_sort_order: i32,
    ) -> Result<Category, CategoryError> {
        // Validate circular reference
        self.validate_circular_reference(id, new_parent_id.clone())
            .await?;

        // Validate depth
        if new_parent_id.is_some() {
            self.validate_depth(new_parent_id.clone()).await?;
        }

        // Get current category
        let mut category = self
            .find_by_id(id)
            .await
            .ok_or_else(|| CategoryError::NotFound("カテゴリが見つかりません".to_string()))?;

        // Update parent and sort order
        category.parent_id = new_parent_id.map(|s| s.to_string());
        category.sort_order = new_sort_order;
        category.updated_at = Utc::now();

        // Update in database
        self.update(category).await
    }

    async fn count_children(&self, id: &str) -> i64 {
        let query = "SELECT COUNT(*) as count FROM categories WHERE parent_id = $1";

        match sqlx::query(query).bind(id).fetch_one(&self.pool).await {
            Ok(row) => row.get("count"),
            Err(e) => {
                error!("Error counting children for category {}: {}", id, e);
                0
            }
        }
    }

    // async fn count_products(&self, _id: &str) -> i64 {
    //     // For now, return 0 as we don't have products table linked yet
    //     // In the future, this would count products linked to this category
    //     0
    // }

    async fn validate_depth(&self, parent_id: Option<String>) -> Result<(), CategoryError> {
        if let Some(parent_id) = parent_id {
            match self.find_path(&parent_id).await {
                Ok(path) => {
                    if path.depth >= 5 {
                        return Err(CategoryError::MaxDepthExceeded(
                            "最大階層数(5階層)を超過しています".to_string(),
                        ));
                    }
                }
                Err(_) => {
                    return Err(CategoryError::NotFound(
                        "親カテゴリが見つかりません".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    async fn validate_circular_reference(
        &self,
        id: &str,
        new_parent_id: Option<String>,
    ) -> Result<(), CategoryError> {
        if let Some(new_parent_id) = new_parent_id {
            // Cannot set self as parent
            if id == new_parent_id {
                return Err(CategoryError::CircularReference(
                    "自分自身を親カテゴリに設定することはできません".to_string(),
                ));
            }

            // Check if new_parent_id is a descendant of current category
            match self.find_path(&new_parent_id).await {
                Ok(path) => {
                    if path.contains(id) {
                        return Err(CategoryError::CircularReference(
                            "循環参照が発生するため、この操作は実行できません".to_string(),
                        ));
                    }
                }
                Err(_) => {
                    return Err(CategoryError::NotFound(
                        "親カテゴリが見つかりません".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{clients::Cli, Container};
    use testcontainers_modules::postgres::{self, Postgres};

    async fn setup_postgres() -> (PgPool, Container<'static, Postgres>) {
        let docker = Box::leak(Box::new(Cli::default()));
        let container = docker.run(postgres::Postgres::default());
        let host_port = container.get_host_port_ipv4(5432);

        let conn_str = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let mut retries = 0;
        let max_retries = 10;
        let pool = loop {
            if retries >= max_retries {
                panic!(
                    "Failed to connect to Postgres after {} retries",
                    max_retries
                );
            }

            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(&conn_str)
                .await
            {
                Ok(pool) => {
                    // simple ping query
                    if sqlx::query("SELECT 1").execute(&pool).await.is_ok() {
                        break pool;
                    } else {
                        retries += 1;
                        eprintln!("Postgres ping failed, retrying... (attempt {})", retries);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    retries += 1;
                    eprintln!(
                        "Failed to connect to Postgres: {}, retrying... (attempt {})",
                        e, retries
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        };

        (pool, container)
    }

    #[tokio::test]
    async fn test_postgres_category_crud_operations() {
        let (pool, _container) = setup_postgres().await;
        let repo = PostgresCategoryRepository::new(pool.clone());

        // Create the table
        repo.init_table()
            .await
            .expect("Failed to create categories table");

        // Test data
        let category = Category::new(
            "cat_123".to_string(),
            "Electronics".to_string(),
            Some("Electronic devices".to_string()),
            None,
            1,
        );

        // 1. Create category test
        let created_category = repo
            .create(category.clone())
            .await
            .expect("Failed to create category");
        assert_eq!(created_category.id, category.id);
        assert_eq!(created_category.name, category.name);
        assert_eq!(created_category.description, category.description);

        // 2. Find by id test
        let found_category = repo.find_by_id("cat_123").await;
        assert!(found_category.is_some());
        let found_category = found_category.unwrap();
        assert_eq!(found_category.id, category.id);
        assert_eq!(found_category.name, category.name);

        // 3. Find all test
        let all_categories = repo.find_all(true).await;
        assert_eq!(all_categories.len(), 1);
        assert_eq!(all_categories[0].id, category.id);

        // 4. Create child category
        let child_category = Category::new(
            "cat_456".to_string(),
            "Smartphones".to_string(),
            Some("Smart devices".to_string()),
            Some("cat_123".to_string()),
            1,
        );

        let created_child = repo
            .create(child_category.clone())
            .await
            .expect("Failed to create child category");
        assert_eq!(created_child.parent_id, Some("cat_123".to_string()));

        // 5. Find children test
        // let children = repo.find_children("cat_123", true).await;
        // assert_eq!(children.len(), 1);
        // assert_eq!(children[0].id, "cat_456");

        // 6. Find path test
        let path = repo.find_path("cat_456").await.expect("Failed to get path");
        assert_eq!(path.depth, 2);
        assert!(path.contains("cat_123"));
        assert!(path.contains("cat_456"));

        // 7. Update category test
        let mut updated_category = found_category.clone();
        updated_category
            .update_name("Updated Electronics".to_string())
            .expect("Failed to update name");

        let result = repo
            .update(updated_category)
            .await
            .expect("Failed to update category");
        assert_eq!(result.name, "Updated Electronics");

        // 8. Delete child first (cannot delete parent with children)
        let child_deleted = repo
            .delete("cat_456")
            .await
            .expect("Failed to delete child");
        assert!(child_deleted);

        // 9. Delete parent
        let parent_deleted = repo
            .delete("cat_123")
            .await
            .expect("Failed to delete parent");
        assert!(parent_deleted);

        // 10. Verify deletion
        let all_categories_after_delete = repo.find_all(true).await;
        assert_eq!(all_categories_after_delete.len(), 0);
    }

    #[tokio::test]
    async fn test_postgres_category_validation() {
        let (pool, _container) = setup_postgres().await;
        let repo = PostgresCategoryRepository::new(pool.clone());

        repo.init_table()
            .await
            .expect("Failed to create categories table");

        // Test duplicate name validation
        let category1 = Category::new(
            "cat_1".to_string(),
            "Electronics".to_string(),
            None,
            None,
            1,
        );

        let category2 = Category::new(
            "cat_2".to_string(),
            "Electronics".to_string(), // Same name, same parent (None)
            None,
            None,
            2,
        );

        // Create first category
        repo.create(category1)
            .await
            .expect("Failed to create first category");

        // Try to create second category with same name - should fail
        let result = repo.create(category2).await;
        assert!(result.is_err());
        match result {
            Err(CategoryError::NameDuplicate(_)) => (),
            _ => panic!("Expected NameDuplicate error"),
        }
    }
}
