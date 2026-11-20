//! Download a stock's most recent document to a local directory.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example download_document -- VNM ./vietstock_downloads

mod common;

use tracing::{info, warn};
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let args: Vec<String> = std::env::args().collect();
    let stock_code = args.get(1).map_or("VNM", String::as_str).to_uppercase();
    let output_dir = args.get(2).map_or_else(
        || std::path::PathBuf::from("./vietstock_downloads"),
        std::path::PathBuf::from,
    );

    let mut client = VietstockClient::new()?;
    client.init_session(&stock_code).await?;
    let documents = client.get_documents(&stock_code).await?;

    let Some(doc) = documents.iter().max_by_key(|d| d.last_update) else {
        warn!(%stock_code, "no documents found");
        return Ok(());
    };
    info!(
        title = %doc.title.trim(),
        last_update = %doc.last_update.format("%Y-%m-%d"),
        ext = %doc.file_ext,
        url = %doc.url,
        "downloading most recent document"
    );

    let path = client.download_document_to_file(doc, &output_dir).await?;
    let bytes = std::fs::metadata(&path)?.len();
    info!(path = %path.display(), bytes, "downloaded");
    Ok(())
}
