use actix_web::{web, App, HttpServer, middleware};
use tracing_actix_web::TracingLogger;
use actix_web::dev::Service;
use std::time::{Instant, Duration};

use crate::infrastructure::metrics::{Metrics, metrics_handler};
use crate::presentation::api::{
    item_handler::ItemHandler,
    user_handler::UserHandler,
    category_handler::{configure_category_routes},
    product_handler::{configure_product_routes},
};
use crate::infrastructure::di::container::AppContainer;

/// HTTPサーバーを構築する
pub fn build_http_server(
    container: &AppContainer,
    addr: &str,
) -> std::io::Result<actix_web::dev::Server> {
    let server = HttpServer::new({
        let item_handler = container.item_handler.clone();
        let user_handler = container.user_handler.clone();
        let category_handler = container.category_handler.clone();
        let product_handler = container.product_handler.clone();
        let keycloak_auth = container.keycloak_auth.clone();
        
        move || {
            App::new()
                .app_data(item_handler.clone())
                .app_data(user_handler.clone())
                .app_data(category_handler.clone())
                .app_data(product_handler.clone())
                .app_data(keycloak_auth.clone())
                // Configure JSON handling with size limit
                .app_data(
                    web::JsonConfig::default()
                        .limit(4096) // 4KB limit for JSON payloads
                        .error_handler(|err, _req| {
                            use crate::infrastructure::error::AppError;
                            AppError::bad_request(err.to_string()).into()
                        })
                )
                // Enable response compression
                .wrap(middleware::Compress::default())
                // Normalize paths (remove trailing slashes)
                .wrap(middleware::NormalizePath::trim())
                // HTTP request tracing middleware
                .wrap(TracingLogger::default())
                // Metrics middleware: record request counts and durations
                .wrap_fn(|req, srv| {
                    // Clone path for labeling; skip metrics endpoint
                    let path = req.path().to_string();
                    let start = Instant::now();
                    let fut = srv.call(req);
                    async move {
                        let res = fut.await?;
                        if path != "/api/metrics" {
                            let elapsed = start.elapsed().as_secs_f64();
                            // Record request duration
                            Metrics::record_duration("rust_webapi", &path, elapsed);
                            // Count success vs error based on status code
                            if res.status().is_server_error() {
                                Metrics::record_error("rust_webapi", &path);
                            } else {
                                Metrics::record_success("rust_webapi", &path);
                            }
                        }
                        Ok(res)
                    }
                })
                .route("/", web::get().to(ItemHandler::index))
                .service(
                    web::scope("/api")
                        // 認証不要のエンドポイント
                        .route("/health", web::get().to(|| async { "OK" }))
                        .route("/metrics", web::get().to(metrics_handler))
                        // 認証必要のエンドポイント
                        .route("/items", web::get().to(ItemHandler::get_items))
                        .route("/items", web::post().to(ItemHandler::create_item))
                        .route("/items/{id}", web::get().to(ItemHandler::get_item))
                        .route("/items/{id}", web::put().to(ItemHandler::update_item))
                        .route("/items/{id}", web::delete().to(ItemHandler::delete_item))
                        // New product deletion API routes
                        .route(
                            "/products/{id}",
                            web::delete().to(ItemHandler::logical_delete_item),
                        )
                        .route(
                            "/products/{id}/permanent",
                            web::delete().to(ItemHandler::physical_delete_item),
                        )
                        .route(
                            "/products/{id}/restore",
                            web::post().to(ItemHandler::restore_item),
                        )
                        .route(
                            "/products/{id}/deletion-check",
                            web::get().to(ItemHandler::validate_item_deletion),
                        )
                        .route(
                            "/products/batch",
                            web::delete().to(ItemHandler::batch_delete_items),
                        )
                        .route(
                            "/products/deleted",
                            web::get().to(ItemHandler::get_deleted_items),
                        )
                        .route(
                            "/products/{id}/deletion-log",
                            web::get().to(ItemHandler::get_item_deletion_log),
                        )
                        .route(
                            "/deletion-logs",
                            web::get().to(ItemHandler::get_deletion_logs),
                        )
                        .route("/users", web::get().to(UserHandler::get_users))
                        .route("/users", web::post().to(UserHandler::create_user))
                        .route("/users/{id}", web::get().to(UserHandler::get_user))
                        .route("/users/{id}", web::put().to(UserHandler::update_user))
                        .route("/users/{id}", web::delete().to(UserHandler::delete_user))
                        // Configure categories and products routes
                        .configure(configure_category_routes)
                        .configure(configure_product_routes),
                )
        }
    })
    // Performance optimizations
    .workers(num_cpus::get() * 2) // Optimize worker threads
    .keep_alive(Duration::from_secs(75)) // Keep-alive timeout
    .client_request_timeout(Duration::from_secs(60)) // Client request timeout
    .client_disconnect_timeout(Duration::from_secs(5)) // Client disconnect timeout
    .bind(addr)?
    .run();
    
    Ok(server)
} 