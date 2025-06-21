use actix_web::middleware::Logger;
use log::info;
use serde_json::json;
use std::time::SystemTime;

/// Actix-web用のカスタムJSONロガーを作成する関数
#[allow(dead_code)]
pub fn json_logger() -> Logger {
    Logger::new("%{r}a \"%r\" %s %b %T").custom_request_replace("actix_logging", |req| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let request_path = req.path().to_string();
        let request_method = req.method().to_string();

        // リクエスト情報をJSONとしてログに記録
        let log_data = json!({
            "timestamp": now,
            "type": "request",
            "path": request_path,
            "method": request_method,
            "remote_addr": req.connection_info().realip_remote_addr().unwrap_or("unknown"),
        });

        info!("{}", log_data);
        "".to_string()
    })
}
