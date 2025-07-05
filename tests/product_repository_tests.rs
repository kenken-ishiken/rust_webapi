mod helpers;

use helpers::postgres::PostgresContainer;
use rust_webapi::infrastructure::repository::postgres::product_repository::PostgresProductRepository;
use rust_webapi::app_domain::repository::product_repository::ProductRepository;
use rust_webapi::app_domain::model::product::{Product, ProductStatus, ProductError, Price, Inventory, Dimensions, ShippingInfo};
use rust_decimal::Decimal;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_postgres_product_repository_basic_crud() {
    // Create a PostgreSQL container
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Test data
    let mut product = Product::new(
        "test-product-1".to_string(),
        "Test Product".to_string(),
        "SKU-001".to_string(),
        ProductStatus::Active,
    ).unwrap();
    
    product.description = Some("Test Description".to_string());
    product.brand = Some("Test Brand".to_string());
    product.category_id = Some("cat01".to_string());
    product.weight = Some(Decimal::new(1000, 2)); // 10.00
    product.dimensions = Some(Dimensions::new(
        Decimal::new(10, 0),
        Decimal::new(20, 0),
        Decimal::new(5, 0)
    ).unwrap());
    product.shipping_info = ShippingInfo {
        shipping_class: "standard".to_string(),
        free_shipping: false,
        shipping_fee: Decimal::new(500, 2),
    };

    // 1. Test product creation
    let created_product = repo.create(product.clone()).await.unwrap();
    assert_eq!(created_product.id, product.id);
    assert_eq!(created_product.name, product.name);
    assert_eq!(created_product.sku, product.sku);
    assert_eq!(created_product.description, product.description);
    assert_eq!(created_product.brand, product.brand);
    assert_eq!(created_product.category_id, product.category_id);
    assert_eq!(created_product.weight, product.weight);

    // 2. Test finding product by ID
    let found_product = repo.find_by_id("test-product-1").await;
    assert!(found_product.is_some());
    let found_product = found_product.unwrap();
    assert_eq!(found_product.id, "test-product-1");
    assert_eq!(found_product.name, "Test Product");
    assert_eq!(found_product.sku, "SKU-001");

    // 3. Test finding product by SKU
    let found_by_sku = repo.find_by_sku("SKU-001").await;
    assert!(found_by_sku.is_some());
    let found_by_sku = found_by_sku.unwrap();
    assert_eq!(found_by_sku.id, "test-product-1");

    // 4. Test finding a non-existent product
    let not_found = repo.find_by_id("non-existent").await;
    assert!(not_found.is_none());

    // 5. Test updating a product
    let mut updated_product = found_product.clone();
    updated_product.name = "Updated Product".to_string();
    updated_product.description = Some("Updated Description".to_string());
    updated_product.updated_at = Utc::now();

    let update_result = repo.update(updated_product.clone()).await;
    assert!(update_result.is_ok());

    let updated = repo.find_by_id("test-product-1").await.unwrap();
    assert_eq!(updated.name, "Updated Product");
    assert_eq!(updated.description, Some("Updated Description".to_string()));

    // 6. Test SKU existence check
    assert!(repo.exists_by_sku("SKU-001", None).await);
    assert!(!repo.exists_by_sku("SKU-999", None).await);
    assert!(!repo.exists_by_sku("SKU-001", Some("test-product-1")).await); // Exclude self

    // 7. Test deleting a product
    let delete_result = repo.delete("test-product-1").await;
    assert!(delete_result.is_ok());

    // Verify deletion
    let deleted_product = repo.find_by_id("test-product-1").await;
    assert!(deleted_product.is_none());
}

#[tokio::test]
async fn test_postgres_product_repository_price_operations() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Create a product first
    let product = Product::new(
        "test-product-2".to_string(),
        "Price Test Product".to_string(),
        "SKU-002".to_string(),
        ProductStatus::Active,
    ).unwrap();

    repo.create(product).await.unwrap();

    // Test price operations
    let price = Price {
        selling_price: Decimal::new(1000, 2), // 10.00
        list_price: Some(Decimal::new(1200, 2)), // 12.00
        discount_price: Some(Decimal::new(900, 2)), // 9.00
        currency: "JPY".to_string(),
        tax_included: true,
        effective_from: None,
        effective_until: None,
    };

    // Update price
    let updated_price = repo.update_price("test-product-2", price.clone()).await;
    assert!(updated_price.is_ok());
    let updated_price = updated_price.unwrap();
    assert_eq!(updated_price.selling_price, price.selling_price);
    assert_eq!(updated_price.list_price, price.list_price);
    assert_eq!(updated_price.discount_price, price.discount_price);

    // Get current price
    let current_price = repo.get_current_price("test-product-2").await;
    assert!(current_price.is_some());
    let current_price = current_price.unwrap();
    assert_eq!(current_price.selling_price, Decimal::new(1000, 2));
}

