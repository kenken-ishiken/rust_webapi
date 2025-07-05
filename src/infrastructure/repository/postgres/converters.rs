use rust_decimal::Decimal;
use sqlx::Row;

use crate::app_domain::model::product::{
    Dimensions, Inventory, Price, Product, ProductHistory, ProductImage, ProductStatus,
    ShippingInfo,
};

/// SQLクエリ結果をProductエンティティに変換
pub fn row_to_product(row: &sqlx::postgres::PgRow) -> Product {
    let dimensions = if let (Some(width), Some(height), Some(depth)) = (
        row.try_get::<Option<Decimal>, _>("width").unwrap_or(None),
        row.try_get::<Option<Decimal>, _>("height").unwrap_or(None),
        row.try_get::<Option<Decimal>, _>("depth").unwrap_or(None),
    ) {
        Some(Dimensions {
            width,
            height,
            depth,
        })
    } else {
        None
    };

    let shipping_info = ShippingInfo {
        shipping_class: row
            .try_get("shipping_class")
            .unwrap_or_else(|_| "standard".to_string()),
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

/// SQLクエリ結果をPriceエンティティに変換
pub fn row_to_price(row: &sqlx::postgres::PgRow) -> Price {
    Price {
        selling_price: row.get("selling_price"),
        list_price: row.try_get("list_price").unwrap_or(None),
        discount_price: row.try_get("discount_price").unwrap_or(None),
        currency: row
            .try_get("currency")
            .unwrap_or_else(|_| "JPY".to_string()),
        tax_included: row.try_get("tax_included").unwrap_or(true),
        effective_from: row.try_get("effective_from").unwrap_or(None),
        effective_until: row.try_get("effective_until").unwrap_or(None),
    }
}

/// SQLクエリ結果をInventoryエンティティに変換
pub fn row_to_inventory(row: &sqlx::postgres::PgRow) -> Inventory {
    Inventory {
        quantity: row.get("quantity"),
        reserved_quantity: row.get("reserved_quantity"),
        alert_threshold: row.try_get("alert_threshold").unwrap_or(None),
        track_inventory: row.try_get("track_inventory").unwrap_or(true),
        allow_backorder: row.try_get("allow_backorder").unwrap_or(false),
    }
}

/// SQLクエリ結果をProductImageエンティティに変換
pub fn row_to_product_image(row: &sqlx::postgres::PgRow) -> ProductImage {
    ProductImage {
        id: row.get("id"),
        url: row.get("url"),
        alt_text: row.try_get("alt_text").unwrap_or(None),
        sort_order: row.get("sort_order"),
        is_main: row.get("is_main"),
    }
}

/// SQLクエリ結果をProductHistoryエンティティに変換
pub fn row_to_product_history(row: &sqlx::postgres::PgRow) -> ProductHistory {
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
