use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{sync::Once, time::Duration};
use testcontainers_modules::postgres;

static INIT: Once = Once::new();

pub struct PostgresContainer {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl PostgresContainer {
    pub fn new() -> Self {
        INIT.call_once(|| {
            // This code runs only once across all tests
            let _ = env_logger::builder().is_test(true).try_init();
        });

        // Use a static Docker client
        let host_port = {
            // Create a static Docker client
            static DOCKER: std::sync::OnceLock<testcontainers::clients::Cli> = std::sync::OnceLock::new();
            let docker = DOCKER.get_or_init(|| testcontainers::clients::Cli::default());
            
            CONTAINER.with(|c| {
                if c.borrow().is_none() {
                    // PostgreSQL container setup with testcontainers-modules
                    let container = docker.run(postgres::Postgres::default());
                    
                    // Get the mapped port
                    let port = container.get_host_port_ipv4(5432);
                    
                    // Store the container in thread_local storage
                    *c.borrow_mut() = Some(container);
                    
                    port
                } else {
                    // Container already exists, get the port
                    c.borrow().as_ref().unwrap().get_host_port_ipv4(5432)
                }
            })
        };

        Self {
            host: "localhost".to_string(),
            port: host_port,
            user: "postgres".to_string(),
            password: "postgres".to_string(), // Default password for testcontainers postgres module
            database: "postgres".to_string(), // Default database for testcontainers postgres module
        }
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }

    pub async fn create_pool(&self) -> Pool<Postgres> {
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&self.connection_string())
            .await
            .expect("Failed to create database pool")
    }

    pub async fn run_migrations(&self, pool: &Pool<Postgres>) {
        // Run migrations from the initdb directory
        let migration_query = include_str!("../../initdb/01_create_tables.sql");
        sqlx::query(migration_query)
            .execute(pool)
            .await
            .expect("Failed to run migrations");
    }
}

// Use thread_local storage to keep the container alive
thread_local! {
    static CONTAINER: std::cell::RefCell<Option<testcontainers::Container<'static, postgres::Postgres>>> = std::cell::RefCell::new(None);
}