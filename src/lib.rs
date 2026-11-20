//! # Vietstock Finance API Client
//!
//! A Rust client for the Vietstock Finance API (`finance.vietstock.vn`).
//! Provides access to:
//!
//! - **Documents** — Financial statements, annual reports, resolutions, prospectuses
//! - **Trading Info** — Real-time price, volume, market cap, valuation metrics
//! - **Company Overview** — Performance, financial statements, news, events
//! - **Ratings** — Technical indicators, fundamental analysis (A–E letter grades)
//! - **Bonds & Planning** — Bond listings, business targets, implementation status
//! - **Market Data** — VN-Index, HNX-Index, gold, oil, headline news
//! - **Authentication** — Anonymous JWT, email/password login, social OAuth URLs
//!
//! All types are based on **captured JSON responses** from live browser sessions.
//!
//! Requests go through [`wreq`] with a Chrome TLS/HTTP2 fingerprint profile, so
//! no headless browser is needed.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use vietstock::VietstockClient;
//!
//! # async fn example() -> Result<(), vietstock::ApiError> {
//! let mut client = VietstockClient::new()?;
//! client.init_session("VCB").await?;
//!
//! // Performance metrics
//! let perf = client.get_performance("VCB").await?;
//! tracing::info!(close = perf.data[0].close_price, "VCB close (VND)");
//!
//! // Trading info with full fundamentals
//! let trading = client.get_trading_info("VCB").await?;
//! tracing::info!(pe = trading.pe, pb = trading.pb, "VCB valuation");
//!
//! // Financial statements
//! if let Some((periods, components, _, _)) = client.get_finance_info("VCB", 2, "BCTQ").await? {
//!     for (name, items) in &components {
//!         tracing::info!(%name, items = items.len(), "statement section");
//!     }
//! }
//!
//! // Technical rating
//! let tech = client.get_technical_rating("VCB").await?;
//! if let Some(rating) = tech.data.first() {
//!     tracing::info!(rating = %rating.final_ratings, rsi = rating.rsi, "technical rating");
//! }
//!
//! // Fundamental rating
//! let funda = client.get_fundamental_rating("VCB").await?;
//! let (_factors, summaries) = &funda.data;
//! if let Some(s) = summaries.first() {
//!     tracing::info!(rating = %s.rating, score = s.vs_rating, "fundamental rating");
//! }
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod date;
pub mod error;
pub mod models;

// ── Client ──
pub use client::{
    FINANCE_INFO_DEFAULT_UNIT, FINANCE_INFO_UNIT_DONG, VietstockClient, VietstockClientBuilder,
};

// ── Error ──
pub use error::ApiError;

// ── Date utilities ──
pub use date::{
    deserialize_ms_date, deserialize_opt_ms_date, deserialize_trim_trailing, parse_ms_date,
};

// ── Models: Document ──
pub use models::{Document, DocumentType};

// ── Models: Trading ──
pub use models::{TradingDayOverview, TradingInfo, TradingOverviewResponse};

// ── Models: Auth ──
pub use models::{AuthTokenResponse, LoginResponse};

// ── Models: Performance ──
pub use models::{Performance, PerformanceResponse};

// ── Models: Financial Statements ──
pub use models::{FinanceInfoLineItem, FinanceInfoResponse, PeriodMetadata};

// ── Models: Financial Statement Tabs ──
pub use models::{
    FinancialStatement, FinancialTab, ReportDataWrapper, ReportNorm, ReportPeriod, ReportValueRow,
};

// ── Models: News ──
pub use models::{HeaderNewsItem, NewsItem};

// ── Models: Events ──
pub use models::{
    ChannelTypeDef, CorporateEvent, EventCalendarItem, EventTypeDef, EventTypeLookup,
};

// ── Models: Analysis Reports ──
pub use models::AnalysisReport;

// ── Models: Insider Trading ──
pub use models::{InsiderTradingRecord, TransferTypeDef};

// ── Models: Ratings ──
pub use models::{
    FundamentalFactor, FundamentalRatingResponse, FundamentalSummary, TechnicalRating,
    TechnicalRatingResponse,
};

// ── Models: Bonds ──
pub use models::{
    BondChartPoint, BondItem, BondMarketOverview, BondNewsItem, BondRelatedResponse,
    BondTradingItem,
};

// ── Models: Planning ──
pub use models::{
    PlanYear, PlannedTarget, PlannedTargetData, PlannedTargetResponse, TargetImplementation,
    TargetImplementationData, TargetImplementationResponse,
};

// ── Models: Reference Price ──
pub use models::{DataRefPrice, RefPriceResponse, StockColorDay};

// ── Models: Market Data ──
pub use models::MarketPrice;

// ── Models: Templates ──
pub use models::TemplateItem;

// ── Models: Industry / Sector ──
pub use models::{
    FinanceIndexDef, FinanceIndexGroup, FinanceIndexGroupResponse, IndustryFinancialIndex,
    IndustryGics, IndustryHeatmapItem, IndustryIndexPoint, IndustryNewsItem, IndustryPerformance,
    SectorReport,
};
