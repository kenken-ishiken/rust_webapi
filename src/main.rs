mod domain;
mod application;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use log::{info, error};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use crate::domain::repository::item_repository::ItemRepositoryImpl;
use crate::infrastructure::repository::item_repository::PostgresItemRepository;
use crate::application::service::item_service::ItemService;
use crate::presentation::api::item_handler::ItemHandler;

use crate::domain::repository::user_repository::UserRepositoryImpl;
use crate::infrastructure::repository::user_repository::PostgresUserRepository;
use crate::application::service::user_service::UserService;
use crate::presentation::api::user_handler::UserHandler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の読み込み
    dotenv().ok();
    
    // ロギングの初期化
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
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
    
    // ハンドラーの作成
    let item_handler = web::Data::new(ItemHandler::new(item_service.clone()));
    let user_handler = web::Data::new(UserHandler::new(user_service.clone()));

    info!("サーバーを開始します: http://127.0.0.1:8080");
    
    // HTTPサーバーの設定と起動
    HttpServer::new(move || {
        App::new()
            .app_data(item_handler.clone())
            .app_data(user_handler.clone())
            .route("/", web::get().to(ItemHandler::index))
            .service(
                web::scope("/api")
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