#[tokio::test]
async fn test_postgres_product_repository_inventory_operations() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Create a product first
    let product = Product::new(
        "test-product-3".to_string(),
        "Inventory Test Product".to_string(),
        "SKU-003".to_string(),
        ProductStatus::Active,
    ).unwrap();

    repo.create(product).await.unwrap();

    // Test inventory operations
    let inventory = Inventory {
        quantity: 100,
        reserved_quantity: 10,
        alert_threshold: Some(5),
        track_inventory: true,
        allow_backorder: false,
    };

    // Update inventory
    let updated_inventory = repo.update_inventory("test-product-3", inventory.clone()).await;
    assert!(updated_inventory.is_ok());
    let updated_inventory = updated_inventory.unwrap();
    assert_eq!(updated_inventory.quantity, 100);
    assert_eq!(updated_inventory.reserved_quantity, 10);
    assert_eq!(updated_inventory.alert_threshold, Some(5));

    // Get current inventory
    let current_inventory = repo.get_inventory("test-product-3").await;
    assert!(current_inventory.is_some());
    let current_inventory = current_inventory.unwrap();
    assert_eq!(current_inventory.quantity, 100);
    assert_eq!(current_inventory.track_inventory, true);
}

#[tokio::test]
async fn test_postgres_product_repository_tags_and_attributes() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Create a product first
    let product = Product::new(
        "test-product-4".to_string(),
        "Tags Test Product".to_string(),
        "SKU-004".to_string(),
        ProductStatus::Active,
    ).unwrap();

    repo.create(product).await.unwrap();

    // Test tags operations
    let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
    let add_result = repo.add_tags("test-product-4", tags.clone()).await;
    assert!(add_result.is_ok());

    let retrieved_tags = repo.get_tags("test-product-4").await;
    assert_eq!(retrieved_tags.len(), 3);
    assert!(retrieved_tags.contains(&"tag1".to_string()));
    assert!(retrieved_tags.contains(&"tag2".to_string()));
    assert!(retrieved_tags.contains(&"tag3".to_string()));

    // Test replace tags
    let new_tags = vec!["newtag1".to_string(), "newtag2".to_string()];
    let replace_result = repo.replace_tags("test-product-4", new_tags.clone()).await;
    assert!(replace_result.is_ok());

    let replaced_tags = repo.get_tags("test-product-4").await;
    assert_eq!(replaced_tags.len(), 2);
    assert!(replaced_tags.contains(&"newtag1".to_string()));
    assert!(replaced_tags.contains(&"newtag2".to_string()));

    // Test attributes operations
    let mut attributes = HashMap::new();
    attributes.insert("color".to_string(), "red".to_string());
    attributes.insert("size".to_string(), "large".to_string());
    attributes.insert("material".to_string(), "cotton".to_string());

    let set_result = repo.set_attributes("test-product-4", attributes.clone()).await;
    assert!(set_result.is_ok());

    let retrieved_attributes = repo.get_attributes("test-product-4").await;
    assert_eq!(retrieved_attributes.len(), 3);
    assert_eq!(retrieved_attributes.get("color"), Some(&"red".to_string()));
    assert_eq!(retrieved_attributes.get("size"), Some(&"large".to_string()));
    assert_eq!(retrieved_attributes.get("material"), Some(&"cotton".to_string()));
}

#[tokio::test]
async fn test_postgres_product_repository_search() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Create multiple products for search testing
    let products = vec![
        ("test-product-5", "Search Product 1", "SKU-005", "cat01"),
        ("test-product-6", "Search Product 2", "SKU-006", "cat01"),
        ("test-product-7", "Different Product", "SKU-007", "cat02"),
        ("test-product-8", "Another Product", "SKU-008", "cat01"),
    ];

    for (id, name, sku, category) in products {
        let mut product = Product::new(
            id.to_string(),
            name.to_string(),
            sku.to_string(),
            ProductStatus::Active,
        ).unwrap();
        product.category_id = Some(category.to_string());
        repo.create(product).await.unwrap();
    }

    // Test text search
    let search_results = repo.search(
        "Search",
        None,
        None,
        None,
        None,
        false,
        None,
        None,
    ).await;
    assert_eq!(search_results.len(), 2);

    // Test category filter
    let category_results = repo.search(
        "",
        Some("cat01"),
        None,
        None,
        None,
        false,
        None,
        None,
    ).await;
    assert_eq!(category_results.len(), 3);

    // Test limit
    let limited_results = repo.search(
        "",
        Some("cat01"),
        None,
        None,
        None,
        false,
        Some(2),
        None,
    ).await;
    assert_eq!(limited_results.len(), 2);

    // Test offset
    let offset_results = repo.search(
        "",
        Some("cat01"),
        None,
        None,
        None,
        false,
        Some(2),
        Some(1),
    ).await;
    assert_eq!(offset_results.len(), 2);
}

