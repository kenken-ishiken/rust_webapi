mod helpers;

use domain::model::item::Item;
use domain::model::user::User;
use helpers::test_environment::TestRepositoryFactory;

/// Macro to create a test that runs with the best available environment
macro_rules! integration_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = TestRepositoryFactory::new().await;
            println!(
                "🧪 Running test '{}' with {} environment",
                stringify!($test_name),
                factory.environment_type()
            );

            let test_fn = $test_body;
            test_fn(factory).await;
        }
    };
}

/// Macro to create a test that only runs with PostgreSQL
macro_rules! postgres_only_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = TestRepositoryFactory::new().await;

            if !factory.is_postgres() {
                println!(
                    "⏭️  Skipping PostgreSQL-only test '{}' (Docker not available)",
                    stringify!($test_name)
                );
                return;
            }

            println!(
                "🧪 Running PostgreSQL-only test '{}'",
                stringify!($test_name)
            );

            let test_fn = $test_body;
            test_fn(factory).await;
        }
    };
}

/// Macro to create a test that only runs with in-memory implementations
macro_rules! in_memory_only_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let factory = TestRepositoryFactory::with_environment(
                helpers::test_environment::TestEnvironment::InMemory,
            );

            println!(
                "🧪 Running in-memory-only test '{}'",
                stringify!($test_name)
            );

            let test_fn = $test_body;
            test_fn(factory).await;
        }
    };
}

