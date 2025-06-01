use async_trait::async_trait;
use std::collections::HashMap;

use crate::app_domain::model::product::{
    Product, ProductError, Price, Inventory, ProductImage, ProductHistory,
};

#[async_trait]
pub trait ProductRepository: Send + Sync {
    // Basic CRUD operations
    async fn find_by_id(&self, id: &str) -> Option<Product>;
    async fn find_by_sku(&self, sku: &str) -> Option<Product>;
    // async fn find_all(&self,
    //     category_id: Option<&str>,
    //     status: Option<&str>,
    //     limit: Option<i64>,
    //     offset: Option<i64>,
    // ) -> Vec<Product>;
    async fn create(&self, product: Product) -> Result<Product, ProductError>;
    async fn update(&self, product: Product) -> Result<Product, ProductError>;
    async fn delete(&self, id: &str) -> Result<(), ProductError>;
    async fn exists_by_sku(&self, sku: &str, exclude_id: Option<&str>) -> bool;

    // Price operations
    async fn get_current_price(&self, product_id: &str) -> Option<Price>;
    // async fn get_price_history(&self, product_id: &str, limit: Option<i64>) -> Vec<Price>;
    async fn update_price(&self, product_id: &str, price: Price) -> Result<Price, ProductError>;

    // Inventory operations
    async fn get_inventory(&self, product_id: &str) -> Option<Inventory>;
    async fn update_inventory(&self, product_id: &str, inventory: Inventory) -> Result<Inventory, ProductError>;
    // async fn reserve_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError>;
    // async fn release_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError>;

    // Image operations
    async fn get_images(&self, product_id: &str) -> Vec<ProductImage>;
    async fn add_image(&self, product_id: &str, image: ProductImage) -> Result<ProductImage, ProductError>;
    async fn update_image(&self, product_id: &str, image: ProductImage) -> Result<ProductImage, ProductError>;
    async fn delete_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError>;
    async fn reorder_images(&self, product_id: &str, image_orders: Vec<(String, i32)>) -> Result<(), ProductError>;
    async fn set_main_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError>;

    // Tag operations
    async fn get_tags(&self, product_id: &str) -> Vec<String>;
    async fn add_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError>;
    // async fn remove_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError>;
    async fn replace_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError>;

    // Attribute operations
    async fn get_attributes(&self, product_id: &str) -> HashMap<String, String>;
    async fn set_attributes(&self, product_id: &str, attributes: HashMap<String, String>) -> Result<(), ProductError>;
    // async fn set_attribute(&self, product_id: &str, name: &str, value: &str) -> Result<(), ProductError>;
    // async fn remove_attribute(&self, product_id: &str, name: &str) -> Result<(), ProductError>;

    // History operations
    async fn get_history(&self, 
        product_id: &str, 
        field_name: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<ProductHistory>;
    // async fn add_history_entry(&self,
    //     product_id: &str,
    //     field_name: &str,
    //     old_value: Option<&str>,
    //     new_value: Option<&str>,
    //     changed_by: Option<&str>,
    //     reason: Option<&str>,
    // ) -> Result<(), ProductError>;

    // Batch operations
    // async fn update_batch(&self, updates: Vec<(String, Product)>) -> Result<Vec<Product>, ProductError>;
    // async fn update_prices_batch(&self, updates: Vec<(String, Price)>) -> Result<Vec<Price>, ProductError>;
    // async fn update_inventory_batch(&self, updates: Vec<(String, Inventory)>) -> Result<Vec<Inventory>, ProductError>;

    // Search and filtering
    #[allow(clippy::too_many_arguments)]
    async fn search(&self, 
        query: &str,
        category_id: Option<&str>,
        tags: Option<Vec<&str>>,
        min_price: Option<rust_decimal::Decimal>,
        max_price: Option<rust_decimal::Decimal>,
        in_stock_only: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Product>;

    // async fn find_by_category_recursive(&self, category_id: &str) -> Vec<Product>;
    async fn find_low_stock_products(&self, threshold: Option<i32>) -> Vec<(Product, Inventory)>;
    async fn find_out_of_stock_products(&self) -> Vec<Product>;
}

// #[cfg(test)]
// use mockall::automock;

// #[cfg(test)]
// #[automock]
// #[async_trait]
// pub trait MockableProductRepository: Send + Sync {
    // async fn find_by_id(&self, id: &str) -> Option<Product>;
    // async fn find_by_sku(&self, sku: &str) -> Option<Product>;
    // async fn create(&self, product: Product) -> Result<Product, ProductError>;
    // async fn update(&self, product: Product) -> Result<Product, ProductError>;
    // async fn delete(&self, id: &str) -> Result<(), ProductError>;
    // async fn update_price(&self, product_id: &str, price: Price) -> Result<Price, ProductError>;
    // async fn update_inventory(&self, product_id: &str, inventory: Inventory) -> Result<Inventory, ProductError>;
    // async fn add_image(&self, product_id: &str, image: ProductImage) -> Result<ProductImage, ProductError>;
// }