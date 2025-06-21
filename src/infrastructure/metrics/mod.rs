use actix_web::{HttpResponse, Responder};
use lazy_static::lazy_static;
use prometheus::{
    gather, register_histogram_vec, register_int_counter_vec, Encoder, HistogramVec, IntCounterVec,
    Registry, TextEncoder,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref API_SUCCESS_COUNTER: IntCounterVec = register_int_counter_vec!(
        "api_success_count",
        "Number of successful API calls",
        &["service", "endpoint"]
    ).unwrap();
    pub static ref API_ERROR_COUNTER: IntCounterVec = register_int_counter_vec!(
        "api_error_count",
        "Number of failed API calls",
        &["service", "endpoint"]
    ).unwrap();
    // Histogram for request durations in seconds, labeled by service and endpoint
    pub static ref API_REQUEST_DURATION_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "api_request_duration_seconds",
        "HTTP request duration in seconds",
        &["service", "endpoint"]
    ).unwrap();
}

pub fn init_metrics() {
    let r = REGISTRY.clone();
    r.register(Box::new(API_SUCCESS_COUNTER.clone())).unwrap();
    r.register(Box::new(API_ERROR_COUNTER.clone())).unwrap();
    // Register histogram for request durations
    r.register(Box::new(API_REQUEST_DURATION_HISTOGRAM.clone()))
        .unwrap();
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

pub async fn metrics_handler() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(String::from_utf8(buffer).unwrap())
}
// Unit tests for metrics endpoint
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App};

    #[actix_web::test]
    async fn test_metrics_handler_outputs_metrics() {
        // Ensure metrics are initialized via register macros
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
        let body_str = std::str::from_utf8(&body).expect("Response not UTF-8");
        // Check that counters and histogram metrics are present with correct labels
        assert!(
            body_str.contains("api_success_count"),
            "Missing success counter"
        );
        assert!(
            body_str.contains("api_error_count"),
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
