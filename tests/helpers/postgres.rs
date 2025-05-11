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

        // PostgreSQL container setup with testcontainers-modules
        let docker = testcontainers::clients::Cli::default();
        let container = docker.run(postgres::Postgres::default());

        // Get the mapped port
        let host_port = container.get_host_port_ipv4(5432);

        // The container will be automatically cleaned up when it goes out of scope
        // To keep it alive for the duration of the test, we need to keep reference to it
        // We store it in thread_local storage
        CONTAINER.with(|c| {
            *c.borrow_mut() = Some(container);
        });

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
    static CONTAINER: std::cell::RefCell<Option<testcontainers::Container<postgres::Postgres>>> = std::cell::RefCell::new(None);
}