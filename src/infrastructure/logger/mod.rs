use slog::{Drain, Logger, o};
use slog_json::Json;
use slog_async::Async;
use slog_scope::GlobalLoggerGuard;
use std::sync::Mutex;
use std::io;

pub mod actix_logger;

/// JSONロガーを初期化する関数
#[allow(dead_code)]
pub fn init_json_logger() -> Result<GlobalLoggerGuard, Box<dyn std::error::Error>> {
    // JSONドレインの設定
    let json_drain = Json::new(io::stdout())
        .add_default_keys()
        .build()
        .fuse();

    // 非同期ドレインの設定
    let drain = Async::new(json_drain)
        .build()
        .fuse();

    // スレッドセーフなドレインの設定
    let drain = Mutex::new(drain).fuse();

    // ルートロガーの作成
    let logger = Logger::root(
        drain,
        o!(
            "version" => env!("CARGO_PKG_VERSION"),
            "app" => env!("CARGO_PKG_NAME")
        )
    );

    // グローバルロガーとして設定
    let guard = slog_scope::set_global_logger(logger);

    // stdlogアダプターを設定（log crateのマクロをslogにリダイレクト）
    slog_stdlog::init()?;

    Ok(guard)
}

/// 標準出力用のロガーを初期化する関数（開発環境用）
#[allow(dead_code)]
pub fn init_stdout_logger() -> Result<GlobalLoggerGuard, Box<dyn std::error::Error>> {
    // ターミナル用のデコレータ
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();

    // 非同期ドレインの設定
    let drain = Async::new(drain).build().fuse();

    // スレッドセーフなドレインの設定
    let drain = Mutex::new(drain).fuse();

    // ルートロガーの作成
    let logger = Logger::root(
        drain,
        o!(
            "version" => env!("CARGO_PKG_VERSION"),
            "app" => env!("CARGO_PKG_NAME")
        )
    );

    // グローバルロガーとして設定
    let guard = slog_scope::set_global_logger(logger);

    // stdlogアダプターを設定
    slog_stdlog::init()?;

    Ok(guard)
}
