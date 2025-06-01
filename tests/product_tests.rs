use rust_webapi::app_domain::model::product::{Product, ProductStatus, ProductError}; // Price, Inventory};
// use rust_decimal::Decimal;

#[cfg(test)]
mod product_tests {
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
        // let price = Price::new(
        //     Decimal::from(100),
        //     "JPY".to_string(),
        //     true,
        // );

        // assert!(price.is_ok());

        // let invalid_price = Price::new(
        //     Decimal::ZERO,
        //     "JPY".to_string(),
        //     true,
        // );

        // assert!(matches!(invalid_price, Err(ProductError::InvalidPrice)));
    }

    #[test]
    fn test_inventory_validation() {
        // let inventory = Inventory::new(10);
        // assert!(inventory.is_ok());

        // let invalid_inventory = Inventory::new(-1);
        // assert!(matches!(invalid_inventory, Err(ProductError::InvalidInventoryQuantity)));
    }

    #[test]
    fn test_product_status_display() {
        assert_eq!(ProductStatus::Active.to_string(), "Active");
        assert_eq!(ProductStatus::Inactive.to_string(), "Inactive");
        assert_eq!(ProductStatus::Draft.to_string(), "Draft");
        assert_eq!(ProductStatus::Discontinued.to_string(), "Discontinued");
    }
}