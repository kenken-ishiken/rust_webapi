use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::error;

use crate::app_domain::model::product::{
    Inventory, Price, Product, ProductError, ProductHistory, ProductImage,
};
use crate::app_domain::repository::product_repository::ProductRepository;
use super::converters::{
    row_to_product, row_to_inventory
};
use super::product_extensions::ProductExtensions;
use super::product_metadata::ProductMetadata;

pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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

        match sqlx::query(query).bind(id).fetch_optional(&self.pool).await {
            Ok(Some(row)) => Some(row_to_product(&row)),
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
            Ok(Some(row)) => Some(row_to_product(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error finding product by sku {}: {}", sku, e);
                None
            }
        }
    }

    // async fn find_all(&self,
    //     category_id: Option<&str>,
    //     status: Option<&str>,
    //     limit: Option<i64>,
    //     offset: Option<i64>,
    // ) -> Vec<Product> {
    //     let mut query = "SELECT id, name, description, sku, brand, status, category_id,
    //                            width, height, depth, weight, shipping_class, free_shipping, shipping_fee,
    //                            created_at, updated_at
    //                      FROM products WHERE 1=1".to_string();

    //     let mut param_index = 1;

    //     if let Some(_cat_id) = category_id {
    //         query.push_str(&format!(" AND category_id = ${}", param_index));
    //         param_index += 1;
    //     }

    //     if let Some(_status_val) = status {
    //         query.push_str(&format!(" AND status = ${}", param_index));
    //         param_index += 1;
    //     }

    //     query.push_str(" ORDER BY created_at DESC");

    //     if let Some(_limit_val) = limit {
    //         query.push_str(&format!(" LIMIT ${}", param_index));
    //         param_index += 1;
    //     }

    //     if let Some(_offset_val) = offset {
    //         query.push_str(&format!(" OFFSET ${}", param_index));
    //     }

    //     let mut sqlx_query = sqlx::query(&query);
    //     if let Some(cat_id) = category_id {
    //         sqlx_query = sqlx_query.bind(cat_id);
    //     }
    //     if let Some(status_val) = status {
    //         sqlx_query = sqlx_query.bind(status_val);
    //     }
    //     if let Some(limit_val) = limit {
    //         sqlx_query = sqlx_query.bind(limit_val);
    //     }
    //     if let Some(offset_val) = offset {
    //         sqlx_query = sqlx_query.bind(offset_val);
    //     }

    //     match sqlx_query.fetch_all(&self.pool).await {
    //         Ok(rows) => rows.iter().map(row_to_product).collect(),
    //         Err(e) => {
    //             error!("Error finding products: {}", e);
    //             vec![]
    //         }
    //     }
    // }

    async fn create(&self, product: Product) -> Result<Product, ProductError> {
        let mut tx = self
            .pool
            .begin()
            .await
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

                tx.commit()
                    .await
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

        match sqlx::query(query).bind(id).execute(&self.pool).await {
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
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.get_current_price(product_id).await
    }

    // async fn get_price_history(&self, product_id: &str, limit: Option<i64>) -> Vec<Price> {
    //     let mut query = "SELECT selling_price, list_price, discount_price, currency, tax_included,
    //                            effective_from, effective_until
    //                      FROM product_prices
    //                      WHERE product_id = $1
    //                      ORDER BY created_at DESC".to_string();

    //     if let Some(limit_val) = limit {
    //         query.push_str(&format!(" LIMIT {}", limit_val));
    //     }

    //     match sqlx::query(&query)
    //         .bind(product_id)
    //         .fetch_all(&self.pool)
    //         .await
    //     {
    //         Ok(rows) => rows.iter().map(row_to_price).collect(),
    //         Err(e) => {
    //             error!("Error getting price history for product {}: {}", product_id, e);
    //             vec![]
    //         }
    //     }
    // }

    async fn update_price(&self, product_id: &str, price: Price) -> Result<Price, ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.update_price(product_id, price).await
    }

    async fn get_inventory(&self, product_id: &str) -> Option<Inventory> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.get_inventory(product_id).await
    }

    async fn update_inventory(
        &self,
        product_id: &str,
        inventory: Inventory,
    ) -> Result<Inventory, ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.update_inventory(product_id, inventory).await
    }

    // async fn reserve_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError> {
    //     if quantity <= 0 {
    //         return Err(ProductError::InvalidInventoryQuantity);
    //     }

    //     let query = "UPDATE product_inventory
    //                  SET reserved_quantity = reserved_quantity + $2
    //                  WHERE product_id = $1 AND quantity >= reserved_quantity + $2";

    //     match sqlx::query(query)
    //         .bind(product_id)
    //         .bind(quantity)
    //         .execute(&self.pool)
    //         .await
    //     {
    //         Ok(result) if result.rows_affected() > 0 => Ok(()),
    //         Ok(_) => Err(ProductError::InvalidInventoryQuantity),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    // async fn release_inventory(&self, product_id: &str, quantity: i32) -> Result<(), ProductError> {
    //     if quantity <= 0 {
    //         return Err(ProductError::InvalidInventoryQuantity);
    //     }

    //     let query = "UPDATE product_inventory
    //                  SET reserved_quantity = GREATEST(0, reserved_quantity - $2)
    //                  WHERE product_id = $1";

    //     match sqlx::query(query)
    //         .bind(product_id)
    //         .bind(quantity)
    //         .execute(&self.pool)
    //         .await
    //     {
    //         Ok(result) if result.rows_affected() > 0 => Ok(()),
    //         Ok(_) => Err(ProductError::ProductNotFound),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    async fn get_images(&self, product_id: &str) -> Vec<ProductImage> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.get_images(product_id).await
    }

    async fn add_image(
        &self,
        product_id: &str,
        image: ProductImage,
    ) -> Result<ProductImage, ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.add_image(product_id, image).await
    }

    async fn update_image(
        &self,
        product_id: &str,
        image: ProductImage,
    ) -> Result<ProductImage, ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.update_image(product_id, image).await
    }

    async fn delete_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.delete_image(product_id, image_id).await
    }

    async fn reorder_images(
        &self,
        product_id: &str,
        image_orders: Vec<(String, i32)>,
    ) -> Result<(), ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.reorder_images(product_id, image_orders).await
    }

    async fn set_main_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError> {
        let extensions = ProductExtensions { pool: &self.pool };
        extensions.set_main_image(product_id, image_id).await
    }

    async fn get_tags(&self, product_id: &str) -> Vec<String> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.get_tags_for_product(product_id).await
    }

    async fn add_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.add_tags(product_id, tags).await
    }

    // async fn remove_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
    //     if tags.is_empty() {
    //         return Ok(());
    //     }

    //     let placeholders: Vec<String> = (1..=tags.len()).map(|i| format!("${}", i + 1)).collect();
    //     let query = format!("DELETE FROM product_tags WHERE product_id = $1 AND tag IN ({})",
    //                        placeholders.join(", "));

    //     let mut sqlx_query = sqlx::query(&query).bind(product_id);
    //     for tag in tags {
    //         sqlx_query = sqlx_query.bind(tag);
    //     }

    //     match sqlx_query.execute(&self.pool).await {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    async fn replace_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.replace_tags(product_id, tags).await
    }

    async fn get_attributes(&self, product_id: &str) -> HashMap<String, String> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.get_attributes_for_product(product_id).await
    }

    async fn set_attributes(
        &self,
        product_id: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), ProductError> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.set_attributes(product_id, attributes).await
    }

    // async fn set_attribute(&self, product_id: &str, name: &str, value: &str) -> Result<(), ProductError> {
    //     let query = "INSERT INTO product_attributes (product_id, attribute_name, attribute_value)
    //                  VALUES ($1, $2, $3)
    //                  ON CONFLICT (product_id, attribute_name)
    //                  DO UPDATE SET attribute_value = $3, updated_at = NOW()";

    //     match sqlx::query(query)
    //         .bind(product_id)
    //         .bind(name)
    //         .bind(value)
    //         .execute(&self.pool)
    //         .await
    //     {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    // async fn remove_attribute(&self, product_id: &str, name: &str) -> Result<(), ProductError> {
    //     let query = "DELETE FROM product_attributes WHERE product_id = $1 AND attribute_name = $2";

    //     match sqlx::query(query)
    //         .bind(product_id)
    //         .bind(name)
    //         .execute(&self.pool)
    //         .await
    //     {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    async fn get_history(
        &self,
        product_id: &str,
        field_name: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Vec<ProductHistory> {
        let metadata = ProductMetadata { pool: &self.pool };
        metadata.get_history(product_id, field_name, limit, offset).await
    }

    // async fn add_history_entry(&self,
    //     product_id: &str,
    //     field_name: &str,
    //     old_value: Option<&str>,
    //     new_value: Option<&str>,
    //     changed_by: Option<&str>,
    //     reason: Option<&str>,
    // ) -> Result<(), ProductError> {
    //     let query = "INSERT INTO product_history (product_id, field_name, old_value, new_value, changed_by, reason)
    //                  VALUES ($1, $2, $3, $4, $5, $6)";

    //     match sqlx::query(query)
    //         .bind(product_id)
    //         .bind(field_name)
    //         .bind(old_value)
    //         .bind(new_value)
    //         .bind(changed_by)
    //         .bind(reason)
    //         .execute(&self.pool)
    //         .await
    //     {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(ProductError::DatabaseError(e.to_string())),
    //     }
    // }

    // async fn update_batch(&self, updates: Vec<(String, Product)>) -> Result<Vec<Product>, ProductError> {
    //     let mut results = Vec::new();
    //     let mut tx = self.pool.begin().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     for (id, product) in updates {
    //         let query = "UPDATE products
    //                      SET name = $2, description = $3, sku = $4, brand = $5, status = $6, category_id = $7,
    //                          width = $8, height = $9, depth = $10, weight = $11,
    //                          shipping_class = $12, free_shipping = $13, shipping_fee = $14, updated_at = NOW()
    //                      WHERE id = $1";

    //         let width = product.dimensions.as_ref().map(|d| d.width);
    //         let height = product.dimensions.as_ref().map(|d| d.height);
    //         let depth = product.dimensions.as_ref().map(|d| d.depth);

    //         let result = sqlx::query(query)
    //             .bind(&id)
    //             .bind(&product.name)
    //             .bind(&product.description)
    //             .bind(&product.sku)
    //             .bind(&product.brand)
    //             .bind(product.status.to_string())
    //             .bind(&product.category_id)
    //             .bind(width)
    //             .bind(height)
    //             .bind(depth)
    //             .bind(product.weight)
    //             .bind(&product.shipping_info.shipping_class)
    //             .bind(product.shipping_info.free_shipping)
    //             .bind(product.shipping_info.shipping_fee)
    //             .execute(&mut *tx)
    //             .await;

    //         match result {
    //             Ok(_) => results.push(product),
    //             Err(e) => {
    //                 let _ = tx.rollback().await;
    //                 return Err(ProductError::DatabaseError(e.to_string()));
    //             }
    //         }
    //     }

    //     tx.commit().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     Ok(results)
    // }

    // async fn update_prices_batch(&self, updates: Vec<(String, Price)>) -> Result<Vec<Price>, ProductError> {
    //     let mut results = Vec::new();
    //     let mut tx = self.pool.begin().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     for (product_id, price) in updates {
    //         price.validate()?;

    //         let query = "INSERT INTO product_prices (product_id, selling_price, list_price, discount_price,
    //                                                currency, tax_included, effective_from, effective_until)
    //                      VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";

    //         let result = sqlx::query(query)
    //             .bind(&product_id)
    //             .bind(price.selling_price)
    //             .bind(price.list_price)
    //             .bind(price.discount_price)
    //             .bind(&price.currency)
    //             .bind(price.tax_included)
    //             .bind(price.effective_from)
    //             .bind(price.effective_until)
    //             .execute(&mut *tx)
    //             .await;

    //         match result {
    //             Ok(_) => results.push(price),
    //             Err(e) => {
    //                 let _ = tx.rollback().await;
    //                 return Err(ProductError::DatabaseError(e.to_string()));
    //             }
    //         }
    //     }

    //     tx.commit().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     Ok(results)
    // }

    // async fn update_inventory_batch(&self, updates: Vec<(String, Inventory)>) -> Result<Vec<Inventory>, ProductError> {
    //     let mut results = Vec::new();
    //     let mut tx = self.pool.begin().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     for (product_id, inventory) in updates {
    //         inventory.validate()?;

    //         let query = "UPDATE product_inventory
    //                      SET quantity = $2, reserved_quantity = $3, alert_threshold = $4,
    //                          track_inventory = $5, allow_backorder = $6, updated_at = NOW()
    //                      WHERE product_id = $1";

    //         let result = sqlx::query(query)
    //             .bind(&product_id)
    //             .bind(inventory.quantity)
    //             .bind(inventory.reserved_quantity)
    //             .bind(inventory.alert_threshold)
    //             .bind(inventory.track_inventory)
    //             .bind(inventory.allow_backorder)
    //             .execute(&mut *tx)
    //             .await;

    //         match result {
    //             Ok(_) => results.push(inventory),
    //             Err(e) => {
    //                 let _ = tx.rollback().await;
    //                 return Err(ProductError::DatabaseError(e.to_string()));
    //             }
    //         }
    //     }

    //     tx.commit().await
    //         .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

    //     Ok(results)
    // }

    #[allow(clippy::too_many_arguments)]
    async fn search(
        &self,
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
            conditions.push(format!(
                "(p.name ILIKE ${} OR p.description ILIKE ${} OR p.sku ILIKE ${})",
                param_index,
                param_index + 1,
                param_index + 2
            ));
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
            joins.push(
                "AND (pp.effective_until IS NULL OR pp.effective_until >= NOW())".to_string(),
            );

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
            Ok(rows) => rows.iter().map(row_to_product).collect(),
            Err(e) => {
                error!("Error searching products: {}", e);
                vec![]
            }
        }
    }

    // async fn find_by_category_recursive(&self, category_id: &str) -> Vec<Product> {
    //     // This would require a recursive CTE to find all subcategories
    //     // For simplicity, just finding direct children for now
    //     let query = "SELECT p.id, p.name, p.description, p.sku, p.brand, p.status, p.category_id,
    //                        p.width, p.height, p.depth, p.weight, p.shipping_class, p.free_shipping, p.shipping_fee,
    //                        p.created_at, p.updated_at
    //                  FROM products p
    //                  WHERE p.category_id = $1
    //                     OR p.category_id IN (
    //                         SELECT id FROM categories WHERE parent_id = $1
    //                     )
    //                  ORDER BY p.created_at DESC";

    //     match sqlx::query(query)
    //         .bind(category_id)
    //         .fetch_all(&self.pool)
    //         .await
    //     {
    //         Ok(rows) => rows.iter().map(row_to_product).collect(),
    //         Err(e) => {
    //             error!("Error finding products by category {}: {}", category_id, e);
    //             vec![]
    //         }
    //     }
    // }

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
            Ok(rows) => rows
                .iter()
                .map(|row| {
                    let product = row_to_product(row);
                    let inventory = row_to_inventory(row);
                    (product, inventory)
                })
                .collect(),
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

        match sqlx::query(query).fetch_all(&self.pool).await {
            Ok(rows) => rows.iter().map(row_to_product).collect(),
            Err(e) => {
                error!("Error finding out of stock products: {}", e);
                vec![]
            }
        }
    }
}
