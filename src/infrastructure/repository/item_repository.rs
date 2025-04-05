use std::collections::HashMap;
use std::sync::Mutex;
use sqlx::{PgPool, Row};
use async_trait::async_trait;
use crate::domain::model::item::Item;
use crate::domain::repository::item_repository::ItemRepository;

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