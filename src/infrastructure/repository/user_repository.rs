use async_trait::async_trait;
use domain::model::user::User;
use domain::repository::user_repository::UserRepository;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Mutex;

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

impl Default for InMemoryUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_all(&self) -> Vec<User> {
        let users = self
            .users
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        users.values().cloned().collect()
    }

    async fn find_by_id(&self, id: u64) -> Option<User> {
        let users = self
            .users
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        users.get(&id).cloned()
    }

    async fn create(&self, user: User) -> User {
        let mut users = self
            .users
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        users.insert(user.id, user.clone());
        user
    }

    async fn update(&self, user: User) -> Option<User> {
        let mut users = self
            .users
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let std::collections::hash_map::Entry::Occupied(mut e) = users.entry(user.id) {
            e.insert(user.clone());
            Some(user)
        } else {
            None
        }
    }

    async fn delete(&self, id: u64) -> bool {
        let mut users = self
            .users
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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

    // テスト用にテーブルを初期化するメソッド
    #[cfg(any(test, feature = "testing"))]
    #[allow(dead_code)]
    pub async fn init_table(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id BIGINT PRIMARY KEY,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_all(&self) -> Vec<User> {
        let result = sqlx::query("SELECT id, username, email FROM users")
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => rows
                .iter()
                .map(|row| User {
                    id: row.get::<i64, _>("id") as u64,
                    username: row.get("username"),
                    email: row.get("email"),
                })
                .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{clients::Cli, Container};
    use testcontainers_modules::postgres::{self, Postgres};

    async fn setup_postgres() -> (PgPool, Container<'static, Postgres>) {
        // PostgreSQLコンテナの起動（デフォルト設定で実行）
        let docker = Box::leak(Box::new(Cli::default()));
        let container = docker.run(postgres::Postgres::default());

        // PostgreSQLへの接続情報の取得
        let host_port = container.get_host_port_ipv4(5432);

        // 接続プールの作成
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
    async fn test_postgres_crud_operations() {
        // PostgreSQLコンテナの初期化
        let (pool, _container) = setup_postgres().await;

        // リポジトリの作成とテーブルの初期化
        let repo = PostgresUserRepository::new(pool.clone());

        // Create the table directly
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id BIGINT PRIMARY KEY,
                username VARCHAR(255) NOT NULL,
                email VARCHAR(255) NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

        // テストデータ
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };

        // 1. ユーザー作成のテスト
        let created_user = repo.create(user.clone()).await;
        assert_eq!(created_user.id, user.id);
        assert_eq!(created_user.username, user.username);
        assert_eq!(created_user.email, user.email);

        // 2. 単一ユーザー取得のテスト
        let found_user = repo.find_by_id(1).await;
        assert!(found_user.is_some());
        if let Some(found_user) = found_user {
            assert_eq!(found_user.id, user.id);
            assert_eq!(found_user.username, user.username);
            assert_eq!(found_user.email, user.email);
        } else {
            panic!("Expected to find user with id 1");
        }

        // 3. 存在しないユーザー取得のテスト
        let not_found = repo.find_by_id(999).await;
        assert!(not_found.is_none());

        // 4. 全ユーザー取得のテスト
        let all_users = repo.find_all().await;
        assert_eq!(all_users.len(), 1);
        assert_eq!(all_users[0].id, user.id);

        // 5. ユーザー更新のテスト
        let updated_user = User {
            id: 1,
            username: "updateduser".to_string(),
            email: "updated@example.com".to_string(),
        };

        let result = repo.update(updated_user.clone()).await;
        assert!(result.is_some());
        if let Some(result) = result {
            assert_eq!(result.username, "updateduser");
            assert_eq!(result.email, "updated@example.com");
        } else {
            panic!("Expected to update user with id 1");
        }

        // 6. ユーザー削除のテスト
        let deleted = repo.delete(1).await;
        assert!(deleted);

        // 削除後の検証
        let all_users_after_delete = repo.find_all().await;
        assert_eq!(all_users_after_delete.len(), 0);

        // 7. 存在しないユーザーの削除テスト
        let not_deleted = repo.delete(999).await;
        assert!(!not_deleted);
    }
}
