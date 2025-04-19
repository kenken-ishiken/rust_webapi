mod application;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use dotenv::dotenv;
// Tracing for structured logging
use tracing::{info, error};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::fmt::format::FmtSpan;
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
use std::time::Instant;
use actix_web::dev::Service;

use domain::repository::item_repository::ItemRepositoryImpl;
use crate::infrastructure::repository::item_repository::PostgresItemRepository;
use crate::application::service::item_service::ItemService;
use crate::presentation::api::item_handler::ItemHandler;

use domain::repository::user_repository::UserRepositoryImpl;
use crate::infrastructure::repository::user_repository::PostgresUserRepository;
use crate::application::service::user_service::UserService;
use crate::presentation::api::user_handler::UserHandler;

use crate::infrastructure::auth::keycloak::{KeycloakAuth, KeycloakConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の読み込み
    dotenv().ok();

    // Initialize LogTracer to capture log crate events in tracing
    LogTracer::init().expect("Failed to initialize LogTracer");
    // Initialize tracing subscriber for structured JSON logging
    // Initialize tracing subscriber for structured JSON logging; ignore if already set
    if let Err(e) = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT)
        .try_init()
    {
        eprintln!("Warning: failed to install tracing subscriber (may already be set): {:?}", e);
    }
    info!("Structured JSON logging initialized");
    
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
    let item_repository: ItemRepositoryImpl = Arc::new(PostgresItemRepository::new(pool.clone()));
    let user_repository: UserRepositoryImpl = Arc::new(PostgresUserRepository::new(pool.clone()));

    // サービスの作成
    let item_service = Arc::new(ItemService::new(item_repository.clone()));
    let user_service = Arc::new(UserService::new(user_repository.clone()));

    // Keycloak認証の設定
    let keycloak_config = KeycloakConfig::from_env();
    let keycloak_auth = web::Data::new(KeycloakAuth::new(keycloak_config));

    // ハンドラーの作成
    let item_handler = web::Data::new(ItemHandler::new(item_service.clone()));
    let user_handler = web::Data::new(UserHandler::new(user_service.clone()));

    info!("サーバーを開始します: http://127.0.0.1:8080");

    // HTTPサーバーの設定と起動
    HttpServer::new(move || {
        App::new()
            .app_data(item_handler.clone())
            .app_data(user_handler.clone())
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
                    .route("/users", web::get().to(UserHandler::get_users))
                    .route("/users", web::post().to(UserHandler::create_user))
                    .route("/users/{id}", web::get().to(UserHandler::get_user))
                    .route("/users/{id}", web::put().to(UserHandler::update_user))
                    .route("/users/{id}", web::delete().to(UserHandler::delete_user))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
