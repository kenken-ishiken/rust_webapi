mod app_domain;
mod application;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::sync::Arc;
// Tracing for structured logging
use tracing::{error, info};
// gRPC imports
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;
use tracing_actix_web::TracingLogger;

// Remove slog-based JSON logger; using tracing for structured logging
// Middleware: TracingLogger for HTTP request logging
// (replace slog middleware with tracing-actix-web)
use crate::infrastructure::metrics::{
    increment_error_counter, increment_success_counter, init_metrics, metrics_handler,
    observe_request_duration,
};
use crate::infrastructure::tracing::init_tracing;
use crate::infrastructure::startup_error::{StartupError, StartupResult};
use crate::infrastructure::config::AppConfig;
use actix_web::dev::Service;
use std::time::Instant;

use crate::app_domain::repository::item_repository::ItemRepository;
use crate::application::service::item_service::ItemService;
use crate::infrastructure::repository::item_repository::PostgresItemRepository;
use crate::presentation::api::item_handler::ItemHandler;

use crate::application::service::user_service::UserService;
use crate::infrastructure::repository::user_repository::PostgresUserRepository;
use crate::presentation::api::user_handler::UserHandler;
use domain::repository::user_repository::UserRepositoryImpl;

use crate::infrastructure::auth::keycloak::{KeycloakAuth, KeycloakConfig};

use crate::app_domain::repository::category_repository::CategoryRepository;
use crate::application::service::category_service::CategoryService;
use crate::infrastructure::repository::category_repository::PostgresCategoryRepository;
use crate::presentation::api::category_handler::{configure_category_routes, CategoryHandler};

use crate::app_domain::repository::product_repository::ProductRepository;
use crate::application::service::product_service::ProductService;
use crate::infrastructure::repository::product_repository::PostgresProductRepository;
use crate::presentation::api::product_handler::{configure_product_routes, ProductHandler};

// gRPC imports
use crate::presentation::grpc::item_service::{ItemServiceImpl, ItemServiceServer};
use crate::presentation::grpc::user_service::{UserServiceImpl, UserServiceServer};

#[tokio::main]
async fn main() -> StartupResult<()> {
    // 環境変数の読み込み
    dotenv().ok();

    // 設定の読み込みと検証
    let config = AppConfig::from_env()?;
    config.validate()?;
    
    info!("Configuration loaded successfully");

    // Initialize tracing and OpenTelemetry (Datadog compatible)
    init_tracing().map_err(|e| StartupError::TracingInit(e.to_string()))?;
    info!("Tracing initialized");

    init_metrics();
    info!("Metrics initialized");

    // データベース接続プールの作成
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;
    
    info!("✅ PostgreSQL データベース接続に成功しました");

    // リポジトリの作成
    let item_repository: Arc<dyn ItemRepository + Send + Sync> =
        Arc::new(PostgresItemRepository::new(pool.clone()));
    let user_repository: UserRepositoryImpl = Arc::new(PostgresUserRepository::new(pool.clone()));
    let category_repository: Arc<dyn CategoryRepository> =
        Arc::new(PostgresCategoryRepository::new(pool.clone()));
    let product_repository: Arc<dyn ProductRepository> =
        Arc::new(PostgresProductRepository::new(pool.clone()));

    // サービスの作成
    let item_service = Arc::new(ItemService::new(item_repository.clone()));
    let user_service = Arc::new(UserService::new(user_repository.clone()));
    let category_service = Arc::new(CategoryService::new(category_repository.clone()));
    let product_service = Arc::new(ProductService::new(product_repository.clone()));

    // Keycloak認証の設定
    let keycloak_config = KeycloakConfig::from_auth_config(&config.auth);
    let keycloak_auth = web::Data::new(KeycloakAuth::new(keycloak_config));

    // ハンドラーの作成
    let item_handler = web::Data::new(ItemHandler::new(item_service.clone()));
    let user_handler = web::Data::new(UserHandler::new(user_service.clone()));
    let category_handler = web::Data::new(CategoryHandler::new(category_service.clone()));
    let product_handler = web::Data::new(ProductHandler::new(product_service.clone()));

    info!(
        "サーバーを開始します: HTTP: http://{}:{}, gRPC: http://{}:{}",
        config.server.http_host, config.server.http_port,
        config.server.grpc_host, config.server.grpc_port
    );

    // gRPCサービスの作成
    let grpc_user_service = UserServiceImpl::new(user_service.clone());
    let grpc_item_service = ItemServiceImpl::new(item_service.clone());

    // HTTPサーバーアドレスとgRPCサーバーアドレスを作成
    let http_addr = format!("{}:{}", config.server.http_host, config.server.http_port);
    let grpc_addr = format!("{}:{}", config.server.grpc_host, config.server.grpc_port)
        .parse()
        .map_err(|_| StartupError::Configuration("Invalid gRPC address".to_string()))?;

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
                    .route("/users/{id}", web::delete().to(UserHandler::delete_user)),
            )
            .configure(configure_category_routes)
            .configure(configure_product_routes)
    })
    .bind(&http_addr)?;

    // gRPCサーバーの設定
    let grpc_server = Server::builder()
        .add_service(UserServiceServer::new(grpc_user_service))
        .add_service(ItemServiceServer::new(grpc_item_service))
        .serve(grpc_addr);

    // 両方のサーバーを並行して実行
    tokio::select! {
        result = http_server.run() => {
            if let Err(e) = result {
                error!("HTTPサーバーエラー: {}", e);
                return Err(StartupError::ServerBind(e.to_string()));
            }
        }
        result = grpc_server => {
            if let Err(e) = result {
                error!("gRPCサーバーエラー: {}", e);
                return Err(StartupError::GrpcServer(e.to_string()));
            }
        }
    }

    info!("サーバーが正常に停止しました");
    Ok(())
}