#[tokio::test]
async fn test_postgres_product_repository_stock_queries() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Create products with different stock levels
    let products_data = vec![
        ("test-product-9", "Low Stock Product", "SKU-009", 5, 0),
        ("test-product-10", "Out of Stock Product", "SKU-010", 0, 0),
        ("test-product-11", "High Stock Product", "SKU-011", 100, 5),
        ("test-product-12", "Reserved Stock Product", "SKU-012", 10, 15), // More reserved than available
    ];

    for (id, name, sku, quantity, reserved) in products_data {
        let product = Product::new(
            id.to_string(),
            name.to_string(),
            sku.to_string(),
            ProductStatus::Active,
        ).unwrap();
        repo.create(product).await.unwrap();

        let inventory = Inventory {
            quantity,
            reserved_quantity: reserved,
            alert_threshold: Some(10),
            track_inventory: true,
            allow_backorder: false,
        };
        repo.update_inventory(id, inventory).await.unwrap();
    }

    // Test low stock products
    let low_stock_products = repo.find_low_stock_products(Some(10)).await;
    assert_eq!(low_stock_products.len(), 3); // Products 9, 10, and 12

    // Test out of stock products
    let out_of_stock_products = repo.find_out_of_stock_products().await;
    assert_eq!(out_of_stock_products.len(), 2); // Products 10 and 12 (reserved > available)
}

#[tokio::test]
async fn test_postgres_product_repository_error_handling() {
    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = PostgresProductRepository::new(pool.clone());
    postgres.run_migrations(&pool).await;

    // Test duplicate SKU error
    let product1 = Product::new(
        "test-product-13".to_string(),
        "First Product".to_string(),
        "DUPLICATE-SKU".to_string(),
        ProductStatus::Active,
    ).unwrap();

    let product2 = Product::new(
        "test-product-14".to_string(),
        "Second Product".to_string(),
        "DUPLICATE-SKU".to_string(),
        ProductStatus::Active,
    ).unwrap();

    // First creation should succeed
    let first_result = repo.create(product1).await;
    assert!(first_result.is_ok());

    // Second creation with same SKU should fail
    let second_result = repo.create(product2).await;
    assert!(matches!(second_result, Err(ProductError::SkuAlreadyExists)));

    // Test updating non-existent product
    let non_existent_product = Product::new(
        "non-existent-id".to_string(),
        "Non-existent Product".to_string(),
        "NON-EXISTENT-SKU".to_string(),
        ProductStatus::Active,
    ).unwrap();

    let update_result = repo.update(non_existent_product).await;
    assert!(matches!(update_result, Err(ProductError::ProductNotFound)));

    // Test deleting non-existent product
    let delete_result = repo.delete("non-existent-id").await;
    assert!(matches!(delete_result, Err(ProductError::ProductNotFound)));

    // Test price update for non-existent product
    let price = Price {
        selling_price: Decimal::new(1000, 2),
        list_price: None,
        discount_price: None,
        currency: "JPY".to_string(),
        tax_included: true,
        effective_from: None,
        effective_until: None,
    };

    let price_update_result = repo.update_price("non-existent-id", price).await;
    assert!(matches!(price_update_result, Err(ProductError::ProductNotFound)));

    // Test inventory update for non-existent product
    let inventory = Inventory {
        quantity: 10,
        reserved_quantity: 0,
        alert_threshold: Some(5),
        track_inventory: true,
        allow_backorder: false,
    };

    let inventory_update_result = repo.update_inventory("non-existent-id", inventory).await;
    assert!(matches!(inventory_update_result, Err(ProductError::ProductNotFound)));
}

#[tokio::test]
async fn test_postgres_product_repository_concurrent_operations() {
    use std::sync::Arc;
    use tokio::task;

    let postgres = PostgresContainer::new();
    let pool = postgres.create_pool().await;
    let repo = Arc::new(PostgresProductRepository::new(pool.clone()));
    postgres.run_migrations(&pool).await;

    // Test concurrent product creation
    let mut create_handles = vec![];

    for i in 1..=10 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            let product = Product::new(
                format!("concurrent-product-{}", i),
                format!("Concurrent Product {}", i),
                format!("CONCURRENT-SKU-{:03}", i),
                ProductStatus::Active,
            ).unwrap();
            repo_clone.create(product).await
        });
        create_handles.push(handle);
    }

    // Wait for all creations to complete
    for handle in create_handles {
        let result = handle.await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Test concurrent reads
    let mut read_handles = vec![];

    for i in 1..=10 {
        let repo_clone = Arc::clone(&repo);
        let handle = task::spawn(async move {
            repo_clone.find_by_id(&format!("concurrent-product-{}", i)).await
        });
        read_handles.push(handle);
    }

    // Verify all reads succeed
    for (i, handle) in read_handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, format!("concurrent-product-{}", i + 1));
    }
} 