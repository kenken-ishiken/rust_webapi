mod helpers;

use helpers::postgres::PostgresContainer;
use sqlx::Row;

#[tokio::test]
async fn test_postgres_container() {
    // Create a PostgreSQL container
    let postgres = PostgresContainer::new();

    // Get the connection string
    let connection_string = postgres.connection_string();
    println!("Connection string: {}", connection_string);

    // Create a database pool
    let pool = postgres.create_pool().await;

    // Run migrations
    postgres.run_migrations(&pool).await;

    // Execute a simple query to verify the connection
    let row = sqlx::query("SELECT 1 as result")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute query");

    let result: i32 = row.get("result");
    assert_eq!(result, 1);

    // This also verifies that our migrations were applied
    // Assuming we have a users table from migrations
    let users_count = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to count users");

    let count: i64 = users_count.get("count");
    assert_eq!(count, 0); // Expecting an empty table
}
