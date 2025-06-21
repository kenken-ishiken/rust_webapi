use tracing_log::LogTracer;
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with JSON log output.
///
/// This sets up `tracing` to emit JSON logs.
/// OpenTelemetry integration has been temporarily disabled due to
/// compatibility issues between the updated OpenTelemetry crates (v0.30.0)
/// and the tracing-opentelemetry crate. We'll need to update the
/// tracing-opentelemetry crate or adjust our implementation to work with
/// the new OpenTelemetry API once the compatibility issues are resolved.
///
/// The original implementation included:
/// - Setting global propagator for distributed tracing context
/// - Creating a resource with service information
/// - Setting up a tracer provider with sampling
/// - Integration with OpenTelemetry for distributed tracing
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Forward log crate events to `tracing`
    LogTracer::init()?;

    // Create JSON log formatter layer
    let fmt_layer = fmt::layer()
        .json()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .with_current_span(true)
        .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT);

    // Initialize subscriber with layers
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer)
        .init();

    Ok(())
}
