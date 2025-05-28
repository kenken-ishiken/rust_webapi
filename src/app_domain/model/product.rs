use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sku: String,
    pub brand: Option<String>,
    pub status: ProductStatus,
    pub category_id: Option<String>,
    pub dimensions: Option<Dimensions>,
    pub weight: Option<Decimal>,
    pub shipping_info: ShippingInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub selling_price: Decimal,
    pub list_price: Option<Decimal>,
    pub discount_price: Option<Decimal>,
    pub currency: String,
    pub tax_included: bool,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub alert_threshold: Option<i32>,
    pub track_inventory: bool,
    pub allow_backorder: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: String,
    pub url: String,
    pub alt_text: Option<String>,
    pub sort_order: i32,
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: Decimal,
    pub height: Decimal,
    pub depth: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingInfo {
    pub shipping_class: String,
    pub free_shipping: bool,
    pub shipping_fee: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProductStatus {
    Active,
    Inactive,
    Draft,
    Discontinued,
}

impl std::fmt::Display for ProductStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductStatus::Active => write!(f, "Active"),
            ProductStatus::Inactive => write!(f, "Inactive"),
            ProductStatus::Draft => write!(f, "Draft"),
            ProductStatus::Discontinued => write!(f, "Discontinued"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductHistory {
    pub id: i64,
    pub product_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProductError {
    InvalidName,
    InvalidSku,
    SkuAlreadyExists,
    InvalidPrice,
    InvalidPriceRelationship,
    InvalidInventoryQuantity,
    InvalidDimensions,
    InvalidWeight,
    InvalidShippingFee,
    TooManyImages,
    ImageNotFound,
    InvalidImageOrder,
    CategoryNotFound,
    ProductNotFound,
    InsufficientPermissions,
    DatabaseError(String),
}

impl std::fmt::Display for ProductError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductError::InvalidName => write!(f, "Product name is invalid"),
            ProductError::InvalidSku => write!(f, "Product SKU is invalid"),
            ProductError::SkuAlreadyExists => write!(f, "SKU already exists"),
            ProductError::InvalidPrice => write!(f, "Price is invalid"),
            ProductError::InvalidPriceRelationship => write!(f, "Price relationship is invalid"),
            ProductError::InvalidInventoryQuantity => write!(f, "Inventory quantity is invalid"),
            ProductError::InvalidDimensions => write!(f, "Dimensions are invalid"),
            ProductError::InvalidWeight => write!(f, "Weight is invalid"),
            ProductError::InvalidShippingFee => write!(f, "Shipping fee is invalid"),
            ProductError::TooManyImages => write!(f, "Too many images"),
            ProductError::ImageNotFound => write!(f, "Image not found"),
            ProductError::InvalidImageOrder => write!(f, "Invalid image order"),
            ProductError::CategoryNotFound => write!(f, "Category not found"),
            ProductError::ProductNotFound => write!(f, "Product not found"),
            ProductError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            ProductError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ProductError {}

impl Product {
    pub fn new(
        id: String,
        name: String,
        sku: String,
        status: ProductStatus,
    ) -> Result<Self, ProductError> {
        Self::validate_name(&name)?;
        Self::validate_sku(&sku)?;

        Ok(Product {
            id,
            name,
            description: None,
            sku,
            brand: None,
            status,
            category_id: None,
            dimensions: None,
            weight: None,
            shipping_info: ShippingInfo::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn validate_name(name: &str) -> Result<(), ProductError> {
        if name.trim().is_empty() || name.len() > 200 {
            return Err(ProductError::InvalidName);
        }
        Ok(())
    }

    pub fn validate_sku(sku: &str) -> Result<(), ProductError> {
        if sku.trim().is_empty() || sku.len() > 50 {
            return Err(ProductError::InvalidSku);
        }
        
        if !sku.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ProductError::InvalidSku);
        }
        
        Ok(())
    }

    pub fn update_name(&mut self, name: String) -> Result<(), ProductError> {
        Self::validate_name(&name)?;
        self.name = name;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_sku(&mut self, sku: String) -> Result<(), ProductError> {
        Self::validate_sku(&sku)?;
        self.sku = sku;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn update_brand(&mut self, brand: Option<String>) {
        self.brand = brand;
        self.updated_at = Utc::now();
    }

    pub fn update_status(&mut self, status: ProductStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn update_category(&mut self, category_id: Option<String>) {
        self.category_id = category_id;
        self.updated_at = Utc::now();
    }

    pub fn update_dimensions(&mut self, dimensions: Option<Dimensions>) -> Result<(), ProductError> {
        if let Some(ref dims) = dimensions {
            dims.validate()?;
        }
        self.dimensions = dimensions;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_weight(&mut self, weight: Option<Decimal>) -> Result<(), ProductError> {
        if let Some(w) = weight {
            if w <= Decimal::ZERO {
                return Err(ProductError::InvalidWeight);
            }
        }
        self.weight = weight;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_shipping_info(&mut self, shipping_info: ShippingInfo) -> Result<(), ProductError> {
        shipping_info.validate()?;
        self.shipping_info = shipping_info;
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl Price {
    pub fn new(
        selling_price: Decimal,
        currency: String,
        tax_included: bool,
    ) -> Result<Self, ProductError> {
        if selling_price <= Decimal::ZERO {
            return Err(ProductError::InvalidPrice);
        }

        Ok(Price {
            selling_price,
            list_price: None,
            discount_price: None,
            currency,
            tax_included,
            effective_from: None,
            effective_until: None,
        })
    }

    pub fn validate(&self) -> Result<(), ProductError> {
        if self.selling_price <= Decimal::ZERO {
            return Err(ProductError::InvalidPrice);
        }

        if let Some(list_price) = self.list_price {
            if list_price <= Decimal::ZERO || self.selling_price > list_price {
                return Err(ProductError::InvalidPriceRelationship);
            }
        }

        if let Some(discount_price) = self.discount_price {
            if discount_price <= Decimal::ZERO || discount_price > self.selling_price {
                return Err(ProductError::InvalidPriceRelationship);
            }
        }

        if let (Some(from), Some(until)) = (self.effective_from, self.effective_until) {
            if from > until {
                return Err(ProductError::InvalidPriceRelationship);
            }
        }

        Ok(())
    }
}

impl Inventory {
    pub fn new(quantity: i32) -> Result<Self, ProductError> {
        if quantity < 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        Ok(Inventory {
            quantity,
            reserved_quantity: 0,
            alert_threshold: None,
            track_inventory: true,
            allow_backorder: false,
        })
    }

    pub fn validate(&self) -> Result<(), ProductError> {
        if self.quantity < 0 || self.reserved_quantity < 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        if self.reserved_quantity > self.quantity {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        if let Some(threshold) = self.alert_threshold {
            if threshold < 0 {
                return Err(ProductError::InvalidInventoryQuantity);
            }
        }

        Ok(())
    }

    pub fn update_quantity(&mut self, quantity: i32) -> Result<(), ProductError> {
        if quantity < 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }
        
        if quantity < self.reserved_quantity {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        self.quantity = quantity;
        Ok(())
    }

    pub fn reserve_quantity(&mut self, amount: i32) -> Result<(), ProductError> {
        if amount < 0 {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        let new_reserved = self.reserved_quantity + amount;
        if new_reserved > self.quantity {
            return Err(ProductError::InvalidInventoryQuantity);
        }

        self.reserved_quantity = new_reserved;
        Ok(())
    }
}

impl Dimensions {
    pub fn new(width: Decimal, height: Decimal, depth: Decimal) -> Result<Self, ProductError> {
        if width <= Decimal::ZERO || height <= Decimal::ZERO || depth <= Decimal::ZERO {
            return Err(ProductError::InvalidDimensions);
        }

        Ok(Dimensions {
            width,
            height,
            depth,
        })
    }

    pub fn validate(&self) -> Result<(), ProductError> {
        if self.width <= Decimal::ZERO || self.height <= Decimal::ZERO || self.depth <= Decimal::ZERO {
            return Err(ProductError::InvalidDimensions);
        }
        Ok(())
    }
}

impl ShippingInfo {
    pub fn validate(&self) -> Result<(), ProductError> {
        if self.shipping_fee < Decimal::ZERO {
            return Err(ProductError::InvalidShippingFee);
        }
        Ok(())
    }
}

impl Default for ShippingInfo {
    fn default() -> Self {
        ShippingInfo {
            shipping_class: "standard".to_string(),
            free_shipping: false,
            shipping_fee: Decimal::ZERO,
        }
    }
}

impl ProductImage {
    pub fn new(
        id: String,
        url: String,
        sort_order: i32,
        is_main: bool,
    ) -> Result<Self, ProductError> {
        if url.trim().is_empty() {
            return Err(ProductError::ImageNotFound);
        }

        if sort_order < 0 {
            return Err(ProductError::InvalidImageOrder);
        }

        Ok(ProductImage {
            id,
            url,
            alt_text: None,
            sort_order,
            is_main,
        })
    }

    pub fn validate(&self) -> Result<(), ProductError> {
        if self.url.trim().is_empty() {
            return Err(ProductError::ImageNotFound);
        }

        if self.sort_order < 0 {
            return Err(ProductError::InvalidImageOrder);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let product = Product::new(
            "prod_123".to_string(),
            "Test Product".to_string(),
            "TEST-SKU-123".to_string(),
            ProductStatus::Draft,
        );

        assert!(product.is_ok());
        let product = product.unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.sku, "TEST-SKU-123");
        assert_eq!(product.status, ProductStatus::Draft);
    }

    #[test]
    fn test_invalid_product_name() {
        let result = Product::new(
            "prod_123".to_string(),
            "".to_string(),
            "TEST-SKU-123".to_string(),
            ProductStatus::Draft,
        );

        assert!(matches!(result, Err(ProductError::InvalidName)));
    }

    #[test]
    fn test_invalid_sku() {
        let result = Product::new(
            "prod_123".to_string(),
            "Test Product".to_string(),
            "INVALID SKU!".to_string(),
            ProductStatus::Draft,
        );

        assert!(matches!(result, Err(ProductError::InvalidSku)));
    }

    #[test]
    fn test_price_validation() {
        let price = Price::new(
            Decimal::from(100),
            "JPY".to_string(),
            true,
        );

        assert!(price.is_ok());

        let invalid_price = Price::new(
            Decimal::ZERO,
            "JPY".to_string(),
            true,
        );

        assert!(matches!(invalid_price, Err(ProductError::InvalidPrice)));
    }

    #[test]
    fn test_inventory_validation() {
        let inventory = Inventory::new(10);
        assert!(inventory.is_ok());

        let invalid_inventory = Inventory::new(-1);
        assert!(matches!(invalid_inventory, Err(ProductError::InvalidInventoryQuantity)));
    }

    #[test]
    fn test_dimensions_validation() {
        let dimensions = Dimensions::new(
            Decimal::from(10),
            Decimal::from(20),
            Decimal::from(30),
        );
        assert!(dimensions.is_ok());

        let invalid_dimensions = Dimensions::new(
            Decimal::ZERO,
            Decimal::from(20),
            Decimal::from(30),
        );
        assert!(matches!(invalid_dimensions, Err(ProductError::InvalidDimensions)));
    }
}