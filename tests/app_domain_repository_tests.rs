mod helpers;

use helpers::mock_builder::{ItemMockBuilder, MockBuilder, TestDataFactory, TestAssertions};
use rust_webapi::app_domain::repository::item_repository::ItemRepository;

#[tokio::test]
async fn test_mock_item_repository_find_all() {
    // テストデータの準備
    let expected_items = TestDataFactory::create_items(2);
    
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_find_all_returning(expected_items.clone())
        .build();

    // テスト実行
    let result = mock_repo.find_all().await.unwrap();

    // アサーション
    assert_eq!(result.len(), 2);
    TestAssertions::assert_item_eq(&result[0], &expected_items[0]);
    TestAssertions::assert_item_eq(&result[1], &expected_items[1]);
}

#[tokio::test]
async fn test_mock_item_repository_find_all_empty() {
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_find_all_empty()
        .build();

    // テスト実行
    let result = mock_repo.find_all().await.unwrap();

    // アサーション
    assert_eq!(result.len(), 0);
}

#[tokio::test]
async fn test_mock_item_repository_find_by_id() {
    // テストデータの準備
    let expected_item = TestDataFactory::create_item(1, "Found Item");
    
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_find_by_id_returning(1, Some(expected_item.clone()))
        .with_find_by_id_not_found(999)
        .build();

    // テスト実行 - アイテムが見つかる場合
    let result = mock_repo.find_by_id(1).await.unwrap();
    assert!(result.is_some());
    TestAssertions::assert_item_eq(&result.unwrap(), &expected_item);

    // テスト実行 - アイテムが見つからない場合
    let result = mock_repo.find_by_id(999).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mock_item_repository_create() {
    // テストデータの準備
    let input_item = TestDataFactory::create_item(1, "New Item");
    
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_create_success_when(move |item| {
            item.id == 1 && item.name == "New Item"
        })
        .build();

    // テスト実行
    let result = mock_repo.create(input_item.clone()).await.unwrap();

    // アサーション
    TestAssertions::assert_item_eq(&result, &input_item);
}

#[tokio::test]
async fn test_mock_item_repository_update_success() {
    // テストデータの準備
    let update_item = TestDataFactory::create_item(1, "Updated Item");
    
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_update_success()
        .build();

    // テスト実行
    let result = mock_repo.update(update_item.clone()).await.unwrap();
    TestAssertions::assert_item_eq(&result, &update_item);
}

#[tokio::test]
async fn test_mock_item_repository_update_not_found() {
    // テストデータの準備
    let non_existing_item = TestDataFactory::create_item(999, "Non-existing");
    
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_update_not_found(999)
        .build();

    // テスト実行
    let result = mock_repo.update(non_existing_item).await;
    
    // アサーション
    TestAssertions::assert_app_error_not_found(result.map(|_| ()));
}

#[tokio::test]
async fn test_mock_item_repository_delete_success() {
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_logical_delete_success(1)
        .build();

    // テスト実行
    let result = mock_repo.logical_delete(1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_item_repository_delete_not_found() {
    // MockBuilderを使用してモック設定
    let mock_repo = ItemMockBuilder::new()
        .with_logical_delete_not_found(999)
        .build();

    // テスト実行
    let result = mock_repo.logical_delete(999).await;
    
    // アサーション
    TestAssertions::assert_app_error_not_found(result);
}

#[tokio::test]
async fn test_mock_repository_multiple_calls() {
    // テストデータの準備
    let items = TestDataFactory::create_items(2);
    
    // MockBuilderを使用してモック設定（複数回呼び出し）
    let mock_repo = ItemMockBuilder::new()
        .with_find_all_times(3, items.clone())
        .with_find_by_id_any_returning(None)
        .build();

    // テスト実行 - find_allを3回呼び出し
    for _ in 0..3 {
        let result = mock_repo.find_all().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    // テスト実行 - find_by_idを1回呼び出し
    let result = mock_repo.find_by_id(1).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_mock_repository_sequence_operations() {
    // テストデータの準備
    let item = TestDataFactory::create_item(1, "Sequence Test");
    let updated_item = TestDataFactory::create_item(1, "Updated Sequence Test");
    
    // MockBuilderを使用してモック設定（シーケンス操作）
    let mock_repo = ItemMockBuilder::new()
        .with_create_success()
        .with_find_by_id_returning(1, Some(item.clone()))
        .with_update_success()
        .with_logical_delete_success(1)
        .build();

    // テスト実行 - 一連の操作
    let created = mock_repo.create(item.clone()).await.unwrap();
    TestAssertions::assert_item_eq(&created, &item);

    let found = mock_repo.find_by_id(1).await.unwrap();
    assert!(found.is_some());
    TestAssertions::assert_item_eq(&found.unwrap(), &item);

    let updated = mock_repo.update(updated_item.clone()).await.unwrap();
    TestAssertions::assert_item_eq(&updated, &updated_item);

    let delete_result = mock_repo.logical_delete(1).await;
    assert!(delete_result.is_ok());
}

#[tokio::test]
async fn test_mock_repository_error_scenarios() {
    // MockBuilderを使用してモック設定（エラーシナリオ）
    let mock_repo = ItemMockBuilder::new()
        .with_find_all_empty()
        .with_find_by_id_any_returning(None)
        .with_update_not_found(1)
        .with_logical_delete_not_found(1)
        .build();

    // 空の結果
    let result = mock_repo.find_all().await.unwrap();
    assert_eq!(result.len(), 0);

    // アイテムが見つからない
    let result = mock_repo.find_by_id(1).await.unwrap();
    assert!(result.is_none());

    // 更新失敗
    let item = TestDataFactory::create_item(1, "Test");
    let result = mock_repo.update(item).await;
    TestAssertions::assert_app_error_not_found(result.map(|_| ()));

    // 削除失敗
    let result = mock_repo.logical_delete(1).await;
    TestAssertions::assert_app_error_not_found(result);
} 