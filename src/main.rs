mod app_domain;
mod application;
mod infrastructure;
mod presentation;

use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

use crate::infrastructure::config::AppConfig;
use crate::infrastructure::di::container::AppContainer;
use crate::infrastructure::di::server::build_http_server;
use crate::infrastructure::metrics::init_metrics;
use crate::infrastructure::startup_error::{StartupError, StartupResult};
use crate::infrastructure::tracing::init_tracing;

#[tokio::main]
async fn main() -> StartupResult<()> {
    // 環境変数の読み込み
    dotenv().ok();

    // 設定の読み込みと検証
    let config = AppConfig::from_env()?;
    config.validate()?;
    info!("Configuration loaded successfully");

    // Initialize tracing and metrics
    init_tracing().map_err(|e| StartupError::TracingInit(e.to_string()))?;
    info!("Tracing initialized");
    init_metrics();
    info!("Metrics initialized");

    // データベース接続プールの作成
    let mut pool_options = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.database.connect_timeout));
    
    // オプション設定
    if let Some(idle_timeout) = config.database.idle_timeout {
        pool_options = pool_options.idle_timeout(std::time::Duration::from_secs(idle_timeout));
    }
    
    if let Some(max_lifetime) = config.database.max_lifetime {
        pool_options = pool_options.max_lifetime(std::time::Duration::from_secs(max_lifetime));
    }
    
    let pool = pool_options
        .connect(&config.database.url)
        .await?;
    info!("✅ PostgreSQL データベース接続に成功しました (pool: min={}, max={})", 
        config.database.min_connections, config.database.max_connections);

    // 依存性注入コンテナの作成
    let container = AppContainer::new(pool, &config);

    // サーバーアドレスの準備
    let http_addr = format!("{}:{}", config.server.http_host, config.server.http_port);
    let grpc_addr = format!("{}:{}", config.server.grpc_host, config.server.grpc_port)
        .parse()
        .map_err(|_| StartupError::Configuration("Invalid gRPC address".to_string()))?;

    info!(
        "サーバーを開始します: HTTP: http://{}, gRPC: http://{}",
        http_addr, grpc_addr
    );

    // HTTPサーバーとgRPCサーバーの構築
    let http_server = build_http_server(&container, &http_addr)?;
    let grpc_server = container.build_grpc_server().serve(grpc_addr);

    // 両方のサーバーを並行して実行
    tokio::select! {
        result = http_server => {
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
