use domain::model::item::Item;
use domain::model::user::User;
use domain::repository::item_repository::ItemRepository;
use domain::repository::user_repository::UserRepository;
use rust_webapi::infrastructure::repository::item_repository::PostgresItemRepository;
use rust_webapi::infrastructure::repository::user_repository::PostgresUserRepository;
use helpers::postgres::PostgresContainer;

mod helpers;

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_item_repository_duplicate_id() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
    };
    
    // Create first item
    let created1 = repo.create(item.clone()).await;
    assert_eq!(created1.id, 1);
    
    // Try to create duplicate ID - should handle gracefully
    // Note: PostgreSQL should prevent this due to PRIMARY KEY constraint
    // The implementation should handle this error gracefully
    let created2 = repo.create(item.clone()).await;
    
    // The implementation might return the original item or handle the error
    // This tests the error handling behavior
    assert_eq!(created2.id, 1);
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_user_repository_duplicate_id() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresUserRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    let user = User {
        id: 1,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
    };
    
    // Create first user
    let created1 = repo.create(user.clone()).await;
    assert_eq!(created1.id, 1);
    
    // Try to create duplicate ID
    let created2 = repo.create(user.clone()).await;
    assert_eq!(created2.id, 1);
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_large_data() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Test with very large text data
    let large_description = "A".repeat(100000); // 100KB of text
    let item = Item {
        id: 1,
        name: "Large Data Item".to_string(),
        description: Some(large_description.clone()),
    };
    
    let created = repo.create(item).await;
    assert_eq!(created.description, Some(large_description));
    
    // Verify retrieval
    let found = repo.find_by_id(1).await;
    assert!(found.is_some());
    let found_item = found.unwrap();
    assert_eq!(found_item.description.as_ref().unwrap().len(), 100000);
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_unicode_handling() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Test with various Unicode characters
    let unicode_data = vec![
        ("Japanese", "こんにちは世界", "これは日本語のテストです"),
        ("Emoji", "Test 🚀 Item", "Description with emojis 🎉 🌟 ⭐"),
        ("Arabic", "مرحبا بالعالم", "وصف باللغة العربية"),
        ("Chinese", "你好世界", "中文描述测试"),
        ("Mixed", "Test مع العربية and 日本語", "Mixed language description"),
    ];
    
    for (i, (test_name, name, description)) in unicode_data.into_iter().enumerate() {
        let item = Item {
            id: (i + 1) as u64,
            name: name.to_string(),
            description: Some(description.to_string()),
        };
        
        let created = repo.create(item.clone()).await;
        assert_eq!(created.name, name, "Failed for {}", test_name);
        assert_eq!(created.description, Some(description.to_string()), "Failed for {}", test_name);
        
        // Verify retrieval
        let found = repo.find_by_id((i + 1) as u64).await;
        assert!(found.is_some(), "Failed to find item for {}", test_name);
        let found_item = found.unwrap();
        assert_eq!(found_item.name, name, "Retrieval failed for {}", test_name);
        assert_eq!(found_item.description, Some(description.to_string()), "Retrieval failed for {}", test_name);
    }
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_null_and_empty_values() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Test with null description
    let item_null = Item {
        id: 1,
        name: "Item with null description".to_string(),
        description: None,
    };
    
    let created = repo.create(item_null).await;
    assert_eq!(created.description, None);
    
    // Test with empty name (edge case)
    let item_empty_name = Item {
        id: 2,
        name: "".to_string(),
        description: Some("Has description but empty name".to_string()),
    };
    
    let created = repo.create(item_empty_name).await;
    assert_eq!(created.name, "");
    
    // Test with empty description
    let item_empty_desc = Item {
        id: 3,
        name: "Has name".to_string(),
        description: Some("".to_string()),
    };
    
    let created = repo.create(item_empty_desc).await;
    assert_eq!(created.description, Some("".to_string()));
    
    // Verify all items can be retrieved correctly
    let all_items = repo.find_all().await;
    assert_eq!(all_items.len(), 3);
    
    // Find and verify each item
    let item1 = repo.find_by_id(1).await.unwrap();
    assert_eq!(item1.description, None);
    
    let item2 = repo.find_by_id(2).await.unwrap();
    assert_eq!(item2.name, "");
    
    let item3 = repo.find_by_id(3).await.unwrap();
    assert_eq!(item3.description, Some("".to_string()));
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_special_characters() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Test with SQL injection attempt (should be safely handled by sqlx)
    let malicious_name = "'; DROP TABLE items; --";
    let malicious_description = "'; DELETE FROM items WHERE 1=1; --";
    
    let item = Item {
        id: 1,
        name: malicious_name.to_string(),
        description: Some(malicious_description.to_string()),
    };
    
    let created = repo.create(item).await;
    assert_eq!(created.name, malicious_name);
    assert_eq!(created.description, Some(malicious_description.to_string()));
    
    // Verify the table still exists and item was stored safely
    let found = repo.find_by_id(1).await;
    assert!(found.is_some());
    
    // Test with various special characters
    let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    let item_special = Item {
        id: 2,
        name: format!("Special chars: {}", special_chars),
        description: Some(format!("Description with: {}", special_chars)),
    };
    
    let created = repo.create(item_special.clone()).await;
    assert_eq!(created.name, item_special.name);
    assert_eq!(created.description, item_special.description);
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_boundary_values() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Test with ID 0
    let item_zero = Item {
        id: 0,
        name: "Zero ID".to_string(),
        description: Some("Testing zero ID".to_string()),
    };
    
    let created = repo.create(item_zero).await;
    assert_eq!(created.id, 0);
    
    let found = repo.find_by_id(0).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 0);
    
    // Test with maximum u64 value (might not work due to PostgreSQL bigint limits)
    // PostgreSQL bigint is signed 64-bit, so max value is 2^63-1
    let max_safe_id = (1u64 << 63) - 1;
    let item_max = Item {
        id: max_safe_id,
        name: "Max ID".to_string(),
        description: Some("Testing maximum safe ID".to_string()),
    };
    
    let created = repo.create(item_max).await;
    assert_eq!(created.id, max_safe_id);
    
    let found = repo.find_by_id(max_safe_id).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, max_safe_id);
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_concurrent_operations() {
    use std::sync::Arc;
    use tokio::task;
    
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = Arc::new(PostgresItemRepository::new(pool.clone()));
    postgres.run_migrations(&pool).await;
    
    // Test concurrent inserts
    let mut insert_handles = vec![];
    
    for i in 1..=50 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            let item = Item {
                id: i,
                name: format!("Concurrent Item {}", i),
                description: Some(format!("Concurrent Description {}", i)),
            };
            repo_clone.create(item).await
        });
        insert_handles.push(handle);
    }
    
    // Wait for all inserts to complete
    for handle in insert_handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
    
    // Verify all items were inserted
    let all_items = repo.find_all().await;
    assert_eq!(all_items.len(), 50);
    
    // Test concurrent reads
    let mut read_handles = vec![];
    
    for i in 1..=50 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            repo_clone.find_by_id(i).await
        });
        read_handles.push(handle);
    }
    
    // Verify all reads succeed
    for (i, handle) in read_handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, (i + 1) as u64);
    }
    
    // Test concurrent updates
    let mut update_handles = vec![];
    
    for i in 1..=25 { // Update first half
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            let updated_item = Item {
                id: i,
                name: format!("Updated Concurrent Item {}", i),
                description: Some(format!("Updated Concurrent Description {}", i)),
            };
            repo_clone.update(updated_item).await
        });
        update_handles.push(handle);
    }
    
    // Wait for all updates to complete
    for handle in update_handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
    }
    
    // Verify updates
    for i in 1..=25 {
        let found = repo.find_by_id(i).await;
        assert!(found.is_some());
        let item = found.unwrap();
        assert_eq!(item.name, format!("Updated Concurrent Item {}", i));
    }
}

#[tokio::test]
#[ignore = "Skipping due to connection issues in CI environment"]
async fn test_postgres_repository_transaction_behavior() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresItemRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;
    
    // Create initial item
    let item1 = Item {
        id: 1,
        name: "Original Item".to_string(),
        description: Some("Original Description".to_string()),
    };
    
    let created = repo.create(item1).await;
    assert_eq!(created.id, 1);
    
    // Test that updates are atomic
    let updated_item = Item {
        id: 1,
        name: "Updated Item".to_string(),
        description: Some("Updated Description".to_string()),
    };
    
    let update_result = repo.update(updated_item.clone()).await;
    assert!(update_result.is_some());
    
    // Verify the update was applied completely
    let found = repo.find_by_id(1).await;
    assert!(found.is_some());
    let found_item = found.unwrap();
    assert_eq!(found_item.name, "Updated Item");
    assert_eq!(found_item.description, Some("Updated Description".to_string()));
    
    // Test that deletes are atomic
    let deleted = repo.delete(1).await;
    assert!(deleted);
    
    // Verify the item is completely gone
    let not_found = repo.find_by_id(1).await;
    assert!(not_found.is_none());
    
    let all_items = repo.find_all().await;
    assert_eq!(all_items.len(), 0);
}