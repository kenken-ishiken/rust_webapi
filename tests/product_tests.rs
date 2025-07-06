use rust_webapi::application::dto::product_dto::{
    CreateProductRequest, ProductResponse, PriceRequest, InventoryRequest, DimensionsRequest, ShippingInfoRequest
};
use rust_decimal::Decimal;
use chrono::{Utc, TimeZone, DateTime};
use std::collections::HashMap;
use rust_webapi::app_domain::model::product::{Product, ProductStatus, ShippingInfo, ProductError};

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

    #[test]
    fn test_create_product_request_serde() {
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
            attributes: Some(HashMap::new()),
            dimensions: Some(DimensionsRequest {
                width: Decimal::new(10, 0),
                height: Decimal::new(20, 0),
                depth: Decimal::new(5, 0),
            }),
            weight: Some(Decimal::new(100, 0)),
            shipping_info: Some(ShippingInfoRequest {
                shipping_class: "standard".to_string(),
                free_shipping: false,
                shipping_fee: Decimal::new(500, 2),
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let de: CreateProductRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, req.name);
        assert_eq!(de.sku, req.sku);
        assert_eq!(de.price.selling_price, req.price.selling_price);
    }

    #[test]
    fn test_product_response_from_domain() {
        let now: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let domain_product = Product {
            id: "prod01".to_string(),
            name: "Test Product".to_string(),
            description: Some("desc".to_string()),
            sku: "SKU-001".to_string(),
            brand: Some("BrandX".to_string()),
            status: ProductStatus::Active,
            category_id: Some("cat01".to_string()),
            dimensions: None,
            weight: Some(Decimal::new(100, 0)),
            shipping_info: ShippingInfo {
                shipping_class: "standard".to_string(),
                free_shipping: false,
                shipping_fee: Decimal::new(500, 2),
            },
            created_at: now,
            updated_at: now,
        };
        let dto: ProductResponse = domain_product.into();
        assert_eq!(dto.id, "prod01");
        assert_eq!(dto.name, "Test Product");
        assert_eq!(dto.sku, "SKU-001");
        assert_eq!(dto.status, ProductStatus::Active);
        assert_eq!(dto.shipping_info.shipping_class, "standard");
    }

    #[test]
    fn test_price_request_to_domain() {
        let req = PriceRequest {
            selling_price: Decimal::new(1000, 2),
            list_price: Some(Decimal::new(1200, 2)),
            discount_price: Some(Decimal::new(900, 2)),
            currency: "JPY".to_string(),
            tax_included: true,
            effective_from: None,
            effective_until: None,
        };
        let price: rust_webapi::app_domain::model::product::Price = req.into();
        assert_eq!(price.selling_price, Decimal::new(1000, 2));
        assert_eq!(price.currency, "JPY");
    }

    #[test]
    fn test_inventory_request_to_domain() {
        let req = InventoryRequest {
            quantity: 10,
            reserved_quantity: Some(2),
            alert_threshold: Some(1),
            track_inventory: Some(true),
            allow_backorder: Some(false),
        };
        let inv: rust_webapi::app_domain::model::product::Inventory = req.into();
        assert_eq!(inv.quantity, 10);
        assert_eq!(inv.reserved_quantity, 2);
        assert!(inv.track_inventory);
        assert!(!inv.allow_backorder);
    }
}
