use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::app_domain::model::product::{
    Dimensions, Inventory, Price, Product, ProductError, ProductImage, ShippingInfo,
};
use crate::app_domain::repository::product_repository::ProductRepository;
use crate::application::dto::product_dto::{
    BatchUpdateRequest, BatchUpdateResponse, BatchUpdateResult, CreateProductRequest,
    ImageReorderRequest, InventoryRequest, InventoryResponse, PatchProductRequest, PriceRequest,
    PriceResponse, ProductHistoryQuery, ProductHistoryResponse, ProductImageRequest,
    ProductImageResponse, ProductListResponse, ProductResponse, ProductSearchQuery,
    UpdateProductRequest,
};
use crate::infrastructure::metrics::Metrics;

pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub fn new(repository: Arc<dyn ProductRepository>) -> Self {
        Self { repository }
    }

    pub async fn find_by_id(&self, id: &str) -> Result<ProductResponse, ProductError> {
        Metrics::with_metrics("product", "find_by_id", async {
            match self.repository.find_by_id(id).await {
                Some(product) => {
                    let mut response = ProductResponse::from(product);

                    // Populate related data
                    if let Some(price) = self.repository.get_current_price(id).await {
                        response.price = Some(price.into());
                    }

                    if let Some(inventory) = self.repository.get_inventory(id).await {
                        response.inventory = Some(inventory.into());
                    }

                    let images = self.repository.get_images(id).await;
                    response.images = images.into_iter().map(Into::into).collect();

                    response.tags = self.repository.get_tags(id).await;
                    response.attributes = self.repository.get_attributes(id).await;

                    info!("Fetched product {}", id);
                    Ok(response)
                }
                None => {
                    error!("Product {} not found", id);
                    Err(ProductError::ProductNotFound)
                }
            }
        })
        .await
    }

    pub async fn find_by_sku(&self, sku: &str) -> Result<ProductResponse, ProductError> {
        Metrics::with_metrics("product", "find_by_sku", async {
            match self.repository.find_by_sku(sku).await {
                Some(product) => {
                    let id = product.id.clone();
                    let mut response = ProductResponse::from(product);

                    // Populate related data
                    if let Some(price) = self.repository.get_current_price(&id).await {
                        response.price = Some(price.into());
                    }

                    if let Some(inventory) = self.repository.get_inventory(&id).await {
                        response.inventory = Some(inventory.into());
                    }

                    let images = self.repository.get_images(&id).await;
                    response.images = images.into_iter().map(Into::into).collect();

                    response.tags = self.repository.get_tags(&id).await;
                    response.attributes = self.repository.get_attributes(&id).await;

                    info!("Fetched product by SKU {}", sku);
                    Ok(response)
                }
                None => {
                    error!("Product with SKU {} not found", sku);
                    Err(ProductError::ProductNotFound)
                }
            }
        })
        .await
    }

    pub async fn search(&self, query: ProductSearchQuery) -> ProductListResponse {
        let tag_vec: Option<Vec<&str>> = query
            .tags
            .as_ref()
            .map(|tags| tags.split(',').map(|s| s.trim()).collect());

        let products = self
            .repository
            .search(
                &query.q.unwrap_or_default(),
                query.category_id.as_deref(),
                tag_vec,
                query.min_price,
                query.max_price,
                query.in_stock_only.unwrap_or(false),
                query.limit,
                query.offset,
            )
            .await;

        let mut product_responses = Vec::new();
        for product in &products {
            let mut response = ProductResponse::from(product.clone());

            // Populate basic related data for listing
            if let Some(price) = self.repository.get_current_price(&product.id).await {
                response.price = Some(price.into());
            }

            if let Some(inventory) = self.repository.get_inventory(&product.id).await {
                response.inventory = Some(inventory.into());
            }

            response.tags = self.repository.get_tags(&product.id).await;

            product_responses.push(response);
        }

        let total = product_responses.len() as i64;
        let has_more = query.limit.is_some_and(|limit| total >= limit);

        Metrics::record_success("product", "search");
        info!("Found {} products", products.len());

        ProductListResponse {
            products: product_responses,
            total,
            has_more,
        }
    }

    pub async fn create(
        &self,
        request: CreateProductRequest,
    ) -> Result<ProductResponse, ProductError> {
        // Check if SKU already exists
        if self.repository.exists_by_sku(&request.sku, None).await {
            Metrics::record_error("product", "create");
            return Err(ProductError::SkuAlreadyExists);
        }

        let product_id = Uuid::new_v4().to_string();

        // Create dimensions if provided
        let dimensions = if let Some(dim_req) = request.dimensions {
            Some(Dimensions::new(
                dim_req.width,
                dim_req.height,
                dim_req.depth,
            )?)
        } else {
            None
        };

        // Create shipping info
        let shipping_info = request.shipping_info.map(Into::into).unwrap_or_default();

        // Create product
        let mut product = Product::new(
            product_id.clone(),
            request.name,
            request.sku,
            request.status,
        )?;
        product.description = request.description;
        product.brand = request.brand;
        product.category_id = request.category_id;
        product.dimensions = dimensions;
        product.weight = request.weight;
        product.shipping_info = shipping_info;

        // Validate weight if provided
        if let Some(weight) = product.weight {
            product.update_weight(Some(weight))?;
        }

        // Create the product
        let _created_product = self.repository.create(product).await?;

        // Set initial price
        let price = Price::from(request.price);
        price.validate()?;
        self.repository
            .update_price(&product_id, price.clone())
            .await?;

        // Set initial inventory
        let inventory = Inventory::from(request.inventory);
        inventory.validate()?;
        self.repository
            .update_inventory(&product_id, inventory.clone())
            .await?;

        // Add tags if provided
        if let Some(tags) = request.tags {
            if !tags.is_empty() {
                self.repository.add_tags(&product_id, tags).await?;
            }
        }

        // Add attributes if provided
        if let Some(attributes) = request.attributes {
            if !attributes.is_empty() {
                self.repository
                    .set_attributes(&product_id, attributes)
                    .await?;
            }
        }

        Metrics::record_success("product", "create");
        info!("Created product {}", product_id);

        // Return full response
        self.find_by_id(&product_id).await
    }

    pub async fn update(
        &self,
        id: &str,
        request: UpdateProductRequest,
    ) -> Result<ProductResponse, ProductError> {
        let mut product = self
            .repository
            .find_by_id(id)
            .await
            .ok_or(ProductError::ProductNotFound)?;

        // Check SKU uniqueness if being updated
        if let Some(ref new_sku) = request.sku {
            if new_sku != &product.sku && self.repository.exists_by_sku(new_sku, Some(id)).await {
                Metrics::record_error("product", "update");
                return Err(ProductError::SkuAlreadyExists);
            }
        }

        // Update fields
        if let Some(name) = request.name {
            product.update_name(name)?;
        }

        if let Some(description) = request.description {
            product.update_description(Some(description));
        }

        if let Some(sku) = request.sku {
            product.update_sku(sku)?;
        }

        if let Some(brand) = request.brand {
            product.update_brand(Some(brand));
        }

        if let Some(status) = request.status {
            product.update_status(status);
        }

        if let Some(category_id) = request.category_id {
            product.update_category(Some(category_id));
        }

        if let Some(dimensions_req) = request.dimensions {
            let dimensions = Some(Dimensions::new(
                dimensions_req.width,
                dimensions_req.height,
                dimensions_req.depth,
            )?);
            product.update_dimensions(dimensions)?;
        }

        if let Some(weight) = request.weight {
            product.update_weight(Some(weight))?;
        }

        if let Some(shipping_info_req) = request.shipping_info {
            let shipping_info = ShippingInfo::from(shipping_info_req);
            product.update_shipping_info(shipping_info)?;
        }

        // Update the product
        self.repository.update(product).await?;

        // Update price if provided
        if let Some(price_req) = request.price {
            let price = Price::from(price_req);
            price.validate()?;
            self.repository.update_price(id, price).await?;
        }

        // Update inventory if provided
        if let Some(inventory_req) = request.inventory {
            let inventory = Inventory::from(inventory_req);
            inventory.validate()?;
            self.repository.update_inventory(id, inventory).await?;
        }

        // Update tags if provided
        if let Some(tags) = request.tags {
            self.repository.replace_tags(id, tags).await?;
        }

        // Update attributes if provided
        if let Some(attributes) = request.attributes {
            self.repository.set_attributes(id, attributes).await?;
        }

        Metrics::record_success("product", "update");
        info!("Updated product {}", id);

        self.find_by_id(id).await
    }

    pub async fn patch(
        &self,
        id: &str,
        request: PatchProductRequest,
    ) -> Result<ProductResponse, ProductError> {
        let mut product = self
            .repository
            .find_by_id(id)
            .await
            .ok_or(ProductError::ProductNotFound)?;

        // Update basic fields
        if let Some(name) = request.name {
            product.update_name(name)?;
        }

        if let Some(description) = request.description {
            product.update_description(Some(description));
        }

        if let Some(status) = request.status {
            product.update_status(status);
        }

        if let Some(category_id) = request.category_id {
            product.update_category(Some(category_id));
        }

        // Update the product
        self.repository.update(product).await?;

        // Update price if provided
        if let Some(price_patch) = request.price {
            if let Some(current_price) = self.repository.get_current_price(id).await {
                let mut new_price = current_price;

                if let Some(selling_price) = price_patch.selling_price {
                    new_price.selling_price = selling_price;
                }

                if let Some(list_price) = price_patch.list_price {
                    new_price.list_price = Some(list_price);
                }

                if let Some(discount_price) = price_patch.discount_price {
                    new_price.discount_price = Some(discount_price);
                }

                new_price.validate()?;
                self.repository.update_price(id, new_price).await?;
            }
        }

        // Update inventory if provided
        if let Some(inventory_patch) = request.inventory {
            if let Some(mut current_inventory) = self.repository.get_inventory(id).await {
                if let Some(quantity) = inventory_patch.quantity {
                    current_inventory.update_quantity(quantity)?;
                }

                if let Some(reserved_quantity) = inventory_patch.reserved_quantity {
                    current_inventory.reserved_quantity = reserved_quantity;
                }

                if let Some(alert_threshold) = inventory_patch.alert_threshold {
                    current_inventory.alert_threshold = Some(alert_threshold);
                }

                if let Some(track_inventory) = inventory_patch.track_inventory {
                    current_inventory.track_inventory = track_inventory;
                }

                if let Some(allow_backorder) = inventory_patch.allow_backorder {
                    current_inventory.allow_backorder = allow_backorder;
                }

                current_inventory.validate()?;
                self.repository
                    .update_inventory(id, current_inventory)
                    .await?;
            }
        }

        Metrics::record_success("product", "patch");
        info!("Patched product {}", id);

        self.find_by_id(id).await
    }

    pub async fn update_price(
        &self,
        id: &str,
        request: PriceRequest,
    ) -> Result<PriceResponse, ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "update_price");
            return Err(ProductError::ProductNotFound);
        }

        let price = Price::from(request);
        price.validate()?;

        let updated_price = self.repository.update_price(id, price).await?;

        Metrics::record_success("product", "update_price");
        info!("Updated price for product {}", id);

        Ok(updated_price.into())
    }

    pub async fn update_inventory(
        &self,
        id: &str,
        request: InventoryRequest,
    ) -> Result<InventoryResponse, ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "update_inventory");
            return Err(ProductError::ProductNotFound);
        }

        let inventory = Inventory::from(request);
        inventory.validate()?;

        let updated_inventory = self.repository.update_inventory(id, inventory).await?;

        Metrics::record_success("product", "update_inventory");
        info!("Updated inventory for product {}", id);

        Ok(updated_inventory.into())
    }

    pub async fn add_image(
        &self,
        id: &str,
        request: ProductImageRequest,
    ) -> Result<ProductImageResponse, ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "add_image");
            return Err(ProductError::ProductNotFound);
        }

        let image = ProductImage::from(request);
        let added_image = self.repository.add_image(id, image).await?;

        Metrics::record_success("product", "add_image");
        info!("Added image {} to product {}", added_image.id, id);

        Ok(added_image.into())
    }

    pub async fn update_image(
        &self,
        id: &str,
        image_id: &str,
        request: ProductImageRequest,
    ) -> Result<ProductImageResponse, ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "update_image");
            return Err(ProductError::ProductNotFound);
        }

        let mut image = ProductImage::from(request);
        image.id = image_id.to_string();

        let updated_image = self.repository.update_image(id, image).await?;

        Metrics::record_success("product", "update_image");
        info!("Updated image {} for product {}", image_id, id);

        Ok(updated_image.into())
    }

    pub async fn delete_image(&self, id: &str, image_id: &str) -> Result<(), ProductError> {
        self.repository.delete_image(id, image_id).await?;

        Metrics::record_success("product", "delete_image");
        info!("Deleted image {} from product {}", image_id, id);

        Ok(())
    }

    pub async fn reorder_images(
        &self,
        id: &str,
        request: ImageReorderRequest,
    ) -> Result<(), ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "reorder_images");
            return Err(ProductError::ProductNotFound);
        }

        let image_orders = request
            .image_orders
            .into_iter()
            .map(|item| (item.image_id, item.sort_order))
            .collect();

        self.repository.reorder_images(id, image_orders).await?;

        Metrics::record_success("product", "reorder_images");
        info!("Reordered images for product {}", id);

        Ok(())
    }

    pub async fn set_main_image(&self, id: &str, image_id: &str) -> Result<(), ProductError> {
        self.repository.set_main_image(id, image_id).await?;

        Metrics::record_success("product", "set_main_image");
        info!("Set main image {} for product {}", image_id, id);

        Ok(())
    }

    pub async fn get_history(
        &self,
        id: &str,
        query: ProductHistoryQuery,
    ) -> Result<ProductHistoryResponse, ProductError> {
        // Verify product exists
        if self.repository.find_by_id(id).await.is_none() {
            Metrics::record_error("product", "get_history");
            return Err(ProductError::ProductNotFound);
        }

        let history_items = self
            .repository
            .get_history(id, query.field.as_deref(), query.limit, query.offset)
            .await;

        let total = history_items.len() as i64;
        let has_more = query.limit.is_some_and(|limit| total >= limit);

        let history_response = history_items.into_iter().map(Into::into).collect();

        Metrics::record_success("product", "get_history");
        info!("Fetched {} history items for product {}", total, id);

        Ok(ProductHistoryResponse {
            history: history_response,
            total,
            has_more,
        })
    }

    pub async fn batch_update(
        &self,
        request: BatchUpdateRequest,
    ) -> Result<BatchUpdateResponse, ProductError> {
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for update_item in request.updates {
            let result = match self.apply_batch_update(&update_item).await {
                Ok(product) => {
                    success_count += 1;
                    BatchUpdateResult {
                        id: update_item.id.clone(),
                        success: true,
                        product: Some(product),
                        error: None,
                    }
                }
                Err(error) => {
                    error_count += 1;
                    BatchUpdateResult {
                        id: update_item.id.clone(),
                        success: false,
                        product: None,
                        error: Some(error.to_string()),
                    }
                }
            };
            results.push(result);
        }

        Metrics::record_success("product", "batch_update");
        info!(
            "Batch update completed: {} success, {} errors",
            success_count, error_count
        );

        Ok(BatchUpdateResponse {
            results,
            success_count,
            error_count,
        })
    }

    async fn apply_batch_update(
        &self,
        update_item: &crate::application::dto::product_dto::BatchUpdateItem,
    ) -> Result<ProductResponse, ProductError> {
        let mut product = self
            .repository
            .find_by_id(&update_item.id)
            .await
            .ok_or(ProductError::ProductNotFound)?;

        // Update name if provided
        if let Some(ref name) = update_item.name {
            product.update_name(name.clone())?;
        }

        // Update status if provided
        if let Some(ref status) = update_item.status {
            product.update_status(status.clone());
        }

        // Update the product
        self.repository.update(product).await?;

        // Update price if provided
        if let Some(ref price_patch) = update_item.price {
            if let Some(current_price) = self.repository.get_current_price(&update_item.id).await {
                let mut new_price = current_price;

                if let Some(selling_price) = price_patch.selling_price {
                    new_price.selling_price = selling_price;
                }

                if let Some(list_price) = price_patch.list_price {
                    new_price.list_price = Some(list_price);
                }

                if let Some(discount_price) = price_patch.discount_price {
                    new_price.discount_price = Some(discount_price);
                }

                new_price.validate()?;
                self.repository
                    .update_price(&update_item.id, new_price)
                    .await?;
            }
        }

        // Update inventory if provided
        if let Some(ref inventory_patch) = update_item.inventory {
            if let Some(mut current_inventory) =
                self.repository.get_inventory(&update_item.id).await
            {
                if let Some(quantity) = inventory_patch.quantity {
                    current_inventory.update_quantity(quantity)?;
                }

                if let Some(reserved_quantity) = inventory_patch.reserved_quantity {
                    current_inventory.reserved_quantity = reserved_quantity;
                }

                if let Some(alert_threshold) = inventory_patch.alert_threshold {
                    current_inventory.alert_threshold = Some(alert_threshold);
                }

                if let Some(track_inventory) = inventory_patch.track_inventory {
                    current_inventory.track_inventory = track_inventory;
                }

                if let Some(allow_backorder) = inventory_patch.allow_backorder {
                    current_inventory.allow_backorder = allow_backorder;
                }

                current_inventory.validate()?;
                self.repository
                    .update_inventory(&update_item.id, current_inventory)
                    .await?;
            }
        }

        self.find_by_id(&update_item.id).await
    }

    pub async fn find_low_stock_products(
        &self,
        threshold: Option<i32>,
    ) -> Vec<(ProductResponse, InventoryResponse)> {
        let low_stock_products = self.repository.find_low_stock_products(threshold).await;

        let mut results = Vec::new();
        for (product, inventory) in low_stock_products {
            let product_response = ProductResponse::from(product);
            let inventory_response = InventoryResponse::from(inventory);
            results.push((product_response, inventory_response));
        }

        Metrics::record_success("product", "find_low_stock");
        info!("Found {} low stock products", results.len());

        results
    }

    pub async fn find_out_of_stock_products(&self) -> Vec<ProductResponse> {
        let out_of_stock_products = self.repository.find_out_of_stock_products().await;

        let results = out_of_stock_products
            .into_iter()
            .map(ProductResponse::from)
            .collect::<Vec<_>>();

        Metrics::record_success("product", "find_out_of_stock");
        info!("Found {} out of stock products", results.len());

        results
    }
}
