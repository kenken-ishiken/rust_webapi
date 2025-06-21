use std::sync::Arc;
use sqlx::PgPool;
use actix_web::web;

use crate::app_domain::repository::{
    category_repository::CategoryRepository,
    item_repository::ItemRepository,
    product_repository::ProductRepository,
};
use crate::application::service::{
    category_service::CategoryService,
    item_service::ItemService,
    product_service::ProductService,
    user_service::UserService,
};
use crate::infrastructure::repository::{
    category_repository::PostgresCategoryRepository,
    item_repository::PostgresItemRepository,
    product_repository::PostgresProductRepository,
    user_repository::PostgresUserRepository,
};
use crate::presentation::api::{
    category_handler::CategoryHandler,
    item_handler::ItemHandler,
    product_handler::ProductHandler,
    user_handler::UserHandler,
};
use crate::presentation::grpc::{
    item_service::{ItemServiceImpl, ItemServiceServer},
    user_service::{UserServiceImpl, UserServiceServer},
};
use crate::infrastructure::auth::keycloak::{KeycloakAuth, KeycloakConfig};
use crate::infrastructure::config::AppConfig;
use domain::repository::user_repository::UserRepositoryImpl;

/// アプリケーションの依存関係を管理するコンテナ
pub struct AppContainer {
    // Repositories
    pub item_repository: Arc<dyn ItemRepository + Send + Sync>,
    pub user_repository: UserRepositoryImpl,
    pub category_repository: Arc<dyn CategoryRepository>,
    pub product_repository: Arc<dyn ProductRepository>,
    
    // Services
    pub item_service: Arc<ItemService>,
    pub user_service: Arc<UserService>,
    pub category_service: Arc<CategoryService>,
    pub product_service: Arc<ProductService>,
    
    // Handlers
    pub item_handler: web::Data<ItemHandler>,
    pub user_handler: web::Data<UserHandler>,
    pub category_handler: web::Data<CategoryHandler>,
    pub product_handler: web::Data<ProductHandler>,
    
    // Auth
    pub keycloak_auth: web::Data<KeycloakAuth>,
    
    // gRPC Services
    pub grpc_user_service: UserServiceImpl,
    pub grpc_item_service: ItemServiceImpl,
}

impl AppContainer {
    /// 新しいAppContainerを作成する
    pub fn new(pool: PgPool, config: &AppConfig) -> Self {
        // リポジトリの作成
        let item_repository: Arc<dyn ItemRepository + Send + Sync> =
            Arc::new(PostgresItemRepository::new(pool.clone()));
        let user_repository: UserRepositoryImpl = 
            Arc::new(PostgresUserRepository::new(pool.clone()));
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

        // gRPCサービスの作成
        let grpc_user_service = UserServiceImpl::new(user_service.clone());
        let grpc_item_service = ItemServiceImpl::new(item_service.clone());

        Self {
            item_repository,
            user_repository,
            category_repository,
            product_repository,
            item_service,
            user_service,
            category_service,
            product_service,
            item_handler,
            user_handler,
            category_handler,
            product_handler,
            keycloak_auth,
            grpc_user_service,
            grpc_item_service,
        }
    }
    
    /// gRPCサーバーを構築する
    pub fn build_grpc_server(&self) -> tonic::transport::server::Router {
        tonic::transport::Server::builder()
            .add_service(UserServiceServer::new(self.grpc_user_service.clone()))
            .add_service(ItemServiceServer::new(self.grpc_item_service.clone()))
    }
} 