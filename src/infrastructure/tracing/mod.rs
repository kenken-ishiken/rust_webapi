use opentelemetry::{global, sdk::Resource, KeyValue};
use opentelemetry::sdk::trace as sdktrace;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_log::LogTracer;

/// Initialize tracing and OpenTelemetry exporter compatible with Datadog.
///
/// This sets up `tracing` to emit JSON logs with `trace_id` and `span_id`
/// fields so that logs and traces can be correlated in Datadog.
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Forward log crate events to `tracing`
    LogTracer::init()?;

    // Use W3C trace context propagation
    global::set_text_map_propagator(opentelemetry::sdk::propagation::TraceContextPropagator::new());

    // Service name resource
    let service_name = env!("CARGO_PKG_NAME");
    let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

    // Build OTLP exporter. Endpoint and credentials can be configured via
    // environment variables such as `OTEL_EXPORTER_OTLP_ENDPOINT` and
    // `OTEL_EXPORTER_OTLP_HEADERS` which are compatible with the Datadog agent.
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(sdktrace::Config::default().with_resource(resource))
        .with_exporter(opentelemetry_otlp::new_exporter().tonic().with_env())
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}
