use sqlx::{PgPool, Row};
use tracing::error;

use super::converters::{row_to_inventory, row_to_price, row_to_product_image};
use crate::app_domain::model::product::{Inventory, Price, ProductError, ProductImage};

/// Product repository extensions for price, inventory, and image management
pub struct ProductExtensions<'a> {
    pub pool: &'a PgPool,
}

impl ProductExtensions<'_> {
    pub async fn get_current_price(&self, product_id: &str) -> Option<Price> {
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
            .fetch_optional(self.pool)
            .await
        {
            Ok(Some(row)) => Some(row_to_price(&row)),
            Ok(None) => None,
            Err(e) => {
                error!(
                    "Error getting current price for product {}: {}",
                    product_id, e
                );
                None
            }
        }
    }

    pub async fn update_price(
        &self,
        product_id: &str,
        price: Price,
    ) -> Result<Price, ProductError> {
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
            .execute(self.pool)
            .await
        {
            Ok(_) => Ok(price),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    pub async fn get_inventory(&self, product_id: &str) -> Option<Inventory> {
        let query =
            "SELECT quantity, reserved_quantity, alert_threshold, track_inventory, allow_backorder
                     FROM product_inventory 
                     WHERE product_id = $1";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_optional(self.pool)
            .await
        {
            Ok(Some(row)) => Some(row_to_inventory(&row)),
            Ok(None) => None,
            Err(e) => {
                error!("Error getting inventory for product {}: {}", product_id, e);
                None
            }
        }
    }

    pub async fn update_inventory(
        &self,
        product_id: &str,
        inventory: Inventory,
    ) -> Result<Inventory, ProductError> {
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
            .execute(self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(inventory),
            Ok(_) => Err(ProductError::ProductNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    pub async fn get_images(&self, product_id: &str) -> Vec<ProductImage> {
        let query = "SELECT id, url, alt_text, sort_order, is_main
                     FROM product_images 
                     WHERE product_id = $1 
                     ORDER BY sort_order, created_at";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(row_to_product_image).collect(),
            Err(e) => {
                error!("Error getting images for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    pub async fn add_image(
        &self,
        product_id: &str,
        image: ProductImage,
    ) -> Result<ProductImage, ProductError> {
        // Check image count limit
        let count_query = "SELECT COUNT(*) as count FROM product_images WHERE product_id = $1";
        let count_result = sqlx::query(count_query)
            .bind(product_id)
            .fetch_one(self.pool)
            .await;

        if let Ok(row) = count_result {
            let count: i64 = row.get("count");
            if count >= 10 {
                return Err(ProductError::TooManyImages);
            }
        }

        image.validate()?;

        let mut tx = self
            .pool
            .begin()
            .await
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

        let query =
            "INSERT INTO product_images (id, product_id, url, alt_text, sort_order, is_main)
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
                tx.commit()
                    .await
                    .map_err(|e| ProductError::DatabaseError(e.to_string()))?;
                Ok(image)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(ProductError::DatabaseError(e.to_string()))
            }
        }
    }

    pub async fn update_image(
        &self,
        product_id: &str,
        image: ProductImage,
    ) -> Result<ProductImage, ProductError> {
        image.validate()?;

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        // If setting as main image, unset others
        if image.is_main {
            let unset_query =
                "UPDATE product_images SET is_main = false WHERE product_id = $1 AND id != $2";
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
                tx.commit()
                    .await
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

    pub async fn delete_image(&self, product_id: &str, image_id: &str) -> Result<(), ProductError> {
        let query = "DELETE FROM product_images WHERE product_id = $1 AND id = $2";

        match sqlx::query(query)
            .bind(product_id)
            .bind(image_id)
            .execute(self.pool)
            .await
        {
            Ok(result) if result.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ProductError::ImageNotFound),
            Err(e) => Err(ProductError::DatabaseError(e.to_string())),
        }
    }

    pub async fn reorder_images(
        &self,
        product_id: &str,
        image_orders: Vec<(String, i32)>,
    ) -> Result<(), ProductError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for (image_id, sort_order) in image_orders {
            if sort_order < 0 {
                let _ = tx.rollback().await;
                return Err(ProductError::InvalidImageOrder);
            }

            let query =
                "UPDATE product_images SET sort_order = $3 WHERE product_id = $1 AND id = $2";

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

        tx.commit()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn set_main_image(
        &self,
        product_id: &str,
        image_id: &str,
    ) -> Result<(), ProductError> {
        let mut tx = self
            .pool
            .begin()
            .await
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
        let set_query =
            "UPDATE product_images SET is_main = true WHERE product_id = $1 AND id = $2";
        let result = sqlx::query(set_query)
            .bind(product_id)
            .bind(image_id)
            .execute(&mut *tx)
            .await;

        match result {
            Ok(result) if result.rows_affected() > 0 => {
                tx.commit()
                    .await
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
}
