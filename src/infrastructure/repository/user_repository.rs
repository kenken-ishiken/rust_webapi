use std::collections::HashMap;
use std::sync::Mutex;
use sqlx::{PgPool, Row};
use async_trait::async_trait;
use crate::domain::model::user::User;
use crate::domain::repository::user_repository::UserRepository;

pub struct InMemoryUserRepository {
    users: Mutex<HashMap<u64, User>>,
}

impl InMemoryUserRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_all(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }

    async fn find_by_id(&self, id: u64) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.get(&id).cloned()
    }

    async fn create(&self, user: User) -> User {
        let mut users = self.users.lock().unwrap();
        users.insert(user.id, user.clone());
        user
    }

    async fn update(&self, user: User) -> Option<User> {
        let mut users = self.users.lock().unwrap();
        if users.contains_key(&user.id) {
            users.insert(user.id, user.clone());
            Some(user)
        } else {
            None
        }
    }

    async fn delete(&self, id: u64) -> bool {
        let mut users = self.users.lock().unwrap();
        users.remove(&id).is_some()
    }
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_all(&self) -> Vec<User> {
        let result = sqlx::query("SELECT id, username, email FROM users")
            .fetch_all(&self.pool)
            .await;
            
        match result {
            Ok(rows) => {
                rows.iter()
                    .map(|row| User {
                        id: row.get::<i64, _>("id") as u64,
                        username: row.get("username"),
                        email: row.get("email"),
                    })
                    .collect()
            }
            Err(e) => {
                log::error!("Error fetching all users: {}", e);
                vec![]
            }
        }
    }

    async fn find_by_id(&self, id: u64) -> Option<User> {
        let result = sqlx::query("SELECT id, username, email FROM users WHERE id = $1")
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await;
            
        match result {
            Ok(Some(row)) => Some(User {
                id: row.get::<i64, _>("id") as u64,
                username: row.get("username"),
                email: row.get("email"),
            }),
            Ok(None) => None,
            Err(e) => {
                log::error!("Error finding user by id {}: {}", id, e);
                None
            }
        }
    }

    async fn create(&self, user: User) -> User {
        let result = sqlx::query(
                "INSERT INTO users (id, username, email) VALUES ($1, $2, $3) RETURNING id, username, email"
            )
            .bind(user.id as i64)
            .bind(&user.username)
            .bind(&user.email)
            .fetch_one(&self.pool)
            .await;
            
        match result {
            Ok(row) => User {
                id: row.get::<i64, _>("id") as u64,
                username: row.get("username"),
                email: row.get("email"),
            },
            Err(e) => {
                log::error!("Error creating user: {}", e);
                user
            }
        }
    }

    async fn update(&self, user: User) -> Option<User> {
        let result = sqlx::query(
                "UPDATE users SET username = $2, email = $3 WHERE id = $1 RETURNING id, username, email"
            )
            .bind(user.id as i64)
            .bind(&user.username)
            .bind(&user.email)
            .fetch_optional(&self.pool)
            .await;
            
        match result {
            Ok(Some(row)) => Some(User {
                id: row.get::<i64, _>("id") as u64,
                username: row.get("username"),
                email: row.get("email"),
            }),
            Ok(None) => None,
            Err(e) => {
                log::error!("Error updating user {}: {}", user.id, e);
                None
            }
        }
    }

    async fn delete(&self, id: u64) -> bool {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id as i64)
            .execute(&self.pool)
            .await;
            
        match result {
            Ok(res) => res.rows_affected() > 0,
            Err(e) => {
                log::error!("Error deleting user {}: {}", id, e);
                false
            }
        }
    }
}