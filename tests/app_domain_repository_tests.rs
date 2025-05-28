use domain::model::item::Item;
use rust_webapi::app_domain::repository::item_repository::{ItemRepository, MockItemRepository};
use rust_webapi::app_domain::repository::item_repository::predicate::*;
use rust_webapi::infrastructure::error::AppError;

#[tokio::test]
async fn test_mock_item_repository_find_all() {
    let mut mock_repo = MockItemRepository::new();
    
    let expected_items = vec![
        Item {
            id: 1,
            name: "Item 1".to_string(),
            description: Some("Description 1".to_string()),
            deleted: false,
            deleted_at: None,
        },
        Item {
            id: 2,
            name: "Item 2".to_string(),
            description: None,
            deleted: false,
            deleted_at: None,
        },
    ];
    
    mock_repo
        .expect_find_all()
        .times(1)
    .returning({
        let items = expected_items.clone();
        move || Ok(items.clone())
    });

    let result = mock_repo.find_all().await.unwrap();
    
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, 1);
    assert_eq!(result[0].name, "Item 1");
    assert_eq!(result[1].id, 2);
    assert_eq!(result[1].name, "Item 2");
}

#[tokio::test]
async fn test_mock_item_repository_find_by_id() {
    let mut mock_repo = MockItemRepository::new();
    
    let expected_item = Item {
        id: 1,
        name: "Found Item".to_string(),
        description: Some("Found Description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    mock_repo
        .expect_find_by_id()
        .with(eq(1u64))
        .times(1)
    .returning({
            let item = expected_item.clone();
            move |_| Ok(Some(item.clone()))
        });
    
    mock_repo
        .expect_find_by_id()
        .with(eq(999u64))
        .times(1)
    .returning(|_| Ok(None));
    
    // Test found item
    let result = mock_repo.find_by_id(1).await.unwrap();
    assert!(result.is_some());
    let found = result.unwrap();
    assert_eq!(found.id, 1);
    assert_eq!(found.name, "Found Item");
    
    // Test not found item
    let result = mock_repo.find_by_id(999).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mock_item_repository_create() {
    let mut mock_repo = MockItemRepository::new();
    
    let input_item = Item {
        id: 1,
        name: "New Item".to_string(),
        description: Some("New Description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    
    mock_repo
        .expect_create()
        .with(function(move |item: &Item| {
            item.id == 1 && item.name == "New Item"
        }))
        .times(1)
        .returning(move |item| Ok(item));

    let result = mock_repo.create(input_item).await.unwrap();
    
    assert_eq!(result.id, 1);
    assert_eq!(result.name, "New Item");
    assert_eq!(result.description, Some("New Description".to_string()));
}

#[tokio::test]
async fn test_mock_item_repository_update() {
    let mut mock_repo = MockItemRepository::new();
    
    let update_item = Item {
        id: 1,
        name: "Updated Item".to_string(),
        description: Some("Updated Description".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    // Test successful update
    mock_repo
        .expect_update()
        .with(function(move |item: &Item| item.id == 1))
        .times(1)
    .returning(move |item| Ok(item));

    let result = mock_repo.update(update_item.clone()).await.unwrap();
    assert_eq!(result.name, "Updated Item");
    
    // Test failed update (item not found)
    let non_existing_item = Item {
        id: 999,
        name: "Non-existing".to_string(),
        description: None,
        deleted: false,
        deleted_at: None,
    };
    
    mock_repo
        .expect_update()
        .with(function(move |item: &Item| item.id == 999))
        .times(1)
        .returning(|_| Err(AppError::NotFound("err".to_string())));

    let result = mock_repo.update(non_existing_item).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_item_repository_delete() {
    let mut mock_repo = MockItemRepository::new();
    
    // Test successful delete
    mock_repo
        .expect_delete()
        .with(eq(1u64))
        .times(1)
        .returning(|_| Ok(()));

    mock_repo.delete(1).await.unwrap();

    // Test failed delete (item not found)
    mock_repo
        .expect_delete()
        .with(eq(999u64))
        .times(1)
        .returning(|_| Err(AppError::NotFound("err".to_string())));

    let result = mock_repo.delete(999).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_repository_call_count_verification() {
    let mut mock_repo = MockItemRepository::new();
    
    // Set up expectations with specific call counts
    mock_repo
        .expect_find_all()
        .times(3)
        .returning(|| Ok(vec![]));

    mock_repo
        .expect_find_by_id()
        .with(eq(1u64))
        .times(2)
        .returning(|_| Ok(None));
    
    // Call the methods the expected number of times
    for _ in 0..3 {
        mock_repo.find_all().await.unwrap();
    }

    for _ in 0..2 {
        mock_repo.find_by_id(1).await.unwrap();
    }
    
    // If we didn't call the expected number of times, the test would fail when the mock is dropped
}

#[tokio::test]
async fn test_mock_repository_parameter_validation() {
    let mut mock_repo = MockItemRepository::new();
    
    // Test with specific parameter constraints
    mock_repo
        .expect_find_by_id()
        .with(function(|id: &u64| *id > 0 && *id < 100))
        .times(1)
        .returning(|_| Ok(None));

    // This should succeed (ID is in valid range)
    let result = mock_repo.find_by_id(50).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mock_repository_sequence_operations() {
    let mut mock_repo = MockItemRepository::new();
    
    let item = Item {
        id: 1,
        name: "Sequence Test".to_string(),
        description: Some("Testing sequence".to_string()),
        deleted: false,
        deleted_at: None,
    };
    
    // Set up a sequence of operations
    mock_repo
        .expect_create()
        .times(1)
        .returning(move |item| Ok(item));

    mock_repo
        .expect_find_by_id()
        .with(eq(1u64))
        .times(1)
        .returning({
            let item = item.clone();
            move |_| Ok(Some(item.clone()))
        });

    mock_repo
        .expect_update()
        .times(1)
        .returning(move |item| Ok(item));

    mock_repo
        .expect_delete()
        .with(eq(1u64))
        .times(1)
        .returning(|_| Ok(()));

    // Execute the sequence
    let created = mock_repo.create(item.clone()).await.unwrap();
    assert_eq!(created.id, 1);

    let found = mock_repo.find_by_id(1).await.unwrap();
    assert!(found.is_some());

    let updated_item = Item {
        id: 1,
        name: "Updated Sequence Test".to_string(),
        description: Some("Updated testing sequence".to_string()),
        deleted: false,
        deleted_at: None,
    };

    let updated = mock_repo.update(updated_item).await.unwrap();
    assert_eq!(updated.name, "Updated Sequence Test");

    mock_repo.delete(1).await.unwrap();
}

#[tokio::test]
async fn test_mock_repository_error_simulation() {
    let mut mock_repo = MockItemRepository::new();
    
    // Simulate different scenarios that might occur in real implementations
    
    // Empty result scenario
    mock_repo
        .expect_find_all()
        .times(1)
        .returning(|| Ok(vec![]));

    let result = mock_repo.find_all().await.unwrap();
    assert_eq!(result.len(), 0);

    // Item not found scenario
    mock_repo
        .expect_find_by_id()
        .with(always())
        .times(1)
        .returning(|_| Ok(None));

    let result = mock_repo.find_by_id(1).await.unwrap();
    assert!(result.is_none());

    // Update failure scenario
    mock_repo
        .expect_update()
        .with(always())
        .times(1)
        .returning(|_| Err(AppError::NotFound("err".to_string())));

    let item = Item {
        id: 1,
        name: "Test".to_string(),
        description: None,
        deleted: false,
        deleted_at: None,
    };

    let result = mock_repo.update(item).await;
    assert!(result.is_err());

    // Delete failure scenario
    mock_repo
        .expect_delete()
        .with(always())
        .times(1)
        .returning(|_| Err(AppError::NotFound("err".to_string())));

    let result = mock_repo.delete(1).await;
    assert!(result.is_err());
}