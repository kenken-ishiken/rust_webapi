use std::collections::HashMap;
use std::sync::Mutex;
use sqlx::{PgPool, Row};
use async_trait::async_trait;
use domain::model::item::{Item, DeletionValidation, RelatedDataCount, DeletionLog, DeletionType};
use crate::app_domain::repository::item_repository::ItemRepository;
use crate::infrastructure::error::{AppError, AppResult};
use chrono::Utc;
use tracing::error;

pub struct InMemoryItemRepository {
    items: Mutex<HashMap<u64, Item>>,
}

impl InMemoryItemRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            items: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryItemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ItemRepository for InMemoryItemRepository {
    async fn find_all(&self) -> AppResult<Vec<Item>> {
        let items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(items.values()
            .filter(|item| !item.deleted)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: u64) -> AppResult<Option<Item>> {
        let items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(items.get(&id)
            .filter(|item| !item.deleted)
            .cloned())
    }

    async fn create(&self, item: Item) -> AppResult<Item> {
        let mut items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        items.insert(item.id, item.clone());
        Ok(item)
    }

    async fn update(&self, item: Item) -> AppResult<Item> {
        let mut items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        if let Some(existing) = items.get(&item.id) {
            if !existing.deleted {
                items.insert(item.id, item.clone());
                return Ok(item);
            }
        }
        Err(AppError::NotFound(format!("Item with id {} not found", item.id)))
    }

    async fn delete(&self, id: u64) -> AppResult<()> {
        // For backward compatibility, we'll make this perform a physical delete
        self.physical_delete(id).await
    }
    
    async fn logical_delete(&self, id: u64) -> AppResult<()> {
        let mut items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        if let Some(mut item) = items.get(&id).cloned() {
            if !item.deleted {
                item.deleted = true;
                item.deleted_at = Some(chrono::Utc::now());
                items.insert(id, item);
                return Ok(());
            }
        }
        Err(AppError::NotFound(format!("Item with id {} not found", id)))
    }
    
    async fn physical_delete(&self, id: u64) -> AppResult<()> {
        let mut items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        if items.remove(&id).is_some() {
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Item with id {} not found", id)))
        }
    }
    
    async fn restore(&self, id: u64) -> AppResult<()> {
        let mut items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        if let Some(mut item) = items.get(&id).cloned() {
            if item.deleted {
                item.deleted = false;
                item.deleted_at = None;
                items.insert(id, item);
                return Ok(());
            }
        }
        Err(AppError::NotFound(format!("Item with id {} not found or not deleted", id)))
    }
    
    async fn find_deleted(&self) -> AppResult<Vec<Item>> {
        let items = self.items.lock()
            .map_err(|_| AppError::InternalServerError("Failed to acquire lock".to_string()))?;
        Ok(items.values()
            .filter(|item| item.deleted)
            .cloned()
            .collect())
    }
    
    async fn validate_deletion(&self, _id: u64) -> AppResult<DeletionValidation> {
        // Simplified implementation for in-memory repository
        Ok(DeletionValidation {
            can_delete: true,
            related_data: RelatedDataCount {
                related_orders: 0,
                related_reviews: 0,
                related_categories: 0,
            },
        })
    }
    
    async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> AppResult<Vec<u64>> {
        let mut successful_ids = Vec::new();
        
        for id in ids {
            let result = if is_physical {
                self.physical_delete(id).await
            } else {
                self.logical_delete(id).await
            };
            
            if result.is_ok() {
                successful_ids.push(id);
            }
        }
        
        Ok(successful_ids)
    }
    
    async fn get_deletion_logs(&self, _item_id: Option<u64>) -> AppResult<Vec<DeletionLog>> {
        // In-memory implementation doesn't track deletion logs
        Ok(Vec::new())
    }
}

pub struct PostgresItemRepository {
    pool: PgPool,
}

impl PostgresItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // Helper method to log deletions
    async fn log_deletion(&self, item_id: u64, deletion_type: &str, deleted_by: &str) -> Result<(), sqlx::Error> {
        // Get the item name for logging
        let item_name = match self.find_by_id(item_id).await.ok().flatten() {
            Some(item) => item.name,
            None => {
                // Try to find in deleted items
                let deleted_items = self.find_deleted().await.unwrap_or_default();
                deleted_items.iter()
                    .find(|item| item.id == item_id)
                    .map(|item| item.name.clone())
                    .unwrap_or_else(|| format!("Unknown Item {}", item_id))
            }
        };
        
        let now = Utc::now();
        
        sqlx::query(
            "INSERT INTO deletion_logs (item_id, item_name, deletion_type, deleted_at, deleted_by) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(item_id as i64)
        .bind(&item_name)
        .bind(deletion_type)
        .bind(now)
        .bind(deleted_by)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    // テーブルを初期化するメソッド（テスト用）
    #[cfg(any(test, feature = "testing"))]
    #[allow(dead_code)]
    pub async fn init_table(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS items (
                id BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT,
                deleted BOOLEAN NOT NULL DEFAULT FALSE,
                deleted_at TIMESTAMP WITH TIME ZONE
            )"
        )
        .execute(&self.pool)
        .await
        .map(|_| ())?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS deletion_logs (
                id SERIAL PRIMARY KEY,
                item_id BIGINT NOT NULL,
                item_name VARCHAR(255) NOT NULL,
                deletion_type VARCHAR(20) NOT NULL,
                deleted_at TIMESTAMP WITH TIME ZONE NOT NULL,
                deleted_by VARCHAR(255) NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl ItemRepository for PostgresItemRepository {
    async fn find_all(&self) -> AppResult<Vec<Item>> {
        let rows = sqlx::query("SELECT id, name, description, deleted, deleted_at FROM items WHERE deleted = FALSE")
            .fetch_all(&self.pool)
            .await?;
            
        Ok(rows.iter()
            .map(|row| Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
                deleted: row.get("deleted"),
                deleted_at: row.get("deleted_at"),
            })
            .collect())
    }

    async fn find_by_id(&self, id: u64) -> AppResult<Option<Item>> {
        let row = sqlx::query("SELECT id, name, description, deleted, deleted_at FROM items WHERE id = $1 AND deleted = FALSE")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|row| Item {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            description: row.get("description"),
            deleted: row.get("deleted"),
            deleted_at: row.get("deleted_at"),
        }))
    }

    async fn create(&self, item: Item) -> AppResult<Item> {
        let row = sqlx::query(
                "INSERT INTO items (id, name, description, deleted, deleted_at) VALUES ($1, $2, $3, $4, $5) RETURNING id, name, description, deleted, deleted_at"
            )
            .bind(item.id as i64)
            .bind(&item.name)
            .bind(&item.description)
            .bind(item.deleted)
            .bind(item.deleted_at)
            .fetch_one(&self.pool)
            .await?;
            
        Ok(Item {
            id: row.get::<i64, _>("id") as u64,
            name: row.get("name"),
            description: row.get("description"),
            deleted: row.get("deleted"),
            deleted_at: row.get("deleted_at"),
        })
    }

    async fn update(&self, item: Item) -> AppResult<Item> {
        let row = sqlx::query(
                "UPDATE items SET name = $2, description = $3, deleted = $4, deleted_at = $5 WHERE id = $1 AND deleted = FALSE RETURNING id, name, description, deleted, deleted_at"
            )
            .bind(item.id as i64)
            .bind(&item.name)
            .bind(&item.description)
            .bind(item.deleted)
            .bind(item.deleted_at)
            .fetch_optional(&self.pool)
            .await?;
            
        match row {
            Some(row) => Ok(Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
                deleted: row.get("deleted"),
                deleted_at: row.get("deleted_at"),
            }),
            None => Err(AppError::NotFound(format!("Item with id {} not found", item.id))),
        }
    }

