mod helpers;

use domain::model::item::Item;
use domain::repository::item_repository::ItemRepository;
use domain::model::user::User;
use domain::repository::user_repository::UserRepository;
use helpers::postgres::PostgresContainer;
use rust_webapi::infrastructure::repository::item_repository::PostgresItemRepository;
use rust_webapi::infrastructure::repository::user_repository::PostgresUserRepository;

#[tokio::test]
async fn test_postgres_item_repository() {
    // Create a PostgreSQL container
    let postgres = PostgresContainer::new();
    
    // Create a database pool
    let pool = postgres.create_pool().await;
    
    // Create the repository and initialize the table
    let repo = PostgresItemRepository::new(pool.clone());
    repo.init_table().await.expect("Failed to create items table");
    
    // Test data
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
    };
    
    // 1. Test item creation
    let created_item = repo.create(item.clone()).await;
    assert_eq!(created_item.id, item.id);
    assert_eq!(created_item.name, item.name);
    assert_eq!(created_item.description, item.description);
    
    // 2. Test finding item by ID
    let found_item = repo.find_by_id(1).await;
    assert!(found_item.is_some());
    let found_item = found_item.unwrap();
    assert_eq!(found_item.id, item.id);
    assert_eq!(found_item.name, item.name);
    assert_eq!(found_item.description, item.description);
    
    // 3. Test finding a non-existent item
    let not_found = repo.find_by_id(999).await;
    assert!(not_found.is_none());
    
    // 4. Test getting all items
    let all_items = repo.find_all().await;
    assert_eq!(all_items.len(), 1);
    assert_eq!(all_items[0].id, item.id);
    
    // 5. Test updating an item
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
    
    // 6. Test deleting an item
    let deleted = repo.delete(1).await;
    assert!(deleted);
    
    // Verify deletion
    let all_items_after_delete = repo.find_all().await;
    assert_eq!(all_items_after_delete.len(), 0);
    
    // 7. Test deleting a non-existent item
    let not_deleted = repo.delete(999).await;
    assert!(!not_deleted);
}

// Test batch operations
#[tokio::test]
async fn test_postgres_batch_operations() {
    // Create a PostgreSQL container
    let postgres = PostgresContainer::new();
    
    // Create a database pool
    let pool = postgres.create_pool().await;
    
    // Create the repository and initialize the table
    let repo = PostgresItemRepository::new(pool.clone());
    repo.init_table().await.expect("Failed to create items table");
    
    // Create multiple items
    let items = vec![
        Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
        },
        Item {
            id: 2,
            name: "Item 2".to_string(),
            description: None,
        },
        Item {
            id: 3,
            name: "Item 3".to_string(),
            description: Some("Description 3".to_string()),
        },
    ];
    
    // Insert all items
    for item in items.clone() {
        repo.create(item).await;
    }
    
    // Test batch retrieval
    let all_items = repo.find_all().await;
    assert_eq!(all_items.len(), 3);
    
    // Verify items are sorted by ID
    assert_eq!(all_items[0].id, 1);
    assert_eq!(all_items[1].id, 2);
    assert_eq!(all_items[2].id, 3);
    
    // Test batch updates (changing all descriptions)
    for mut item in all_items {
        item.description = Some("Updated".to_string());
        let _ = repo.update(item).await;
    }
    
    // Verify all descriptions were updated
    let updated_items = repo.find_all().await;
    for item in updated_items {
        assert_eq!(item.description, Some("Updated".to_string()));
    }
}

#[tokio::test]
async fn test_postgres_user_repository() {
    // Create a PostgreSQL container
    let postgres = PostgresContainer::new();

    // Create a database pool
    let pool = postgres.create_pool().await;

    // Create the repository and initialize the table
    let repo = PostgresUserRepository::new(pool.clone());
    repo.init_table().await.expect("Failed to create users table");

    // Test data
    let user = User {
        id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };

    // 1. Test user creation
    let created_user = repo.create(user.clone()).await;
    assert_eq!(created_user.id, user.id);
    assert_eq!(created_user.username, user.username);
    assert_eq!(created_user.email, user.email);

    // 2. Test finding user by ID
    let found_user = repo.find_by_id(1).await;
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.username, user.username);
    assert_eq!(found_user.email, user.email);

    // 3. Test finding a non-existent user
    let not_found = repo.find_by_id(999).await;
    assert!(not_found.is_none());

    // 4. Test getting all users
    let all_users = repo.find_all().await;
    assert_eq!(all_users.len(), 1);
    assert_eq!(all_users[0].id, user.id);

    // 5. Test updating a user
    let updated_user = User {
        id: 1,
        username: "updateduser".to_string(),
        email: "updated@example.com".to_string(),
    };

    let result = repo.update(updated_user.clone()).await;
    assert!(result.is_some());
    let result = result.unwrap();
    assert_eq!(result.username, "updateduser");
    assert_eq!(result.email, "updated@example.com");

    // 6. Test deleting a user
    let deleted = repo.delete(1).await;
    assert!(deleted);

    // Verify deletion
    let all_users_after_delete = repo.find_all().await;
    assert_eq!(all_users_after_delete.len(), 0);

    // 7. Test deleting a non-existent user
    let not_deleted = repo.delete(999).await;
    assert!(!not_deleted);
}