use rust_webapi::app_domain::model::product::{Product, ProductStatus};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_product_module_compilation() {
        // This test just verifies that the product module compiles correctly
        let product = Product::new(
            "test_id".to_string(),
            "Test Product".to_string(),
            "TEST-123".to_string(),
            ProductStatus::Active,
        );

        assert!(product.is_ok());
    }
}
