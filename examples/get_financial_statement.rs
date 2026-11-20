//! Run the three-step statement pipeline for one company and summarise what comes back.
//!
//! Anonymous sessions get the line items but not the periods/values
//! (`RequestUpgradeAccount_Permission`); the example then falls back to the free
//! summary endpoint.
//!
//! Usage:
//!   cargo run -p vietstock-rs --example get_financial_statement -- VNM CDKT NAM [UNITED_ID] [AUDITED_STATUS_ID]

mod common;

use tracing::{info, warn};
use vietstock::{FinancialTab, VietstockClient};

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    common::init_tracing();
    let args: Vec<String> = std::env::args().collect();
    let stock_code = args.get(1).map_or("VNM", String::as_str).to_uppercase();
    let tab = match args.get(2).map(|s| s.to_uppercase()).as_deref() {
        Some("BCTT") => FinancialTab::Bctt,
        Some("KQKD") => FinancialTab::Kqkd,
        Some("LCTT") => FinancialTab::Lctt,
        Some("CSTC") => FinancialTab::Cstc,
        _ => FinancialTab::Cdkt,
    };
    let period_type = args.get(3).map_or("NAM", String::as_str).to_uppercase();
    // -1 = all. UnitedId: 0 consolidated, 1 parent company.
    let united_id: i32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(-1);
    let audited_status_id: i32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(-1);

    let mut client = VietstockClient::new()?;
    client.init_session(&stock_code).await?;

    let norms = client.get_report_norms(&stock_code, tab).await?;
    let first: Vec<&str> = norms
        .iter()
        .take(5)
        .map(|n| n.report_norm_name.trim())
        .collect();
    info!(%stock_code, ?tab, line_items = norms.len(), ?first, "line items");

    let periods = match client
        .get_report_periods(
            &stock_code,
            tab,
            &period_type,
            1,
            united_id,
            audited_status_id,
            false,
            "Time_ASC",
        )
        .await
    {
        Ok(periods) => periods,
        Err(e) if e.is_permission_denied() => {
            warn!(united_id, audited_status_id, error = %e, "periods refused for this account");
            summary(&mut client, &stock_code).await;
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    info!(count = periods.len(), %period_type, "periods");
    for p in periods.iter().rev().take(8) {
        info!(
            year = p.year_period,
            term = p.report_term_id,
            scope = %p.united_name,
            audit = %p.audit_status_name,
            auditor = %p.cty_kiem_toan,
            opinion = %p.y_kien_kiem_toan,
            visible = p.is_show_data_permission,
            "period"
        );
    }

    // Values for the latest few periods only (one request, at most 9 periods).
    let latest: Vec<(i32, i32, bool)> = periods
        .iter()
        .rev()
        .take(4)
        .map(|p| (p.report_data_id, p.year_period, p.is_show_data_permission))
        .collect();
    if !latest.is_empty() {
        let rows = client
            .get_report_values(&stock_code, tab, &latest, 1, 1)
            .await?;
        let filled = rows.iter().filter(|r| r.value1.is_some()).count();
        info!(
            rows = rows.len(),
            periods = latest.len(),
            filled_first_period = filled,
            "values"
        );
    }
    Ok(())
}

/// The summary on the company overview page is free; log what it returns.
async fn summary(client: &mut VietstockClient, stock_code: &str) {
    match client.get_finance_info(stock_code, 2, "BCTQ").await {
        Ok(Some((periods, components, _, _))) => {
            let labels: Vec<String> = periods
                .iter()
                .map(|p| {
                    format!(
                        "{}/{} {} {}",
                        p.term_code, p.year_period, p.united, p.audited_status
                    )
                })
                .collect();
            info!(?labels, "summary /data/financeinfo (quarterly)");
            for (section, items) in &components {
                let first = items.first().map_or("-", |i| i.name.as_str());
                info!(%section, items = items.len(), first, "summary section");
            }
        }
        Ok(None) => warn!("summary /data/financeinfo: empty"),
        Err(e) => warn!(error = %e, "summary /data/financeinfo failed"),
    }
}
