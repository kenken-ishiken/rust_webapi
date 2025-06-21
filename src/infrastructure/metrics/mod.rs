use actix_web::{HttpResponse, Result as ActixResult};
use lazy_static::lazy_static;
use prometheus::{
    CounterVec, HistogramVec, Opts, Registry, TextEncoder, Encoder,
};
use std::time::Instant;

lazy_static! {
    static ref API_SUCCESS_COUNTER: CounterVec = CounterVec::new(
        Opts::new("api_success_total", "Total number of successful API requests"),
        &["service", "endpoint"]
    ).expect("Failed to create API_SUCCESS_COUNTER");
    
    static ref API_ERROR_COUNTER: CounterVec = CounterVec::new(
        Opts::new("api_error_total", "Total number of API errors"),
        &["service", "endpoint"]
    ).expect("Failed to create API_ERROR_COUNTER");
    
    static ref API_REQUEST_DURATION_HISTOGRAM: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "api_request_duration_seconds", 
            "API request duration in seconds"
        ),
        &["service", "endpoint"]
    ).expect("Failed to create API_REQUEST_DURATION_HISTOGRAM");
    
    static ref REGISTRY: Registry = Registry::new();
}

/// メトリクス記録の統一マクロ
/// 
/// # 使用例
/// ```rust
/// // 成功カウンター
/// metrics!(success, "user", "find_by_id");
/// 
/// // エラーカウンター
/// metrics!(error, "user", "find_by_id");
/// 
/// // リクエスト時間測定
/// let timer = metrics!(timer, "user", "find_by_id");
/// // ... 処理 ...
/// timer.observe();
/// ```
#[macro_export]
macro_rules! metrics {
    // 成功カウンター
    (success, $service:expr, $endpoint:expr) => {
        $crate::infrastructure::metrics::increment_success_counter($service, $endpoint);
    };
    
    // エラーカウンター
    (error, $service:expr, $endpoint:expr) => {
        $crate::infrastructure::metrics::increment_error_counter($service, $endpoint);
    };
    
    // タイマー開始
    (timer, $service:expr, $endpoint:expr) => {
        $crate::infrastructure::metrics::MetricsTimer::new($service, $endpoint)
    };
    
    // 時間直接測定
    (duration, $service:expr, $endpoint:expr, $seconds:expr) => {
        $crate::infrastructure::metrics::observe_request_duration($service, $endpoint, $seconds);
    };
}

/// メトリクス記録用のタイマー
/// 
/// # 例
/// ```rust
/// let timer = MetricsTimer::new("user", "find_by_id");
/// // ... 処理 ...
/// timer.observe(); // 自動的に経過時間を記録
/// ```
pub struct MetricsTimer {
    service: String,
    endpoint: String,
    start_time: Instant,
}

impl MetricsTimer {
    /// 新しいタイマーを作成
    pub fn new(service: &str, endpoint: &str) -> Self {
        Self {
            service: service.to_string(),
            endpoint: endpoint.to_string(),
            start_time: Instant::now(),
        }
    }
    
    /// 経過時間を観測してメトリクスに記録
    pub fn observe(self) {
        let elapsed = self.start_time.elapsed();
        let seconds = elapsed.as_secs_f64();
        observe_request_duration(&self.service, &self.endpoint, seconds);
    }
    
    /// 経過時間を取得（秒）
    pub fn elapsed_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

impl Drop for MetricsTimer {
    /// タイマーが破棄される際に自動的に測定結果を記録
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let seconds = elapsed.as_secs_f64();
        observe_request_duration(&self.service, &self.endpoint, seconds);
    }
}

/// 高レベルメトリクス記録API
pub struct Metrics;

impl Metrics {
    /// 操作の成功を記録
    pub fn record_success(service: &str, operation: &str) {
        increment_success_counter(service, operation);
        tracing::debug!("Metrics: {} {} success", service, operation);
    }
    
    /// 操作のエラーを記録
    pub fn record_error(service: &str, operation: &str) {
        increment_error_counter(service, operation);
        tracing::warn!("Metrics: {} {} error", service, operation);
    }
    
    /// 操作の実行時間を記録
    pub fn record_duration(service: &str, operation: &str, seconds: f64) {
        observe_request_duration(service, operation, seconds);
        tracing::debug!("Metrics: {} {} duration: {:.3}s", service, operation, seconds);
    }
    
    /// 操作をタイマー付きで実行
    pub async fn with_timer<F, T>(service: &str, operation: &str, f: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let timer = MetricsTimer::new(service, operation);
        let result = f.await;
        timer.observe();
        result
    }
    