    async fn delete(&self, id: u64) -> AppResult<()> {
        // For backward compatibility, we'll make this method perform a physical delete
        self.physical_delete(id).await
    }
    
    async fn logical_delete(&self, id: u64) -> AppResult<()> {
        let now = Utc::now();
        let result = sqlx::query(
            "UPDATE items SET deleted = TRUE, deleted_at = $2 WHERE id = $1 AND deleted = FALSE"
        )
        .bind(id as i64)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Log the deletion
            if let Err(e) = self.log_deletion(id, "Logical", "system").await {
                error!("Failed to log logical deletion for item {}: {}", id, e);
            }
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Item with id {} not found", id)))
        }
    }
    
    async fn physical_delete(&self, id: u64) -> AppResult<()> {
        // Get the item name before deletion for logging
        let item = self.find_by_id(id).await?;
        
        let result = sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() > 0 {
            if item.is_some() {
                // Log the deletion if the item existed
                if let Err(e) = self.log_deletion(id, "Physical", "system").await {
                    error!("Failed to log physical deletion for item {}: {}", id, e);
                }
            }
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Item with id {} not found", id)))
        }
    }
    
    async fn restore(&self, id: u64) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE items SET deleted = FALSE, deleted_at = NULL WHERE id = $1 AND deleted = TRUE"
        )
        .bind(id as i64)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            // Log the restoration
            if let Err(e) = self.log_deletion(id, "Restore", "system").await {
                error!("Failed to log restoration for item {}: {}", id, e);
            }
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Item with id {} not found or not deleted", id)))
        }
    }
    
    async fn find_deleted(&self) -> AppResult<Vec<Item>> {
        let rows = sqlx::query("SELECT id, name, description, deleted, deleted_at FROM items WHERE deleted = TRUE")
            .fetch_all(&self.pool)
            .await?;
            
        Ok(rows.iter()
            .map(|row| Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
                deleted: row.get("deleted"),
                deleted_at: row.get("deleted_at"),
            })
            .collect())
    }
    
    async fn validate_deletion(&self, _id: u64) -> AppResult<DeletionValidation> {
        // This is a placeholder implementation. In a real application,
        // you would check for related entities like orders, reviews, etc.
        let related_data = RelatedDataCount {
            related_orders: 0,
            related_reviews: 0,
            related_categories: 0,
        };
        
        Ok(DeletionValidation {
            can_delete: true,
            related_data,
        })
    }
    
    async fn batch_delete(&self, ids: Vec<u64>, is_physical: bool) -> AppResult<Vec<u64>> {
        let mut successful_deletions = Vec::new();
        
        for id in ids {
            let result = if is_physical {
                self.physical_delete(id).await
            } else {
                self.logical_delete(id).await
            };
            
            if result.is_ok() {
                successful_deletions.push(id);
            }
        }
        
        Ok(successful_deletions)
    }
    
    async fn get_deletion_logs(&self, item_id: Option<u64>) -> AppResult<Vec<DeletionLog>> {
        let query = match item_id {
            Some(id) => {
                sqlx::query("SELECT id, item_id, item_name, deletion_type, deleted_at, deleted_by FROM deletion_logs WHERE item_id = $1 ORDER BY deleted_at DESC")
                    .bind(id as i64)
            },
            None => {
                sqlx::query("SELECT id, item_id, item_name, deletion_type, deleted_at, deleted_by FROM deletion_logs ORDER BY deleted_at DESC")
            }
        };
        
        let rows = query.fetch_all(&self.pool).await?;
        
        Ok(rows.iter()
            .map(|row| {
                let deletion_type_str: String = row.get("deletion_type");
                let deletion_type = match deletion_type_str.as_str() {
                    "Logical" => DeletionType::Logical,
                    "Physical" => DeletionType::Physical,
                    "Restore" => DeletionType::Restore,
                    _ => DeletionType::Logical, // Default
                };
                
                DeletionLog {
                    id: row.get::<i64, _>("id") as u64,
                    item_id: row.get::<i64, _>("item_id") as u64,
                    item_name: row.get("item_name"),
                    deletion_type,
                    deleted_at: row.get("deleted_at"),
                    deleted_by: row.get("deleted_by"),
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers_modules::postgres;

    async fn setup_postgres() -> PgPool {
        // PostgreSQLコンテナの起動（デフォルト設定で実行）
        let docker = testcontainers::clients::Cli::default();
        let container = docker.run(postgres::Postgres::default());

        // PostgreSQLへの接続情報の取得
        let host_port = container.get_host_port_ipv4(5432);

        // 接続プールの作成
        let conn_str = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        let mut retries = 0;
        loop {
            match sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(std::time::Duration::from_secs(5))
                .connect(&conn_str)
                .await
            {
                Ok(pool) => {
                    if sqlx::query("SELECT 1").execute(&pool).await.is_ok() {
                        break pool;
                    } else {
                        retries += 1;
                        eprintln!("Postgres ping failed, retrying... (attempt {})", retries);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                },
                Err(e) => panic!("Failed to connect to Postgres after retries: {}", e),
            }
        }
    }
    
    #[tokio::test]
    async fn test_postgres_crud_operations() {
        // PostgreSQLコンテナの初期化
        let pool = setup_postgres().await;
        
        // リポジトリの作成とテーブルの初期化
        let repo = PostgresItemRepository::new(pool.clone());
        
        // Create the table directly
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS items (
                id BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT
            )"
        )
        .execute(&pool)
        .await
        .expect("Failed to create items table");
        
        // テストデータ
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };
        
        // 1. アイテム作成のテスト
        let created_item = repo.create(item.clone()).await.unwrap();
        assert_eq!(created_item.id, item.id);
        assert_eq!(created_item.name, item.name);
        assert_eq!(created_item.description, item.description);
        
        // 2. 単一アイテム取得のテスト
        let found_item = repo.find_by_id(1).await.unwrap();
        assert!(found_item.is_some());
        let found_item = found_item.unwrap();
        assert_eq!(found_item.id, item.id);
        assert_eq!(found_item.name, item.name);
        assert_eq!(found_item.description, item.description);
        
        // 3. 存在しないアイテム取得のテスト
        let not_found = repo.find_by_id(999).await.unwrap();
        assert!(not_found.is_none());
        
        // 4. 全アイテム取得のテスト
        let all_items = repo.find_all().await.unwrap();
        assert_eq!(all_items.len(), 1);
        assert_eq!(all_items[0].id, item.id);
        
        // 5. アイテム更新のテスト
        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
            deleted: false,
            deleted_at: None,
        };
        
        let result = repo.update(updated_item.clone()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.name, "Updated Item");
        assert_eq!(result.description, Some("Updated Description".to_string()));
        
        // 6. アイテム削除のテスト
        let deleted = repo.delete(1).await;
        assert!(deleted.is_ok());
        
        // 削除後の検証
        let all_items_after_delete = repo.find_all().await.unwrap();
        assert_eq!(all_items_after_delete.len(), 0);
        
        // 7. 存在しないアイテムの削除テスト
        let not_deleted = repo.delete(999).await;
        assert!(not_deleted.is_err());
    }
}