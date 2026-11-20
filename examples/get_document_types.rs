//! List the document type categories. No session needed.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example get_document_types

mod common;

use tracing::info;
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let client = VietstockClient::new()?;
    let types = client.get_document_types().await?;

    info!(count = types.len(), "document types");
    for dt in &types {
        info!(
            id = dt.document_type_id,
            order = dt.display_order,
            name = %dt.document_name,
            name_en = %dt.document_name_en,
            "document type"
        );
    }
    Ok(())
}
