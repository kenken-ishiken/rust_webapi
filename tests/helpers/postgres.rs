#![allow(dead_code)]
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{sync::Once, time::Duration};
use testcontainers_modules::postgres;

static INIT: Once = Once::new();

/// Parse SQL statements from a string, handling PL/pgSQL function blocks correctly
fn parse_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut in_dollar_quote = false;
    let mut dollar_quote_tag = String::new();
    let chars: Vec<char> = sql.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        // Check for dollar quote start/end
        if ch == '$' && i + 1 < chars.len() {
            // Look for a dollar quote pattern
            let mut j = i + 1;
            let mut tag = String::from("$");
            
            // Collect the tag (letters, numbers, underscores between $$)
            while j < chars.len() && chars[j] != '$' {
                tag.push(chars[j]);
                j += 1;
            }
            
            if j < chars.len() && chars[j] == '$' {
                tag.push('$');
                
                if !in_dollar_quote {
                    // Starting a dollar quote
                    in_dollar_quote = true;
                    dollar_quote_tag = tag.clone();
                    current_statement.push_str(&tag);
                    i = j;
                } else if tag == dollar_quote_tag {
                    // Ending the dollar quote
                    in_dollar_quote = false;
                    dollar_quote_tag.clear();
                    current_statement.push_str(&tag);
                    i = j;
                }
            } else {
                current_statement.push(ch);
            }
        } else if ch == ';' && !in_dollar_quote {
            // Found statement terminator outside of dollar quotes
            current_statement.push(ch);
            let statement = current_statement.trim().to_string();
            if !statement.is_empty() && !statement.starts_with("--") {
                statements.push(statement);
            }
            current_statement.clear();
        } else {
            current_statement.push(ch);
        }
        
        i += 1;
    }
    
    // Add any remaining statement
    let final_statement = current_statement.trim().to_string();
    if !final_statement.is_empty() && !final_statement.starts_with("--") {
        statements.push(final_statement);
    }
    
    statements
}

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
            static DOCKER: std::sync::OnceLock<testcontainers::clients::Cli> =
                std::sync::OnceLock::new();
            let docker = DOCKER.get_or_init(testcontainers::clients::Cli::default);

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
        let mut retries = 0;
        const MAX_RETRIES: usize = 10;

        loop {
            match PgPoolOptions::new()
                .max_connections(5)
                .acquire_timeout(Duration::from_secs(5))
                .connect(&self.connection_string())
                .await
            {
                Ok(pool) => {
                    // Verify the connection is ready by running a simple query
                    let mut ready_ok = false;
                    for _ in 0..5 {
                        match sqlx::query("SELECT 1").execute(&pool).await {
                            Ok(_) => {
                                ready_ok = true;
                                break;
                            }
                            Err(e) => {
                                eprintln!("Postgres not ready yet (ping): {}", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                    if ready_ok {
                        break pool;
                    } else {
                        retries += 1;
                        if retries >= MAX_RETRIES {
                            panic!(
                                "Failed to connect to Postgres after {} retries",
                                MAX_RETRIES
                            );
                        }
                        continue;
                    }
                }
                Err(e) => {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        panic!(
                            "Failed to create database pool after {} retries: {}",
                            MAX_RETRIES, e
                        );
                    }
                    eprintln!("Failed to create pool (attempt {}): {}", retries, e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    pub async fn run_migrations(&self, pool: &Pool<Postgres>) {
        
        // Read and execute the complete migration file
        let migration_content = include_str!("../../initdb/01_create_tables.sql");
        
        // Get a connection from the pool for raw execution
        let mut conn = pool.acquire().await.expect("Failed to acquire connection");
        
        // Use raw SQL execution which supports multiple statements
        match sqlx::raw_sql(migration_content)
            .execute(&mut *conn)
            .await
        {
            Ok(_) => {
                println!("Migrations executed successfully");
            }
            Err(e) => {
                eprintln!("Failed to execute migrations as batch: {}", e);
                
                // If raw execution fails, drop the connection and get a new one
                drop(conn);
                
                // Try a more sophisticated approach - parse and execute in order
                let mut conn = pool.acquire().await.expect("Failed to acquire connection");
                
                // Create the update function first, as it's needed by triggers
                let function_sql = r#"
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = CURRENT_TIMESTAMP;
                    RETURN NEW;
                END;
                $$ language 'plpgsql'"#;
                
                if let Err(e) = sqlx::query(function_sql).execute(&mut *conn).await {
                    eprintln!("Failed to create update function: {}", e);
                }
                
                // Now execute the rest of the migrations
                let statements = parse_sql_statements(migration_content);
                
                for statement in statements {
                    let trimmed = statement.trim();
                    if trimmed.is_empty() || trimmed.contains("CREATE OR REPLACE FUNCTION update_updated_at_column") {
                        continue; // Skip empty or already executed function
                    }
                    
                    if let Err(e) = sqlx::query(&statement).execute(&mut *conn).await {
                        eprintln!("Warning: Failed to execute migration statement: {}", e);
                        if statement.len() > 100 {
                            eprintln!("Statement: {}...", &statement[..100]);
                        } else {
                            eprintln!("Statement: {}", statement);
                        }
                    }
                }
            }
        }
    }
}

// Use thread_local storage to keep the container alive
thread_local! {
    static CONTAINER: std::cell::RefCell<Option<testcontainers::Container<'static, postgres::Postgres>>> = const { std::cell::RefCell::new(None) };
}
