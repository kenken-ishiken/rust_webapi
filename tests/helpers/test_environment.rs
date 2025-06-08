#![allow(dead_code)]
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use rust_webapi::app_domain::repository::item_repository::ItemRepository;
use rust_webapi::app_domain::repository::category_repository::CategoryRepository;
use rust_webapi::infrastructure::repository::item_repository::{InMemoryItemRepository, PostgresItemRepository};
use rust_webapi::infrastructure::repository::user_repository::{InMemoryUserRepository, PostgresUserRepository};
use rust_webapi::infrastructure::repository::category_repository::PostgresCategoryRepository;
use domain::repository::user_repository::UserRepository;

/// Test environment configuration
#[derive(Debug, Clone)]
pub enum TestEnvironment {
    /// Use PostgreSQL with testcontainers (requires Docker)
    PostgreSQL(Pool<Postgres>),
    /// Use in-memory implementations (no Docker required)
    InMemory,
}

/// Test repository factory that provides appropriate implementations based on environment
pub struct TestRepositoryFactory {
    environment: TestEnvironment,
}

impl TestRepositoryFactory {
    /// Create a new test repository factory
    /// 
    /// This will attempt to use PostgreSQL with testcontainers if Docker is available,
    /// otherwise it will fall back to in-memory implementations.
    pub async fn new() -> Self {
        let environment = Self::detect_environment().await;
        Self { environment }
    }

    /// Force creation with a specific environment (useful for testing specific scenarios)
    pub fn with_environment(environment: TestEnvironment) -> Self {
        Self { environment }
    }

    /// Detect the best available test environment
    async fn detect_environment() -> TestEnvironment {
        // Try to use PostgreSQL with testcontainers
        match Self::try_postgres().await {
            Ok(pool) => {
                println!("✅ Using PostgreSQL testcontainer for integration tests");
                TestEnvironment::PostgreSQL(pool)
            }
            Err(e) => {
                println!("⚠️  PostgreSQL testcontainer not available ({}), falling back to in-memory implementations", e);
                TestEnvironment::InMemory
            }
        }
    }

    /// Try to create a PostgreSQL testcontainer
    async fn try_postgres() -> Result<Pool<Postgres>, Box<dyn std::error::Error + Send + Sync>> {
        use testcontainers_modules::postgres;
        use sqlx::postgres::PgPoolOptions;
        use std::time::Duration;

        // Check if Docker is available
        if !Self::is_docker_available() {
            return Err("Docker is not available".into());
        }

        // Try to create a testcontainer
        let docker = testcontainers::clients::Cli::default();
        let container = docker.run(postgres::Postgres::default());
        let host_port = container.get_host_port_ipv4(5432);

        let conn_str = format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            host_port
        );

        // Try to connect with a timeout
        let pool = tokio::time::timeout(
            Duration::from_secs(30),
            PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&conn_str)
        ).await??;

        // Verify connection with a simple query
        sqlx::query("SELECT 1").execute(&pool).await?;

        // Run migrations
        let migration_query = include_str!("../../initdb/01_create_tables.sql");
        sqlx::query(migration_query).execute(&pool).await?;

        Ok(pool)
    }

    /// Check if Docker is available
    fn is_docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Create an item repository
    pub fn create_item_repository(&self) -> Arc<dyn ItemRepository> {
        match &self.environment {
            TestEnvironment::PostgreSQL(pool) => {
                Arc::new(PostgresItemRepository::new(pool.clone()))
            }
            TestEnvironment::InMemory => {
                Arc::new(InMemoryItemRepository::new())
            }
        }
    }

    /// Create a user repository
    pub fn create_user_repository(&self) -> Arc<dyn UserRepository> {
        match &self.environment {
            TestEnvironment::PostgreSQL(pool) => {
                Arc::new(PostgresUserRepository::new(pool.clone()))
            }
            TestEnvironment::InMemory => {
                Arc::new(InMemoryUserRepository::new())
            }
        }
    }

    /// Create a category repository (PostgreSQL only for now)
    pub fn create_category_repository(&self) -> Option<Arc<dyn CategoryRepository>> {
        match &self.environment {
            TestEnvironment::PostgreSQL(pool) => {
                Some(Arc::new(PostgresCategoryRepository::new(pool.clone())))
            }
            TestEnvironment::InMemory => {
                // No in-memory implementation available for categories yet
                None
            }
        }
    }

    /// Get the environment type for test reporting
    pub fn environment_type(&self) -> &str {
        match &self.environment {
            TestEnvironment::PostgreSQL(_) => "PostgreSQL",
            TestEnvironment::InMemory => "InMemory",
        }
    }

    /// Check if using PostgreSQL
    pub fn is_postgres(&self) -> bool {
        matches!(self.environment, TestEnvironment::PostgreSQL(_))
    }

    /// Check if using in-memory
    pub fn is_in_memory(&self) -> bool {
        matches!(self.environment, TestEnvironment::InMemory)
    }
}

/// Macro to create a test that runs with the best available environment
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = $crate::helpers::test_environment::TestRepositoryFactory::new().await;
            println!("🧪 Running test '{}' with {} environment", 
                stringify!($test_name), 
                factory.environment_type()
            );
            
            let test_fn: Box<dyn Fn($crate::helpers::test_environment::TestRepositoryFactory) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>> = 
                Box::new(|factory| Box::pin($test_body(factory)));
            
            test_fn(factory).await;
        }
    };
}

/// Macro to create a test that only runs with PostgreSQL
#[macro_export]
macro_rules! postgres_only_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = $crate::helpers::test_environment::TestRepositoryFactory::new().await;
            
            if !factory.is_postgres() {
                println!("⏭️  Skipping PostgreSQL-only test '{}' (Docker not available)", stringify!($test_name));
                return;
            }
            
            println!("🧪 Running PostgreSQL-only test '{}'", stringify!($test_name));
            
            let test_fn: Box<dyn Fn($crate::helpers::test_environment::TestRepositoryFactory) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>> = 
                Box::new(|factory| Box::pin($test_body(factory)));
            
            test_fn(factory).await;
        }
    };
}

/// Macro to create a test that only runs with in-memory implementations
#[macro_export]
macro_rules! in_memory_only_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = $crate::helpers::test_environment::TestRepositoryFactory::with_environment(
                $crate::helpers::test_environment::TestEnvironment::InMemory
            );
            
            println!("🧪 Running in-memory-only test '{}'", stringify!($test_name));
            
            let test_fn: Box<dyn Fn($crate::helpers::test_environment::TestRepositoryFactory) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>> = 
                Box::new(|factory| Box::pin($test_body(factory)));
            
            test_fn(factory).await;
        }
    };
}