mod domain;
mod application;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use log::info;

use crate::domain::repository::item_repository::ItemRepositoryImpl;
use crate::infrastructure::repository::item_repository::InMemoryItemRepository;
use crate::application::service::item_service::ItemService;
use crate::presentation::api::item_handler::ItemHandler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ロギングの初期化
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // リポジトリの作成
    let item_repository: ItemRepositoryImpl = Arc::new(InMemoryItemRepository::new());
    
    // サービスの作成
    let item_service = Arc::new(ItemService::new(item_repository.clone()));
    
    // ハンドラーの作成
    let item_handler = web::Data::new(ItemHandler::new(item_service.clone()));

    info!("サーバーを開始します: http://127.0.0.1:8080");
    
    // HTTPサーバーの設定と起動
    HttpServer::new(move || {
        App::new()
            .app_data(item_handler.clone())
            .route("/", web::get().to(ItemHandler::index))
            .service(
                web::scope("/api")
                    .route("/items", web::get().to(ItemHandler::get_items))
                    .route("/items", web::post().to(ItemHandler::create_item))
                    .route("/items/{id}", web::get().to(ItemHandler::get_item))
                    .route("/items/{id}", web::put().to(ItemHandler::update_item))
                    .route("/items/{id}", web::delete().to(ItemHandler::delete_item))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