integration_test!(
    test_item_crud_operations,
    |factory: TestRepositoryFactory| async move {
        let repo = factory.create_item_repository();

        // Test data
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        // 1. Create item
        let created_item = repo.create(item.clone()).await.unwrap();
        assert_eq!(created_item.id, item.id);
        assert_eq!(created_item.name, item.name);
        assert_eq!(created_item.description, item.description);

        // 2. Find by ID
        let found_item = repo.find_by_id(1).await.unwrap();
        assert!(found_item.is_some());
        let found_item = found_item.unwrap();
        assert_eq!(found_item.id, item.id);
        assert_eq!(found_item.name, item.name);

        // 3. Find all
        let all_items = repo.find_all().await.unwrap();
        assert_eq!(all_items.len(), 1);
        assert_eq!(all_items[0].id, item.id);

        // 4. Update item
        let updated_item = Item {
            id: 1,
            name: "Updated Item".to_string(),
            description: Some("Updated Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        let result = repo.update(updated_item.clone()).await.unwrap();
        assert_eq!(result.name, "Updated Item");
        assert_eq!(result.description, Some("Updated Description".to_string()));

        // 5. Logical delete
        repo.logical_delete(1).await.unwrap();

        // Verify item is not in active list
        let all_items_after_delete = repo.find_all().await.unwrap();
        assert_eq!(all_items_after_delete.len(), 0);

        // Verify item is in deleted list
        let deleted_items = repo.find_deleted().await.unwrap();
        assert_eq!(deleted_items.len(), 1);
        assert_eq!(deleted_items[0].id, 1);
        assert!(deleted_items[0].deleted);

        // 6. Restore item
        repo.restore(1).await.unwrap();

        // Verify item is back in active list
        let all_items_after_restore = repo.find_all().await.unwrap();
        assert_eq!(all_items_after_restore.len(), 1);
        assert_eq!(all_items_after_restore[0].id, 1);
        assert!(!all_items_after_restore[0].deleted);

        // 7. Physical delete
        repo.physical_delete(1).await.unwrap();

        // Verify item is completely gone
        let all_items_final = repo.find_all().await.unwrap();
        assert_eq!(all_items_final.len(), 0);

        let deleted_items_final = repo.find_deleted().await.unwrap();
        assert_eq!(deleted_items_final.len(), 0);
    }
);

integration_test!(
    test_user_crud_operations,
    |factory: TestRepositoryFactory| async move {
        let repo = factory.create_user_repository();

        // Test data
        let user = User {
            id: 1,
            username: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // 1. Create user
        let created_user = repo.create(user.clone()).await;
        assert_eq!(created_user.id, user.id);
        assert_eq!(created_user.username, user.username);
        assert_eq!(created_user.email, user.email);

        // 2. Find by ID
        let found_user = repo.find_by_id(1).await;
        assert!(found_user.is_some());
        let found_user = found_user.unwrap();
        assert_eq!(found_user.id, user.id);
        assert_eq!(found_user.username, user.username);
        assert_eq!(found_user.email, user.email);

        // 3. Find all
        let all_users = repo.find_all().await;
        assert_eq!(all_users.len(), 1);
        assert_eq!(all_users[0].id, user.id);

        // 4. Update user
        let updated_user = User {
            id: 1,
            username: "Updated User".to_string(),
            email: "updated@example.com".to_string(),
        };

        let result = repo.update(updated_user.clone()).await;
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.username, "Updated User");
        assert_eq!(result.email, "updated@example.com");

        // 5. Delete user
        let deleted = repo.delete(1).await;
        assert!(deleted);

        // Verify user is deleted
        let all_users_after_delete = repo.find_all().await;
        assert_eq!(all_users_after_delete.len(), 0);

        let not_found = repo.find_by_id(1).await;
        assert!(not_found.is_none());
    }
);

integration_test!(
    test_batch_operations,
    |factory: TestRepositoryFactory| async move {
        let repo = factory.create_item_repository();

        // Create multiple items
        for i in 1..=5 {
            let item = Item {
                id: i,
                name: format!("Item {}", i),
                description: Some(format!("Description {}", i)),
                deleted: false,
                deleted_at: None,
            };
            repo.create(item).await.unwrap();
        }

        // Verify all items created
        let all_items = repo.find_all().await.unwrap();
        assert_eq!(all_items.len(), 5);

        // Batch logical delete
        let ids_to_delete = vec![1, 3, 5];
        let deleted_ids = repo
            .batch_delete(ids_to_delete.clone(), false)
            .await
            .unwrap();
        assert_eq!(deleted_ids.len(), 3);
        assert_eq!(deleted_ids, ids_to_delete);

        // Verify remaining active items
        let active_items = repo.find_all().await.unwrap();
        assert_eq!(active_items.len(), 2);
        let active_ids: Vec<u64> = active_items.iter().map(|item| item.id).collect();
        assert!(active_ids.contains(&2));
        assert!(active_ids.contains(&4));

        // Verify deleted items
        let deleted_items = repo.find_deleted().await.unwrap();
        assert_eq!(deleted_items.len(), 3);
        let deleted_ids: Vec<u64> = deleted_items.iter().map(|item| item.id).collect();
        assert!(deleted_ids.contains(&1));
        assert!(deleted_ids.contains(&3));
        assert!(deleted_ids.contains(&5));

        // Batch physical delete
        let remaining_ids = vec![2, 4];
        let physically_deleted_ids = repo
            .batch_delete(remaining_ids.clone(), true)
            .await
            .unwrap();
        assert_eq!(physically_deleted_ids.len(), 2);
        assert_eq!(physically_deleted_ids, remaining_ids);

        // Verify all active items are gone
        let final_active_items = repo.find_all().await.unwrap();
        assert_eq!(final_active_items.len(), 0);

        // Deleted items should still be there (only logically deleted ones)
        let final_deleted_items = repo.find_deleted().await.unwrap();
        assert_eq!(final_deleted_items.len(), 3);
    }
);

postgres_only_test!(
    test_postgres_specific_features,
    |factory: TestRepositoryFactory| async move {
        // This test only runs when PostgreSQL is available
        let repo = factory.create_item_repository();

        // Test PostgreSQL-specific features like deletion logs
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        repo.create(item).await.unwrap();
        repo.logical_delete(1).await.unwrap();

        // Check deletion logs (this feature might only be available in PostgreSQL implementation)
        let logs = repo.get_deletion_logs(Some(1)).await.unwrap();
        // The exact behavior depends on the PostgreSQL implementation
        // This is just an example of PostgreSQL-specific testing
        println!("Deletion logs: {:?}", logs);
    }
);

in_memory_only_test!(
    test_in_memory_specific_features,
    |factory: TestRepositoryFactory| async move {
        // This test only runs with in-memory implementations
        let repo = factory.create_item_repository();

        // Test in-memory specific behavior
        let item = Item {
            id: 1,
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            deleted: false,
            deleted_at: None,
        };

        repo.create(item).await.unwrap();

        // In-memory implementations might have different characteristics
        // For example, they might not persist deletion logs
        let logs = repo.get_deletion_logs(Some(1)).await.unwrap();
        assert_eq!(logs.len(), 0); // In-memory implementation doesn't track logs
    }
);

#[tokio::test]
async fn test_parallel_execution() {
    // Test that multiple tests can run in parallel without interfering with each other
    let factory1 = TestRepositoryFactory::new().await;
    let factory2 = TestRepositoryFactory::new().await;

    let repo1 = factory1.create_item_repository();
    let repo2 = factory2.create_item_repository();

    // Run operations in parallel
    let (result1, result2) = tokio::join!(
        async {
            let item = Item {
                id: 1,
                name: "Item 1".to_string(),
                description: Some("Description 1".to_string()),
                deleted: false,
                deleted_at: None,
            };
            repo1.create(item).await.unwrap();
            repo1.find_all().await.unwrap()
        },
        async {
            let item = Item {
                id: 2,
                name: "Item 2".to_string(),
                description: Some("Description 2".to_string()),
                deleted: false,
                deleted_at: None,
            };
            repo2.create(item).await.unwrap();
            repo2.find_all().await.unwrap()
        }
    );

    // Each repository should only see its own items (for in-memory implementations)
    // For PostgreSQL, they might share the same database, so we just check they both work
    assert_eq!(result1.len(), 1);
    assert_eq!(result1[0].id, 1);

    assert_eq!(result2.len(), 1);
    assert_eq!(result2[0].id, 2);

    println!("✅ Parallel execution test completed successfully");
}
