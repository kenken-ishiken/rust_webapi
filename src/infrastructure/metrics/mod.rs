use actix_web::{HttpResponse, Result as ActixResult};
use lazy_static::lazy_static;
use prometheus::{
    CounterVec, HistogramVec, Opts, Registry, TextEncoder, Encoder,
};

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
}
