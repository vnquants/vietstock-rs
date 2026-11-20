//! Fetch the trading snapshot (prices, volume, valuation) for a stock.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example get_trading_info -- VNM

mod common;

use tracing::info;
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let stock_code = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "VNM".to_string())
        .to_uppercase();

    let mut client = VietstockClient::new()?;
    client.init_session(&stock_code).await?;
    let t = client.get_trading_info(&stock_code).await?;

    info!(
        code = %t.stock_code,
        date = %t.trading_date.format("%Y-%m-%d"),
        last = t.last_price,
        open = t.open_price,
        high = t.highest_price,
        low = t.lowest_price,
        prior_close = t.prior_close_price,
        ceiling = t.ceiling_price,
        floor = t.floor_price,
        vwap = t.avr_price,
        "prices (VND)"
    );
    info!(
        change = t.change,
        change_pct = t.per_change,
        volume = t.total_vol,
        value_vnd = t.total_val,
        market_cap_vnd = t.market_capital,
        "change and volume"
    );
    info!(
        eps = t.eps,
        pe = t.pe,
        bvps = t.bvps,
        pb = t.pb,
        low_52w = t.min_52w,
        high_52w = t.max_52w,
        "valuation"
    );
    Ok(())
}
