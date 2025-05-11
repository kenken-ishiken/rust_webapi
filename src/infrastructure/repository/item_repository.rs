use std::collections::HashMap;
use std::sync::Mutex;
use sqlx::{PgPool, Row};
use async_trait::async_trait;
use domain::model::item::Item;
use domain::repository::item_repository::ItemRepository;

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

#[async_trait]
impl ItemRepository for InMemoryItemRepository {
    async fn find_all(&self) -> Vec<Item> {
        let items = self.items.lock().unwrap();
        items.values().cloned().collect()
    }

    async fn find_by_id(&self, id: u64) -> Option<Item> {
        let items = self.items.lock().unwrap();
        items.get(&id).cloned()
    }

    async fn create(&self, item: Item) -> Item {
        let mut items = self.items.lock().unwrap();
        items.insert(item.id, item.clone());
        item
    }

    async fn update(&self, item: Item) -> Option<Item> {
        let mut items = self.items.lock().unwrap();
        if items.contains_key(&item.id) {
            items.insert(item.id, item.clone());
            Some(item)
        } else {
            None
        }
    }

    async fn delete(&self, id: u64) -> bool {
        let mut items = self.items.lock().unwrap();
        items.remove(&id).is_some()
    }
}

pub struct PostgresItemRepository {
    pool: PgPool,
}

impl PostgresItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // テーブルを初期化するメソッド（テスト用）
    #[cfg(test)]
    pub async fn init_table(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS items (
                id BIGINT PRIMARY KEY,
                name VARCHAR(255) NOT NULL,
                description TEXT
            )"
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl ItemRepository for PostgresItemRepository {
    async fn find_all(&self) -> Vec<Item> {
        let result = sqlx::query("SELECT id, name, description FROM items")
            .fetch_all(&self.pool)
            .await;
            
        match result {
            Ok(rows) => {
                rows.iter()
                    .map(|row| Item {
                        id: row.get::<i64, _>("id") as u64,
                        name: row.get("name"),
                        description: row.get("description"),
                    })
                    .collect()
            }
            Err(e) => {
                log::error!("Error fetching all items: {}", e);
                vec![]
            }
        }
    }

    async fn find_by_id(&self, id: u64) -> Option<Item> {
        let result = sqlx::query("SELECT id, name, description FROM items WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await;
            
        match result {
            Ok(Some(row)) => Some(Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
            }),
            Ok(None) => None,
            Err(e) => {
                log::error!("Error finding item by id {}: {}", id, e);
                None
            }
        }
    }

    async fn create(&self, item: Item) -> Item {
        let result = sqlx::query(
                "INSERT INTO items (id, name, description) VALUES ($1, $2, $3) RETURNING id, name, description"
            )
            .bind(item.id as i64)
            .bind(&item.name)
            .bind(&item.description)
            .fetch_one(&self.pool)
            .await;
            
        match result {
            Ok(row) => Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
            },
            Err(e) => {
                log::error!("Error creating item: {}", e);
                item
            }
        }
    }

    async fn update(&self, item: Item) -> Option<Item> {
        let result = sqlx::query(
                "UPDATE items SET name = $2, description = $3 WHERE id = $1 RETURNING id, name, description"
            )
            .bind(item.id as i64)
            .bind(&item.name)
            .bind(&item.description)
            .fetch_optional(&self.pool)
            .await;
            
        match result {
            Ok(Some(row)) => Some(Item {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                description: row.get("description"),
            }),
            Ok(None) => None,
            Err(e) => {
                log::error!("Error updating item {}: {}", item.id, e);
                None
            }
        }
    }

    async fn delete(&self, id: u64) -> bool {
        let result = sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await;

        match result {
            Ok(res) => res.rows_affected() > 0,
            Err(e) => {
                log::error!("Error deleting item {}: {}", id, e);
                false
            }
        }
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
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!(
                "postgres://postgres:postgres@localhost:{}/postgres",
                host_port
            ))
            .await
            .expect("Failed to connect to Postgres");

        pool
    }
    
    #[tokio::test]
    async fn test_postgres_crud_operations() {
        // PostgreSQLコンテナの初期化
        let pool = setup_postgres().await;
        
        // リポジトリの作成とテーブルの初期化
        let repo = PostgresItemRepository::new(pool);
        repo.init_table().await.expect("Failed to create items table");
        
        // テストデータ
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
        };
        
        // 1. アイテム作成のテスト
        let created_item = repo.create(item.clone()).await;
        assert_eq!(created_item.id, item.id);
        assert_eq!(created_item.name, item.name);
        assert_eq!(created_item.description, item.description);
        
        // 2. 単一アイテム取得のテスト
        let found_item = repo.find_by_id(1).await;
        assert!(found_item.is_some());
        let found_item = found_item.unwrap();
        assert_eq!(found_item.id, item.id);
        assert_eq!(found_item.name, item.name);
        assert_eq!(found_item.description, item.description);
        
        // 3. 存在しないアイテム取得のテスト
        let not_found = repo.find_by_id(999).await;
        assert!(not_found.is_none());
        
        // 4. 全アイテム取得のテスト
        let all_items = repo.find_all().await;
        assert_eq!(all_items.len(), 1);
        assert_eq!(all_items[0].id, item.id);
        
        // 5. アイテム更新のテスト
        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
        };
        
        let result = repo.update(updated_item.clone()).await;
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.name, "Updated Item");
        assert_eq!(result.description, Some("Updated Description".to_string()));
        
        // 6. アイテム削除のテスト
        let deleted = repo.delete(1).await;
        assert!(deleted);
        
        // 削除後の検証
        let all_items_after_delete = repo.find_all().await;
        assert_eq!(all_items_after_delete.len(), 0);
        
        // 7. 存在しないアイテムの削除テスト
        let not_deleted = repo.delete(999).await;
        assert!(!not_deleted);
    }
}