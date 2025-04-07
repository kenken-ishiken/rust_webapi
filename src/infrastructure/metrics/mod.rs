use prometheus::{
    register_int_counter_vec, IntCounterVec, Registry, TextEncoder, Encoder, gather,
};
use std::sync::Arc;
use std::sync::Mutex;
use actix_web::{web, HttpResponse, Responder};
use lazy_static::lazy_static;

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
}

pub fn init_metrics() {
    let r = REGISTRY.clone();
    r.register(Box::new(API_SUCCESS_COUNTER.clone())).unwrap();
    r.register(Box::new(API_ERROR_COUNTER.clone())).unwrap();
}

pub fn increment_success_counter(service: &str, endpoint: &str) {
    API_SUCCESS_COUNTER.with_label_values(&[service, endpoint]).inc();
}

pub fn increment_error_counter(service: &str, endpoint: &str) {
    API_ERROR_COUNTER.with_label_values(&[service, endpoint]).inc();
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
