use opentelemetry::KeyValue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_log::LogTracer;

/// Initialize tracing and OpenTelemetry exporter compatible with Datadog.
///
/// This sets up `tracing` to emit JSON logs with `trace_id` and `span_id`
/// fields so that logs and traces can be correlated in Datadog.
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Forward log crate events to `tracing`
    LogTracer::init()?;

    // Set global propagator for distributed tracing context
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new()
    );

    // Service name resource
    let service_name = env!("CARGO_PKG_NAME");
    
    // Create a resource with service information
    let resource = opentelemetry_sdk::Resource::default()
        .with_schema_url("https://opentelemetry.io/schemas/1.21.0")
        .with_attributes(vec![KeyValue::new("service.name", service_name)]);
    
    // Create a trace provider builder
    let provider_builder = opentelemetry_sdk::trace::TracerProviderBuilder::default()
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(resource)
                .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        );
    
    // Create the tracer provider
    let provider = provider_builder.build();
    
    // Set the provider as the global default
    opentelemetry::global::set_tracer_provider(provider);
    
    // Get a tracer from the global provider
    let tracer = opentelemetry::global::tracer(service_name);
    
    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create JSON log formatter layer
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT);

    // Initialize subscriber with layers
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(telemetry)
        .with(fmt_layer)
        .init();

    Ok(())
}
