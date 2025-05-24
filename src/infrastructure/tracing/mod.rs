use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_log::LogTracer;

/// Initialize tracing with JSON log output.
/// 
/// This sets up `tracing` to emit JSON logs.
/// OpenTelemetry integration will be added in a future update.
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
