mod application;
mod app_domain;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use dotenvy::dotenv;
// Tracing for structured logging
use tracing::{info, error};
// gRPC imports
use tonic::transport::Server;
use tracing_actix_web::TracingLogger;
use sqlx::postgres::PgPoolOptions;

// Remove slog-based JSON logger; using tracing for structured logging
// Middleware: TracingLogger for HTTP request logging
// (replace slog middleware with tracing-actix-web)
use crate::infrastructure::metrics::{
    init_metrics,
    metrics_handler,
    increment_success_counter,
    increment_error_counter,
    observe_request_duration,
};
use crate::infrastructure::tracing::init_tracing;
use std::time::Instant;
use actix_web::dev::Service;

use crate::app_domain::repository::item_repository::ItemRepository;
use crate::infrastructure::repository::item_repository::PostgresItemRepository;
use crate::application::service::item_service::ItemService;
use crate::presentation::api::item_handler::ItemHandler;

use domain::repository::user_repository::UserRepositoryImpl;
use crate::infrastructure::repository::user_repository::PostgresUserRepository;
use crate::application::service::user_service::UserService;
use crate::presentation::api::user_handler::UserHandler;

use crate::infrastructure::auth::keycloak::{KeycloakAuth, KeycloakConfig};

use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::infrastructure::repository::category_repository::PostgresCategoryRepository;
use crate::application::service::category_service::CategoryService;
use crate::presentation::api::category_handler::{CategoryHandler, configure_category_routes};

use crate::app_domain::repository::product_repository::ProductRepository;
use crate::infrastructure::repository::product_repository::PostgresProductRepository;
use crate::application::service::product_service::ProductService;
use crate::presentation::api::product_handler::{ProductHandler, configure_product_routes};

// gRPC imports
use crate::presentation::grpc::user_service::{UserServiceImpl, UserServiceServer};
use crate::presentation::grpc::item_service::{ItemServiceImpl, ItemServiceServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 環境変数の読み込み
    dotenv().ok();

    // Initialize tracing and OpenTelemetry (Datadog compatible)
    init_tracing().expect("failed to initialize tracing");
    info!("Tracing initialized");
    
    init_metrics();
    info!("Metrics initialized");

    // データベース接続プールの作成
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url).await {
            Ok(pool) => {
                info!("✅ PostgreSQL データベース接続に成功しました");
                pool
            },
            Err(e) => {
                error!("❌ PostgreSQL データベース接続に失敗しました: {}", e);
                std::process::exit(1);
            }
        };

    // リポジトリの作成
    let item_repository: Arc<dyn ItemRepository + Send + Sync> = Arc::new(PostgresItemRepository::new(pool.clone()));
    let user_repository: UserRepositoryImpl = Arc::new(PostgresUserRepository::new(pool.clone()));
    let category_repository: Arc<dyn CategoryRepository> = Arc::new(PostgresCategoryRepository::new(pool.clone()));
    let product_repository: Arc<dyn ProductRepository> = Arc::new(PostgresProductRepository::new(pool.clone()));

    // サービスの作成
    let item_service = Arc::new(ItemService::new(item_repository.clone()));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let category_service = Arc::new(CategoryService::new(category_repository.clone()));
    let product_service = Arc::new(ProductService::new(product_repository.clone()));

    // Keycloak認証の設定
    let keycloak_config = KeycloakConfig::from_env();
    let keycloak_auth = web::Data::new(KeycloakAuth::new(keycloak_config));

    // ハンドラーの作成
    let item_handler = web::Data::new(ItemHandler::new(item_service.clone()));
    let user_handler = web::Data::new(UserHandler::new(user_service.clone()));
    let category_handler = web::Data::new(CategoryHandler::new(category_service.clone()));
    let product_handler = web::Data::new(ProductHandler::new(product_service.clone()));

    info!("サーバーを開始します: HTTP: http://127.0.0.1:8080, gRPC: http://127.0.0.1:50051");

    // gRPCサービスの作成
    let grpc_user_service = UserServiceImpl::new(user_service.clone());
    let grpc_item_service = ItemServiceImpl::new(item_service.clone());

    // HTTPサーバーの設定
    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(item_handler.clone())
            .app_data(user_handler.clone())
            .app_data(category_handler.clone())
            .app_data(product_handler.clone())
            .app_data(keycloak_auth.clone())
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
                        // Observe request duration
                        observe_request_duration("rust_webapi", &path, elapsed);
                        // Count success vs error based on status code
                        if res.status().is_server_error() {
                            increment_error_counter("rust_webapi", &path);
                        } else {
                            increment_success_counter("rust_webapi", &path);
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
                    .route("/products/{id}", web::delete().to(ItemHandler::logical_delete_item))
                    .route("/products/{id}/permanent", web::delete().to(ItemHandler::physical_delete_item))
                    .route("/products/{id}/restore", web::post().to(ItemHandler::restore_item))
                    .route("/products/{id}/deletion-check", web::get().to(ItemHandler::validate_item_deletion))
                    .route("/products/batch", web::delete().to(ItemHandler::batch_delete_items))
                    .route("/products/deleted", web::get().to(ItemHandler::get_deleted_items))
                    .route("/products/{id}/deletion-log", web::get().to(ItemHandler::get_item_deletion_log))
                    .route("/deletion-logs", web::get().to(ItemHandler::get_deletion_logs))
                    .route("/users", web::get().to(UserHandler::get_users))
                    .route("/users", web::post().to(UserHandler::create_user))
                    .route("/users/{id}", web::get().to(UserHandler::get_user))
                    .route("/users/{id}", web::put().to(UserHandler::update_user))
                    .route("/users/{id}", web::delete().to(UserHandler::delete_user))
            )
            .configure(configure_category_routes)
            .configure(configure_product_routes)
    })
    .bind("127.0.0.1:8080")?;

    // gRPCサーバーの設定
    let grpc_server = Server::builder()
        .add_service(UserServiceServer::new(grpc_user_service))
        .add_service(ItemServiceServer::new(grpc_item_service))
        .serve("127.0.0.1:50051".parse().unwrap());

    // 両方のサーバーを並行して実行
    let result = tokio::try_join!(
        http_server.run(),
        grpc_server
    );

    match result {
        Ok(_) => {
            info!("サーバーが正常に停止しました");
            Ok(())
        }
        Err(e) => {
            error!("サーバーエラー: {}", e);
            Err(e.into())
        }
    }
}
