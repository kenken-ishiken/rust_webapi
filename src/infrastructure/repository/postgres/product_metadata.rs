use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::error;

use crate::app_domain::model::product::{ProductError, ProductHistory};
use super::converters::row_to_product_history;

/// Product repository extensions for tags, attributes, and history management
pub struct ProductMetadata<'a> {
    pub pool: &'a PgPool,
}

impl<'a> ProductMetadata<'a> {
    pub async fn get_tags_for_product(&self, product_id: &str) -> Vec<String> {
        let query = "SELECT tag FROM product_tags WHERE product_id = $1 ORDER BY tag";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(self.pool)
            .await
        {
            Ok(rows) => rows.iter().map(|row| row.get("tag")).collect(),
            Err(e) => {
                error!("Error fetching tags for product {}: {}", product_id, e);
                vec![]
            }
        }
    }

    pub async fn add_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        if tags.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool
            .begin()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        for tag in tags {
            let query =
                "INSERT INTO product_tags (product_id, tag) VALUES ($1, $2) ON CONFLICT DO NOTHING";

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

        tx.commit()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn replace_tags(&self, product_id: &str, tags: Vec<String>) -> Result<(), ProductError> {
        let mut tx = self.pool
            .begin()
            .await
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

        tx.commit()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_attributes_for_product(&self, product_id: &str) -> HashMap<String, String> {
        let query =
            "SELECT attribute_name, attribute_value FROM product_attributes WHERE product_id = $1";

        match sqlx::query(query)
            .bind(product_id)
            .fetch_all(self.pool)
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
                error!(
                    "Error fetching attributes for product {}: {}",
                    product_id, e
                );
                HashMap::new()
            }
        }
    }

    pub async fn set_attributes(
        &self,
        product_id: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), ProductError> {
        let mut tx = self.pool
            .begin()
            .await
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
            let insert_query =
                "INSERT INTO product_attributes (product_id, attribute_name, attribute_value)
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

        tx.commit()
            .await
            .map_err(|e| ProductError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_history(
        &self,
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

        match sqlx_query.fetch_all(self.pool).await {
            Ok(rows) => rows.iter().map(row_to_product_history).collect(),
            Err(e) => {
                error!("Error getting history for product {}: {}", product_id, e);
                vec![]
            }
        }
    }
} 