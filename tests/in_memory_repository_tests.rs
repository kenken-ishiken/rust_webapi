use domain::model::item::Item;
use domain::model::user::User;
use rust_webapi::app_domain::repository::item_repository::ItemRepository;
use domain::repository::user_repository::UserRepository;
use rust_webapi::infrastructure::repository::item_repository::InMemoryItemRepository;
use rust_webapi::infrastructure::repository::user_repository::InMemoryUserRepository;

#[tokio::test]
async fn test_in_memory_item_repository_operations() {
    let repo = InMemoryItemRepository::new();
    
    // Test empty repository
    let all_items = repo.find_all().await.unwrap();
    assert_eq!(all_items.len(), 0);

    let not_found = repo.find_by_id(1).await.unwrap();
    assert!(not_found.is_none());
    
    // Test create
    let item1 = Item {
        id: 1,
        name: "Item 1".to_string(),
        description: Some("Description 1".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item1.clone()).await.unwrap();
    assert_eq!(created.id, 1);
    assert_eq!(created.name, "Item 1");
    
    // Test find_by_id
    let found = repo.find_by_id(1).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 1);
    
    // Test find_all
    let all_items = repo.find_all().await.unwrap();
    assert_eq!(all_items.len(), 1);
    
    // Test update existing
    let updated_item = Item {
        id: 1,
        name: "Updated Item".to_string(),
        description: Some("Updated Description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    let result = repo.update(updated_item.clone()).await;
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.name, "Updated Item");
    
    // Test update non-existing

    let non_existing = Item {
        id: 999,
        name: "Non-existing".to_string(),
        description: None,
        deleted: false,
        deleted_at: None,
    };
    
    let result = repo.update(non_existing).await;
    assert!(result.is_err());
    
    // Test delete existing
    repo.delete(1).await.unwrap();
    
    // Verify deletion
    let all_items = repo.find_all().await.unwrap();
    assert_eq!(all_items.len(), 0);
    
    // Test delete non-existing
    let result = repo.delete(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_in_memory_user_repository_operations() {
    let repo = InMemoryUserRepository::new();
    
    // Test empty repository
    let all_users = repo.find_all().await;
    assert_eq!(all_users.len(), 0);
    
    let not_found = repo.find_by_id(1).await;
    assert!(not_found.is_none());
    
    // Test create
    let user1 = User {
        id: 1,
        username: "user1".to_string(),
        email: "user1@example.com".to_string(),
    };
    
    let created = repo.create(user1.clone()).await;
    assert_eq!(created.id, 1);
    assert_eq!(created.username, "user1");
    assert_eq!(created.email, "user1@example.com");
    
    // Test find_by_id
    let found = repo.find_by_id(1).await;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 1);
    
    // Test find_all
    let all_users = repo.find_all().await;
    assert_eq!(all_users.len(), 1);
    
    // Test update existing
    let updated_user = User {
        id: 1,
        username: "updated_user".to_string(),
        email: "updated@example.com".to_string(),
    };
    
    let result = repo.update(updated_user.clone()).await;
    assert!(result.is_some());
    let updated = result.unwrap();
    assert_eq!(updated.username, "updated_user");
    assert_eq!(updated.email, "updated@example.com");
    
    // Test update non-existing
    let non_existing = User {
        id: 999,
        username: "nonexistent".to_string(),
        email: "none@example.com".to_string(),
    };
    
    let result = repo.update(non_existing).await;
    assert!(result.is_none());
    
    // Test delete existing
    let deleted = repo.delete(1).await;
    assert!(deleted);
    
    // Verify deletion
    let all_users = repo.find_all().await;
    assert_eq!(all_users.len(), 0);
    
    // Test delete non-existing
    let not_deleted = repo.delete(999).await;
    assert!(!not_deleted);
}

#[tokio::test]
async fn test_concurrent_access_item_repository() {
    use std::sync::Arc;
    use tokio::task;
    
    let repo = Arc::new(InMemoryItemRepository::new());
    
    // Create items concurrently
    let mut handles = vec![];
    
    for i in 1..=10 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            let item = Item {
                id: i,
                name: format!("Item {}", i),
                description: Some(format!("Description {}", i)),
                deleted: false,
                deleted_at: None,
            };
            repo_clone.create(item).await.unwrap()
        });
        handles.push(handle);
    }
    
    // Wait for all creates to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all items were created
    let all_items = repo.find_all().await.unwrap();
    assert_eq!(all_items.len(), 10);
    
    // Test concurrent reads
    let mut read_handles = vec![];
    
    for i in 1..=10 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            repo_clone.find_by_id(i).await.unwrap()
        });
        read_handles.push(handle);
    }
    
    // Verify all reads succeed
    for (i, handle) in read_handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_batch_operations_performance() {
    let repo = InMemoryItemRepository::new();
    
    // Create large number of items
    let mut items = vec![];
    for i in 1..=1000 {
        let item = Item {
            id: i,
            name: format!("Item {}", i),
            description: if i % 2 == 0 { Some(format!("Desc {}", i)) } else { None },
            deleted: false,
            deleted_at: None,
        };
        items.push(item);
    }
    
    // Batch create
    for item in items {
        repo.create(item).await.unwrap();
    }
    
    // Verify batch read
    let all_items = repo.find_all().await.unwrap();
    assert_eq!(all_items.len(), 1000);
    
    // Test batch update (every even ID)
    for i in (2..=1000).step_by(2) {
        let updated_item = Item {
            id: i,
            name: format!("Updated Item {}", i),
            description: Some(format!("Updated Desc {}", i)),
            deleted: false,
            deleted_at: None,
        };
        let result = repo.update(updated_item).await;
        assert!(result.is_ok());
    }
    
    // Verify updates
    for i in (2..=1000).step_by(2) {
        let found = repo.find_by_id(i).await.unwrap();
        assert!(found.is_some());
        let item = found.unwrap();
        assert_eq!(item.name, format!("Updated Item {}", i));
    }
    
    // Test batch delete (every third ID)
    let mut deleted_count = 0;
    for i in (3..=1000).step_by(3) {
        if repo.delete(i).await.is_ok() {
            deleted_count += 1;
        }
    }
    
    // Verify deletions
    let remaining_items = repo.find_all().await.unwrap();
    assert_eq!(remaining_items.len(), 1000 - deleted_count);
}

