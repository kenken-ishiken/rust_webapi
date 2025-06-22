use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use std::sync::Arc;
use tracing::{error, info};

use crate::application::dto::product_dto::{
    BatchUpdateRequest, CreateProductRequest, ImageReorderRequest, InventoryRequest,
    PatchProductRequest, PriceRequest, ProductErrorResponse, ProductHistoryQuery,
    ProductImageRequest, ProductSearchQuery, UpdateProductRequest,
};
use crate::application::service::product_service::ProductService;
use crate::application::service::deletion_facade::DeletionFacade;
use crate::app_domain::service::deletion_service::DeleteKind;
use crate::infrastructure::auth::middleware::KeycloakUser;
use crate::infrastructure::error::AppError;

pub struct ProductHandler {
    service: Arc<ProductService>,
    deletion_facade: Arc<DeletionFacade>,
}

impl ProductHandler {
    pub fn new(service: Arc<ProductService>, deletion_facade: Arc<DeletionFacade>) -> Self {
        Self { service, deletion_facade }
    }

    // GET /api/products/{id}
    pub async fn get_product(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Fetching product {}", product_id);

        match data.service.find_by_id(&product_id).await {
            Ok(product) => {
                info!("Successfully fetched product {}", product_id);
                Ok(HttpResponse::Ok().json(product))
            }
            Err(error) => {
                error!("Failed to fetch product {}: {}", product_id, error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // GET /api/products/sku/{sku}
    pub async fn get_product_by_sku(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
    ) -> ActixResult<impl Responder> {
        let sku = path.into_inner();

        info!("Fetching product by SKU {}", sku);

        match data.service.find_by_sku(&sku).await {
            Ok(product) => {
                info!("Successfully fetched product by SKU {}", sku);
                Ok(HttpResponse::Ok().json(product))
            }
            Err(error) => {
                error!("Failed to fetch product by SKU {}: {}", sku, error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // GET /api/products (with search/filter capabilities)
    pub async fn search_products(
        data: web::Data<ProductHandler>,
        query: web::Query<ProductSearchQuery>,
    ) -> ActixResult<impl Responder> {
        info!("Searching products with query: {:?}", query.q);

        let response = data.service.search(query.into_inner()).await;

        info!("Found {} products", response.total);
        Ok(HttpResponse::Ok().json(response))
    }

    // POST /api/products
    pub async fn create_product(
        data: web::Data<ProductHandler>,
        _user: KeycloakUser,
        request: web::Json<CreateProductRequest>,
    ) -> ActixResult<impl Responder> {
        info!("Creating new product with SKU {}", request.sku);

        match data.service.create(request.into_inner()).await {
            Ok(product) => {
                info!("Successfully created product {}", product.id);
                Ok(HttpResponse::Created().json(product))
            }
            Err(error) => {
                error!("Failed to create product: {}", error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_SKU_DUPLICATE" => Ok(HttpResponse::Conflict().json(error_response)),
                    "PRODUCT_INVALID_NAME"
                    | "PRODUCT_INVALID_SKU"
                    | "INVALID_PRICE_RANGE"
                    | "INVALID_INVENTORY_QUANTITY" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/{id}
    pub async fn update_product(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<UpdateProductRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Updating product {}", product_id);

        match data.service.update(&product_id, request.into_inner()).await {
            Ok(product) => {
                info!("Successfully updated product {}", product_id);
                Ok(HttpResponse::Ok().json(product))
            }
            Err(error) => {
                error!("Failed to update product {}: {}", product_id, error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "PRODUCT_SKU_DUPLICATE" => Ok(HttpResponse::Conflict().json(error_response)),
                    "PRODUCT_INVALID_NAME"
                    | "PRODUCT_INVALID_SKU"
                    | "INVALID_PRICE_RANGE"
                    | "INVALID_INVENTORY_QUANTITY" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    "CATEGORY_NOT_FOUND" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PATCH /api/products/{id}
    pub async fn patch_product(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<PatchProductRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Patching product {}", product_id);

        match data.service.patch(&product_id, request.into_inner()).await {
            Ok(product) => {
                info!("Successfully patched product {}", product_id);
                Ok(HttpResponse::Ok().json(product))
            }
            Err(error) => {
                error!("Failed to patch product {}: {}", product_id, error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "INVALID_PRICE_RANGE" | "INVALID_INVENTORY_QUANTITY" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // DELETE /api/products/{id}
    pub async fn delete_product(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Deleting product {}", product_id);

        match data.deletion_facade.delete_product(product_id.clone(), DeleteKind::Physical).await {
            Ok(_) => {
                info!("Successfully deleted product {}", product_id);
                Ok(HttpResponse::NoContent().finish())
            }
            Err(error) => {
                error!("Failed to delete product {}: {}", product_id, error);
                match error {
                    AppError::NotFound(_) => {
                        Ok(HttpResponse::NotFound().json(serde_json::json!({
                            "error": {
                                "code": "PRODUCT_NOT_FOUND",
                                "message": "商品が見つかりません"
                            }
                        })))
                    }
                    _ => {
                        Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": {
                                "code": "INTERNAL_SERVER_ERROR",
                                "message": "削除処理中にエラーが発生しました"
                            }
                        })))
                    }
                }
            }
        }
    }

    // PUT /api/products/{id}/price
    pub async fn update_product_price(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<PriceRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Updating price for product {}", product_id);

        match data
            .service
            .update_price(&product_id, request.into_inner())
            .await
        {
            Ok(price) => {
                info!("Successfully updated price for product {}", product_id);
                Ok(HttpResponse::Ok().json(price))
            }
            Err(error) => {
                error!(
                    "Failed to update price for product {}: {}",
                    product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "INVALID_PRICE_RANGE" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/{id}/inventory
    pub async fn update_product_inventory(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<InventoryRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Updating inventory for product {}", product_id);

        match data
            .service
            .update_inventory(&product_id, request.into_inner())
            .await
        {
            Ok(inventory) => {
                info!("Successfully updated inventory for product {}", product_id);
                Ok(HttpResponse::Ok().json(inventory))
            }
            Err(error) => {
                error!(
                    "Failed to update inventory for product {}: {}",
                    product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "INVALID_INVENTORY_QUANTITY" => {
                        Ok(HttpResponse::BadRequest().json(error_response))
                    }
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // POST /api/products/{id}/images
    pub async fn add_product_image(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<ProductImageRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Adding image to product {}", product_id);

        match data
            .service
            .add_image(&product_id, request.into_inner())
            .await
        {
            Ok(image) => {
                info!(
                    "Successfully added image {} to product {}",
                    image.id, product_id
                );
                Ok(HttpResponse::Created().json(image))
            }
            Err(error) => {
                error!("Failed to add image to product {}: {}", product_id, error);
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "MAX_IMAGES_EXCEEDED" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/{id}/images/{image_id}
    pub async fn update_product_image(
        data: web::Data<ProductHandler>,
        path: web::Path<(String, String)>,
        _user: KeycloakUser,
        request: web::Json<ProductImageRequest>,
    ) -> ActixResult<impl Responder> {
        let (product_id, image_id) = path.into_inner();

        info!("Updating image {} for product {}", image_id, product_id);

        match data
            .service
            .update_image(&product_id, &image_id, request.into_inner())
            .await
        {
            Ok(image) => {
                info!(
                    "Successfully updated image {} for product {}",
                    image_id, product_id
                );
                Ok(HttpResponse::Ok().json(image))
            }
            Err(error) => {
                error!(
                    "Failed to update image {} for product {}: {}",
                    image_id, product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "IMAGE_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // DELETE /api/products/{id}/images/{image_id}
    pub async fn delete_product_image(
        data: web::Data<ProductHandler>,
        path: web::Path<(String, String)>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let (product_id, image_id) = path.into_inner();

        info!("Deleting image {} from product {}", image_id, product_id);

        match data.service.delete_image(&product_id, &image_id).await {
            Ok(_) => {
                info!(
                    "Successfully deleted image {} from product {}",
                    image_id, product_id
                );
                Ok(HttpResponse::NoContent().finish())
            }
            Err(error) => {
                error!(
                    "Failed to delete image {} from product {}: {}",
                    image_id, product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "IMAGE_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/{id}/images/reorder
    pub async fn reorder_product_images(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        _user: KeycloakUser,
        request: web::Json<ImageReorderRequest>,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Reordering images for product {}", product_id);

        match data
            .service
            .reorder_images(&product_id, request.into_inner())
            .await
        {
            Ok(_) => {
                info!("Successfully reordered images for product {}", product_id);
                Ok(HttpResponse::Ok()
                    .json(serde_json::json!({"message": "Images reordered successfully"})))
            }
            Err(error) => {
                error!(
                    "Failed to reorder images for product {}: {}",
                    product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "INVALID_IMAGE_ORDER" => Ok(HttpResponse::BadRequest().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/{id}/images/{image_id}/main
    pub async fn set_main_product_image(
        data: web::Data<ProductHandler>,
        path: web::Path<(String, String)>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let (product_id, image_id) = path.into_inner();

        info!("Setting main image {} for product {}", image_id, product_id);

        match data.service.set_main_image(&product_id, &image_id).await {
            Ok(_) => {
                info!(
                    "Successfully set main image {} for product {}",
                    image_id, product_id
                );
                Ok(HttpResponse::Ok()
                    .json(serde_json::json!({"message": "Main image set successfully"})))
            }
            Err(error) => {
                error!(
                    "Failed to set main image {} for product {}: {}",
                    image_id, product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    "IMAGE_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // PUT /api/products/batch
    pub async fn batch_update_products(
        data: web::Data<ProductHandler>,
        _user: KeycloakUser,
        request: web::Json<BatchUpdateRequest>,
    ) -> ActixResult<impl Responder> {
        let update_count = request.updates.len();
        info!("Batch updating {} products", update_count);

        match data.service.batch_update(request.into_inner()).await {
            Ok(response) => {
                info!(
                    "Batch update completed: {} success, {} errors",
                    response.success_count, response.error_count
                );
                Ok(HttpResponse::Ok().json(response))
            }
            Err(error) => {
                error!("Failed to perform batch update: {}", error);
                let error_response: ProductErrorResponse = error.into();
                Ok(HttpResponse::InternalServerError().json(error_response))
            }
        }
    }

    // GET /api/products/{id}/history
    pub async fn get_product_history(
        data: web::Data<ProductHandler>,
        path: web::Path<String>,
        query: web::Query<ProductHistoryQuery>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let product_id = path.into_inner();

        info!("Fetching history for product {}", product_id);

        match data
            .service
            .get_history(&product_id, query.into_inner())
            .await
        {
            Ok(history) => {
                info!(
                    "Successfully fetched {} history items for product {}",
                    history.total, product_id
                );
                Ok(HttpResponse::Ok().json(history))
            }
            Err(error) => {
                error!(
                    "Failed to fetch history for product {}: {}",
                    product_id, error
                );
                let error_response: ProductErrorResponse = error.into();
                match error_response.code.as_str() {
                    "PRODUCT_NOT_FOUND" => Ok(HttpResponse::NotFound().json(error_response)),
                    _ => Ok(HttpResponse::InternalServerError().json(error_response)),
                }
            }
        }
    }

    // GET /api/products/reports/low-stock
    pub async fn get_low_stock_products(
        data: web::Data<ProductHandler>,
        query: web::Query<LowStockQuery>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        let threshold = query.threshold;

        info!(
            "Fetching low stock products with threshold: {:?}",
            threshold
        );

        let products = data.service.find_low_stock_products(threshold).await;

        info!("Found {} low stock products", products.len());
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "products": products,
            "total": products.len()
        })))
    }

    // GET /api/products/reports/out-of-stock
    pub async fn get_out_of_stock_products(
        data: web::Data<ProductHandler>,
        _user: KeycloakUser,
    ) -> ActixResult<impl Responder> {
        info!("Fetching out of stock products");

        let products = data.service.find_out_of_stock_products().await;

        info!("Found {} out of stock products", products.len());
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "products": products,
            "total": products.len()
        })))
    }
}

// Query parameters for low stock report
#[derive(serde::Deserialize)]
pub struct LowStockQuery {
    pub threshold: Option<i32>,
}

// Product configuration function to register all routes
pub fn configure_product_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .route("", web::get().to(ProductHandler::search_products))
            .route("", web::post().to(ProductHandler::create_product))
            .route("/{id}", web::get().to(ProductHandler::get_product))
            .route("/{id}", web::put().to(ProductHandler::update_product))
            .route("/{id}", web::patch().to(ProductHandler::patch_product))
            .route("/{id}", web::delete().to(ProductHandler::delete_product))
            // Alternative access by SKU
            .route(
                "/sku/{sku}",
                web::get().to(ProductHandler::get_product_by_sku),
            )
            // Price operations
            .route(
                "/{id}/price",
                web::put().to(ProductHandler::update_product_price),
            )
            // Inventory operations
            .route(
                "/{id}/inventory",
                web::put().to(ProductHandler::update_product_inventory),
            )
            // Image operations
            .route(
                "/{id}/images",
                web::post().to(ProductHandler::add_product_image),
            )
            .route(
                "/{id}/images/{image_id}",
                web::put().to(ProductHandler::update_product_image),
            )
            .route(
                "/{id}/images/{image_id}",
                web::delete().to(ProductHandler::delete_product_image),
            )
            .route(
                "/{id}/images/reorder",
                web::put().to(ProductHandler::reorder_product_images),
            )
            .route(
                "/{id}/images/{image_id}/main",
                web::put().to(ProductHandler::set_main_product_image),
            )
            // Batch operations
            .route(
                "/batch",
                web::put().to(ProductHandler::batch_update_products),
            )
            // History
            .route(
                "/{id}/history",
                web::get().to(ProductHandler::get_product_history),
            )
            // Reports
            .route(
                "/reports/low-stock",
                web::get().to(ProductHandler::get_low_stock_products),
            )
            .route(
                "/reports/out-of-stock",
                web::get().to(ProductHandler::get_out_of_stock_products),
            ),
    );
}
