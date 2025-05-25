use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app_domain::model::product::{
    Product, ProductStatus, Price, Inventory, ProductImage, Dimensions, 
    ShippingInfo, ProductHistory, ProductError,
};

// Request DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub sku: String,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub price: PriceRequest,
    pub inventory: InventoryRequest,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub attributes: Option<HashMap<String, String>>,
    pub dimensions: Option<DimensionsRequest>,
    pub weight: Option<Decimal>,
    pub shipping_info: Option<ShippingInfoRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub brand: Option<String>,
    pub status: Option<ProductStatus>,
    pub price: Option<PriceRequest>,
    pub inventory: Option<InventoryRequest>,
    pub category_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub attributes: Option<HashMap<String, String>>,
    pub dimensions: Option<DimensionsRequest>,
    pub weight: Option<Decimal>,
    pub shipping_info: Option<ShippingInfoRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<PricePatchRequest>,
    pub inventory: Option<InventoryPatchRequest>,
    pub status: Option<ProductStatus>,
    pub category_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceRequest {
    pub selling_price: Decimal,
    pub list_price: Option<Decimal>,
    pub discount_price: Option<Decimal>,
    pub currency: String,
    pub tax_included: bool,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricePatchRequest {
    pub selling_price: Option<Decimal>,
    pub list_price: Option<Decimal>,
    pub discount_price: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryRequest {
    pub quantity: i32,
    pub reserved_quantity: Option<i32>,
    pub alert_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
    pub allow_backorder: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryPatchRequest {
    pub quantity: Option<i32>,
    pub reserved_quantity: Option<i32>,
    pub alert_threshold: Option<i32>,
    pub track_inventory: Option<bool>,
    pub allow_backorder: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DimensionsRequest {
    pub width: Decimal,
    pub height: Decimal,
    pub depth: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShippingInfoRequest {
    pub shipping_class: String,
    pub free_shipping: bool,
    pub shipping_fee: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductImageRequest {
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_main: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageReorderRequest {
    pub image_orders: Vec<ImageOrderItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageOrderItem {
    pub image_id: String,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUpdateRequest {
    pub updates: Vec<BatchUpdateItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUpdateItem {
    pub id: String,
    pub name: Option<String>,
    pub price: Option<PricePatchRequest>,
    pub inventory: Option<InventoryPatchRequest>,
    pub status: Option<ProductStatus>,
}

// Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sku: String,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub price: Option<PriceResponse>,
    pub inventory: Option<InventoryResponse>,
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub images: Vec<ProductImageResponse>,
    pub dimensions: Option<DimensionsResponse>,
    pub weight: Option<Decimal>,
    pub shipping_info: ShippingInfoResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductResponse>,
    pub total: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceResponse {
    pub selling_price: Decimal,
    pub list_price: Option<Decimal>,
    pub discount_price: Option<Decimal>,
    pub currency: String,
    pub tax_included: bool,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryResponse {
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub alert_threshold: Option<i32>,
    pub track_inventory: bool,
    pub allow_backorder: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductImageResponse {
    pub id: String,
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_main: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DimensionsResponse {
    pub width: Decimal,
    pub height: Decimal,
    pub depth: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShippingInfoResponse {
    pub shipping_class: String,
    pub free_shipping: bool,
    pub shipping_fee: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductHistoryResponse {
    pub history: Vec<ProductHistoryItem>,
    pub total: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductHistoryItem {
    pub id: i64,
    pub product_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUpdateResponse {
    pub results: Vec<BatchUpdateResult>,
    pub success_count: i32,
    pub error_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUpdateResult {
    pub id: String,
    pub success: bool,
    pub product: Option<ProductResponse>,
    pub error: Option<String>,
}

// Query DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSearchQuery {
    pub q: Option<String>,
    pub category_id: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub in_stock_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductHistoryQuery {
    pub field: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Error Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<ProductErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductErrorDetails {
    pub field: Option<String>,
    pub value: Option<String>,
    pub constraint: Option<String>,
    pub additional_info: Option<HashMap<String, String>>,
}

// Conversion implementations
impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        ProductResponse {
            id: product.id,
            name: product.name,
            description: product.description,
            sku: product.sku,
            brand: product.brand,
            status: product.status,
            price: None, // Will be populated separately
            inventory: None, // Will be populated separately
            category_id: product.category_id,
            tags: Vec::new(), // Will be populated separately
            attributes: HashMap::new(), // Will be populated separately
            images: Vec::new(), // Will be populated separately
            dimensions: product.dimensions.map(Into::into),
            weight: product.weight,
            shipping_info: product.shipping_info.into(),
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }
}

impl From<Price> for PriceResponse {
    fn from(price: Price) -> Self {
        PriceResponse {
            selling_price: price.selling_price,
            list_price: price.list_price,
            discount_price: price.discount_price,
            currency: price.currency,
            tax_included: price.tax_included,
            effective_from: price.effective_from,
            effective_until: price.effective_until,
        }
    }
}

impl From<Inventory> for InventoryResponse {
    fn from(inventory: Inventory) -> Self {
        InventoryResponse {
            quantity: inventory.quantity,
            reserved_quantity: inventory.reserved_quantity,
            alert_threshold: inventory.alert_threshold,
            track_inventory: inventory.track_inventory,
            allow_backorder: inventory.allow_backorder,
        }
    }
}

impl From<ProductImage> for ProductImageResponse {
    fn from(image: ProductImage) -> Self {
        ProductImageResponse {
            id: image.id,
            url: image.url,
            alt_text: image.alt_text,
            sort_order: image.sort_order,
            is_main: image.is_main,
        }
    }
}

impl From<Dimensions> for DimensionsResponse {
    fn from(dimensions: Dimensions) -> Self {
        DimensionsResponse {
            width: dimensions.width,
            height: dimensions.height,
            depth: dimensions.depth,
        }
    }
}

impl From<ShippingInfo> for ShippingInfoResponse {
    fn from(shipping_info: ShippingInfo) -> Self {
        ShippingInfoResponse {
            shipping_class: shipping_info.shipping_class,
            free_shipping: shipping_info.free_shipping,
            shipping_fee: shipping_info.shipping_fee,
        }
    }
}

impl From<ProductHistory> for ProductHistoryItem {
    fn from(history: ProductHistory) -> Self {
        ProductHistoryItem {
            id: history.id,
            product_id: history.product_id,
            field: history.field_name,
            old_value: history.old_value,
            new_value: history.new_value,
            changed_by: history.changed_by,
            changed_at: history.changed_at,
            reason: history.reason,
        }
    }
}

impl From<PriceRequest> for Price {
    fn from(request: PriceRequest) -> Self {
        Price {
            selling_price: request.selling_price,
            list_price: request.list_price,
            discount_price: request.discount_price,
            currency: request.currency,
            tax_included: request.tax_included,
            effective_from: request.effective_from,
            effective_until: request.effective_until,
        }
    }
}

impl From<InventoryRequest> for Inventory {
    fn from(request: InventoryRequest) -> Self {
        Inventory {
            quantity: request.quantity,
            reserved_quantity: request.reserved_quantity.unwrap_or(0),
            alert_threshold: request.alert_threshold,
            track_inventory: request.track_inventory.unwrap_or(true),
            allow_backorder: request.allow_backorder.unwrap_or(false),
        }
    }
}

impl From<DimensionsRequest> for Dimensions {
    fn from(request: DimensionsRequest) -> Self {
        Dimensions {
            width: request.width,
            height: request.height,
            depth: request.depth,
        }
    }
}

impl From<ShippingInfoRequest> for ShippingInfo {
    fn from(request: ShippingInfoRequest) -> Self {
        ShippingInfo {
            shipping_class: request.shipping_class,
            free_shipping: request.free_shipping,
            shipping_fee: request.shipping_fee,
        }
    }
}

impl From<ProductImageRequest> for ProductImage {
    fn from(request: ProductImageRequest) -> Self {
        ProductImage {
            id: uuid::Uuid::new_v4().to_string(),
            url: request.url,
            alt_text: request.alt_text,
            sort_order: request.sort_order,
            is_main: request.is_main,
        }
    }
}

impl From<ProductError> for ProductErrorResponse {
    fn from(error: ProductError) -> Self {
        let (code, message, details) = match error {
            ProductError::InvalidName => (
                "PRODUCT_INVALID_NAME".to_string(),
                "商品名が無効です".to_string(),
                Some(ProductErrorDetails {
                    field: Some("name".to_string()),
                    value: None,
                    constraint: Some("1文字以上200文字以下である必要があります".to_string()),
                    additional_info: None,
                }),
            ),
            ProductError::InvalidSku => (
                "PRODUCT_INVALID_SKU".to_string(),
                "商品コードが無効です".to_string(),
                Some(ProductErrorDetails {
                    field: Some("sku".to_string()),
                    value: None,
                    constraint: Some("英数字とハイフンのみ、1文字以上50文字以下".to_string()),
                    additional_info: None,
                }),
            ),
            ProductError::SkuAlreadyExists => (
                "PRODUCT_SKU_DUPLICATE".to_string(),
                "SKUが重複しています".to_string(),
                None,
            ),
            ProductError::InvalidPrice => (
                "INVALID_PRICE_RANGE".to_string(),
                "価格の範囲が不正です".to_string(),
                Some(ProductErrorDetails {
                    field: Some("price".to_string()),
                    value: None,
                    constraint: Some("価格は0より大きい必要があります".to_string()),
                    additional_info: None,
                }),
            ),
            ProductError::InvalidPriceRelationship => (
                "INVALID_PRICE_RANGE".to_string(),
                "販売価格は定価以下、割引価格は販売価格以下である必要があります".to_string(),
                None,
            ),
            ProductError::InvalidInventoryQuantity => (
                "INVALID_INVENTORY_QUANTITY".to_string(),
                "在庫数量が不正です".to_string(),
                Some(ProductErrorDetails {
                    field: Some("inventory.quantity".to_string()),
                    value: None,
                    constraint: Some("在庫数量は0以上、予約数量は在庫数量以下".to_string()),
                    additional_info: None,
                }),
            ),
            ProductError::TooManyImages => (
                "MAX_IMAGES_EXCEEDED".to_string(),
                "最大画像数を超過しています".to_string(),
                Some(ProductErrorDetails {
                    field: Some("images".to_string()),
                    value: None,
                    constraint: Some("最大10枚まで".to_string()),
                    additional_info: None,
                }),
            ),
            ProductError::ProductNotFound => (
                "PRODUCT_NOT_FOUND".to_string(),
                "商品が見つかりません".to_string(),
                None,
            ),
            ProductError::CategoryNotFound => (
                "CATEGORY_NOT_FOUND".to_string(),
                "指定されたカテゴリが存在しません".to_string(),
                None,
            ),
            ProductError::InsufficientPermissions => (
                "INSUFFICIENT_PERMISSIONS".to_string(),
                "編集権限がありません".to_string(),
                None,
            ),
            _ => (
                "INTERNAL_ERROR".to_string(),
                error.to_string(),
                None,
            ),
        };

        ProductErrorResponse {
            code,
            message,
            details,
        }
    }
}