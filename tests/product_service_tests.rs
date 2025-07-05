use std::sync::Arc;
use async_trait::async_trait;
use rust_webapi::application::service::product_service::ProductService;
use rust_webapi::app_domain::repository::product_repository::ProductRepository;
use rust_webapi::app_domain::model::product::{Product, ProductStatus, ProductError, Price, Inventory, Dimensions, ShippingInfo};
use rust_webapi::application::dto::product_dto::{CreateProductRequest, PriceRequest, InventoryRequest, DimensionsRequest, ShippingInfoRequest};
use rust_decimal::Decimal;

struct MockProductRepository {
    exists: bool,
    created: Option<Product>,
}

#[async_trait]
impl ProductRepository for MockProductRepository {
    async fn find_by_id(&self, _id: &str) -> Option<Product> {
        self.created.clone()
    }
    async fn find_by_sku(&self, _sku: &str) -> Option<Product> { None }
    async fn create(&self, product: Product) -> Result<Product, ProductError> {
        Ok(product.clone())
    }
    async fn update(&self, _product: Product) -> Result<Product, ProductError> { Ok(_product) }
    async fn delete(&self, _id: &str) -> Result<(), ProductError> { Ok(()) }
    async fn exists_by_sku(&self, _sku: &str, _exclude_id: Option<&str>) -> bool { self.exists }
    async fn get_current_price(&self, _product_id: &str) -> Option<Price> { None }
    async fn update_price(&self, _product_id: &str, price: Price) -> Result<Price, ProductError> { Ok(price) }
    async fn get_inventory(&self, _product_id: &str) -> Option<Inventory> { None }
    async fn update_inventory(&self, _product_id: &str, inventory: Inventory) -> Result<Inventory, ProductError> { Ok(inventory) }
    async fn get_images(&self, _product_id: &str) -> Vec<rust_webapi::app_domain::model::product::ProductImage> { vec![] }
    async fn add_image(&self, _product_id: &str, _image: rust_webapi::app_domain::model::product::ProductImage) -> Result<rust_webapi::app_domain::model::product::ProductImage, ProductError> { Ok(_image) }
    async fn update_image(&self, _product_id: &str, _image: rust_webapi::app_domain::model::product::ProductImage) -> Result<rust_webapi::app_domain::model::product::ProductImage, ProductError> { Ok(_image) }
    async fn delete_image(&self, _product_id: &str, _image_id: &str) -> Result<(), ProductError> { Ok(()) }
    async fn reorder_images(&self, _product_id: &str, _image_orders: Vec<(String, i32)>) -> Result<(), ProductError> { Ok(()) }
    async fn set_main_image(&self, _product_id: &str, _image_id: &str) -> Result<(), ProductError> { Ok(()) }
    async fn get_tags(&self, _product_id: &str) -> Vec<String> { vec![] }
    async fn add_tags(&self, _product_id: &str, _tags: Vec<String>) -> Result<(), ProductError> { Ok(()) }
    async fn replace_tags(&self, _product_id: &str, _tags: Vec<String>) -> Result<(), ProductError> { Ok(()) }
    async fn get_attributes(&self, _product_id: &str) -> std::collections::HashMap<String, String> { std::collections::HashMap::new() }
    async fn set_attributes(&self, _product_id: &str, _attributes: std::collections::HashMap<String, String>) -> Result<(), ProductError> { Ok(()) }
    async fn get_history(&self, _product_id: &str, _field_name: Option<&str>, _limit: Option<i64>, _offset: Option<i64>) -> Vec<rust_webapi::app_domain::model::product::ProductHistory> { vec![] }
    async fn search(&self, _query: &str, _category_id: Option<&str>, _tags: Option<Vec<&str>>, _min_price: Option<Decimal>, _max_price: Option<Decimal>, _in_stock_only: bool, _limit: Option<i64>, _offset: Option<i64>) -> Vec<Product> { vec![] }
    async fn find_low_stock_products(&self, _threshold: Option<i32>) -> Vec<(Product, Inventory)> { vec![] }
    async fn find_out_of_stock_products(&self) -> Vec<Product> { vec![] }
}

#[tokio::test]
async fn test_create_product_duplicate_sku() {
    let repo = Arc::new(MockProductRepository { exists: true, created: None });
    let service = ProductService::new(repo);
    let req = CreateProductRequest {
        name: "Test Product".to_string(),
        description: None,
        sku: "SKU-001".to_string(),
        brand: None,
        status: ProductStatus::Active,
        price: PriceRequest {
            selling_price: Decimal::new(1000, 2),
            list_price: None,
            discount_price: None,
            currency: "JPY".to_string(),
            tax_included: true,
            effective_from: None,
            effective_until: None,
        },
        inventory: InventoryRequest {
            quantity: 10,
            reserved_quantity: None,
            alert_threshold: None,
            track_inventory: None,
            allow_backorder: None,
        },
        category_id: None,
        tags: None,
        attributes: None,
        dimensions: None,
        weight: None,
        shipping_info: None,
    };
    let result = service.create(req).await;
    assert!(matches!(result, Err(ProductError::SkuAlreadyExists)));
}

#[tokio::test]
async fn test_create_product_success() {
    let product = Product::new(
        "dummy_id".to_string(),
        "Test Product".to_string(),
        "SKU-001".to_string(),
        ProductStatus::Active,
    ).unwrap();
    let repo = Arc::new(MockProductRepository { exists: false, created: Some(product) });
    let service = ProductService::new(repo);
    let req = CreateProductRequest {
        name: "Test Product".to_string(),
        description: Some("desc".to_string()),
        sku: "SKU-001".to_string(),
        brand: Some("BrandX".to_string()),
        status: ProductStatus::Active,
        price: PriceRequest {
            selling_price: Decimal::new(1000, 2),
            list_price: Some(Decimal::new(1200, 2)),
            discount_price: Some(Decimal::new(900, 2)),
            currency: "JPY".to_string(),
            tax_included: true,
            effective_from: None,
            effective_until: None,
        },
        inventory: InventoryRequest {
            quantity: 10,
            reserved_quantity: Some(2),
            alert_threshold: Some(1),
            track_inventory: Some(true),
            allow_backorder: Some(false),
        },
        category_id: Some("cat01".to_string()),
        tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        attributes: Some(std::collections::HashMap::new()),
        dimensions: Some(DimensionsRequest { width: Decimal::new(10, 0), height: Decimal::new(20, 0), depth: Decimal::new(5, 0) }),
        weight: Some(Decimal::new(100, 0)),
        shipping_info: Some(ShippingInfoRequest { shipping_class: "standard".to_string(), free_shipping: false, shipping_fee: Decimal::new(500, 2) }),
    };
    let result = service.create(req).await;
    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.name, "Test Product");
    assert_eq!(product.sku, "SKU-001");
} 