use sqlx::{PgPool, Row};
use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use tracing::error;
use std::collections::HashMap;

use crate::app_domain::model::product::{
    Product, ProductStatus, ProductError, Price, Inventory, ProductImage,
    Dimensions, ShippingInfo, ProductHistory,
};
use crate::app_domain::repository::product_repository::ProductRepository;

pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_product(row: &sqlx::postgres::PgRow) -> Product {
        let dimensions = if let (Some(width), Some(height), Some(depth)) = (
            row.try_get::<Option<Decimal>, _>("width").unwrap_or(None),
            row.try_get::<Option<Decimal>, _>("height").unwrap_or(None),
            row.try_get::<Option<Decimal>, _>("depth").unwrap_or(None),
        ) {
            Some(Dimensions { width, height, depth })
        } else {
            None
        };

        let shipping_info = ShippingInfo {
            shipping_class: row.try_get("shipping_class").unwrap_or_else(|_| "standard".to_string()),
            free_shipping: row.try_get("free_shipping").unwrap_or(false),
            shipping_fee: row.try_get("shipping_fee").unwrap_or(Decimal::ZERO),
        };

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Active" => ProductStatus::Active,
            "Inactive" => ProductStatus::Inactive,
            "Draft" => ProductStatus::Draft,
            "Discontinued" => ProductStatus::Discontinued,
            _ => ProductStatus::Draft,
        };

        Product {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            sku: row.get("sku"),
            brand: row.get("brand"),
            status,
            category_id: row.get("category_id"),
            dimensions,
            weight: row.try_get("weight").unwrap_or(None),
            shipping_info,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_price(row: &sqlx::postgres::PgRow) -> Price {
        Price {
            selling_price: row.get("selling_price"),
            list_price: row.try_get("list_price").unwrap_or(None),
            discount_price: row.try_get("discount_price").unwrap_or(None),
            currency: row.try_get("currency").unwrap_or_else(|_| "JPY".to_string()),
            tax_included: row.try_get("tax_included").unwrap_or(true),
            effective_from: row.try_get("effective_from").unwrap_or(None),
            effective_until: row.try_get("effective_until").unwrap_or(None),
        }
    }

    fn row_to_inventory(row: &sqlx::postgres::PgRow) -> Inventory {
        Inventory {
            quantity: row.get("quantity"),
            reserved_quantity: row.get("reserved_quantity"),
            alert_threshold: row.try_get("alert_threshold").unwrap_or(None),
            track_inventory: row.try_get("track_inventory").unwrap_or(true),
            allow_backorder: row.try_get("allow_backorder").unwrap_or(false),
        }
    }

    fn row_to_product_image(row: &sqlx::postgres::PgRow) -> ProductImage {
        ProductImage {
            id: row.get("id"),
            url: row.get("url"),
            alt_text: row.try_get("alt_text").unwrap_or(None),
            sort_order: row.get("sort_order"),
            is_main: row.get("is_main"),
        }
    }

    fn row_to_product_history(row: &sqlx::postgres::PgRow) -> ProductHistory {
        ProductHistory {
            id: row.get("id"),
            product_id: row.get("product_id"),
            field_name: row.get("field_name"),
            old_value: row.try_get("old_value").unwrap_or(None),
            new_value: row.try_get("new_value").unwrap_or(None),
            changed_by: row.try_get("changed_by").unwrap_or(None),
            reason: row.try_get("reason").unwrap_or(None),
            changed_at: row.get("changed_at"),
        }
    }

    async fn get_tags_for_product(&self, product_id: &str) -> Vec<String> {
        let query = "SELECT tag FROM product_tags WHERE product_id = $1 ORDER BY tag";
        
        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(|row| row.get("tag")).collect(),
            Err(e) => {
                error!("Error fetching tags for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    async fn get_attributes_for_product(&self, product_id: &str) -> HashMap<String, String> {
        let query = "SELECT attribute_name, attribute_value FROM product_attributes WHERE product_id = $1";
        
        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => {
                let mut attributes = HashMap::new();
                for row in rows {
                    attributes.insert(row.get("attribute_name"), row.get("attribute_value"));
                }
                attributes
            }
            Err(e) => {
                error!("Error fetching attributes for product {}: {}", product_id, e);
                HashMap::new()
            }
        }
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn find_by_id(&self, id: &str) -> Option<Product> {
        let query = "SELECT id, name, description, sku, brand, status, category_id, 
                           width, height, depth, weight, shipping_class, free_shipping, shipping_fee,
                           created_at, updated_at 
                     FROM products 
                     WHERE id = $1";

        match sqlx::query(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Some(Self::row_to_product(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error finding product by id {}: {}", id, e);
                None
            }
        }
    }

    async fn find_by_sku(&self, sku: &str) -> Option<Product> {
        let query = "SELECT id, name, description, sku, brand, status, category_id, 
                           width, height, depth, weight, shipping_class, free_shipping, shipping_fee,
                           created_at, updated_at 
                     FROM products 
                     WHERE sku = $1";

        match sqlx::query(query)
            .bind(sku)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Some(Self::row_to_product(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error finding product by sku {}: {}", sku, e);
                None
            }
        }
    }

    async fn find_all(&self, 
        category_id: Option<&str>,
        status: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Product> {
        let mut query = "SELECT id, name, description, sku, brand, status, category_id, 
                               width, height, depth, weight, shipping_class, free_shipping, shipping_fee,
                               created_at, updated_at 
                         FROM products WHERE 1=1".to_string();
        
        let mut param_index = 1;

        if let Some(_cat_id) = category_id {
            query.push_str(&format!(" AND category_id = ${}", param_index));
            param_index += 1;
        }

        if let Some(_status_val) = status {
            query.push_str(&format!(" AND status = ${}", param_index));
            param_index += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(_limit_val) = limit {
            query.push_str(&format!(" LIMIT ${}", param_index));
            param_index += 1;
        }

        if let Some(_offset_val) = offset {
            query.push_str(&format!(" OFFSET ${}", param_index));
        }

        let mut sqlx_query = sqlx::query(&query);
        if let Some(cat_id) = category_id {
            sqlx_query = sqlx_query.bind(cat_id);
        }
        if let Some(status_val) = status {
            sqlx_query = sqlx_query.bind(status_val);
        }
        if let Some(limit_val) = limit {
            sqlx_query = sqlx_query.bind(limit_val);
        }
        if let Some(offset_val) = offset {
            sqlx_query = sqlx_query.bind(offset_val);
        }

        match sqlx_query.fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(Self::row_to_product).collect(),
            Err(e) => {
                error!("Error finding products: {}", e);
                vec![]
            }
        }
    }

    async fn create(&self, product: Product) -> Result<Product, ProductError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // Insert main product record
        let query = "INSERT INTO products (id, name, description, sku, brand, status, category_id, 
                                         width, height, depth, weight, shipping_class, free_shipping, shipping_fee,
                                         created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)";

        let width = product.dimensions.as_ref().map(|d| d.width);
        let height = product.dimensions.as_ref().map(|d| d.height);
        let depth = product.dimensions.as_ref().map(|d| d.depth);

        let result = sqlx::query(query)
            .bind(&product.id)
            .bind(&product.name)
            .bind(&product.description)
            .bind(&product.sku)
            .bind(&product.brand)
            .bind(product.status.to_string())
            .bind(&product.category_id)
            .bind(width)
            .bind(height)
            .bind(depth)
            .bind(product.weight)
            .bind(&product.shipping_info.shipping_class)
            .bind(product.shipping_info.free_shipping)
            .bind(product.shipping_info.shipping_fee)
            .bind(product.created_at)
            .bind(product.updated_at)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => {
                // Create initial inventory record
                let inventory_query = "INSERT INTO product_inventory (product_id, quantity, reserved_quantity, track_inventory, allow_backorder)
                                     VALUES ($1, 0, 0, true, false)";
                
                if let Err(e) = sqlx::query(inventory_query)
                    .bind(&product.id)
                    .execute(&mut *tx)
                    .await
                {
                    let _ = tx.rollback().await;
                    return Err(ProductError::DatabaseError(e.to_string()));
                }

                tx.commit().await
                    .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
                
                Ok(product)
            }
            Err(sqlx::Error::Database(db_err)) => {
                let _ = tx.rollback().await;
                if db_err.constraint() == Some("products_sku_key") {
                    Err(ProductError::SkuAlreadyExists)
                } else {
                    Err(ProductError::DatabaseError(db_err.to_string()))
                }
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(ProductError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn update(&self, product: Product) -> Result<Product, ProductError> {
        let query = "UPDATE products 
                     SET name = $2, description = $3, sku = $4, brand = $5, status = $6, category_id = $7,
                         width = $8, height = $9, depth = $10, weight = $11, 
                         shipping_class = $12, free_shipping = $13, shipping_fee = $14, updated_at = $15
                     WHERE id = $1";

        let width = product.dimensions.as_ref().map(|d| d.width);
        let height = product.dimensions.as_ref().map(|d| d.height);
        let depth = product.dimensions.as_ref().map(|d| d.depth);

        let result = sqlx::query(query)
            .bind(&product.id)
            .bind(&product.name)
            .bind(&product.description)
            .bind(&product.sku)
            .bind(&product.brand)
            .bind(product.status.to_string())
            .bind(&product.category_id)
            .bind(width)
            .bind(height)
            .bind(depth)
            .bind(product.weight)
            .bind(&product.shipping_info.shipping_class)
            .bind(product.shipping_info.free_shipping)
            .bind(product.shipping_info.shipping_fee)
            .bind(Utc::now())
            .execute(&self.pool)
            .await;

        match result {
            Ok(result) if result.rows_affected() > 0 => Ok(product),
            Ok(_) => Err(ProductError::ProductNotFound),
            Err(sqlx::Error::Database(db_err)) => {
                if db_err.constraint() == Some("products_sku_key") {
                    Err(ProductError::SkuAlreadyExists)
                } else {
                    Err(ProductError::DatabaseError(db_err.to_string()))
                }
            }
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), ProductError> {
        let query = "DELETE FROM products WHERE id = $1";

        match sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ProductError::ProductNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn exists_by_sku(&self, sku: &str, exclude_id: Option<&str>) -> bool {
        let query = if exclude_id.is_some() {
            "SELECT COUNT(*) as count FROM products WHERE sku = $1 AND id != $2"
        } else {
            "SELECT COUNT(*) as count FROM products WHERE sku = $1"
        };

        let mut sqlx_query = sqlx::query(query).bind(sku);
        if let Some(id) = exclude_id {
            sqlx_query = sqlx_query.bind(id);
        }

        match sqlx_query.fetch_one(&self.pool).await {
            Ok(row) => {
                let count: i64 = row.get("count");
                count > 0
            }
            Err(e) => {
                error!("Error checking SKU existence: {}", e);
                false
            }
        }
    }

    async fn get_current_price(&self, product_id: &str) -> Option<Price> {
        let query = "SELECT selling_price, list_price, discount_price, currency, tax_included,
                           effective_from, effective_until
                     FROM product_prices 
                     WHERE product_id = $1 
                       AND (effective_from IS NULL OR effective_from <= NOW())
                       AND (effective_until IS NULL OR effective_until >= NOW())
                     ORDER BY created_at DESC 
                     LIMIT 1";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Some(Self::row_to_price(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error getting current price for product {}: {}", product_id, e);
                None
            }
        }
    }

    async fn get_price_history(&self, product_id: &str, limit: Option<i64>) -> Vec<Price> {
        let mut query = "SELECT selling_price, list_price, discount_price, currency, tax_included,
                               effective_from, effective_until
                         FROM product_prices 
                         WHERE product_id = $1 
                         ORDER BY created_at DESC".to_string();

        if let Some(limit_val) = limit {
            query.push_str(&format!(" LIMIT {}", limit_val));
        }

        match sqlx::query(&query)
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(Self::row_to_price).collect(),
            Err(e) => {
                error!("Error getting price history for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    async fn update_price(&self, product_id: &str, price: Price) -> Result<Price, ProductError> {
        // Validate price before updating
        price.validate()?;

        let query = "INSERT INTO product_prices (product_id, selling_price, list_price, discount_price, 
                                               currency, tax_included, effective_from, effective_until)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

        match sqlx::query(query)
            .bind(product_id)
            .bind(price.selling_price)
            .bind(price.list_price)
            .bind(price.discount_price)
            .bind(&price.currency)
            .bind(price.tax_included)
            .bind(price.effective_from)
            .bind(price.effective_until)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(price),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn get_inventory(&self, product_id: &str) -> Option<Inventory> {
        let query = "SELECT quantity, reserved_quantity, alert_threshold, track_inventory, allow_backorder
                     FROM product_inventory 
                     WHERE product_id = $1";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(Some(row)) => Some(Self::row_to_inventory(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error getting inventory for product {}: {}", product_id, e);
                None
            }
        }
    }

    async fn update_inventory(&self, product_id: &str, inventory: Inventory) -> Result<Inventory, ProductError> {
        inventory.validate()?;

        let query = "UPDATE product_inventory 
                     SET quantity = $2, reserved_quantity = $3, alert_threshold = $4, 
                         track_inventory = $5, allow_backorder = $6, updated_at = NOW()
                     WHERE product_id = $1";

        match sqlx::query(query)
            .bind(product_id)
            .bind(inventory.quantity)
            .bind(inventory.reserved_quantity)
            .bind(inventory.alert_threshold)
            .bind(inventory.track_inventory)
            .bind(inventory.allow_backorder)
            .execute(&self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(inventory),
            Ok(_) => Err(ProductError::ProductNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn reserve_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError> {
        if quantity <= 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        let query = "UPDATE product_inventory 
                     SET reserved_quantity = reserved_quantity + $2
                     WHERE product_id = $1 AND quantity >= reserved_quantity + $2";

        match sqlx::query(query)
            .bind(product_id)
            .bind(quantity)
            .execute(&self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ProductError::InvalidInventoryQuantity),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn release_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError> {
        if quantity <= 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        let query = "UPDATE product_inventory 
                     SET reserved_quantity = GREATEST(0, reserved_quantity - $2)
                     WHERE product_id = $1";

        match sqlx::query(query)
            .bind(product_id)
            .bind(quantity)
            .execute(&self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ProductError::ProductNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn get_images(&self, product_id: &str) -> Vec<ProductImage> {
        let query = "SELECT id, url, alt_text, sort_order, is_main
                     FROM product_images 
                     WHERE product_id = $1 
                     ORDER BY sort_order, created_at";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(Self::row_to_product_image).collect(),
            Err(e) => {
                error!("Error getting images for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    async fn add_image(&self, product_id: &str, image: ProductImage) -> Result<ProductImage, ProductError> {
        // Check image count limit
        let count_query = "SELECT COUNT(*) as count FROM product_images WHERE product_id = $1";
        let count_result = sqlx::query(count_query)
            .bind(product_id)
            .fetch_one(&self.pool)
            .await;

        if let Ok(row) = count_result {
            let count: i64 = row.get("count");
            if count >= 10 {
                return Err(ProductError::TooManyImages);
            }
        }

        image.validate()?;

        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // If setting as main image, unset others
        if image.is_main {
            let unset_query = "UPDATE product_images SET is_main = false WHERE product_id = $1";
            if let Err(e) = sqlx::query(unset_query)
                .bind(product_id)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        let query = "INSERT INTO product_images (id, product_id, url, alt_text, sort_order, is_main)
                     VALUES ($1, $2, $3, $4, $5, $6)";

        let result = sqlx::query(query)
            .bind(&image.id)
            .bind(product_id)
            .bind(&image.url)
            .bind(&image.alt_text)
            .bind(image.sort_order)
            .bind(image.is_main)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(_) => {
                tx.commit().await
                    .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
                Ok(image)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(ProductError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn update_image(&self, product_id: &str, image: ProductImage) -> Result<ProductImage, ProductError> {
        image.validate()?;

        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // If setting as main image, unset others
        if image.is_main {
            let unset_query = "UPDATE product_images SET is_main = false WHERE product_id = $1 AND id != $2";
            if let Err(e) = sqlx::query(unset_query)
                .bind(product_id)
                .bind(&image.id)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        let query = "UPDATE product_images 
                     SET url = $3, alt_text = $4, sort_order = $5, is_main = $6, updated_at = NOW()
                     WHERE product_id = $1 AND id = $2";

        let result = sqlx::query(query)
            .bind(product_id)
            .bind(&image.id)
            .bind(&image.url)
            .bind(&image.alt_text)
            .bind(image.sort_order)
            .bind(image.is_main)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(result) if result.rows_affected() > 0 => {
                tx.commit().await
                    .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
                Ok(image)
            }
            Ok(_) => {
                let _ = tx.rollback().await;
                Err(ProductError::ImageNotFound)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(ProductError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn delete_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError> {
        let query = "DELETE FROM product_images WHERE product_id = $1 AND id = $2";

        match sqlx::query(query)
            .bind(product_id)
            .bind(image_id)
            .execute(&self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ProductError::ImageNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn reorder_images(&self, product_id: &str, image_orders: Vec<(String, i32)>) -> Result<(), ProductError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for (image_id, sort_order) in image_orders {
            if sort_order < 0 {
                let _ = tx.rollback().await;
                return Err(ProductError::InvalidImageOrder);
            }

            let query = "UPDATE product_images SET sort_order = $3 WHERE product_id = $1 AND id = $2";
            
            if let Err(e) = sqlx::query(query)
                .bind(product_id)
                .bind(&image_id)
                .bind(sort_order)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    async fn set_main_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // Unset all main images for this product
        let unset_query = "UPDATE product_images SET is_main = false WHERE product_id = $1";
        if let Err(e) = sqlx::query(unset_query)
            .bind(product_id)
            .execute(&mut *tx)
            .await
        {
            let _ = tx.rollback().await;
            return Err(ProductError::DatabaseError(e.to_string()));
        }

        // Set the specified image as main
        let set_query = "UPDATE product_images SET is_main = true WHERE product_id = $1 AND id = $2";
        let result = sqlx::query(set_query)
            .bind(product_id)
            .bind(image_id)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(result) if result.rows_affected() > 0 => {
                tx.commit().await
                    .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
                Ok(())
            }
            Ok(_) => {
                let _ = tx.rollback().await;
                Err(ProductError::ImageNotFound)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(ProductError::DatabaseError(e.to_string()))
            }
        }
    }

    async fn get_tags(&self, product_id: &str) -> Vec<String> {
        self.get_tags_for_product(product_id).await
    }

    async fn add_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        if tags.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for tag in tags {
            let query = "INSERT INTO product_tags (product_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING";
            
            if let Err(e) = sqlx::query(query)
                .bind(product_id)
                .bind(&tag)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    async fn remove_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        if tags.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<String> = (1..=tags.len()).map(|i| format!("${}", i + 1)).collect();
        let query = format!("DELETE FROM product_tags WHERE product_id = $1 AND tag IN ({})", 
                           placeholders.join(", "));

        let mut sqlx_query = sqlx::query(&query).bind(product_id);
        for tag in tags {
            sqlx_query = sqlx_query.bind(tag);
        }

        match sqlx_query.execute(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn replace_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // Delete all existing tags
        let delete_query = "DELETE FROM product_tags WHERE product_id = $1";
        if let Err(e) = sqlx::query(delete_query)
            .bind(product_id)
            .execute(&mut *tx)
            .await
        {
            let _ = tx.rollback().await;
            return Err(ProductError::DatabaseError(e.to_string()));
        }

        // Insert new tags
        for tag in tags {
            let insert_query = "INSERT INTO product_tags (product_id, tag) VALUES ($1, $2)";
            
            if let Err(e) = sqlx::query(insert_query)
                .bind(product_id)
                .bind(&tag)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    async fn get_attributes(&self, product_id: &str) -> HashMap<String, String> {
        self.get_attributes_for_product(product_id).await
    }

    async fn set_attributes(&self, product_id: &str, attributes: HashMap<String, String>) -> Result<(), ProductError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // Delete all existing attributes
        let delete_query = "DELETE FROM product_attributes WHERE product_id = $1";
        if let Err(e) = sqlx::query(delete_query)
            .bind(product_id)
            .execute(&mut *tx)
            .await
        {
            let _ = tx.rollback().await;
            return Err(ProductError::DatabaseError(e.to_string()));
        }

        // Insert new attributes
        for (name, value) in attributes {
            let insert_query = "INSERT INTO product_attributes (product_id, attribute_name, attribute_value) 
                               VALUES ($1, $2, $3)";
            
            if let Err(e) = sqlx::query(insert_query)
                .bind(product_id)
                .bind(&name)
                .bind(&value)
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback().await;
                return Err(ProductError::DatabaseError(e.to_string()));
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    async fn set_attribute(&self, product_id: &str, name: &str, value: &str) -> Result<(), ProductError> {
        let query = "INSERT INTO product_attributes (product_id, attribute_name, attribute_value) 
                     VALUES ($1, $2, $3)
                     ON CONFLICT (product_id, attribute_name) 
                     DO UPDATE SET attribute_value = $3, updated_at = NOW()";

        match sqlx::query(query)
            .bind(product_id)
            .bind(name)
            .bind(value)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn remove_attribute(&self, product_id: &str, name: &str) -> Result<(), ProductError> {
        let query = "DELETE FROM product_attributes WHERE product_id = $1 AND attribute_name = $2";

        match sqlx::query(query)
            .bind(product_id)
            .bind(name)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn get_history(&self, 
        product_id: &str, 
        field_name: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<ProductHistory> {
        let mut query = "SELECT id, product_id, field_name, old_value, new_value, changed_by, reason, changed_at
                         FROM product_history 
                         WHERE product_id = $1".to_string();
        
        let mut param_index = 2;

        if let Some(_field) = field_name {
            query.push_str(&format!(" AND field_name = ${}", param_index));
            param_index += 1;
        }

        query.push_str(" ORDER BY changed_at DESC");

        if limit.is_some() {
            query.push_str(&format!(" LIMIT ${}", param_index));
            param_index += 1;
        }

        if offset.is_some() {
            query.push_str(&format!(" OFFSET ${}", param_index));
        }

        let mut sqlx_query = sqlx::query(&query);
        sqlx_query = sqlx_query.bind(product_id);
        if let Some(field) = field_name {
            sqlx_query = sqlx_query.bind(field);
        }
        if let Some(limit_val) = limit {
            sqlx_query = sqlx_query.bind(limit_val);
        }
        if let Some(offset_val) = offset {
            sqlx_query = sqlx_query.bind(offset_val);
        }

        match sqlx_query.fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(Self::row_to_product_history).collect(),
            Err(e) => {
                error!("Error getting history for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    async fn add_history_entry(&self, 
        product_id: &str,
        field_name: &str,
        old_value: Option<&str>,
        new_value: Option<&str>,
        changed_by: Option<&str>,
        reason: Option<&str>,
    ) -> Result<(), ProductError> {
        let query = "INSERT INTO product_history (product_id, field_name, old_value, new_value, changed_by, reason)
                     VALUES ($1, $2, $3, $4, $5, $6)";

        match sqlx::query(query)
            .bind(product_id)
            .bind(field_name)
            .bind(old_value)
            .bind(new_value)
            .bind(changed_by)
            .bind(reason)
            .execute(&self.pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    async fn update_batch(&self, updates: Vec<(String, Product)>) -> Result<Vec<Product>, ProductError> {
        let mut results = Vec::new();
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for (id, product) in updates {
            let query = "UPDATE products 
                         SET name = $2, description = $3, sku = $4, brand = $5, status = $6, category_id = $7,
                             width = $8, height = $9, depth = $10, weight = $11, 
                             shipping_class = $12, free_shipping = $13, shipping_fee = $14, updated_at = NOW()
                         WHERE id = $1";

            let width = product.dimensions.as_ref().map(|d| d.width);
            let height = product.dimensions.as_ref().map(|d| d.height);
            let depth = product.dimensions.as_ref().map(|d| d.depth);

            let result = sqlx::query(query)
                .bind(&id)
                .bind(&product.name)
                .bind(&product.description)
                .bind(&product.sku)
                .bind(&product.brand)
                .bind(product.status.to_string())
                .bind(&product.category_id)
                .bind(width)
                .bind(height)
                .bind(depth)
                .bind(product.weight)
                .bind(&product.shipping_info.shipping_class)
                .bind(product.shipping_info.free_shipping)
                .bind(product.shipping_info.shipping_fee)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => results.push(product),
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(ProductError::DatabaseError(e.to_string()));
                }
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(results)
    }

    async fn update_prices_batch(&self, updates: Vec<(String, Price)>) -> Result<Vec<Price>, ProductError> {
        let mut results = Vec::new();
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for (product_id, price) in updates {
            price.validate()?;

            let query = "INSERT INTO product_prices (product_id, selling_price, list_price, discount_price, 
                                                   currency, tax_included, effective_from, effective_until)
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

            let result = sqlx::query(query)
                .bind(&product_id)
                .bind(price.selling_price)
                .bind(price.list_price)
                .bind(price.discount_price)
                .bind(&price.currency)
                .bind(price.tax_included)
                .bind(price.effective_from)
                .bind(price.effective_until)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => results.push(price),
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(ProductError::DatabaseError(e.to_string()));
                }
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(results)
    }

    async fn update_inventory_batch(&self, updates: Vec<(String, Inventory)>) -> Result<Vec<Inventory>, ProductError> {
        let mut results = Vec::new();
        let mut tx = self.pool.begin().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for (product_id, inventory) in updates {
            inventory.validate()?;

            let query = "UPDATE product_inventory 
                         SET quantity = $2, reserved_quantity = $3, alert_threshold = $4, 
                             track_inventory = $5, allow_backorder = $6, updated_at = NOW()
                         WHERE product_id = $1";

            let result = sqlx::query(query)
                .bind(&product_id)
                .bind(inventory.quantity)
                .bind(inventory.reserved_quantity)
                .bind(inventory.alert_threshold)
                .bind(inventory.track_inventory)
                .bind(inventory.allow_backorder)
                .execute(&mut *tx)
                .await;

            match result {
                Ok(_) => results.push(inventory),
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(ProductError::DatabaseError(e.to_string()));
                }
            }
        }

        tx.commit().await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
        
        Ok(results)
    }

    #[allow(clippy::too_many_arguments)]
    async fn search(&self, 
        query: &str,
        category_id: Option<&str>,
        tags: Option<Vec<&str>>,
        min_price: Option<Decimal>,
        max_price: Option<Decimal>,
        in_stock_only: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<Product> {
        let mut sql_query = "SELECT DISTINCT p.id, p.name, p.description, p.sku, p.brand, p.status, p.category_id, 
                                    p.width, p.height, p.depth, p.weight, p.shipping_class, p.free_shipping, p.shipping_fee,
                                    p.created_at, p.updated_at 
                            FROM products p".to_string();
        
        let mut joins = Vec::new();
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        // Text search
        if !query.is_empty() {
            conditions.push(format!("(p.name ILIKE ${} OR p.description ILIKE ${} OR p.sku ILIKE ${})", 
                                  param_index, param_index + 1, param_index + 2));
            let search_pattern = format!("%{}%", query);
            params.push(search_pattern.clone());
            params.push(search_pattern.clone());
            params.push(search_pattern);
            param_index += 3;
        }

        // Category filter
        if let Some(cat_id) = category_id {
            conditions.push(format!("p.category_id = ${}", param_index));
            params.push(cat_id.to_string());
            param_index += 1;
        }

        // Tags filter
        if let Some(tag_list) = tags {
            if !tag_list.is_empty() {
                joins.push("JOIN product_tags pt ON p.id = pt.product_id".to_string());
                let placeholders: Vec<String> = (0..tag_list.len())
                    .map(|i| format!("${}", param_index + i))
                    .collect();
                conditions.push(format!("pt.tag IN ({})", placeholders.join(", ")));
                for tag in tag_list {
                    params.push(tag.to_string());
                    param_index += 1;
                }
            }
        }

        // Price range filter
        if min_price.is_some() || max_price.is_some() {
            joins.push("JOIN product_prices pp ON p.id = pp.product_id".to_string());
            joins.push("AND (pp.effective_from IS NULL OR pp.effective_from <= NOW())".to_string());
            joins.push("AND (pp.effective_until IS NULL OR pp.effective_until >= NOW())".to_string());

            if let Some(min_p) = min_price {
                conditions.push(format!("pp.selling_price >= ${}", param_index));
                params.push(min_p.to_string());
                param_index += 1;
            }

            if let Some(max_p) = max_price {
                conditions.push(format!("pp.selling_price <= ${}", param_index));
                params.push(max_p.to_string());
                param_index += 1;
            }
        }

        // Stock filter
        if in_stock_only {
            joins.push("JOIN product_inventory pi ON p.id = pi.product_id".to_string());
            conditions.push("pi.quantity > pi.reserved_quantity".to_string());
        }

        // Build the complete query
        if !joins.is_empty() {
            sql_query.push(' ');
            sql_query.push_str(&joins.join(" "));
        }

        if !conditions.is_empty() {
            sql_query.push_str(" WHERE ");
            sql_query.push_str(&conditions.join(" AND "));
        }

        sql_query.push_str(" ORDER BY p.created_at DESC");

        if let Some(limit_val) = limit {
            sql_query.push_str(&format!(" LIMIT ${}", param_index));
            let limit_val_str = limit_val.to_string();
            params.push(limit_val_str);
            param_index += 1;
        }

        if let Some(offset_val) = offset {
            sql_query.push_str(&format!(" OFFSET ${}", param_index));
            let offset_val_str = offset_val.to_string();
            params.push(offset_val_str);
        }

        let mut sqlx_query = sqlx::query(&sql_query);
        for param in params {
            sqlx_query = sqlx_query.bind(param);
        }

        match sqlx_query.fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(Self::row_to_product).collect(),
            Err(e) => {
                error!("Error searching products: {}", e);
                vec![]
            }
        }
    }

    async fn find_by_category_recursive(&self, category_id: &str) -> Vec<Product> {
        // This would require a recursive CTE to find all subcategories
        // For simplicity, just finding direct children for now
        let query = "SELECT p.id, p.name, p.description, p.sku, p.brand, p.status, p.category_id, 
                           p.width, p.height, p.depth, p.weight, p.shipping_class, p.free_shipping, p.shipping_fee,
                           p.created_at, p.updated_at 
                     FROM products p
                     WHERE p.category_id = $1 
                        OR p.category_id IN (
                            SELECT id FROM categories WHERE parent_id = $1
                        )
                     ORDER BY p.created_at DESC";

        match sqlx::query(query)
            .bind(category_id)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(Self::row_to_product).collect(),
            Err(e) => {
                error!("Error finding products by category {}: {}", category_id, e);
                vec![]
            }
        }
    }

    async fn find_low_stock_products(&self, threshold: Option<i32>) -> Vec<(Product, Inventory)> {
        let default_threshold = threshold.unwrap_or(10);
        
        let query = "SELECT p.id, p.name, p.description, p.sku, p.brand, p.status, p.category_id, 
                           p.width, p.height, p.depth, p.weight, p.shipping_class, p.free_shipping, p.shipping_fee,
                           p.created_at, p.updated_at,
                           i.quantity, i.reserved_quantity, i.alert_threshold, i.track_inventory, i.allow_backorder
                     FROM products p
                     JOIN product_inventory i ON p.id = i.product_id
                     WHERE i.track_inventory = true 
                       AND (i.quantity - i.reserved_quantity) <= COALESCE(i.alert_threshold, $1)
                     ORDER BY (i.quantity - i.reserved_quantity) ASC";

        match sqlx::query(query)
            .bind(default_threshold)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => {
                rows.iter().map(|row| {
                    let product = Self::row_to_product(row);
                    let inventory = Self::row_to_inventory(row);
                    (product, inventory)
                }).collect()
            }
            Err(e) => {
                error!("Error finding low stock products: {}", e);
                vec![]
            }
        }
    }

    async fn find_out_of_stock_products(&self) -> Vec<Product> {
        let query = "SELECT p.id, p.name, p.description, p.sku, p.brand, p.status, p.category_id, 
                           p.width, p.height, p.depth, p.weight, p.shipping_class, p.free_shipping, p.shipping_fee,
                           p.created_at, p.updated_at
                     FROM products p
                     JOIN product_inventory i ON p.id = i.product_id
                     WHERE i.track_inventory = true 
                       AND (i.quantity - i.reserved_quantity) <= 0
                     ORDER BY p.name";

        match sqlx::query(query)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(Self::row_to_product).collect(),
            Err(e) => {
                error!("Error finding out of stock products: {}", e);
                vec![]
            }
        }
    }
}