#[tokio::test]
async fn test_edge_cases_item_repository() {
    let repo = InMemoryItemRepository::new();
    
    // Test item with very long name
    let long_name = "A".repeat(10000);
    let item_long_name = Item {
        id: 1,
        name: long_name.clone(),
        description: Some("Normal description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item_long_name).await.unwrap();
    assert_eq!(created.name, long_name);
    
    // Test item with empty name
    let item_empty_name = Item {
        id: 2,
        name: "".to_string(),
        description: Some("Has description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item_empty_name).await.unwrap();
    assert_eq!(created.name, "");
    
    // Test item with special characters
    let item_special = Item {
        id: 3,
        name: "Item with 特殊文字 and émojis 🚀".to_string(),
        description: Some("Description with\nnewlines\tand\ttabs".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item_special.clone()).await.unwrap();
    assert_eq!(created.name, item_special.name);
    assert_eq!(created.description, item_special.description);
    
    // Test creating item with ID 0
    let item_zero_id = Item {
        id: 0,
        name: "Zero ID".to_string(),
        description: None,
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item_zero_id).await.unwrap();
    assert_eq!(created.id, 0);
    
    let found = repo.find_by_id(0).await.unwrap();
    assert!(found.is_some());
    
    // Test with maximum u64 ID
    let item_max_id = Item {
        id: u64::MAX,
        name: "Max ID".to_string(),
        description: None,
        deleted: false,
        deleted_at: None,
    };
    
    let created = repo.create(item_max_id).await.unwrap();
    assert_eq!(created.id, u64::MAX);
    
    let found = repo.find_by_id(u64::MAX).await.unwrap();
    assert!(found.is_some());
}