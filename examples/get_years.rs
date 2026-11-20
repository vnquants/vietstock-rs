//! Fetch the years for which a stock has documents. No session needed.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example get_years -- VNM

mod common;

use tracing::{info, warn};
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let stock_code = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "VNM".to_string())
        .to_uppercase();

    let client = VietstockClient::new()?;
    let years = client.get_years(&stock_code).await?;

    if years.is_empty() {
        warn!(%stock_code, "no documents found");
    } else {
        info!(%stock_code, count = years.len(), ?years, "years with documents");
    }
    Ok(())
}
