mod helpers;

use domain::model::item::Item;
use rust_webapi::app_domain::repository::item_repository::ItemRepository;
use rust_webapi::app_domain::service::deletion_service::{
    DeleteKind, DeletionError, DeletionStrategy, ItemDeletionStrategy,
};
use rust_webapi::infrastructure::repository::item_repository::InMemoryItemRepository;
use std::sync::Arc;

/// Item DeletionStrategy のContract Test
#[tokio::test]
async fn test_item_deletion_strategy_logical_deletion() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // テストアイテムを作成
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
        deleted: false,
        deleted_at: None,
    };

    let created = repository
        .create(item)
        .await
        .expect("Failed to create item");
    assert_eq!(created.id, 1);

    // 論理削除を実行
    let result = strategy.delete(1, DeleteKind::Logical).await;
    assert!(result.is_ok(), "Logical deletion should succeed");

    // アイテムが論理削除されていることを確認
    let deleted_items = repository
        .find_deleted()
        .await
        .expect("Find deleted should not fail");
    assert_eq!(deleted_items.len(), 1, "Should have one deleted item");
    let item = &deleted_items[0];
    assert_eq!(item.id, 1, "Deleted item should have correct ID");
    assert!(item.deleted, "Item should be logically deleted");
    assert!(item.deleted_at.is_some(), "deleted_at should be set");
}

#[tokio::test]
async fn test_item_deletion_strategy_physical_deletion() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // テストアイテムを作成
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
        deleted: false,
        deleted_at: None,
    };

    repository
        .create(item)
        .await
        .expect("Failed to create item");

    // 物理削除を実行
    let result = strategy.delete(1, DeleteKind::Physical).await;
    assert!(result.is_ok(), "Physical deletion should succeed");

    // アイテムが物理削除されていることを確認
    let found = repository
        .find_by_id(1)
        .await
        .expect("Find should not fail");
    assert!(found.is_none(), "Item should be physically deleted");
}

#[tokio::test]
async fn test_item_deletion_strategy_restore() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // テストアイテムを作成
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
        deleted: false,
        deleted_at: None,
    };

    repository
        .create(item)
        .await
        .expect("Failed to create item");

    // 論理削除を実行
    let result = strategy.delete(1, DeleteKind::Logical).await;
    assert!(result.is_ok(), "Logical deletion should succeed");

    // 復元を実行
    let result = strategy.delete(1, DeleteKind::Restore).await;
    assert!(result.is_ok(), "Restore should succeed");

    // アイテムが復元されていることを確認
    let found = repository
        .find_by_id(1)
        .await
        .expect("Find should not fail");
    assert!(found.is_some(), "Restored item should be found");
    let item = found.unwrap();
    assert!(!item.deleted, "Item should be restored");
    assert!(item.deleted_at.is_none(), "deleted_at should be cleared");
}

#[tokio::test]
async fn test_item_deletion_strategy_nonexistent_entity() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // 存在しないアイテムの削除を試行
    let result = strategy.delete(999, DeleteKind::Logical).await;
    assert!(result.is_err(), "Deleting nonexistent entity should fail");
    assert!(matches!(result.unwrap_err(), DeletionError::NotFound(_)));

    // 物理削除を試行
    let result = strategy.delete(999, DeleteKind::Physical).await;
    assert!(result.is_err(), "Deleting nonexistent entity should fail");
    assert!(matches!(result.unwrap_err(), DeletionError::NotFound(_)));

    // 復元を試行
    let result = strategy.delete(999, DeleteKind::Restore).await;
    assert!(result.is_err(), "Restoring nonexistent entity should fail");
    assert!(matches!(result.unwrap_err(), DeletionError::NotFound(_)));
}

#[tokio::test]
async fn test_item_deletion_strategy_duplicate_deletion() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // テストアイテムを作成
    let item = Item {
        id: 1,
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
        deleted: false,
        deleted_at: None,
    };

    repository
        .create(item)
        .await
        .expect("Failed to create item");

    // 1回目の論理削除
    let result = strategy.delete(1, DeleteKind::Logical).await;
    assert!(result.is_ok(), "First logical deletion should succeed");

    // 2回目の論理削除（既に削除済み）
    let result = strategy.delete(1, DeleteKind::Logical).await;
    // InMemoryRepositoryの実装では既に削除済みでもエラーにならない（冪等性）
    match result {
        Ok(_) => {
            // 冪等性を保証する実装の場合
            let deleted_items = repository
                .find_deleted()
                .await
                .expect("Find deleted should not fail");
            assert_eq!(deleted_items.len(), 1, "Should have one deleted item");
            assert!(deleted_items[0].deleted, "Item should remain deleted");
        }
        Err(DeletionError::NotFound(_)) => {
            // 既に削除済みとしてエラーを返す実装の場合
            // これも有効な実装
        }
        Err(_) => panic!("Unexpected error type for duplicate deletion"),
    }
}

/// パフォーマンステスト: 複数削除の性能確認
#[tokio::test]
async fn test_item_deletion_strategy_performance() {
    use std::time::Instant;

    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // 複数のアイテムを作成
    let mut entity_ids = Vec::new();
    for i in 1..=10 {
        let item = Item {
            id: i,
            name: format!("Performance Test Item {}", i),
            description: Some(format!("Performance test item {}", i)),
            deleted: false,
            deleted_at: None,
        };
        let created = repository
            .create(item)
            .await
            .expect("Failed to create test item");
        entity_ids.push(created.id);
    }

    // 削除のパフォーマンスを測定
    let start = Instant::now();
    for id in entity_ids {
        let result = strategy.delete(id, DeleteKind::Logical).await;
        assert!(result.is_ok(), "Deletion should succeed for id {}", id);
    }
    let duration = start.elapsed();

    // パフォーマンス基準（10個の削除が1秒以内）
    assert!(
        duration.as_secs() < 1,
        "Deletion performance too slow: {:?}",
        duration
    );

    println!("Deleted 10 items in {:?}", duration);
}

/// 統合テスト: DeletionStrategyのエラー一貫性
#[tokio::test]
async fn test_deletion_strategy_error_consistency() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // 存在しないエンティティに対する削除
    let logical_result = strategy.delete(999, DeleteKind::Logical).await;
    let physical_result = strategy.delete(999, DeleteKind::Physical).await;
    let restore_result = strategy.delete(999, DeleteKind::Restore).await;

    // すべてNotFoundエラーになることを確認
    assert!(logical_result.is_err());
    assert!(physical_result.is_err());
    assert!(restore_result.is_err());
    assert!(matches!(
        logical_result.unwrap_err(),
        DeletionError::NotFound(_)
    ));
    assert!(matches!(
        physical_result.unwrap_err(),
        DeletionError::NotFound(_)
    ));
    assert!(matches!(
        restore_result.unwrap_err(),
        DeletionError::NotFound(_)
    ));
}

/// 境界値テスト: 削除操作の境界条件
#[tokio::test]
async fn test_deletion_strategy_boundary_conditions() {
    let repository = Arc::new(InMemoryItemRepository::new());
    let strategy = ItemDeletionStrategy::new(repository.clone());

    // ID 0の処理
    let result = strategy.delete(0, DeleteKind::Logical).await;
    assert!(result.is_err(), "ID 0 should fail");

    // 最大ID値の処理
    let result = strategy.delete(u64::MAX, DeleteKind::Logical).await;
    assert!(result.is_err(), "MAX ID should fail");
}
