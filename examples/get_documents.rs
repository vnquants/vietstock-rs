//! List a stock's document library and summarise it.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example get_documents -- VNM
//!   cargo run -p vietstock-rs --example get_documents -- VNM --all   # walk every page

mod common;

use tracing::info;
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let args: Vec<String> = std::env::args().collect();
    let stock_code = args.get(1).map_or("VNM", String::as_str).to_uppercase();
    let all_pages = args.iter().any(|a| a == "--all");

    let mut client = VietstockClient::new()?;
    client.init_session(&stock_code).await?;

    let documents = if all_pages {
        client.get_all_documents(&stock_code, None).await?
    } else {
        client.get_documents(&stock_code).await?
    };

    let mut years: Vec<i32> = documents.iter().filter_map(|d| d.year()).collect();
    years.sort_unstable();
    years.dedup();
    let mut exchanges: Vec<&str> = documents.iter().filter_map(|d| d.exchange()).collect();
    exchanges.sort_unstable();
    exchanges.dedup();
    info!(
        %stock_code,
        fetched = documents.len(),
        library_total = documents.first().map_or(0, |d| d.total_row),
        financial_statements = documents.iter().filter(|d| d.url.contains("/BCTC/")).count(),
        pdf = documents.iter().filter(|d| d.is_pdf()).count(),
        zip = documents.iter().filter(|d| d.is_zip()).count(),
        rar = documents.iter().filter(|d| d.is_rar()).count(),
        first_year = years.first(),
        last_year = years.last(),
        ?exchanges,
        "document library"
    );

    let mut recent: Vec<_> = documents.iter().collect();
    recent.sort_by_key(|d| std::cmp::Reverse(d.last_update));
    for doc in recent.iter().take(5) {
        info!(
            last_update = %doc.last_update.format("%Y-%m-%d"),
            ext = %doc.file_ext,
            title = %doc.title.trim(),
            url = %doc.url,
            "recent document"
        );
    }
    Ok(())
}