    /// 操作をメトリクス記録付きで実行（Result型）
    pub async fn with_metrics<F, T, E>(service: &str, operation: &str, f: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let timer = MetricsTimer::new(service, operation);
        let result = f.await;
        
        match &result {
            Ok(_) => Self::record_success(service, operation),
            Err(_) => Self::record_error(service, operation),
        }
        
        timer.observe();
        result
    }
}

pub fn init_metrics() {
    // Register metrics - these are static and should not fail after initial creation
    let _ = REGISTRY.register(Box::new(API_SUCCESS_COUNTER.clone()));
    let _ = REGISTRY.register(Box::new(API_ERROR_COUNTER.clone()));
    let _ = REGISTRY.register(Box::new(API_REQUEST_DURATION_HISTOGRAM.clone()));
}

pub fn increment_success_counter(service: &str, endpoint: &str) {
    API_SUCCESS_COUNTER
        .with_label_values(&[service, endpoint])
        .inc();
}

pub fn increment_error_counter(service: &str, endpoint: &str) {
    API_ERROR_COUNTER
        .with_label_values(&[service, endpoint])
        .inc();
}
// Observe request duration in seconds for a given service and endpoint
pub fn observe_request_duration(service: &str, endpoint: &str, seconds: f64) {
    API_REQUEST_DURATION_HISTOGRAM
        .with_label_values(&[service, endpoint])
        .observe(seconds);
}

pub async fn metrics_handler() -> ActixResult<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|e| {
            tracing::error!("Failed to encode metrics: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to encode metrics")
        })?;
    
    let response = String::from_utf8(buffer)
        .map_err(|e| {
            tracing::error!("Failed to convert metrics to UTF-8: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to convert metrics")
        })?;
    
    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(response))
}
// Unit tests for metrics endpoint
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App};

    #[actix_web::test]
    async fn test_metrics_handler_outputs_metrics() {
        // Initialize metrics before using them
        init_metrics();
        
        // Emit some sample metrics
        increment_success_counter("rust_webapi", "/test");
        increment_error_counter("rust_webapi", "/test");
        observe_request_duration("rust_webapi", "/test", 0.123);

        // Build Actix app with metrics handler
        let app =
            test::init_service(App::new().route("/metrics", web::get().to(metrics_handler))).await;
        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        // Verify HTTP 200 OK
        assert_eq!(resp.status(), StatusCode::OK);
        // Read body as text
        let body = test::read_body(resp).await;
        let body_str = match std::str::from_utf8(&body) {
            Ok(s) => s,
            Err(_) => panic!("Response not UTF-8"),
        };
        // Check that counters and histogram metrics are present with correct labels
        assert!(
            body_str.contains("api_success_total"),
            "Missing success counter"
        );
        assert!(
            body_str.contains("api_error_total"),
            "Missing error counter"
        );
        assert!(
            body_str.contains("api_request_duration_seconds_count"),
            "Missing histogram count"
        );
        assert!(
            body_str.contains("api_request_duration_seconds_sum"),
            "Missing histogram sum"
        );
        assert!(
            body_str.contains("endpoint=\"/test\""),
            "Missing endpoint label"
        );
    }

    #[tokio::test]
    async fn test_metrics_macro() {
        init_metrics();
        
        // テスト用メトリクス記録の動作確認
        increment_success_counter("test_service", "test_endpoint");
        increment_error_counter("test_service", "test_endpoint");
        observe_request_duration("test_service", "test_endpoint", 0.5);
        
        // タイマーのテスト
        let timer = MetricsTimer::new("test_service", "test_timer");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        timer.observe();
    }

    #[tokio::test]
    async fn test_metrics_timer() {
        let timer = MetricsTimer::new("test", "operation");
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let elapsed = timer.elapsed_seconds();
        assert!(elapsed >= 0.01); // 少なくとも10ms経過している
        timer.observe(); // 手動で観測
    }

    #[tokio::test]
    async fn test_metrics_timer_auto_drop() {
        init_metrics();
        {
            let _timer = MetricsTimer::new("test", "auto_drop");
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            // スコープを抜ける際に自動的にdropされて測定される
        }
    }

    #[tokio::test]
    async fn test_metrics_with_timer() {
        init_metrics();
        
        let result = Metrics::with_timer("test", "async_op", async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        }).await;
        
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_metrics_with_metrics_success() {
        init_metrics();
        
        let result = Metrics::with_metrics("test", "success_op", async {
            Ok::<i32, &str>(100)
        }).await;
        
        assert_eq!(result, Ok(100));
    }

    #[tokio::test]
    async fn test_metrics_with_metrics_error() {
        init_metrics();
        
        let result = Metrics::with_metrics("test", "error_op", async {
            Err::<i32, &str>("test error")
        }).await;
        
        assert_eq!(result, Err("test error"));
    }
}
