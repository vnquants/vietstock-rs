//! Shared setup for the examples.

/// Install a `tracing` subscriber. The level comes from `RUST_LOG`
/// (for example `RUST_LOG=vietstock=debug`) and defaults to `info`.
pub fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
