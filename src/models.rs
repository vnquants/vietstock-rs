//! Data structures for the Vietstock Finance API.
//!
//! All types are based on **captured JSON responses** from live browser sessions.
//! Prices are in **VND** (Vietnamese Dong) — integers for trading data,
//! floating-point for some performance/ratio metrics.
//!
//! # Schema Completeness
//!
//! All types in this module have been verified against captured response JSON
//! (see `_workspace/01c_schema_capture.md`). No `\[INFERRED\]` types remain.
//! Fields that were `null` in the captured data use `Option<T>`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::date::{deserialize_ms_date, deserialize_opt_ms_date, deserialize_trim_trailing};

// ===========================================================================
// Document API Types
// ===========================================================================

/// A document type category (e.g., "Financial Statement", "Annual Report").
///
/// Returned by `GET /data/getdocumenttype`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentType {
    #[serde(rename = "DocumentTypeID")]
    pub document_type_id: i32,

    #[serde(rename = "DocumentName")]
    pub document_name: String,

    #[serde(rename = "DocumentNameEn")]
    pub document_name_en: String,

    #[serde(rename = "DisplayOrder")]
    pub display_order: i32,
}

/// A document record with metadata and a CDN download URL.
///
/// Returned by `POST /data/getdocument` (document listing page, 20 per page, `page` and `year` filters)
/// and `POST /data/GetDocument` (overview page — paginated via Offset/Limit).
///
/// The `Url` field points to `static2.vietstock.vn` CDN for direct download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// File extension (e.g., `".zip"`, `".pdf"`).
    /// Trailing spaces from the API are trimmed during deserialization.
    #[serde(rename = "FileExt", deserialize_with = "deserialize_trim_trailing")]
    pub file_ext: String,

    /// Always `null` in observed data. Purpose unknown.
    #[serde(rename = "UpdateTime", default)]
    pub update_time: Option<String>,

    /// Total number of documents available for this stock.
    #[serde(rename = "TotalRow")]
    pub total_row: i32,

    /// Unique document identifier.
    #[serde(rename = "FileInfoID")]
    pub file_info_id: i64,

    /// Direct download URL to `static2.vietstock.vn` CDN.
    #[serde(rename = "Url")]
    pub url: String,

    /// Short display title (Vietnamese).
    #[serde(rename = "Title")]
    pub title: String,

    /// Full descriptive document name (Vietnamese).
    #[serde(rename = "FullName")]
    pub full_name: String,

    /// When Vietstock last updated the file, in Microsoft `/Date(milliseconds)/` format.
    ///
    /// Not the legal publication time: old filings are often re-uploaded years
    /// later (VNM's audited 2010 parent-company statement shows 2016). Use it only
    /// as an upper bound; record your own first-seen time for point-in-time work.
    #[serde(rename = "LastUpdate", deserialize_with = "deserialize_ms_date")]
    pub last_update: DateTime<Utc>,
}

impl Document {
    /// Extract the exchange code from the document's CDN URL.
    ///
    /// URL pattern: `https://static2.vietstock.vn/data/{EXCHANGE}/{YEAR}/...`
    pub fn exchange(&self) -> Option<&str> {
        let after_data = self.url.split("/data/").nth(1)?;
        after_data.split('/').next()
    }

    /// Extract the year from the document's CDN URL.
    pub fn year(&self) -> Option<i32> {
        let after_data = self.url.split("/data/").nth(1)?;
        let year_str = after_data.split('/').nth(1)?;
        year_str.parse().ok()
    }

    /// Returns `true` if this is a PDF document (as opposed to ZIP).
    pub fn is_pdf(&self) -> bool {
        self.file_ext.eq_ignore_ascii_case(".pdf")
    }

    /// Returns `true` if this is a ZIP archive.
    pub fn is_zip(&self) -> bool {
        self.file_ext.eq_ignore_ascii_case(".zip")
    }

    /// Returns `true` if this is a RAR archive (seen on older filings, e.g. 2010).
    pub fn is_rar(&self) -> bool {
        self.file_ext.eq_ignore_ascii_case(".rar")
    }
}

// ===========================================================================
// Trading Types
// ===========================================================================

/// Real-time trading information for a stock.
///
/// Returned by `POST /company/tradinginfo`.
/// All price fields are in **VND** (Vietnamese Dong) as integers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingInfo {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Listed shares outstanding.
    #[serde(rename = "KLCPLH")]
    pub klcplh: i64,

    /// Listed shares (total trading).
    #[serde(rename = "KLCPNY")]
    pub klcpny: i64,

    #[serde(rename = "PriorClosePrice")]
    pub prior_close_price: i64,

    #[serde(rename = "CeilingPrice")]
    pub ceiling_price: i64,

    #[serde(rename = "FloorPrice")]
    pub floor_price: i64,

    #[serde(rename = "TotalVol")]
    pub total_vol: i64,

    #[serde(rename = "TotalVal")]
    pub total_val: i64,

    #[serde(rename = "MarketCapital")]
    pub market_capital: i64,

    #[serde(rename = "HighestPrice")]
    pub highest_price: i64,

    #[serde(rename = "LowestPrice")]
    pub lowest_price: i64,

    #[serde(rename = "OpenPrice")]
    pub open_price: i64,

    #[serde(rename = "LastPrice")]
    pub last_price: i64,

    #[serde(rename = "AvrPrice")]
    pub avr_price: i64,

    #[serde(rename = "Change")]
    pub change: i64,

    #[serde(rename = "PerChange")]
    pub per_change: f64,

    #[serde(rename = "Min52W")]
    pub min_52w: i64,

    #[serde(rename = "Max52W")]
    pub max_52w: i64,

    #[serde(rename = "EPS")]
    pub eps: f64,

    #[serde(rename = "PE")]
    pub pe: f64,

    #[serde(rename = "BVPS")]
    pub bvps: f64,

    #[serde(rename = "PB")]
    pub pb: f64,

    // ── Extended fields (captured from overview page VCB response) ──
    /// 52-week volume.
    #[serde(rename = "Vol52W", default)]
    pub vol_52w: i64,

    /// Outstanding buy volume (shares).
    #[serde(rename = "OutstandingBuy", default)]
    pub outstanding_buy: i64,

    /// Outstanding sell volume (shares).
    #[serde(rename = "OutstandingSell", default)]
    pub outstanding_sell: i64,

    /// Foreign ownership ratio (%).
    #[serde(rename = "OwnedRatio", default)]
    pub owned_ratio: f64,

    /// Dividend amount (VND per share).
    #[serde(rename = "Dividend", default)]
    pub dividend: i64,

    /// Dividend yield (%).
    #[serde(rename = "Yield", default)]
    pub dividend_yield: f64,

    /// Beta coefficient.
    #[serde(rename = "Beta", default)]
    pub beta: f64,

    /// Forward EPS.
    #[serde(rename = "FEPS", default)]
    pub feps: f64,

    /// Price-to-sales ratio.
    #[serde(rename = "PS", default)]
    pub ps: f64,

    /// Total foreign room (shares).
    #[serde(rename = "TotalRoom", default)]
    pub total_room: i64,

    /// Current foreign room (shares).
    #[serde(rename = "CurrRoom", default)]
    pub curr_room: i64,

    /// Remaining foreign room (%).
    #[serde(rename = "RemainRoom", default)]
    pub remain_room: f64,

    /// Foreign buy volume (shares).
    #[serde(rename = "F_BuyVol", default)]
    pub f_buy_vol: i64,

    /// Foreign buy value (VND).
    #[serde(rename = "F_BuyVal", default)]
    pub f_buy_val: i64,

    /// Foreign sell volume (shares).
    #[serde(rename = "F_SellVol", default)]
    pub f_sell_vol: i64,

    /// Foreign sell value (VND).
    #[serde(rename = "F_SellVal", default)]
    pub f_sell_val: i64,

    /// Foreign buy put-through volume.
    #[serde(rename = "F_BuyPutVol", default)]
    pub f_buy_put_vol: i64,

    /// Foreign buy put-through value (VND).
    #[serde(rename = "F_BuyPutVal", default)]
    pub f_buy_put_val: i64,

    /// Foreign sell put-through volume.
    #[serde(rename = "F_SellPutVol", default)]
    pub f_sell_put_vol: i64,

    /// Foreign sell put-through value (VND).
    #[serde(rename = "F_SellPutVal", default)]
    pub f_sell_put_val: i64,

    /// Market status code.
    #[serde(rename = "MarketStatus", default)]
    pub market_status: i64,

    /// Color indicator (1=green, -1=red, 0=unchanged).
    #[serde(rename = "ColorId", default)]
    pub color_id: i64,

    /// Market status name (e.g. "Kết thúc phiên").
    #[serde(rename = "StatusName", default)]
    pub status_name: String,

    /// Stock trading status (usually empty string).
    #[serde(rename = "StockStatus", default)]
    pub stock_status: String,

    /// Total revenue (VND).
    #[serde(rename = "Revenue", default)]
    pub revenue: f64,

    /// Net profit (VND).
    #[serde(rename = "Profit", default)]
    pub profit: f64,
}

// ===========================================================================
// Auth Types
// ===========================================================================

/// Response from `POST /Account/GetAuthToken`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenResponse {
    #[serde(rename = "IsSuccess")]
    pub is_success: bool,
}

/// Response from `POST /Account/Login`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "IsSuccess")]
    pub is_success: bool,

    #[serde(default, rename = "ErrorMessage")]
    pub error_message: Option<String>,

    /// Additional fields that may be present on success.
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

// ===========================================================================
// Performance Types
// ===========================================================================

/// Wrapper for `POST /data/GetPerformanceByStockCode` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResponse {
    pub data: Vec<Performance>,
}

/// Stock price performance across multiple timeframes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performance {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    #[serde(rename = "ClosePrice")]
    pub close_price: f64,

    #[serde(rename = "PerChange")]
    pub per_change: f64,

    #[serde(rename = "ClosePrice1W")]
    pub close_price_1w: f64,

    #[serde(rename = "PerChange1W")]
    pub per_change_1w: f64,

    #[serde(rename = "ClosePrice1M")]
    pub close_price_1m: f64,

    #[serde(rename = "PerChange1M")]
    pub per_change_1m: f64,

    #[serde(rename = "ClosePrice6M")]
    pub close_price_6m: f64,

    #[serde(rename = "PerChange6M")]
    pub per_change_6m: f64,

    #[serde(rename = "ClosePriceYTD")]
    pub close_price_ytd: f64,

    #[serde(rename = "PerChangeYTD")]
    pub per_change_ytd: f64,

    #[serde(rename = "ClosePrice1Y")]
    pub close_price_1y: f64,

    #[serde(rename = "PerChange1Y")]
    pub per_change_1y: f64,

    #[serde(rename = "ClosePrice5Y")]
    pub close_price_5y: f64,

    #[serde(rename = "PerChange5Y")]
    pub per_change_5y: f64,

    #[serde(rename = "ClosePriceAll")]
    pub close_price_all: f64,

    #[serde(rename = "PerChangeAll")]
    pub per_change_all: f64,
}

// ===========================================================================
// FinanceInfo Types (Financial Statements)
// ===========================================================================

/// Period metadata for a financial statement column.
///
/// Each entry describes one reporting period (e.g., Q1/2026, Q4/2025).
/// Returned as the first element of the `financeinfo` response array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodMetadata {
    #[serde(rename = "ID")]
    pub id: i32,

    #[serde(rename = "Row")]
    pub row: i32,

    #[serde(rename = "CompanyID")]
    pub company_id: i32,

    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// Quarter code: "Q1", "Q2", "Q3", "Q4"
    #[serde(rename = "TermCode")]
    pub term_code: String,

    /// Vietnamese quarter name: "Quý 1", "Quý 2", etc.
    #[serde(rename = "TermName")]
    pub term_name: String,

    /// English quarter name: "Quarter 1", "Quarter 2", etc.
    #[serde(rename = "TermNameEN")]
    pub term_name_en: String,

    /// 2=Q1, 3=Q2, 4=Q3, 5=Q4/FY
    #[serde(rename = "ReportTermID")]
    pub report_term_id: i32,

    #[serde(rename = "DisplayOrdering")]
    pub display_ordering: i32,

    /// Scope code, per the response's own legend (element 3): "HN"
    /// consolidated (Hợp nhất), "ĐL" stand-alone, "CTM" parent company.
    #[serde(rename = "United")]
    pub united: String,

    /// Audit code, per the response's own legend (element 2, seen
    /// 2026-11-20): "KT" audited (Kiểm toán), "SX" reviewed (Soát xét),
    /// "CKT" unaudited (Chưa kiểm toán).
    #[serde(rename = "AuditedStatus")]
    pub audited_status: String,

    /// Period begin in YYYYMM format ("202504")
    #[serde(rename = "PeriodBegin")]
    pub period_begin: String,

    /// Period end in YYYYMM format ("202506")
    #[serde(rename = "PeriodEnd")]
    pub period_end: String,

    #[serde(rename = "TotalRow")]
    pub total_row: i32,

    #[serde(rename = "BusinessType")]
    pub business_type: i32,

    #[serde(rename = "ReportNote")]
    pub report_note: Option<String>,

    #[serde(rename = "ReportNoteEn")]
    pub report_note_en: Option<String>,
}

/// A single line item in a financial statement.
///
/// Part of the second element of the `financeinfo` response.
/// Money values are in the request's `Unit` (dong per value): **billions of
/// VND** from [`crate::VietstockClient::get_finance_info`], whole dong with
/// [`crate::FINANCE_INFO_UNIT_DONG`]. Ratio lines (`EPS 4 quý`, `ROS`,
/// `P/E cơ bản`, …) are the same in every unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceInfoLineItem {
    #[serde(rename = "ID")]
    pub id: i32,

    /// Normalized report item identifier.
    #[serde(rename = "ReportNormID")]
    pub report_norm_id: i32,

    /// Line item name (Vietnamese).
    #[serde(rename = "Name")]
    pub name: String,

    /// Line item name (English).
    #[serde(rename = "NameEn")]
    pub name_en: String,

    /// Short name for mobile display (Vietnamese).
    #[serde(rename = "NameMobile")]
    pub name_mobile: String,

    /// Short name for mobile display (English).
    #[serde(rename = "NameMobileEn")]
    pub name_mobile_en: String,

    /// CSS style class ("Normal", "Bold", etc.).
    #[serde(rename = "CssStyle")]
    pub css_style: String,

    /// Indentation level ("Padding1", "Padding2", etc.).
    #[serde(rename = "Padding")]
    pub padding: String,

    /// Parent line item ReportNormID (for hierarchy).
    #[serde(rename = "ParentReportNormID")]
    pub parent_report_norm_id: i32,

    /// The report component this line belongs to.
    #[serde(rename = "ReportComponentName")]
    pub report_component_name: String,

    /// English report component name.
    #[serde(rename = "ReportComponentNameEn")]
    pub report_component_name_en: String,

    /// Display unit (Vietnamese).
    #[serde(rename = "Unit")]
    pub unit: String,

    /// Display unit (English).
    #[serde(rename = "UnitEn")]
    pub unit_en: String,

    #[serde(rename = "OrderType")]
    pub order_type: i32,

    #[serde(rename = "OrderingComponent")]
    pub ordering_component: i32,

    #[serde(rename = "RowNumber")]
    pub row_number: i32,

    #[serde(rename = "ReportComponentTypeID")]
    pub report_component_type_id: Option<i32>,

    /// Number of child line items.
    #[serde(rename = "ChildTotal")]
    pub child_total: i32,

    /// Nesting depth (0 = top level).
    #[serde(rename = "Levels")]
    pub levels: i32,

    /// Value for the most recent period (e.g., Q1/2026), in the request's
    /// unit.
    #[serde(rename = "Value1")]
    pub value1: Option<f64>,

    /// Value for the second period (e.g., Q4/2025).
    #[serde(rename = "Value2")]
    pub value2: Option<f64>,

    /// Value for the third period (e.g., Q3/2025).
    #[serde(rename = "Value3")]
    pub value3: Option<f64>,

    /// Value for the fourth period (e.g., Q2/2025).
    #[serde(rename = "Value4")]
    pub value4: Option<f64>,

    /// Unknown field — always null in captured data.
    #[serde(rename = "Vl")]
    pub vl: Option<f64>,

    /// Whether the data for this row should be displayed.
    #[serde(rename = "IsShowData")]
    pub is_show_data: bool,
}

/// Full response from `POST /data/financeinfo`.
///
/// Response from `POST /data/financeinfo`.
///
/// The API returns a 4-element JSON array:
/// - Element 0: `Vec<PeriodMetadata>` — period columns
/// - Element 1: `HashMap<String, Vec<FinanceInfoLineItem>>` — components
/// - Element 2: `serde_json::Value` — audit status metadata
/// - Element 3: `serde_json::Value` — united type metadata
pub type FinanceInfoResponse = (
    Vec<PeriodMetadata>,
    HashMap<String, Vec<FinanceInfoLineItem>>,
    serde_json::Value,
    serde_json::Value,
);

// ===========================================================================
// Trading Overview Types
// ===========================================================================

/// A single day's trading summary on the overview page.
///
/// Returned by `POST /data/GetStockDeal_OverviewTrading`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDayOverview {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Close price in VND.
    #[serde(rename = "ClosePrice")]
    pub close_price: i64,

    /// 1 = green (up), -1 = red (down), 0 = unchanged.
    #[serde(rename = "ColorID")]
    pub color_id: i32,

    /// Price change in VND.
    #[serde(rename = "Change")]
    pub change: i64,

    /// Percent change.
    #[serde(rename = "PerChange")]
    pub per_change: f64,

    /// Total trading volume (shares).
    #[serde(rename = "TotalVol")]
    pub total_vol: i64,

    /// Total trading value (VND).
    #[serde(rename = "TotalVal")]
    pub total_val: i64,

    /// Number of buy trades.
    #[serde(rename = "TotalBuyTrade")]
    pub total_buy_trade: i64,

    /// Number of sell trades.
    #[serde(rename = "TotalSellTrade")]
    pub total_sell_trade: i64,

    /// Average buy trade size (shares).
    #[serde(rename = "BuyAvgTrade")]
    pub buy_avg_trade: f64,

    /// Average sell trade size (shares).
    #[serde(rename = "SellAvgTrade")]
    pub sell_avg_trade: f64,

    /// Foreign buy value (VND).
    #[serde(rename = "ForeignBuyVal")]
    pub foreign_buy_val: i64,

    /// Foreign sell value (VND).
    #[serde(rename = "ForeignSellVal")]
    pub foreign_sell_val: i64,

    /// Proprietary buy value (VND).
    #[serde(rename = "ProBuyVal")]
    pub pro_buy_val: i64,

    /// Proprietary sell value (VND).
    #[serde(rename = "ProSellVal")]
    pub pro_sell_val: i64,
}

/// Wrapper for the trading overview response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingOverviewResponse {
    /// 5 trading days of data.
    pub data: Vec<TradingDayOverview>,
}

// ===========================================================================
// News Types
// ===========================================================================

/// A news article on the company overview page.
///
/// Returned by `POST /data/GetNews`.
/// Uses Offset/Limit pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "ChannelID")]
    pub channel_id: i32,

    /// Short description / excerpt.
    #[serde(rename = "Head")]
    pub head: String,

    #[serde(rename = "ArticleID")]
    pub article_id: i64,

    #[serde(rename = "Title")]
    pub title: String,

    #[serde(rename = "PublishTime", deserialize_with = "deserialize_ms_date")]
    pub publish_time: DateTime<Utc>,

    /// Full article content (empty when fetched from overview listing).
    #[serde(rename = "Content", default)]
    pub content: String,

    /// Relative URL path (e.g. "/2026/06/...").
    #[serde(rename = "URL")]
    pub url: String,

    /// Total count of news items for this stock.
    #[serde(rename = "TotalRow")]
    pub total_row: i32,

    /// News source name ("FILI", "Tiền phong", etc.).
    #[serde(rename = "Source")]
    pub source: String,

    #[serde(rename = "Icon")]
    pub icon: Option<String>,
}

// ===========================================================================
// Corporate Events Types
// ===========================================================================

/// A corporate event (dividend, meeting, etc.).
///
/// Returned by `POST /data/EventsTypeData`.
/// Nesting: the API returns `[[...]]` (array of arrays).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorporateEvent {
    #[serde(rename = "EventID")]
    pub event_id: i64,

    #[serde(rename = "EventTypeID")]
    pub event_type_id: i32,

    #[serde(rename = "ChannelID")]
    pub channel_id: i32,

    /// Stock ticker code.
    #[serde(rename = "Code")]
    pub code: String,

    /// Full company name.
    #[serde(rename = "CompanyName")]
    pub company_name: String,

    #[serde(rename = "CatID")]
    pub cat_id: i32,

    /// Registration date (Ngày đăng ký cuối cùng).
    #[serde(rename = "GDKHQDate", deserialize_with = "deserialize_ms_date")]
    pub gdkhq_date: DateTime<Utc>,

    /// Not-payment / implementation date.
    #[serde(rename = "NDKCCDate", deserialize_with = "deserialize_ms_date")]
    pub ndkcc_date: DateTime<Utc>,

    #[serde(rename = "Time", deserialize_with = "deserialize_ms_date")]
    pub time: DateTime<Utc>,

    /// Event description note.
    #[serde(rename = "Note")]
    pub note: String,

    /// Event name (e.g. "Trả cổ tức bằng tiền mặt").
    #[serde(rename = "Name")]
    pub name: String,

    /// Exchange: "HOSE", "HNX", "UPCOM".
    #[serde(rename = "Exchange")]
    pub exchange: String,

    #[serde(rename = "Title")]
    pub title: String,

    /// Full event content (HTML).
    #[serde(rename = "Content")]
    pub content: String,

    /// Attachment file URL.
    #[serde(rename = "FileUrl")]
    pub file_url: String,

    #[serde(rename = "DateOrder", deserialize_with = "deserialize_ms_date")]
    pub date_order: DateTime<Utc>,

    #[serde(rename = "Row")]
    pub row: i32,
}

// ===========================================================================
// Analysis Reports Types
// ===========================================================================

/// An analysis report from a brokerage or research firm.
///
/// Returned by `POST /data/Overview_ReportByStockCode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    #[serde(rename = "ReportID")]
    pub report_id: i32,

    #[serde(rename = "Title")]
    pub title: String,

    #[serde(rename = "PublishDate", deserialize_with = "deserialize_ms_date")]
    pub publish_date: DateTime<Utc>,

    /// Report type name (e.g., "Phân tích Doanh nghiệp").
    #[serde(rename = "TypeName", default)]
    pub type_name: String,

    /// Report type ID.
    #[serde(rename = "ReportTypeID", default)]
    pub report_type_id: i32,

    /// Source/brokerage name.
    #[serde(rename = "SourceName", default)]
    pub source_name: String,

    /// Source ID.
    #[serde(rename = "SourceID", default)]
    pub source_id: i32,

    /// Report summary content (HTML).
    #[serde(rename = "Content", default)]
    pub content: String,

    /// Relative URL to the full report page.
    #[serde(rename = "Url", default)]
    pub url: String,

    /// Thumbnail image URL.
    #[serde(rename = "Img", default)]
    pub img: String,

    /// Download count.
    #[serde(rename = "DownloadHit", default)]
    pub download_hit: i32,

    /// Whether the report is downloadable.
    #[serde(rename = "IsDownload", default)]
    pub is_download: bool,

    /// Total number of reports available.
    #[serde(rename = "TotalRow", default)]
    pub total_row: i32,

    #[serde(rename = "StockCode")]
    pub stock_code: String,
}

// ===========================================================================
// Technical Rating Types
// ===========================================================================

/// Full technical analysis rating with all indicators.
///
/// Returned by `POST /TechnicalRating/GetFinalRanking`.
/// Flag fields (FSMA*, FEMA*, FinalRating*) are -1 (bearish), 0 (neutral), 1 (bullish).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalRating {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    // ── Moving Averages ──
    #[serde(rename = "SMA10")]
    pub sma10: f64,
    #[serde(rename = "FSMA10")]
    pub fsma10: f64,
    #[serde(rename = "EMA10")]
    pub ema10: f64,
    #[serde(rename = "FEMA10")]
    pub fema10: f64,
    #[serde(rename = "SMA20")]
    pub sma20: f64,
    #[serde(rename = "FSMA20")]
    pub fsma20: f64,
    #[serde(rename = "EMA20")]
    pub ema20: f64,
    #[serde(rename = "FEMA20")]
    pub fema20: f64,
    #[serde(rename = "SMA30")]
    pub sma30: f64,
    #[serde(rename = "FSMA30")]
    pub fsma30: f64,
    #[serde(rename = "EMA30")]
    pub ema30: f64,
    #[serde(rename = "FEMA30")]
    pub fema30: f64,
    #[serde(rename = "SMA50")]
    pub sma50: f64,
    #[serde(rename = "FSMA50")]
    pub fsma50: f64,
    #[serde(rename = "EMA50")]
    pub ema50: f64,
    #[serde(rename = "FEMA50")]
    pub fema50: f64,
    #[serde(rename = "SMA100")]
    pub sma100: f64,
    #[serde(rename = "FSMA100")]
    pub fsma100: f64,
    #[serde(rename = "EMA100")]
    pub ema100: f64,
    #[serde(rename = "FEMA100")]
    pub fema100: f64,
    #[serde(rename = "SMA200")]
    pub sma200: f64,
    #[serde(rename = "FSMA200")]
    pub fsma200: f64,
    #[serde(rename = "EMA200")]
    pub ema200: f64,
    #[serde(rename = "FEMA200")]
    pub fema200: f64,
    /// Composite SMA rating.
    #[serde(rename = "FinalRatingSMA")]
    pub final_rating_sma: f64,
    /// Hull Moving Average (9).
    #[serde(rename = "HMA9")]
    pub hma9: f64,
    #[serde(rename = "FinalRatingHMA")]
    pub final_rating_hma: f64,
    /// Volume-Weighted Moving Average (20).
    #[serde(rename = "VWMA20")]
    pub vwma20: f64,
    #[serde(rename = "FinalRatingVWMA")]
    pub final_rating_vwma: f64,
    /// Ichimoku Cloud baseline.
    #[serde(rename = "IchimokuBaseLine")]
    pub ichimoku_base_line: f64,
    #[serde(rename = "FinalRatingIchimoku")]
    pub final_rating_ichimoku: f64,
    /// Composite moving averages rating.
    #[serde(rename = "MovingAverages")]
    pub moving_averages: f64,

    // ── Oscillators ──
    #[serde(rename = "RSI")]
    pub rsi: f64,
    #[serde(rename = "FinalRatingRSI")]
    pub final_rating_rsi: f64,
    #[serde(rename = "MACD")]
    pub macd: f64,
    #[serde(rename = "FinalRatingMACD")]
    pub final_rating_macd: f64,
    /// Stochastic Oscillator.
    #[serde(rename = "OS")]
    pub os: f64,
    #[serde(rename = "FinalRatingOS")]
    pub final_rating_os: f64,
    #[serde(rename = "WilliamsR")]
    pub williams_r: f64,
    #[serde(rename = "FinalRatingR")]
    pub final_rating_r: f64,
    #[serde(rename = "ADX")]
    pub adx: f64,
    #[serde(rename = "FinalRatingADX")]
    pub final_rating_adx: f64,
    #[serde(rename = "Momentum10")]
    pub momentum10: f64,
    #[serde(rename = "FinalRatingMomentum")]
    pub final_rating_momentum: f64,
    /// Ultimate Oscillator.
    #[serde(rename = "UO")]
    pub uo: f64,
    #[serde(rename = "FinalRatingUO")]
    pub final_rating_uo: f64,
    #[serde(rename = "CCI20")]
    pub cci20: f64,
    #[serde(rename = "FinalRatingCCI")]
    pub final_rating_cci: f64,
    #[serde(rename = "StochRSI")]
    pub stoch_rsi: f64,
    #[serde(rename = "FinalRatingStochRSI")]
    pub final_rating_stoch_rsi: f64,
    /// Awesome Oscillator.
    #[serde(rename = "AO")]
    pub ao: f64,
    #[serde(rename = "FinalRatingAO")]
    pub final_rating_ao: f64,
    #[serde(rename = "BullBearPower")]
    pub bull_bear_power: f64,
    #[serde(rename = "FinalRatingBBP")]
    pub final_rating_bbp: f64,
    /// Composite oscillators rating.
    #[serde(rename = "Oscillators")]
    pub oscillators: f64,

    /// Final overall technical rating.
    #[serde(rename = "FinalRatings")]
    pub final_ratings: f64,
}

/// Wrapper for technical rating response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalRatingResponse {
    pub success: bool,
    pub data: Vec<TechnicalRating>,
}

// ===========================================================================
// Fundamental Rating Types
// ===========================================================================

/// A single factor group in the fundamental rating.
///
/// Returned by `POST /Data/FundamentalRating_GetByStock`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalFactor {
    #[serde(rename = "GroupIndexId")]
    pub group_index_id: i32,

    /// Factor group name (Quản trị, Thanh khoản, Định giá, Sinh lợi, etc.).
    #[serde(rename = "GroupIndex")]
    pub group_index: String,

    /// Average rating for the group.
    #[serde(rename = "AvgRating")]
    pub avg_rating: f64,

    /// Fundamental rating for the group.
    #[serde(rename = "FundaRating")]
    pub funda_rating: f64,

    /// Industry average for AvgRating.
    #[serde(rename = "AvgRatingInd")]
    pub avg_rating_ind: f64,

    /// Industry average for FundaRating.
    #[serde(rename = "FundaRatingInd")]
    pub funda_rating_ind: f64,
}

/// Overall fundamental rating summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalSummary {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "StockName")]
    pub stock_name: String,

    #[serde(rename = "BusinessTypeID")]
    pub business_type_id: i32,

    /// Industry code (4010 = banking, etc.).
    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    #[serde(rename = "IndustryName")]
    pub industry_name: String,

    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// Numeric rating (0–10).
    #[serde(rename = "VSRating")]
    pub vs_rating: f64,

    /// Letter rating: "A", "B", "C", "D", "E".
    #[serde(rename = "Rating")]
    pub rating: String,
}

/// Wrapper for fundamental rating response.
///
/// The `data` field is a tuple: `[factors[], summary[]]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalRatingResponse {
    pub success: bool,
    pub data: (Vec<FundamentalFactor>, Vec<FundamentalSummary>),
}

// ===========================================================================
// Bond Types
// ===========================================================================

/// A bond listing entry.
///
/// Returned by `POST /Data/GetBondRelated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondItem {
    #[serde(rename = "KeyCode")]
    pub key_code: String,

    /// Bond ticker code.
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "BondCode")]
    pub bond_code: String,

    #[serde(rename = "ReleaseDate", deserialize_with = "deserialize_ms_date")]
    pub release_date: DateTime<Utc>,

    #[serde(rename = "DueDate", deserialize_with = "deserialize_ms_date")]
    pub due_date: DateTime<Utc>,

    /// Days remaining until maturity.
    #[serde(rename = "RemainingMaturity")]
    pub remaining_maturity: i32,

    /// Face value per bond (VND).
    #[serde(rename = "FaceValue")]
    pub face_value: i64,

    /// Issue interest rate (%).
    #[serde(rename = "IssueRate")]
    pub issue_rate: f64,

    /// Issue volume (number of bonds). NOTE: API typo "IssuaVolume".
    #[serde(rename = "IssuaVolume")]
    pub issue_volume: i64,

    /// Total issue value (VND).
    #[serde(rename = "IssueValue")]
    pub issue_value: i64,

    /// Outstanding value (VND).
    #[serde(rename = "OutstandingSharesValue")]
    pub outstanding_shares_value: i64,

    /// Outstanding shares (number of bonds).
    #[serde(rename = "OutstandingShares")]
    pub outstanding_shares: i64,

    #[serde(rename = "CompanyCode")]
    pub company_code: Option<String>,

    #[serde(rename = "CompanyName")]
    pub company_name: Option<String>,

    #[serde(rename = "CompanyURL")]
    pub company_url: Option<String>,

    /// "Cố định" (Fixed) or "Thả nổi" (Floating).
    #[serde(rename = "InterestRateType")]
    pub interest_rate_type: String,

    /// Interest payment period (e.g. "12 Tháng").
    #[serde(rename = "InterestPeriod")]
    pub interest_period: String,

    /// Total number of bond records.
    #[serde(rename = "TotalRecord")]
    pub total_record: i32,
}

/// Wrapper for bond related response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondRelatedResponse {
    #[serde(rename = "RelatedBondInfos")]
    pub related_bond_infos: Vec<BondItem>,
}

// ===========================================================================
// Bond Market Types
// ===========================================================================

/// Market overview statistics for the corporate bonds market.
///
/// Returned by `POST /Bonds/GetBondMarketGeneralInfo`.
/// Uses multipart/form-data content type (unique among Vietstock endpoints).
///
/// **Units:**
/// - `bond_quantity`: Số lượng trái phiếu (count of bonds)
/// - `bond_value`: Tổng mệnh giá (total face value, VND)
/// - `klny` / `kllh`: Khối lượng niêm yết / lưu hành (bonds)
/// - `total_tr_vol` / `total_tr_val`: Tổng KLGD/GTGD khớp lệnh (bonds / VND)
/// - `total_neg_vol` / `total_neg_val`: Tổng KLGD/GTGD thỏa thuận (bonds / VND)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondMarketOverview {
    /// Số lượng trái phiếu.
    #[serde(rename = "BondQuantity")]
    pub bond_quantity: i32,

    /// Tổng mệnh giá trái phiếu (VND).
    #[serde(rename = "BondValue")]
    pub bond_value: f64,

    /// Phần trăm giá cao.
    #[serde(rename = "HighPricePercent")]
    pub high_price_percent: f64,

    /// Khối lượng niêm yết (bonds).
    #[serde(rename = "KLNY")]
    pub klny: f64,

    /// Khối lượng lưu hành (bonds).
    #[serde(rename = "KLLH")]
    pub kllh: f64,

    /// Tổng khối lượng giao dịch khớp lệnh (bonds).
    #[serde(rename = "TotalTrVol")]
    pub total_tr_vol: f64,

    /// Tổng giá trị giao dịch khớp lệnh (VND).
    #[serde(rename = "TotalTrVal")]
    pub total_tr_val: f64,

    /// Tổng khối lượng giao dịch thỏa thuận (bonds).
    #[serde(rename = "TotalNegVol")]
    pub total_neg_vol: f64,

    /// Tổng giá trị giao dịch thỏa thuận (VND).
    #[serde(rename = "TotalNegVal")]
    pub total_neg_val: f64,

    /// Ngày giao dịch gần nhất.
    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,
}

/// A single data point in the bond market trading chart.
///
/// Returned by `POST /Bonds/MarketOverviewChart`.
/// Each entry represents one trading day's aggregated volume and value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondChartPoint {
    /// Trading record ID.
    #[serde(rename = "TrID")]
    pub tr_id: i32,

    /// Trading date.
    #[serde(rename = "TradeDate", deserialize_with = "deserialize_ms_date")]
    pub trade_date: DateTime<Utc>,

    /// Tổng giá trị giao dịch trong ngày (VND).
    #[serde(rename = "TotalValue")]
    pub total_value: f64,

    /// Tổng khối lượng giao dịch trong ngày (bonds).
    #[serde(rename = "TotalVolume")]
    pub total_volume: f64,
}

/// A single row in the bond trading history / listing table.
///
/// Returned by `POST /Bonds/GetLastTrBondHistory` (paginated).
///
/// **Units:**
/// - `close_price`: Ngàn đồng (thousands of VND)
/// - `face_value`: VND per bond
/// - All `*_vol` fields: Trái phiếu (bonds)
/// - All `*_val` fields: VND
/// - `outstanding_shares`: Trái phiếu (bonds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondTradingItem {
    /// Unique bond history record ID.
    #[serde(rename = "TrBondHisID")]
    pub tr_bond_his_id: i64,

    /// Internal stock-bond identifier.
    #[serde(rename = "StockBondID")]
    pub stock_bond_id: i32,

    /// Bond ticker code (e.g., "MML121021").
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    /// Khối lượng lưu hành (bonds).
    #[serde(rename = "OutstandingShares")]
    pub outstanding_shares: f64,

    /// Mệnh giá (face value per bond, VND).
    #[serde(rename = "FaceValue")]
    pub face_value: f64,

    /// Maturity date. `None` for bonds without a fixed maturity.
    #[serde(
        rename = "DueDate",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub due_date: Option<DateTime<Utc>>,

    /// Interest rate (%). `None` for zero-coupon or undisclosed.
    #[serde(rename = "IssueRate", default)]
    pub issue_rate: Option<f64>,

    /// Trading record ID.
    #[serde(rename = "TrID")]
    pub tr_id: i32,

    /// Trading date.
    #[serde(rename = "TradeDate", deserialize_with = "deserialize_ms_date")]
    pub trade_date: DateTime<Utc>,

    /// Giá đóng cửa (thousands of VND).
    #[serde(rename = "ClosePrice")]
    pub close_price: f64,

    /// Color indicator: 1 = price up (green), 0 = unchanged/down.
    #[serde(rename = "Color")]
    pub color: i32,

    /// Tổng KLGD khớp lệnh — chẵn lô (bonds).
    #[serde(rename = "TotalEvenLotVol")]
    pub total_even_lot_vol: f64,

    /// Tổng GTGD khớp lệnh — chẵn lô (VND).
    #[serde(rename = "TotalEvenLotVal")]
    pub total_even_lot_val: f64,

    /// Tổng KLGD khớp lệnh — lô lẻ (bonds).
    #[serde(rename = "TotalOddLotVol")]
    pub total_odd_lot_vol: f64,

    /// Tổng GTGD khớp lệnh — lô lẻ (VND).
    #[serde(rename = "TotalOddLotVal")]
    pub total_odd_lot_val: f64,

    /// Tổng KLGD thỏa thuận — chẵn lô (bonds).
    #[serde(rename = "TotalEvenLotNegotiatedVol")]
    pub total_even_lot_negotiated_vol: f64,

    /// Tổng GTGD thỏa thuận — chẵn lô (VND).
    #[serde(rename = "TotalEvenLotNegotiatedVal")]
    pub total_even_lot_negotiated_val: f64,

    /// Tổng KLGD thỏa thuận — lô lẻ (bonds).
    #[serde(rename = "TotalOddLotNegotiatedVol")]
    pub total_odd_lot_negotiated_vol: f64,

    /// Tổng GTGD thỏa thuận — lô lẻ (VND).
    #[serde(rename = "TotalOddLotNegotiatedVal")]
    pub total_odd_lot_negotiated_val: f64,

    /// Tên tổ chức phát hành (Issuer name).
    #[serde(rename = "CompanyName")]
    pub company_name: String,

    /// Total number of records (for pagination). Same value in every item.
    #[serde(rename = "TotalRecord")]
    pub total_record: i32,

    /// Row number within the current page (1-based).
    #[serde(rename = "RowNum")]
    pub row_num: i32,
}

/// A corporate bond news article.
///
/// Returned by `POST /Bonds/GetBondNews` (paginated, filterable by news type).
///
/// # News type values
///
/// | type | Vietnamese Name | English |
/// |------|-----------------|---------|
/// | 5    | Đăng ký giao dịch | Registration |
/// | 6    | Thay đổi đăng ký giao dịch | Registration Changes |
/// | 4    | Tin thị trường | Market News |
/// | 0    | Thông tin công bố | Announcements |
///
/// **Note:** This is DIFFERENT from [`NewsItem`] — field names use uppercase
/// (`URL`, `TotalRecord`, `PublishTimeConvert`) and there is no `StockCode` or `Head` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondNewsItem {
    /// Unique article identifier.
    #[serde(rename = "ArticleID")]
    pub article_id: i64,

    /// Article title.
    #[serde(rename = "Title")]
    pub title: String,

    /// Publication time (Microsoft date format).
    #[serde(rename = "PublishTime", deserialize_with = "deserialize_ms_date")]
    pub publish_time: DateTime<Utc>,

    /// Human-readable publication time (e.g., "16/06/2026 00:00").
    #[serde(rename = "PublishTimeConvert")]
    pub publish_time_convert: String,

    /// Relative article URL (e.g., "/2026/06/cbtt-ve-viec-...").
    #[serde(rename = "URL")]
    pub url: String,

    /// Total count of articles for this news type (same value in every item on a page).
    #[serde(rename = "TotalRecord")]
    pub total_record: i32,

    /// News source name (e.g., "HNX", "FILI").
    #[serde(rename = "Source")]
    pub source: String,

    /// Source icon (optional). `None` when no icon is available.
    #[serde(rename = "Icon", default)]
    pub icon: Option<String>,
}

// ===========================================================================
// Planned Targets Types
// ===========================================================================

/// A planned business target (revenue, profit target for a specific year).
///
/// Returned by `POST /Data/GetData_PlannedTarget`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTarget {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    /// Plan target ID (5=Revenue, 8=LNTT, 9=LNST).
    #[serde(rename = "PTID")]
    pub pt_id: i32,

    /// Vietnamese description of the target.
    #[serde(rename = "PTName")]
    pub pt_name: String,

    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// Target value in VND.
    #[serde(rename = "Value")]
    pub value: f64,
}

/// A year entry in the planned targets response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanYear {
    #[serde(rename = "Year")]
    pub year: i32,
}

/// Inner data for planned targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTargetData {
    #[serde(rename = "Data_Results")]
    pub data_results: Vec<PlannedTarget>,

    #[serde(rename = "Year_Results")]
    pub year_results: Vec<PlanYear>,
}

/// Full response for planned targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTargetResponse {
    pub data: PlannedTargetData,
}

// ===========================================================================
// Target Implementation Types
// ===========================================================================

/// Implementation status for a planned target.
///
/// Returned by `POST /Data/GetData_PlannedTarget_ImplementStatus`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetImplementation {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// 1=FullYear, 2=Q1, 3=Q2, 4=Q3, 5=Q4.
    #[serde(rename = "ReportTermID")]
    pub report_term_id: i32,

    /// Report norm ID: 4377=LNTT, 4378=LNST, 4399=Doanh thu.
    #[serde(rename = "ReportNormID")]
    pub report_norm_id: i32,

    /// Actual achieved value (VND).
    #[serde(rename = "Value")]
    pub value: f64,

    /// Plan target ID (5=Revenue, 8=LNTT, 9=LNST).
    #[serde(rename = "PTID")]
    pub pt_id: i32,
}

/// Inner data for target implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetImplementationData {
    #[serde(rename = "Data_Results")]
    pub data_results: Vec<TargetImplementation>,
}

/// Full response for target implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetImplementationResponse {
    pub data: TargetImplementationData,
}

// ===========================================================================
// Reference Price Types
// ===========================================================================

/// Reference price data for a stock.
///
/// Returned by `POST /company/GetDataRefPriceByCode`.
/// **Note:** `RefPrice` is in **thousands of VND** (61.6 = 61,600 VND).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRefPrice {
    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Reference price in THOUSANDS of VND (61.6 = 61,600 VND).
    #[serde(rename = "RefPrice")]
    pub ref_price: f64,

    #[serde(rename = "MarketStatus")]
    pub market_status: i32,
}

/// Daily stock color / trading summary data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockColorDay {
    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    #[serde(rename = "LastPrice")]
    pub last_price: i64,

    #[serde(rename = "Change")]
    pub change: i64,

    #[serde(rename = "PerChange")]
    pub per_change: f64,

    #[serde(rename = "TotalVol")]
    pub total_vol: i64,

    #[serde(rename = "BuyAvg")]
    pub buy_avg: f64,

    #[serde(rename = "SellAvg")]
    pub sell_avg: f64,

    #[serde(rename = "ForeignBuyVol")]
    pub foreign_buy_vol: i64,

    #[serde(rename = "ForeignSellVol")]
    pub foreign_sell_vol: i64,

    #[serde(rename = "ColorId")]
    pub color_id: i32,
}

/// Full response for reference price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefPriceResponse {
    #[serde(rename = "dataRefPrice")]
    pub data_ref_price: DataRefPrice,

    /// Last 5 trading days of color data.
    #[serde(rename = "stockColor")]
    pub stock_color: Vec<StockColorDay>,
}

// ===========================================================================
// Market Price Types
// ===========================================================================

/// A market index price entry.
///
/// Returned by `POST /data/getmarketprice`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    /// Index code: "VNIndex", "HNXINDEX", "VS100", "CL", "XAUUSDO", etc.
    #[serde(rename = "Code")]
    pub code: String,

    /// Display name: "VN-Index", "HNX-Index", "Spot Gold", etc.
    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Price")]
    pub price: f64,

    #[serde(rename = "Change")]
    pub change: f64,

    #[serde(rename = "PerChange")]
    pub per_change: f64,

    /// Hex color code (e.g. "#39ca32").
    #[serde(rename = "Color")]
    pub color: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,
}

// ===========================================================================
// Header News Types
// ===========================================================================

/// A headline news item in the top navigation banner.
///
/// Returned by `POST /data/headernews`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderNewsItem {
    #[serde(rename = "Title")]
    pub title: String,

    /// Relative URL to the full article.
    #[serde(rename = "URL")]
    pub url: String,

    #[serde(rename = "PublishTime", deserialize_with = "deserialize_ms_date")]
    pub publish_time: DateTime<Utc>,

    /// News source: "FILI", "Tiền phong", etc.
    #[serde(rename = "Source")]
    pub source: String,

    #[serde(rename = "Icon")]
    pub icon: Option<String>,

    #[serde(rename = "ArticleID")]
    pub article_id: i64,
}

// ===========================================================================
// Template Types
// ===========================================================================

/// A page template metadata entry.
///
/// Returned by `POST /data/GetTemplateByName`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateItem {
    #[serde(rename = "TemplateID")]
    pub template_id: i32,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Content")]
    pub content: String,

    #[serde(rename = "Priority")]
    pub priority: i32,
}

// ===========================================================================
// Event Calendar Types (Market-Wide)
// ===========================================================================

/// A market-wide event calendar item (listing, dividend, shareholder meeting).
///
/// Returned by `POST /data/eventstypedata`.
/// This is DIFFERENT from `CorporateEvent` (returned by `/data/EventsTypeData`)
/// which is company-specific events on the overview page.
///
/// Many date fields can be `null` depending on the event type; use `Option<DateTime<Utc>>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCalendarItem {
    #[serde(rename = "EventID")]
    pub event_id: i64,

    #[serde(rename = "ChannelID")]
    pub channel_id: i32,

    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "CompanyName")]
    pub company_name: String,

    #[serde(rename = "CatID")]
    pub cat_id: i32,

    /// Ex-right date (Ngày đăng ký cuối cùng).
    #[serde(
        rename = "GDKHQDate",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub gdkhq_date: Option<DateTime<Utc>>,

    /// Record date / implementation date.
    #[serde(
        rename = "NDKCCDate",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub ndkcc_date: Option<DateTime<Utc>>,

    /// Event date.
    #[serde(rename = "Time", default, deserialize_with = "deserialize_opt_ms_date")]
    pub time: Option<DateTime<Utc>>,

    /// Event description note (Vietnamese).
    #[serde(rename = "Note")]
    pub note: String,

    #[serde(rename = "EventTypeID")]
    pub event_type_id: i32,

    /// Event type name (Vietnamese).
    #[serde(rename = "Name")]
    pub name: String,

    /// Exchange: "HOSE", "HNX", "UPCoM", "OTC".
    #[serde(rename = "Exchange")]
    pub exchange: String,

    #[serde(rename = "Source", default)]
    pub source: String,

    /// Full event title.
    #[serde(rename = "Title")]
    pub title: String,

    /// HTML content (often empty for listing events).
    #[serde(rename = "Content", default)]
    pub content: String,

    /// Link to disclosure PDF.
    #[serde(rename = "FileUrl", default)]
    pub file_url: String,

    #[serde(
        rename = "DateOrder",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_order: Option<DateTime<Utc>>,

    /// Number of shares involved.
    #[serde(rename = "Volume", default)]
    pub volume: Option<i64>,

    /// Price (for dividend events, in VND). Null for listing events.
    #[serde(rename = "Price", default)]
    pub price: Option<f64>,

    // ── Additional listing-specific fields (default for other event types) ──
    #[serde(rename = "Reason", default)]
    pub reason: String,

    #[serde(rename = "ReasonCBKS", default)]
    pub reason_cbks: String,

    #[serde(
        rename = "DateNYTL",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_nytl: Option<DateTime<Utc>>,

    #[serde(
        rename = "DateHCB",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_hcb: Option<DateTime<Utc>>,

    #[serde(
        rename = "DateGDTL",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_gdtl: Option<DateTime<Utc>>,

    /// Trading date.
    #[serde(
        rename = "DateCTGD",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_ctgd: Option<DateTime<Utc>>,

    #[serde(rename = "CancelVolume", default)]
    pub cancel_volume: Option<i64>,

    #[serde(rename = "TimeDPGD", default)]
    pub time_dpgd: String,

    #[serde(rename = "Row")]
    pub row: i32,
}

// ===========================================================================
// Event Type + Channel Type Lookup Types
// ===========================================================================

/// An event type definition (e.g., Listing, Dividend, Shareholders Meeting).
///
/// Returned as the first array in `POST /data/eventtypebyid` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeDef {
    #[serde(rename = "EventTypeID")]
    pub event_type_id: i32,

    /// Event type name (Vietnamese).
    #[serde(rename = "Name")]
    pub name: String,

    /// Event type name (English). May be `null` for some entries.
    #[serde(rename = "NameEn", default)]
    pub name_en: Option<String>,
}

/// A channel/sub-type definition (sub-types within an event type).
///
/// Returned as the second array in `POST /data/eventtypebyid` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelTypeDef {
    #[serde(rename = "ChannelID")]
    pub channel_id: i32,

    /// Channel name (Vietnamese).
    #[serde(rename = "Name")]
    pub name: String,

    /// Channel name (English). May be `null` for some entries.
    #[serde(rename = "NameEn", default)]
    pub name_en: Option<String>,
}

/// Full response from `POST /data/eventtypebyid`.
///
/// Tuple of `(event_types, channel_types)`.
pub type EventTypeLookup = (Vec<EventTypeDef>, Vec<ChannelTypeDef>);

// ===========================================================================
// Insider Trading Types
// ===========================================================================

/// An insider trading record for a stock.
///
/// Returned by `POST /data/eventstransferdata`.
/// Fields vary depending on whether the record is a "registration" or "execution" row.
/// Use `#[serde(default)]` extensively to handle both types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTradingRecord {
    #[serde(rename = "EventID")]
    pub event_id: i64,

    #[serde(rename = "StockCode")]
    pub stock_code: String,

    #[serde(rename = "FinanceURL", default)]
    pub finance_url: String,

    /// Full HTML content with transaction details.
    #[serde(rename = "Content", default)]
    pub content: String,

    /// Event title.
    #[serde(rename = "Title")]
    pub title: String,

    /// Link to disclosure PDF.
    #[serde(rename = "FileUrl", default)]
    pub file_url: String,

    /// Transaction type name (Vietnamese), e.g. "GD của người liên quan".
    #[serde(rename = "TypeName")]
    pub type_name: String,

    #[serde(rename = "TransferTypeID")]
    pub transfer_type_id: i32,

    /// Position code for internal shareholder (Cổ đông nội bộ).
    #[serde(rename = "PositionCD", default)]
    pub position_cd: String,

    /// Additional position for related person (e.g., "TV HĐQT").
    #[serde(rename = "ExtraPositionNLQ", default)]
    pub extra_position_nlq: String,

    /// Extra position NLQ (exchange-specific).
    #[serde(rename = "ExtraPositionNLQEx", default)]
    pub extra_position_nlq_ex: String,

    /// Extra position for internal shareholder.
    #[serde(rename = "ExtraPositionNN", default)]
    pub extra_position_nn: String,

    /// Relationship type.
    #[serde(rename = "RelationShipType", default)]
    pub relationship_type: String,

    /// Người thực hiện giao dịch - cổ đông (Internal shareholder transactor).
    #[serde(rename = "DTTHCD", default)]
    pub dtthcd: String,

    /// Người thực hiện giao dịch - người liên quan (Related person transactor).
    #[serde(rename = "DTTHLQ", default)]
    pub dtthlq: String,

    /// Đối tượng liên quan (Related party name).
    #[serde(rename = "DTLQLQ", default)]
    pub dtlqlq: String,

    /// Người thực hiện giao dịch (Executor/transactor name).
    #[serde(rename = "NVTH", default)]
    pub nvth: String,

    // ── Registration volumes ──
    /// Registered buy volume (shares).
    #[serde(rename = "RegisterBuyVolume", default)]
    pub register_buy_volume: Option<i64>,

    /// Actual buy volume executed (shares).
    #[serde(rename = "BuyVolume", default)]
    pub buy_volume: Option<i64>,

    /// Registered sell volume (shares).
    #[serde(rename = "RegisterSellVolume", default)]
    pub register_sell_volume: Option<i64>,

    /// Actual sell volume executed (shares).
    #[serde(rename = "SellVolume", default)]
    pub sell_volume: Option<i64>,

    /// Shares held before registration.
    #[serde(rename = "RegisterVolumeBefore", default)]
    pub register_volume_before: Option<i64>,

    /// Shares expected after registration.
    #[serde(rename = "RegisterVolumeAfter", default)]
    pub register_volume_after: Option<i64>,

    /// Actual shares held before execution.
    #[serde(rename = "VolumeBefore", default)]
    pub volume_before: Option<i64>,

    /// Actual shares held after execution.
    #[serde(rename = "VolumeAfter", default)]
    pub volume_after: Option<i64>,

    // ── Dates ──
    /// Expected buy date (start of window).
    #[serde(
        rename = "DateBuyExpected",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_buy_expected: Option<DateTime<Utc>>,

    /// Expected sell date (end of window).
    #[serde(
        rename = "DateSellExpected",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_sell_expected: Option<DateTime<Utc>>,

    /// Actual execution end date.
    #[serde(
        rename = "DateActionTo",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_action_to: Option<DateTime<Utc>>,

    /// Actual execution start date.
    #[serde(
        rename = "DateActionFrom",
        default,
        deserialize_with = "deserialize_opt_ms_date"
    )]
    pub date_action_from: Option<DateTime<Utc>>,

    // ── Percent fields ──
    #[serde(rename = "RegisterBuyVolumePercent", default)]
    pub register_buy_volume_percent: Option<f64>,

    #[serde(rename = "BuyVolumePercent", default)]
    pub buy_volume_percent: Option<f64>,

    #[serde(rename = "RegisterSellVolumePercent", default)]
    pub register_sell_volume_percent: Option<f64>,

    #[serde(rename = "SellVolumePercent", default)]
    pub sell_volume_percent: Option<f64>,

    #[serde(rename = "RegisterVolumeBeforePercent", default)]
    pub register_volume_before_percent: Option<f64>,

    #[serde(rename = "RegisterVolumeAfterPercent", default)]
    pub register_volume_after_percent: Option<f64>,

    #[serde(rename = "VolumeBeforePercent", default)]
    pub volume_before_percent: Option<f64>,

    #[serde(rename = "VolumeAfterPercent", default)]
    pub volume_after_percent: Option<f64>,

    // ── Status ──
    /// Status: "Đăng ký" (Registered) or execution status.
    #[serde(rename = "StatusName", default)]
    pub status_name: String,

    #[serde(rename = "TotalRecord")]
    pub total_record: i32,

    #[serde(rename = "Row")]
    pub row: i32,

    // ── Extra positional fields ──
    #[serde(rename = "PositionCDEx", default)]
    pub position_cd_ex: String,

    #[serde(rename = "ExtraPositionIDNNEx", default)]
    pub extra_position_id_nn_ex: Option<i32>,

    #[serde(rename = "NDDTitle", default)]
    pub ndd_title: Option<i32>,

    #[serde(rename = "NDDTH", default)]
    pub nddth: String,

    #[serde(rename = "NDDPosition", default)]
    pub ndd_position: Option<i32>,

    #[serde(rename = "NDDExtraPosition", default)]
    pub ndd_extra_position: Option<i32>,

    #[serde(rename = "TransferTitleTypeID", default)]
    pub transfer_title_type_id: Option<i32>,
}

// ===========================================================================
// Transfer Type Definitions (Insider Trading)
// ===========================================================================

/// A transaction type definition for insider trading filter.
///
/// Returned by `GET /data/eventstransfertype`.
/// NOTE: The API uses `Title`/`TitleEn` field names (not `Name`/`NameEN`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTypeDef {
    #[serde(rename = "ID")]
    pub id: i32,

    /// Type name (Vietnamese).
    #[serde(rename = "Title")]
    pub title: String,

    /// Type name (English).
    #[serde(rename = "TitleEn")]
    pub title_en: String,

    /// Display priority in the UI.
    #[serde(rename = "Priority")]
    pub priority: i32,
}

// ===========================================================================
// Industry / Sector Types
// ===========================================================================

/// A single node in the GICS industry classification hierarchy (4 levels).
///
/// Returned by `POST /Data/GetListIndustryGICS`. Contains 285 items across
/// all levels of the hierarchy.
///
/// **Code Encoding:**
/// - Level 1: 2-digit codes (10=Energy, 15=Materials, ..., 60=Real Estate)
/// - Level 2: 4-digit codes (first 2 = parent Level 1)
/// - Level 3: 6-digit codes (first 4 = parent Level 2)
/// - Level 4: 8-digit codes (first 6 = parent Level 3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryGics {
    #[serde(rename = "IndustryID")]
    pub industry_id: i32,

    /// GICS level: 1, 2, 3, or 4.
    #[serde(rename = "Level")]
    pub level: i32,

    /// GICS industry code (2/4/6/8 digits depending on level).
    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    /// Parent industry code. 0 for Level 1.
    #[serde(rename = "ParentCode")]
    pub parent_code: i32,

    /// Vietnamese industry name.
    #[serde(rename = "IndustryName")]
    pub industry_name: String,

    /// Relative path to SVG icon.
    #[serde(rename = "ImageURL")]
    pub image_url: String,
}

/// Heatmap data for an industry or sub-industry.
///
/// Returned by `POST /Data/GetHeatmapGICS`. Each item represents an industry
/// node with price change, market cap, and trading statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryHeatmapItem {
    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    #[serde(rename = "ParentCode")]
    pub parent_code: i32,

    #[serde(rename = "Level")]
    pub level: i32,

    #[serde(rename = "DisplayOrder")]
    pub display_order: i32,

    #[serde(rename = "IndustryName")]
    pub industry_name: String,

    /// Present only when `detailStockCode=true` and the item is a stock entry.
    #[serde(rename = "StockFullName", default)]
    pub stock_full_name: Option<String>,

    #[serde(rename = "ImageURL")]
    pub image_url: String,

    /// Trading date (date of the market data).
    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Basic/reference price (VND).
    #[serde(rename = "BasicPrice")]
    pub basic_price: f64,

    #[serde(rename = "CeilingPrice")]
    pub ceiling_price: f64,

    #[serde(rename = "FloorPrice")]
    pub floor_price: f64,

    /// Close price (VND).
    #[serde(rename = "ClosePrice")]
    pub close_price: f64,

    #[serde(rename = "CloseIndex")]
    pub close_index: i32,

    /// Absolute price change today.
    #[serde(rename = "Change")]
    pub change: f64,

    /// Percentage change today.
    #[serde(rename = "PerChange")]
    pub per_change: f64,

    /// Year-to-date percentage change.
    #[serde(rename = "PerChangeYTD")]
    pub per_change_ytd: f64,

    /// Number of stocks in this industry group.
    #[serde(rename = "QttyStockCode")]
    pub qtty_stock_code: i32,

    /// Market capitalization in billions VND.
    #[serde(rename = "MarketCapital")]
    pub market_capital: f64,

    /// Trading volume (thousands of shares).
    #[serde(rename = "TotalVol")]
    pub total_vol: f64,

    /// Trading value (billions VND).
    #[serde(rename = "TotalVal")]
    pub total_val: f64,
}

/// A single data point in an industry index time series.
///
/// Returned by `POST /Data/GetGICSIndex`. Each entry is one trading day's
/// index value for a specific industry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryIndexPoint {
    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    #[serde(rename = "IndustryName")]
    pub industry_name: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Index value at this date.
    #[serde(rename = "Value")]
    pub value: f64,

    #[serde(rename = "Year")]
    pub year: i32,

    #[serde(rename = "Quarter")]
    pub quarter: i32,
}

/// Performance data for an industry over a specific timeframe.
///
/// Returned by `POST /Data/GetGICSPerformance`. Each entry is one industry ×
/// one timeframe combination. The full response has 11 industries × 8
/// timeframes = 88 items (for Level 1).
///
/// **Type codes (timeframes):**
/// | Type | Meaning |
/// |------|---------|
/// | 1    | Today   |
/// | 2    | 1 Week  |
/// | 3    | 1 Month |
/// | 4    | 6 Months|
/// | 5    | YTD     |
/// | 6    | 1 Year  |
/// | 7    | 5 Years |
/// | 8    | 10 Years|
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryPerformance {
    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    #[serde(rename = "IndustryName")]
    pub industry_name: String,

    #[serde(rename = "TradingDate", deserialize_with = "deserialize_ms_date")]
    pub trading_date: DateTime<Utc>,

    /// Percentage change over this timeframe.
    #[serde(rename = "Change")]
    pub change: f64,

    /// Timeframe type code (1=Today, 2=1W, 3=1M, 4=6M, 5=YTD, 6=1Y, 7=5Y, 8=10Y).
    #[serde(rename = "Type")]
    pub perf_type: i32,
}

/// A financial index group definition (e.g., "Valuation ratios", "Profitability ratios").
///
/// Part of the response from `POST /Data/GetFinanceIndexGroupForSector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceIndexGroup {
    #[serde(rename = "FinanceIndexGroupID")]
    pub finance_index_group_id: i32,

    /// Vietnamese group name.
    #[serde(rename = "FinanceIndexName")]
    pub finance_index_name: String,

    /// English group name.
    #[serde(rename = "FinanceIndexNameEn")]
    pub finance_index_name_en: String,

    /// Display priority order.
    #[serde(rename = "DisplayOrder")]
    pub display_order: i32,
}

/// An individual financial index definition (e.g., "PE", "ROE", "EBIT").
///
/// Each index belongs to a [`FinanceIndexGroup`].
/// Part of the response from `POST /Data/GetFinanceIndexGroupForSector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceIndexDef {
    #[serde(rename = "FinanceIndexGroupID")]
    pub finance_index_group_id: i32,

    /// Vietnamese index name.
    #[serde(rename = "FinanceIndexName")]
    pub finance_index_name: String,

    /// English index name (may be empty).
    #[serde(rename = "FinanceIndexNameEn")]
    pub finance_index_name_en: String,

    /// Unique identifier for this index.
    #[serde(rename = "FinanceIndexID")]
    pub finance_index_id: i32,

    /// Short/abbreviated index name (e.g., "EBIT", "PE").
    #[serde(rename = "ShortIndexName")]
    pub short_index_name: String,

    /// Vietnamese unit label (e.g., "Triệu đồng", "Lần").
    #[serde(rename = "Unit")]
    pub unit: String,

    /// English unit label (e.g., "mVND").
    #[serde(rename = "UnitEn")]
    pub unit_en: String,

    #[serde(rename = "OrderDisplay")]
    pub order_display: i32,

    #[serde(rename = "BusinessTypeID")]
    pub business_type_id: i32,
}

/// Full response from `POST /Data/GetFinanceIndexGroupForSector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceIndexGroupResponse {
    /// Always `false` for successful responses.
    pub error: bool,

    /// List of financial index groups (10 groups for all industries).
    pub group: Vec<FinanceIndexGroup>,

    /// List of individual financial index definitions.
    pub index: Vec<FinanceIndexDef>,

    /// Template metadata (structure varies).
    #[serde(default)]
    pub templates: serde_json::Value,
}

/// A financial index value for a specific industry, year, and reporting period.
///
/// Returned by `POST /Data/GetGICSFinancialIndex`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryFinancialIndex {
    /// Links to a `FinanceIndexDef.finance_index_id`.
    #[serde(rename = "FinanceGICSIndexID")]
    pub finance_gics_index_id: i32,

    #[serde(rename = "IndustryCode")]
    pub industry_code: i32,

    /// Fiscal year (e.g., 2026).
    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// 1=Q1, 2=Q2, 3=Q3, 4=Q4, 5=Full Year.
    #[serde(rename = "ReportTermID")]
    pub report_term_id: i32,

    /// The numerical value (unit defined by the parent `FinanceIndexDef`).
    #[serde(rename = "Value")]
    pub value: f64,

    /// Vietnamese unit label.
    #[serde(rename = "Unit")]
    pub unit: String,
}

/// An industry/sector analysis report from a brokerage or research firm.
///
/// Returned by `POST /Data/GetReportSector`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorReport {
    #[serde(rename = "PublishDate", deserialize_with = "deserialize_ms_date")]
    pub publish_date: DateTime<Utc>,

    /// Report title (Vietnamese).
    #[serde(rename = "Title")]
    pub title: String,

    /// Source/brokerage name (e.g., "Vietcap", "SSI Research").
    #[serde(rename = "SourceName")]
    pub source_name: String,

    #[serde(rename = "SourceID")]
    pub source_id: i32,

    /// Full report content (HTML).
    #[serde(rename = "Content")]
    pub content: String,

    /// Report type code.
    #[serde(rename = "Type")]
    pub report_type: i32,

    #[serde(rename = "ReportID")]
    pub report_id: i32,

    /// Industry name (may be `null` for generic/multi-sector reports).
    #[serde(rename = "IndustryName", default)]
    pub industry_name: Option<String>,
}

/// An industry news article.
///
/// Returned by `POST /Data/GetIndustryNews` with page/pageSize pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryNewsItem {
    #[serde(rename = "ArticleID")]
    pub article_id: i64,

    #[serde(rename = "PublishTime", deserialize_with = "deserialize_ms_date")]
    pub publish_time: DateTime<Utc>,

    /// Human-readable relative time (e.g., "5 giờ trước").
    #[serde(rename = "PublishTimeConvert")]
    pub publish_time_convert: String,

    /// Article title.
    #[serde(rename = "Title")]
    pub title: String,

    /// Short excerpt / headline summary.
    #[serde(rename = "Head")]
    pub head: String,

    /// Thumbnail image URL. May be `null` for text-only articles.
    #[serde(rename = "ImageURL", default)]
    pub image_url: Option<String>,

    /// Full article URL (relative path on vietstock.vn).
    #[serde(rename = "ArticleURL")]
    pub article_url: String,

    /// Industry name if filtered; `null` for "All industries" view.
    #[serde(rename = "IndustryName", default)]
    pub industry_name: Option<String>,
}

// ===========================================================================
// Financial Statement Tab Types (BCTT, CDKT, KQKD, LCTT, TM, CSTC, CTKH)
// ===========================================================================

/// A single report row / norm definition.
///
/// Returned by the `GetListReportNorm_{TAB}_ByStockCode` family of endpoints.
/// Each entry defines one line item in a financial statement report.
///
/// **Exception:** CDKT uses `GetListReportNormByStockCode` (no tab prefix)
/// and has a different field set (with `ParentReportNormID`, `CalculatedOrdering`,
/// and no `Unit`/`ReportTypeName`/`ReportTypeCode`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportNorm {
    #[serde(rename = "ReportNormId")]
    pub report_norm_id: i32,

    /// Vietnamese name (e.g., "Thu nhập lãi thuần", "A. TÀI SẢN").
    #[serde(rename = "ReportNormName")]
    pub report_norm_name: String,

    /// Visual style hint: "Normal", "MaxB" (section header), "NormalB" (subsection).
    #[serde(rename = "CssStyle")]
    pub css_style: String,

    /// Parent node ID for hierarchical norms (CDKT). `None` for BCTT/CSTC.
    #[serde(rename = "ParentReportNormID", default)]
    pub parent_report_norm_id: Option<i32>,

    /// Nesting/ordering hint for hierarchical norms (CDKT). `None` for BCTT/CSTC.
    #[serde(rename = "CalculatedOrdering", default)]
    pub calculated_ordering: Option<i32>,

    /// Display unit label (e.g., empty, "Tỷ đồng"). Present in BCTT norms only.
    #[serde(rename = "Unit", default)]
    pub unit: Option<String>,

    /// Parent report type name (e.g., "Kết quả kinh doanh", "Cân đối kế toán").
    /// Present in BCTT norms — which aggregate rows from multiple report types.
    #[serde(rename = "ReportTypeName", default)]
    pub report_type_name: Option<String>,

    /// Report type code: "KQ" = Income Statement, "CD" = Balance Sheet.
    /// Present in BCTT norms.
    #[serde(rename = "ReportTypeCode", default)]
    pub report_type_code: Option<String>,
}

/// A single period / column in the financial data table.
///
/// Returned by the `{TAB}_GetListReportData` family of endpoints.
/// Describes one reporting column (e.g., Q1/2026 Hợp nhất Chưa kiểm toán).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    #[serde(rename = "RowNumber")]
    pub row_number: i32,

    /// The ID used in `GetReportDataDetailValue_*` batch requests.
    #[serde(rename = "ReportDataID")]
    pub report_data_id: i32,

    /// Fiscal year (e.g., 2026).
    #[serde(rename = "YearPeriod")]
    pub year_period: i32,

    /// 2=Q1, 3=Q2, 4=Q3, 5=Q4.
    #[serde(rename = "ReportTermID")]
    pub report_term_id: i32,

    /// Auditor opinion (Vietnamese). Often empty.
    #[serde(rename = "YKienKiemToan", default)]
    pub y_kien_kiem_toan: String,

    /// Audit firm name. Often empty.
    #[serde(rename = "CtyKiemToan", default)]
    pub cty_kiem_toan: String,

    /// 0 = Consolidated (Hợp nhất), 1 = Parent only (Công ty mẹ).
    #[serde(rename = "IsUnited")]
    pub is_united: i32,

    /// "Hợp nhất" or "Công ty mẹ".
    #[serde(rename = "UnitedName")]
    pub united_name: String,

    /// 4 = Unaudited (Chưa kiểm toán), 10 = Audited (Kiểm toán).
    #[serde(rename = "AuditStatusID")]
    pub audit_status_id: i32,

    /// "Chưa kiểm toán" or "Kiểm toán".
    #[serde(rename = "AuditStatusName")]
    pub audit_status_name: String,

    /// Period start in YYYYMM format (e.g., "200901").
    #[serde(rename = "PeriodBegin")]
    pub period_begin: String,

    /// Period end in YYYYMM format (e.g., "200903").
    #[serde(rename = "PeriodEnd")]
    pub period_end: String,

    /// Base period begin for comparison (YYYYMM).
    #[serde(rename = "BasePeriodBegin", default)]
    pub base_period_begin: String,

    /// Base period end for comparison (YYYYMM).
    #[serde(rename = "BasePeriodEnd", default)]
    pub base_period_end: String,

    /// Whether data display is permitted for this period.
    #[serde(rename = "IsShowData_Permission")]
    pub is_show_data_permission: bool,

    /// PDF download link for the financial report. `None` for older periods.
    #[serde(rename = "Url", default)]
    pub url: Option<String>,
}

/// A value row — maps a `ReportNormId` to values for up to 9 periods/columns.
///
/// Returned by `GetReportDataDetailValue_{TAB}_ByReportDataIds`.
/// Values correspond positionally to the `listReportDataIds` array sent in the request.
/// Values are in the unit specified (e.g., 1000000000 = Tỷ đồng).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportValueRow {
    #[serde(rename = "ReportNormId")]
    pub report_norm_id: i32,

    /// "CD" (Balance Sheet) or "KQ" (Income Statement). Present in BCTT values.
    #[serde(rename = "ReportTypeCode", default)]
    pub report_type_code: Option<String>,

    #[serde(rename = "Value1", default)]
    pub value1: Option<f64>,
    #[serde(rename = "Value2", default)]
    pub value2: Option<f64>,
    #[serde(rename = "Value3", default)]
    pub value3: Option<f64>,
    #[serde(rename = "Value4", default)]
    pub value4: Option<f64>,
    #[serde(rename = "Value5", default)]
    pub value5: Option<f64>,
    #[serde(rename = "Value6", default)]
    pub value6: Option<f64>,
    #[serde(rename = "Value7", default)]
    pub value7: Option<f64>,
    #[serde(rename = "Value8", default)]
    pub value8: Option<f64>,
    #[serde(rename = "Value9", default)]
    pub value9: Option<f64>,
}

/// Generic wrapper for list-style API responses that use `{"data": [...]}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDataWrapper<T> {
    pub data: T,
}

/// Supported financial statement tabs.
///
/// These correspond to the `?tab=` query parameter on the Vietstock financials page,
/// and map to the API endpoint prefix used in all three request stages (norms, periods, values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinancialTab {
    /// BCTC Tóm tắt — Summary financials (aggregates CDKT + KQKD rows)
    Bctt,
    /// Cân đối kế toán — Balance Sheet
    Cdkt,
    /// Kết quả kinh doanh — Income Statement
    Kqkd,
    /// Lưu chuyển tiền tệ — Cash Flow
    Lctt,
    /// Thuyết minh — Notes to Financial Statements \[INFERRED\]
    Tm,
    /// Chỉ số tài chính — Financial Ratios
    Cstc,
    /// Chỉ tiêu kế hoạch — Planned Targets \[INFERRED\]
    Ctkh,
}

impl FinancialTab {
    /// Get the URL prefix for this tab's endpoints.
    ///
    /// Used to construct endpoint paths like:
    /// - `GetListReportNorm_{prefix}_ByStockCode`
    /// - `{prefix}_GetListReportData`
    /// - `GetReportDataDetailValue_{prefix}_ByReportDataIds`
    pub fn prefix(&self) -> &str {
        match self {
            FinancialTab::Bctt => "BCTT",
            FinancialTab::Cdkt => "CDKT",
            FinancialTab::Kqkd => "KQKD",
            FinancialTab::Lctt => "LCTT",
            FinancialTab::Tm => "TM",
            FinancialTab::Cstc => "CSTC",
            FinancialTab::Ctkh => "CTKH",
        }
    }
}

/// Complete financial statement for one tab (norms + periods + values).
pub type FinancialStatement = (Vec<ReportNorm>, Vec<ReportPeriod>, Vec<ReportValueRow>);

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // ---- DocumentType ----

    #[test]
    fn test_deserialize_document_type_array() {
        let json = r#"[
            {"DocumentTypeID":1,"DocumentName":"Báo cáo tài chính","DocumentNameEn":"Financial Statement","DisplayOrder":1},
            {"DocumentTypeID":23,"DocumentName":"Nghị quyết HĐQT","DocumentNameEn":"Board of Management","DisplayOrder":1},
            {"DocumentTypeID":2,"DocumentName":"Báo cáo thường niên","DocumentNameEn":"Annual Report","DisplayOrder":5}
        ]"#;
        let types: Vec<DocumentType> = serde_json::from_str(json).unwrap();
        assert_eq!(types.len(), 3);
        assert_eq!(types[0].document_type_id, 1);
        assert_eq!(types[0].document_name, "Báo cáo tài chính");
        assert_eq!(types[0].document_name_en, "Financial Statement");
    }

    // ---- Document ----

    #[test]
    fn test_deserialize_document_zip() {
        let json = r#"{
            "FileExt": ".zip ",
            "UpdateTime": null,
            "TotalRow": 223,
            "FileInfoID": 548794,
            "Url": "https://static2.vietstock.vn/data/UPCOM/2026/TAI LIEU DHCD/VN/test.zip",
            "Title": "Tài liệu đại hội đồng cổ đông thường niên năm 2026 ",
            "FullName": "Tài liệu đại hội đồng cổ đông thường niên năm 2026 ",
            "LastUpdate": "/Date(1780368384730)/"
        }"#;
        let doc: Document = serde_json::from_str(json).unwrap();
        assert_eq!(doc.file_ext, ".zip");
        assert!(doc.update_time.is_none());
        assert_eq!(doc.total_row, 223);
        assert_eq!(doc.file_info_id, 548794);
        assert!(doc.is_zip());
        assert!(!doc.is_pdf());
    }

    #[test]
    fn test_document_exchange_and_year() {
        let doc = Document {
            file_ext: ".pdf".into(),
            update_time: None,
            total_row: 1,
            file_info_id: 1,
            url: "https://static2.vietstock.vn/data/HOSE/2026/KHAC/VN/file.pdf".into(),
            title: "Test".into(),
            full_name: "Test".into(),
            last_update: chrono::Utc::now(),
        };
        assert_eq!(doc.exchange(), Some("HOSE"));
        assert_eq!(doc.year(), Some(2026));
    }

    // ---- TradingInfo ----

    #[test]
    fn test_deserialize_trading_info_vcb() {
        let json = r#"{
            "StockCode": "VCB",
            "TradingDate": "/Date(1781596720150)/",
            "KLCPLH": 8355675094,
            "KLCPNY": 8355675094,
            "PriorClosePrice": 61600,
            "CeilingPrice": 65900,
            "FloorPrice": 57300,
            "TotalVol": 3110000,
            "TotalVal": 192490240000,
            "MarketCapital": 516380720809200,
            "HighestPrice": 62300,
            "LowestPrice": 61600,
            "OpenPrice": 61800,
            "LastPrice": 61800,
            "AvrPrice": 61891,
            "Change": 200,
            "PerChange": 0.32,
            "Min52W": 56000,
            "Max52W": 76000,
            "EPS": 4301.09,
            "PE": 14.32,
            "BVPS": 28008.67,
            "PB": 2.20
        }"#;
        let info: TradingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.stock_code, "VCB");
        assert_eq!(info.last_price, 61800);
        assert!((info.eps - 4301.09).abs() < 0.01);
        assert!((info.pe - 14.32).abs() < 0.01);
    }

    #[test]
    fn test_deserialize_trading_info_extended() {
        let json = r#"{
            "StockCode": "VCB",
            "TradingDate": "/Date(1781596720150)/",
            "KLCPLH": 8355675094, "KLCPNY": 8355675094,
            "PriorClosePrice": 61600, "CeilingPrice": 65900, "FloorPrice": 57300,
            "TotalVol": 3110000, "TotalVal": 192490240000,
            "MarketCapital": 516380720809200,
            "HighestPrice": 62300, "LowestPrice": 61600,
            "OpenPrice": 61800, "LastPrice": 61800, "AvrPrice": 61891,
            "Change": 200, "PerChange": 0.32,
            "Min52W": 56000, "Max52W": 76000, "Vol52W": 7652644,
            "OutstandingBuy": 919600, "OutstandingSell": 206700,
            "OwnedRatio": 20.29, "Dividend": 450, "Yield": 0.01, "Beta": 0.80,
            "EPS": 4301.09, "PE": 14.32, "FEPS": 16.78, "BVPS": 28008.67,
            "PB": 2.20, "PS": 22.77,
            "TotalRoom": 2506702528, "CurrRoom": 811732555, "RemainRoom": 9.71,
            "F_BuyVol": 461900, "F_BuyVal": 28605660000,
            "F_SellVol": 1175800, "F_SellVal": 72743990000,
            "F_BuyPutVol": 0, "F_BuyPutVal": 0,
            "F_SellPutVol": 0, "F_SellPutVal": 0,
            "MarketStatus": 0, "ColorId": 1,
            "StatusName": "Kết thúc phiên", "StockStatus": "",
            "Revenue": 105216484000000.0, "Profit": 35177663000000.0
        }"#;
        let info: TradingInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.vol_52w, 7652644);
        assert!((info.owned_ratio - 20.29).abs() < 0.01);
        assert_eq!(info.dividend, 450);
        assert!((info.beta - 0.80).abs() < 0.01);
        assert!((info.feps - 16.78).abs() < 0.01);
        assert!((info.ps - 22.77).abs() < 0.01);
        assert_eq!(info.status_name, "Kết thúc phiên");
        assert!((info.revenue - 105216484000000.0).abs() < 1.0);
    }

    // ---- Auth ----

    #[test]
    fn test_deserialize_auth_token_response() {
        let r: AuthTokenResponse = serde_json::from_str(r#"{"IsSuccess":true}"#).unwrap();
        assert!(r.is_success);
    }

    // ---- Performance ----

    #[test]
    fn test_deserialize_performance() {
        let json = r#"{"data":[{"StockCode":"VCB","TradingDate":"/Date(1781542800000)/","ClosePrice":61800.0,"PerChange":0.32,"ClosePrice1W":61700.0,"PerChange1W":0.16,"ClosePrice1M":60700.0,"PerChange1M":1.81,"ClosePrice6M":56800.0,"PerChange6M":8.80,"ClosePriceYTD":57500.0,"PerChangeYTD":7.48,"ClosePrice1Y":56235.44,"PerChange1Y":9.90,"ClosePrice5Y":44786.77,"PerChange5Y":37.99,"ClosePriceAll":9150.90,"PerChangeAll":575.34}]}"#;
        let r: PerformanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.len(), 1);
        assert!((r.data[0].close_price - 61800.0).abs() < 0.01);
        assert!(r.data[0].trading_date.year() == 2026);
    }

    // ---- FinanceInfo (Financial Statements) ----

    #[test]
    fn test_deserialize_period_metadata() {
        let json = r#"[{"ID":4,"Row":4,"CompanyID":1167,"YearPeriod":2025,"TermCode":"Q2","TermName":"Quý 2","TermNameEN":"Quarter 2","ReportTermID":3,"DisplayOrdering":25,"United":"HN","AuditedStatus":"CKT","PeriodBegin":"202504","PeriodEnd":"202506","TotalRow":69,"BusinessType":3,"ReportNote":null,"ReportNoteEn":null}]"#;
        let periods: Vec<PeriodMetadata> = serde_json::from_str(json).unwrap();
        assert_eq!(periods.len(), 1);
        let p = &periods[0];
        assert_eq!(p.id, 4);
        assert_eq!(p.year_period, 2025);
        assert_eq!(p.term_code, "Q2");
        assert_eq!(p.report_term_id, 3);
        assert_eq!(p.audited_status, "CKT");
        assert_eq!(p.period_begin, "202504");
        assert_eq!(p.period_end, "202506");
        assert_eq!(p.total_row, 69);
    }

    #[test]
    fn test_deserialize_finance_info_line_item() {
        let json = r#"{
            "ID":1,"ReportNormID":4385,
            "Name":"Thu nhập lãi thuần","NameEn":"I. Net interest income (1-2)",
            "NameMobile":"Thu nhập lãi thuần","NameMobileEn":"NII",
            "CssStyle":"Normal","Padding":"Padding1",
            "ParentReportNormID":4376,
            "ReportComponentName":"Kết quả kinh doanh","ReportComponentNameEn":"Income Statement",
            "Unit":"","UnitEn":"",
            "OrderType":1,"OrderingComponent":1,"RowNumber":10,
            "ReportComponentTypeID":null,"ChildTotal":0,"Levels":0,
            "Value1":17651.00,"Value2":16170.00,"Value3":14657.00,"Value4":14160.00,
            "Vl":null,"IsShowData":true
        }"#;
        let item: FinanceInfoLineItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, 1);
        assert_eq!(item.report_norm_id, 4385);
        assert_eq!(item.name, "Thu nhập lãi thuần");
        assert_eq!(item.name_en, "I. Net interest income (1-2)");
        assert_eq!(item.report_component_name, "Kết quả kinh doanh");
        assert!((item.value1.unwrap() - 17651.0).abs() < 0.01);
        assert!((item.value2.unwrap() - 16170.0).abs() < 0.01);
        assert!(item.vl.is_none());
        assert!(item.is_show_data);
        assert!(item.report_component_type_id.is_none());
    }

    #[test]
    fn test_deserialize_finance_info_response() {
        let json = r#"[
            [{"ID":4,"Row":4,"CompanyID":1167,"YearPeriod":2025,"TermCode":"Q2","TermName":"Quý 2","TermNameEN":"Quarter 2","ReportTermID":3,"DisplayOrdering":25,"United":"HN","AuditedStatus":"CKT","PeriodBegin":"202504","PeriodEnd":"202506","TotalRow":69,"BusinessType":3,"ReportNote":null,"ReportNoteEn":null}],
            {"Kết quả kinh doanh":[{"ID":1,"ReportNormID":4385,"Name":"Thu nhập lãi thuần","NameEn":"NII","NameMobile":"NII","NameMobileEn":"NII","CssStyle":"Normal","Padding":"Padding1","ParentReportNormID":4376,"ReportComponentName":"Kết quả kinh doanh","ReportComponentNameEn":"Income Statement","Unit":"","UnitEn":"","OrderType":1,"OrderingComponent":1,"RowNumber":10,"ReportComponentTypeID":null,"ChildTotal":0,"Levels":0,"Value1":17651.0,"Value2":16170.0,"Value3":14657.0,"Value4":14160.0,"Vl":null,"IsShowData":true}]},
            [],
            []
        ]"#;
        let (periods, components, _audit, _united): FinanceInfoResponse =
            serde_json::from_str(json).unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].year_period, 2025);
        assert!(components.contains_key("Kết quả kinh doanh"));
        let items = &components["Kết quả kinh doanh"];
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Thu nhập lãi thuần");
    }

    // ---- Trading Overview ----

    #[test]
    fn test_deserialize_trading_overview() {
        let json = r#"{"data":[{"StockCode":"VCB","TradingDate":"/Date(1781542800000)/","ClosePrice":61800,"ColorID":1,"Change":200,"PerChange":0.32,"TotalVol":3110000,"TotalVal":192490240000,"TotalBuyTrade":4547,"TotalSellTrade":4238,"BuyAvgTrade":1445.60,"SellAvgTrade":1589.08,"ForeignBuyVal":28605660000,"ForeignSellVal":72743990000,"ProBuyVal":6764350000,"ProSellVal":890797000}]}"#;
        let r: TradingOverviewResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.len(), 1);
        let d = &r.data[0];
        assert_eq!(d.stock_code, "VCB");
        assert_eq!(d.close_price, 61800);
        assert_eq!(d.color_id, 1);
        assert_eq!(d.change, 200);
        assert!((d.per_change - 0.32).abs() < 0.001);
        assert_eq!(d.total_vol, 3110000);
        assert_eq!(d.total_buy_trade, 4547);
    }

    // ---- News ----

    #[test]
    fn test_deserialize_news_item() {
        let json = r#"{"StockCode":"VCB","ChannelID":618,"Head":"Tính đến đầu tháng 6...","ArticleID":1453078,"Title":"Lãi suất tiền gửi đầu tháng 6","PublishTime":"/Date(1781239008260)/","Content":"","URL":"/2026/06/lai-suat...","TotalRow":2528,"Source":"FILI","Icon":null}"#;
        let item: NewsItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.stock_code, "VCB");
        assert_eq!(item.channel_id, 618);
        assert_eq!(item.article_id, 1453078);
        assert_eq!(item.source, "FILI");
        assert!(item.icon.is_none());
    }

    // ---- Corporate Events ----

    #[test]
    fn test_deserialize_corporate_event() {
        let json = r#"[{"EventID":209242,"EventTypeID":1,"ChannelID":13,"Code":"VCB","CompanyName":"Ngân hàng TMCP Ngoại thương Việt Nam","CatID":1,"GDKHQDate":"/Date(1759424400000)/","NDKCCDate":"/Date(1759683600000)/","Time":"/Date(1761238800000)/","Note":"Trả cổ tức năm 2024 bằng tiền, 450 đồng/CP","Name":"Trả cổ tức bằng tiền mặt","Exchange":"HOSE","Title":"VCB: Thông báo...","Content":"<p>Sở Giao dịch...</p>","FileUrl":"https://static2.vietstock.vn/test","DateOrder":"/Date(1759424400000)/","Row":1}]"#;
        let events: Vec<CorporateEvent> = serde_json::from_str(json).unwrap();
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e.event_id, 209242);
        assert_eq!(e.code, "VCB");
        assert_eq!(e.exchange, "HOSE");
        assert_eq!(e.name, "Trả cổ tức bằng tiền mặt");
    }

    // ---- Analysis Reports ----

    #[test]
    fn test_deserialize_analysis_report() {
        let json = r#"{"ReportID":20653,"Title":"VCB: Khuyến nghị MUA","PublishDate":"/Date(1774828800000)/","TypeName":"Phân tích Doanh nghiệp","ReportTypeID":58,"Url":"/bao-cao-phan-tich/20653/...","TotalRow":856,"StockCode":"VCB"}"#;
        let report: AnalysisReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.report_id, 20653);
        assert_eq!(report.stock_code, "VCB");
        assert_eq!(report.total_row, 856);
    }

    // ---- Technical Rating ----

    #[test]
    fn test_deserialize_technical_rating() {
        let json = r#"{"success":true,"data":[{"StockCode":"VCB","TradingDate":"/Date(1781542800000)/","SMA10":61690.0,"FSMA10":1.0,"EMA10":61799.1796,"FEMA10":1.0,"SMA20":62540.0,"FSMA20":-1.0,"EMA20":61897.3497,"FEMA20":-1.0,"SMA30":62063.3333,"FSMA30":-1.0,"EMA30":61791.375,"FEMA30":1.0,"SMA50":61100.0,"FSMA50":1.0,"EMA50":61642.867,"FEMA50":1.0,"SMA100":62628.0,"FSMA100":-1.0,"EMA100":61729.3846,"FEMA100":1.0,"SMA200":62017.7,"FSMA200":-1.0,"EMA200":62933.8543,"FEMA200":-1.0,"FinalRatingSMA":0.0,"HMA9":61642.963,"FinalRatingHMA":1.0,"VWMA20":63344.0755,"FinalRatingVWMA":-1.0,"IchimokuBaseLine":62700.0,"FinalRatingIchimoku":0.0,"MovingAverages":0.0,"RSI":49.7715,"FinalRatingRSI":0.0,"MACD":7.0672,"FinalRatingMACD":-1.0,"OS":21.9048,"FinalRatingOS":0.0,"WilliamsR":-72.7273,"FinalRatingR":0.0,"ADX":33.0937,"FinalRatingADX":0.0,"Momentum10":99.8384,"FinalRatingMomentum":0.0,"UO":32.1785,"FinalRatingUO":0.0,"CCI20":4442.6708,"FinalRatingCCI":0.0,"StochRSI":18.2707,"FinalRatingStochRSI":0.0,"AO":-15.0,"FinalRatingAO":-1.0,"BullBearPower":168.042,"FinalRatingBBP":0.0,"Oscillators":-0.181818,"FinalRatings":-9.0909}]}"#;
        let r: TechnicalRatingResponse = serde_json::from_str(json).unwrap();
        assert!(r.success);
        assert_eq!(r.data.len(), 1);
        let t = &r.data[0];
        assert_eq!(t.stock_code, "VCB");
        assert!((t.sma10 - 61690.0).abs() < 0.01);
        assert!((t.fsma10 - 1.0).abs() < 0.001);
        assert!((t.rsi - 49.7715).abs() < 0.0001);
        assert!((t.macd - 7.0672).abs() < 0.0001);
        assert!((t.final_ratings - (-9.0909)).abs() < 0.0001);
    }

    // ---- Fundamental Rating ----

    #[test]
    fn test_deserialize_fundamental_rating() {
        let json = r#"{"success":true,"data":[[{"GroupIndexId":3,"GroupIndex":"Quản trị","AvgRating":4.33,"FundaRating":8.66,"AvgRatingInd":3.02,"FundaRatingInd":6.04}],[{"StockCode":"VCB","StockName":"Ngân hàng TMCP Ngoại thương Việt Nam","BusinessTypeID":3,"IndustryCode":4010,"IndustryName":"Tổ chức tín dụng","YearPeriod":2024,"VSRating":8.28,"Rating":"A"}]]}"#;
        let r: FundamentalRatingResponse = serde_json::from_str(json).unwrap();
        assert!(r.success);
        let (factors, summaries) = &r.data;
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].group_index, "Quản trị");
        assert!((factors[0].avg_rating - 4.33).abs() < 0.01);
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].stock_code, "VCB");
        assert_eq!(summaries[0].rating, "A");
        assert!((summaries[0].vs_rating - 8.28).abs() < 0.01);
    }

    // ---- Bonds ----

    #[test]
    fn test_deserialize_bond_item() {
        let json = r#"{"RelatedBondInfos":[{"KeyCode":"VCB12401","StockCode":"VCB12401","BondCode":"VCBL2426001","ReleaseDate":"/Date(1731517200000)/","DueDate":"/Date(1794589200000)/","RemainingMaturity":151,"FaceValue":1000000000,"IssueRate":4.9,"IssuaVolume":2000,"IssueValue":2000000000000,"OutstandingSharesValue":2000000000000,"OutstandingShares":2000,"CompanyCode":null,"CompanyName":null,"CompanyURL":null,"InterestRateType":"Cố định","InterestPeriod":"12 Tháng","TotalRecord":76}]}"#;
        let r: BondRelatedResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.related_bond_infos.len(), 1);
        let b = &r.related_bond_infos[0];
        assert_eq!(b.key_code, "VCB12401");
        assert_eq!(b.bond_code, "VCBL2426001");
        assert!((b.issue_rate - 4.9).abs() < 0.01);
        assert_eq!(b.remaining_maturity, 151);
        assert_eq!(b.interest_rate_type, "Cố định");
    }

    // ---- Planned Targets ----

    #[test]
    fn test_deserialize_planned_target() {
        let json = r#"{"data":{"Data_Results":[{"StockCode":"VCB","PTID":8,"PTName":"Lợi nhuận trước thuế kế hoạch","YearPeriod":2023,"Value":42973304650000.0}],"Year_Results":[{"Year":2022},{"Year":2023},{"Year":2024}]}}"#;
        let r: PlannedTargetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.data_results.len(), 1);
        assert_eq!(r.data.data_results[0].pt_id, 8);
        assert_eq!(r.data.data_results[0].year_period, 2023);
        assert_eq!(r.data.year_results.len(), 3);
        assert_eq!(r.data.year_results[0].year, 2022);
    }

    // ---- Target Implementation ----

    #[test]
    fn test_deserialize_target_implementation() {
        let json = r#"{"data":{"Data_Results":[{"StockCode":"VCB","YearPeriod":2026,"ReportTermID":2,"ReportNormID":4377,"Value":11802653000000.0,"PTID":8}]}}"#;
        let r: TargetImplementationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.data_results.len(), 1);
        let imp = &r.data.data_results[0];
        assert_eq!(imp.stock_code, "VCB");
        assert_eq!(imp.year_period, 2026);
        assert_eq!(imp.report_term_id, 2);
        assert!((imp.value - 11802653000000.0).abs() < 1.0);
    }

    // ---- Reference Price ----

    #[test]
    fn test_deserialize_ref_price() {
        let json = r#"{"dataRefPrice":{"StockCode":"VCB","TradingDate":"/Date(1781542800000)/","RefPrice":61.6,"MarketStatus":1},"stockColor":[{"TradingDate":"/Date(1781542800000)/","LastPrice":61800,"Change":200,"PerChange":0.32,"TotalVol":3110000,"BuyAvg":1445.60,"SellAvg":1589.08,"ForeignBuyVol":461900,"ForeignSellVol":1175800,"ColorId":1}]}"#;
        let r: RefPriceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data_ref_price.stock_code, "VCB");
        assert!((r.data_ref_price.ref_price - 61.6).abs() < 0.01);
        assert_eq!(r.stock_color.len(), 1);
        assert_eq!(r.stock_color[0].last_price, 61800);
    }

    // ---- Market Prices ----

    #[test]
    fn test_deserialize_market_price() {
        let json = r##"{"Code":"VNIndex","Name":"VN-Index","Price":1807.94,"Change":8.63,"PerChange":0.48,"Color":"#39ca32","TradingDate":"/Date(1781542800000)/"}"##;
        let mp: MarketPrice = serde_json::from_str(json).unwrap();
        assert_eq!(mp.code, "VNIndex");
        assert!((mp.price - 1807.94).abs() < 0.01);
        assert!((mp.change - 8.63).abs() < 0.01);
        assert_eq!(mp.color, "#39ca32");
    }

    // ---- Header News ----

    #[test]
    fn test_deserialize_header_news() {
        let json = r#"{"Title":"Từ ngày 16/6...","URL":"/2026/06/tu-ngay-166...","PublishTime":"/Date(1781622120000)/","Source":"FILI","Icon":null,"ArticleID":1455123}"#;
        let item: HeaderNewsItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.source, "FILI");
        assert_eq!(item.article_id, 1455123);
        assert!(item.icon.is_none());
    }

    // ---- Event Calendar Item ----

    #[test]
    fn test_deserialize_event_calendar_item() {
        let json = r#"{"EventID":175900,"ChannelID":21,"Code":"AGP","CompanyName":"CTCP Dược phẩm Agimexpharm","CatID":5,"GDKHQDate":null,"NDKCCDate":null,"Time":"/Date(1784048400000)/","Note":"Giao dịch bổ sung - 359,386 CP","EventTypeID":2,"Name":"Niêm yết và giao dịch bổ sung","Exchange":"UPCoM","Source":"","Title":"AGP: Ngày 18/09/2023, Thông báo ngày giao dịch đầu tiên đối với cổ phiếu thay đổi đăng ký giao dịch","Content":"","FileUrl":"https://static2.vietstock.vn/vietstock/2023/9/13/1_agp_2023_9_13_6c8c763_3044tb_0001.pdf","DateOrder":"/Date(1784048400000)/","Volume":359386,"Price":null,"Reason":"","ReasonCBKS":"","DateNYTL":null,"DateHCB":null,"DateGDTL":null,"DateCTGD":"/Date(1784048400000)/","CancelVolume":null,"TimeDPGD":"","Row":1}"#;
        let events: Vec<EventCalendarItem> = serde_json::from_str(&format!("[{}]", json)).unwrap();
        assert_eq!(events.len(), 1);
        let e = &events[0];
        assert_eq!(e.event_id, 175900);
        assert_eq!(e.channel_id, 21);
        assert_eq!(e.code, "AGP");
        assert_eq!(e.company_name, "CTCP Dược phẩm Agimexpharm");
        assert_eq!(e.cat_id, 5);
        assert!(e.gdkhq_date.is_none());
        assert!(e.ndkcc_date.is_none());
        assert!(e.time.is_some());
        assert_eq!(e.note, "Giao dịch bổ sung - 359,386 CP");
        assert_eq!(e.event_type_id, 2);
        assert_eq!(e.name, "Niêm yết và giao dịch bổ sung");
        assert_eq!(e.exchange, "UPCoM");
        assert_eq!(e.volume, Some(359386));
        assert!(e.price.is_none());
        assert!(e.date_ctgd.is_some());
        assert_eq!(e.row, 1);
    }

    // ---- Event Type Lookup ----

    #[test]
    fn test_deserialize_event_type_lookup() {
        let json = r#"[[{"EventTypeID":2,"Name":"Niêm yết","NameEn":"Listing"},{"EventTypeID":1,"Name":"Cổ tức, thưởng và phát hành thêm","NameEn":"Dividend, Bonus and Additional issue"},{"EventTypeID":5,"Name":"Đại hội cổ đông","NameEn":"Shareholders meeting"}],[{"ChannelID":17,"Name":"Niêm yết và giao dịch lần đầu","NameEn":"New listing"},{"ChannelID":56,"Name":"Đình chỉ giao dịch","NameEn":null},{"ChannelID":21,"Name":"Niêm yết và giao dịch bổ sung","NameEn":"Additional listing"},{"ChannelID":19,"Name":"Cảnh báo","NameEn":"Alert"},{"ChannelID":22,"Name":"Kiểm soát","NameEn":"Supervision"},{"ChannelID":20,"Name":"Tạm ngừng giao dịch","NameEn":"Temporary trading suspension"},{"ChannelID":18,"Name":"Hủy niêm yết","NameEn":"Delisting/Move listing"},{"ChannelID":50,"Name":"Hạn chế giao dịch","NameEn":"Under trading restriction"},{"ChannelID":51,"Name":"Cảnh báo và bị kiểm soát","NameEn":"Alert and under supervision"},{"ChannelID":52,"Name":"Cảnh báo và hạn chế giao dịch","NameEn":"Alert and under trading restriction"},{"ChannelID":53,"Name":"Kiểm soát và hạn chế giao dịch","NameEn":"Supervision and under trading restriction"},{"ChannelID":54,"Name":"Cảnh báo, kiểm soát và hạn chế giao dịch","NameEn":"Alert,Supervision and under trading restriction"}]]"#;
        let (event_types, channel_types): EventTypeLookup = serde_json::from_str(json).unwrap();
        assert_eq!(event_types.len(), 3);
        assert_eq!(event_types[0].event_type_id, 2);
        assert_eq!(event_types[0].name, "Niêm yết");
        assert_eq!(event_types[0].name_en.as_deref(), Some("Listing"));
        assert_eq!(event_types[2].event_type_id, 5);

        assert_eq!(channel_types.len(), 12);
        assert_eq!(channel_types[0].channel_id, 17);
        assert_eq!(channel_types[0].name, "Niêm yết và giao dịch lần đầu");
        // ChannelID 56 has null NameEn
        assert_eq!(channel_types[1].channel_id, 56);
        assert!(channel_types[1].name_en.is_none());
    }

    // ---- Insider Trading Record ----

    #[test]
    fn test_deserialize_insider_trading_record() {
        let json = r#"[{"EventID":220989,"StockCode":"VNM","FinanceURL":"https://finance.vietstock.vn/VNM-ctcp-sua-viet-nam.htm","Content":"<p>SCIC thông báo giao dịch cổ phiếu...</p>","Title":"VNM: Thông báo giao dịch cổ phiếu của tổ chức có liên quan đến người nội bộ SCIC","FileUrl":"https://static2.vietstock.vn/vietstock/2026/5/28/20260527___vnm___thong_bao_giao_dich_cp_cua_to_chuc_clq_voi_nnb_sic__ban_cbtt_.pdf","TypeName":"GD của người liên quan","TransferTypeID":3,"PositionCD":"Blank","ExtraPositionNLQ":"TV HĐQT","ExtraPositionNLQEx":"Blank","ExtraPositionNN":"Blank","RelationShipType":"Blank","DTTHCD":"","DTTHLQ":"SCIC","DTLQLQ":"Vũ Trí Thức","NVTH":"","RegisterBuyVolume":1000000,"BuyVolume":null,"RegisterSellVolume":null,"SellVolume":null,"RegisterVolumeBefore":0,"RegisterVolumeAfter":1000000,"VolumeBefore":null,"VolumeAfter":null,"DateBuyExpected":"/Date(1780419600000)/","DateSellExpected":"/Date(1782925200000)/","DateActionTo":null,"PositionCDEx":"Blank","ExtraPositionIDNNEx":23,"DateActionFrom":null,"NDDTitle":6,"NDDTH":"","NDDPosition":23,"NDDExtraPosition":23,"TransferTitleTypeID":1,"RegisterBuyVolumePercent":null,"BuyVolumePercent":null,"RegisterSellVolumePercent":null,"SellVolumePercent":null,"RegisterVolumeBeforePercent":null,"RegisterVolumeAfterPercent":null,"VolumeBeforePercent":null,"VolumeAfterPercent":null,"StatusName":"Đăng ký","TotalRecord":1,"Row":1}]"#;
        let records: Vec<InsiderTradingRecord> = serde_json::from_str(json).unwrap();
        assert_eq!(records.len(), 1);
        let r = &records[0];
        assert_eq!(r.event_id, 220989);
        assert_eq!(r.stock_code, "VNM");
        assert_eq!(r.type_name, "GD của người liên quan");
        assert_eq!(r.transfer_type_id, 3);
        assert_eq!(r.dtthlq, "SCIC");
        assert_eq!(r.dtlqlq, "Vũ Trí Thức");
        assert_eq!(r.extra_position_nlq, "TV HĐQT");
        assert_eq!(r.register_buy_volume, Some(1000000));
        assert_eq!(r.register_volume_before, Some(0));
        assert_eq!(r.register_volume_after, Some(1000000));
        assert!(r.buy_volume.is_none());
        assert!(r.volume_before.is_none());
        assert!(r.date_buy_expected.is_some());
        assert!(r.date_sell_expected.is_some());
        assert!(r.date_action_from.is_none());
        assert_eq!(r.status_name, "Đăng ký");
        assert_eq!(r.total_record, 1);
        assert_eq!(r.row, 1);
    }

    // ---- Transfer Type Definitions ----

    #[test]
    fn test_deserialize_transfer_type_defs() {
        let json = r#"[{"ID":2,"Title":"Giao dịch cổ đông nội bộ","TitleEn":"Internal shareholder Transactions","Priority":1},{"ID":3,"Title":"Giao dịch của người có liên quan","TitleEn":"Related person's Transactions","Priority":2},{"ID":1,"Title":"Giao dịch cổ đông lớn","TitleEn":"Major Shareholder Transactions","Priority":3},{"ID":5,"Title":"Giao dịch cổ phiếu quỹ","TitleEn":"Treasury Stock Transactions","Priority":4}]"#;
        let types: Vec<TransferTypeDef> = serde_json::from_str(json).unwrap();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].id, 2);
        assert_eq!(types[0].title, "Giao dịch cổ đông nội bộ");
        assert_eq!(types[0].title_en, "Internal shareholder Transactions");
        assert_eq!(types[0].priority, 1);
        assert_eq!(types[3].id, 5);
        assert_eq!(types[3].title, "Giao dịch cổ phiếu quỹ");
    }

    // ---- Industry GICS ----

    #[test]
    fn test_deserialize_industry_gics() {
        let json = r#"[
            {"IndustryID":1,"Level":1,"IndustryCode":10,"ParentCode":0,"IndustryName":"Năng lượng","ImageURL":"/content/images/sector/energy.svg"},
            {"IndustryID":2,"Level":2,"IndustryCode":1010,"ParentCode":10,"IndustryName":"Năng lượng","ImageURL":"/content/images/sector/energy.svg"},
            {"IndustryID":57,"Level":4,"IndustryCode":40101010,"ParentCode":401010,"IndustryName":"Lĩnh vực đa ngành","ImageURL":"/content/images/sector/401010.svg"}
        ]"#;
        let items: Vec<IndustryGics> = serde_json::from_str(json).unwrap();
        assert_eq!(items.len(), 3);

        let l1 = &items[0];
        assert_eq!(l1.industry_id, 1);
        assert_eq!(l1.level, 1);
        assert_eq!(l1.industry_code, 10);
        assert_eq!(l1.parent_code, 0);
        assert_eq!(l1.industry_name, "Năng lượng");

        let l2 = &items[1];
        assert_eq!(l2.level, 2);
        assert_eq!(l2.industry_code, 1010);
        assert_eq!(l2.parent_code, 10);

        let l4 = &items[2];
        assert_eq!(l4.level, 4);
        assert_eq!(l4.industry_code, 40101010);
    }

    // ---- Industry Heatmap ----

    #[test]
    fn test_deserialize_industry_heatmap() {
        let json = r#"{
            "IndustryCode": 10,
            "ParentCode": 0,
            "Level": 1,
            "DisplayOrder": 0,
            "IndustryName": "Năng lượng",
            "StockFullName": null,
            "ImageURL": "/content/images/sector/energy.svg",
            "TradingDate": "/Date(1781629200000)/",
            "BasicPrice": 326.18,
            "CeilingPrice": 1000000.0,
            "FloorPrice": 0.0,
            "ClosePrice": 323.91,
            "CloseIndex": 0,
            "Change": -2.27,
            "PerChange": -0.69,
            "PerChangeYTD": 33.15,
            "QttyStockCode": 47,
            "MarketCapital": 244627.836725,
            "TotalVol": 16055.0,
            "TotalVal": 123456.78
        }"#;
        let item: IndustryHeatmapItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.industry_code, 10);
        assert_eq!(item.level, 1);
        assert_eq!(item.industry_name, "Năng lượng");
        assert!(item.stock_full_name.is_none());
        assert!((item.basic_price - 326.18).abs() < 0.01);
        assert!((item.close_price - 323.91).abs() < 0.01);
        assert!((item.change - (-2.27)).abs() < 0.01);
        assert!((item.per_change - (-0.69)).abs() < 0.01);
        assert!((item.per_change_ytd - 33.15).abs() < 0.01);
        assert_eq!(item.qtty_stock_code, 47);
        assert!((item.market_capital - 244627.836725).abs() < 0.01);
    }

    // ---- Industry Performance ----

    #[test]
    fn test_deserialize_industry_performance() {
        let json = r#"[{"IndustryCode":10,"IndustryName":"Năng lượng","TradingDate":"/Date(1781629200000)/","Change":-0.6959347599484947,"Type":1}]"#;
        let items: Vec<IndustryPerformance> = serde_json::from_str(json).unwrap();
        assert_eq!(items.len(), 1);
        let p = &items[0];
        assert_eq!(p.industry_code, 10);
        assert_eq!(p.industry_name, "Năng lượng");
        assert!((p.change - (-0.6959)).abs() < 0.001);
        assert_eq!(p.perf_type, 1); // Today
    }

    // ---- Industry Index Point ----

    #[test]
    fn test_deserialize_industry_index_point() {
        let json = r#"{"IndustryCode":10,"IndustryName":"Năng lượng","TradingDate":"/Date(1773680400000)/","Value":390.55,"Year":0,"Quarter":0}"#;
        let point: IndustryIndexPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.industry_code, 10);
        assert_eq!(point.industry_name, "Năng lượng");
        assert!((point.value - 390.55).abs() < 0.01);
        assert_eq!(point.year, 0);
        assert_eq!(point.quarter, 0);
    }

    // ---- Industry News ----

    #[test]
    fn test_deserialize_industry_news() {
        let json = r#"{"ArticleID":1455130,"PublishTime":"/Date(1781661720000)/","PublishTimeConvert":"5 giờ trước","Title":"Phố Wall hạ mình trước Elon Musk","Head":"Trong thương vụ phát hành cổ phiếu...","ImageURL":"https://image.vietstock.vn/test.jpg","ArticleURL":"https://vietstock.vn/test-article.htm","IndustryName":null}"#;
        let item: IndustryNewsItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.article_id, 1455130);
        assert_eq!(item.title, "Phố Wall hạ mình trước Elon Musk");
        assert_eq!(item.publish_time_convert, "5 giờ trước");
        assert!(item.image_url.is_some());
        assert!(item.industry_name.is_none());
    }

    // ---- Bond Market Overview ----

    #[test]
    fn test_deserialize_bond_market_overview() {
        let json = r#"{"BondQuantity":94,"BondValue":130764586900000.0000,"HighPricePercent":8.434,"KLNY":877227242.0000,"KLLH":1255194355.0000,"TotalTrVol":42351.0000,"TotalTrVal":8705708696.0000,"TotalNegVol":5577731.0000,"TotalNegVal":618246488995.0000,"TradingDate":"/Date(1781629200000)/"}"#;
        let overview: BondMarketOverview = serde_json::from_str(json).unwrap();
        assert_eq!(overview.bond_quantity, 94);
        assert!((overview.bond_value - 130764586900000.0).abs() < 1.0);
        assert!((overview.high_price_percent - 8.434).abs() < 0.001);
        assert!((overview.klny - 877227242.0).abs() < 1.0);
        assert!((overview.kllh - 1255194355.0).abs() < 1.0);
        assert!((overview.total_tr_vol - 42351.0).abs() < 1.0);
        assert!((overview.total_tr_val - 8705708696.0).abs() < 1.0);
        assert!((overview.total_neg_vol - 5577731.0).abs() < 1.0);
        assert!((overview.total_neg_val - 618246488995.0).abs() < 1.0);
        assert_eq!(overview.trading_date.year(), 2026);
    }

    // ---- Bond Chart Point ----

    #[test]
    fn test_deserialize_bond_chart_points() {
        let json = r#"[
            {"TrID":6945,"TradeDate":"/Date(1781542800000)/","TotalValue":2272947364035.0000,"TotalVolume":22026184.0000},
            {"TrID":6944,"TradeDate":"/Date(1781456400000)/","TotalValue":1316747118522.0000,"TotalVolume":12594966.0000}
        ]"#;
        let points: Vec<BondChartPoint> = serde_json::from_str(json).unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].tr_id, 6945);
        assert!((points[0].total_value - 2272947364035.0).abs() < 1.0);
        assert!((points[0].total_volume - 22026184.0).abs() < 1.0);
        assert_eq!(points[1].tr_id, 6944);
        assert_eq!(points[0].trade_date.year(), 2026);
    }

    // ---- Bond Trading Item ----

    #[test]
    fn test_deserialize_bond_trading_item_with_due_date() {
        let json = r#"{"TrBondHisID":73071,"StockBondID":5757,"StockCode":"MML121021","OutstandingShares":19999800,"FaceValue":100000,"DueDate":"/Date(1787677200000)/","IssueRate":null,"TrID":6945,"TradeDate":"/Date(1781542800000)/","ClosePrice":102615.0000,"Color":1,"TotalEvenLotVol":100000.0000,"TotalEvenLotVal":10261500000.0000,"TotalOddLotVol":0.0000,"TotalOddLotVal":0.0000,"TotalEvenLotNegotiatedVol":445.0000,"TotalEvenLotNegotiatedVal":45698480.0000,"TotalOddLotNegotiatedVol":0.0000,"TotalOddLotNegotiatedVal":0.0000,"CompanyName":"CTCP Masan MeatLife","TotalRecord":93,"RowNum":1}"#;
        let item: BondTradingItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.tr_bond_his_id, 73071);
        assert_eq!(item.stock_code, "MML121021");
        assert!((item.close_price - 102615.0).abs() < 0.01);
        assert_eq!(item.color, 1);
        assert!(item.due_date.is_some());
        assert!(item.issue_rate.is_none());
        assert_eq!(item.company_name, "CTCP Masan MeatLife");
        assert_eq!(item.total_record, 93);
        assert_eq!(item.row_num, 1);
    }

    #[test]
    fn test_deserialize_bond_trading_item_null_due_date() {
        let json = r#"{"TrBondHisID":73054,"StockBondID":12935,"StockCode":"HDC425001","OutstandingShares":4999999,"FaceValue":100000,"DueDate":null,"IssueRate":null,"TrID":6945,"TradeDate":"/Date(1781542800000)/","ClosePrice":135500.0000,"Color":1,"TotalEvenLotVol":5019.0000,"TotalEvenLotVal":678016402.0000,"TotalOddLotVol":0.0000,"TotalOddLotVal":0.0000,"TotalEvenLotNegotiatedVol":0.0000,"TotalEvenLotNegotiatedVal":0.0000,"TotalOddLotNegotiatedVol":0.0000,"TotalOddLotNegotiatedVal":0.0000,"CompanyName":"CTCP Phát triển Nhà Bà Rịa - Vũng Tàu","TotalRecord":93,"RowNum":3}"#;
        let item: BondTradingItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.stock_code, "HDC425001");
        assert!(item.due_date.is_none());
        assert!(item.issue_rate.is_none());
        assert!((item.close_price - 135500.0).abs() < 0.01);
        assert_eq!(item.color, 1);
    }

    // ---- Bond News Item ----

    #[test]
    fn test_deserialize_bond_news_item() {
        let json = r#"{"ArticleID":1454938,"Title":"CBTT về việc đăng ký giao dịch trái phiếu MBB12603 của NGÂN HÀNG TMCP QUÂN ĐỘI","PublishTime":"/Date(1781542800000)/","PublishTimeConvert":"16/06/2026 00:00","URL":"/2026/06/cbtt-ve-viec-dang-ky-giao-dich-trai-phieu-mbb12603-cua-ngan-hang-tmcp-quan-doi-3118-1454938.htm","TotalRecord":2425,"Source":"HNX","Icon":null}"#;
        let item: BondNewsItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.article_id, 1454938);
        assert!(item.title.contains("MBB12603"));
        assert_eq!(item.publish_time_convert, "16/06/2026 00:00");
        assert!(item.url.contains("mbb12603"));
        assert_eq!(item.total_record, 2425);
        assert_eq!(item.source, "HNX");
        assert!(item.icon.is_none());
    }

    #[test]
    fn test_deserialize_bond_news_array() {
        let json = r#"[
            {"ArticleID":1454938,"Title":"Test 1","PublishTime":"/Date(1781542800000)/","PublishTimeConvert":"16/06/2026 00:00","URL":"/2026/06/test-1.htm","TotalRecord":2425,"Source":"HNX","Icon":null},
            {"ArticleID":1454691,"Title":"Test 2","PublishTime":"/Date(1781542800000)/","PublishTimeConvert":"16/06/2026 00:00","URL":"/2026/06/test-2.htm","TotalRecord":2425,"Source":"HNX","Icon":null}
        ]"#;
        let items: Vec<BondNewsItem> = serde_json::from_str(json).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].article_id, 1454938);
        assert_eq!(items[1].article_id, 1454691);
    }

    // ---- ReportNorm (BCTT) ----

    #[test]
    fn test_deserialize_report_norm_bctt() {
        let json = r#"{"data":[
            {"ReportNormId":4385,"ReportNormName":"Thu nhập lãi thuần","CssStyle":"Normal","Unit":"","ReportTypeName":"Kết quả kinh doanh","ReportTypeCode":"KQ"},
            {"ReportNormId":4376,"ReportNormName":"Lợi nhuận thuần từ hoạt động KD trước chi phí dự phòng rủi ro tín dụng","CssStyle":"Normal","Unit":"","ReportTypeName":"Kết quả kinh doanh","ReportTypeCode":"KQ"},
            {"ReportNormId":4302,"ReportNormName":"A. TÀI SẢN","CssStyle":"MaxB","Unit":"","ReportTypeName":"Cân đối kế toán","ReportTypeCode":"CD"},
            {"ReportNormId":4380,"ReportNormName":"Lãi cơ bản trên cổ phiếu (VNĐ)","CssStyle":"Normal","Unit":"","ReportTypeName":"Kết quả kinh doanh","ReportTypeCode":"KQ"}
        ]}"#;
        let wrapper: ReportDataWrapper<Vec<ReportNorm>> = serde_json::from_str(json).unwrap();
        assert_eq!(wrapper.data.len(), 4);

        let n0 = &wrapper.data[0];
        assert_eq!(n0.report_norm_id, 4385);
        assert_eq!(n0.report_norm_name, "Thu nhập lãi thuần");
        assert_eq!(n0.css_style, "Normal");
        assert_eq!(n0.report_type_code.as_deref(), Some("KQ"));
        assert!(n0.parent_report_norm_id.is_none());
        assert!(n0.calculated_ordering.is_none());

        let n2 = &wrapper.data[2];
        assert_eq!(n2.report_norm_name, "A. TÀI SẢN");
        assert_eq!(n2.css_style, "MaxB");
        assert_eq!(n2.report_type_code.as_deref(), Some("CD"));
    }

    // ---- ReportNorm (CDKT) — different shape: hierarchical with ParentReportNormID ----

    #[test]
    fn test_deserialize_report_norm_cdkt() {
        let json = r#"{"data":[
            {"ReportNormId":4302,"ReportNormName":"A. TÀI SẢN","CssStyle":"MaxB","ParentReportNormID":4302,"CalculatedOrdering":0},
            {"ReportNormId":4310,"ReportNormName":"I. Tiền mặt, vàng bạc, đá quý","CssStyle":"NormalB","ParentReportNormID":4375,"CalculatedOrdering":2},
            {"ReportNormId":4344,"ReportNormName":"1. Tiền, vàng gửi tại các TCTD khác","CssStyle":"Normal","ParentReportNormID":4312,"CalculatedOrdering":4},
            {"ReportNormId":4375,"ReportNormName":"TỔNG TÀI SẢN","CssStyle":"MaxB","ParentReportNormID":4375,"CalculatedOrdering":0}
        ]}"#;
        let wrapper: ReportDataWrapper<Vec<ReportNorm>> = serde_json::from_str(json).unwrap();
        assert_eq!(wrapper.data.len(), 4);

        let n0 = &wrapper.data[0];
        assert_eq!(n0.report_norm_id, 4302);
        assert_eq!(n0.report_norm_name, "A. TÀI SẢN");
        assert_eq!(n0.css_style, "MaxB");
        assert_eq!(n0.parent_report_norm_id, Some(4302));
        assert_eq!(n0.calculated_ordering, Some(0));
        // CDKT norms do NOT have report_type fields or unit
        assert!(n0.report_type_code.is_none());
        assert!(n0.report_type_name.is_none());

        let n1 = &wrapper.data[1];
        assert_eq!(n1.parent_report_norm_id, Some(4375));
        assert_eq!(n1.calculated_ordering, Some(2));
        assert_eq!(n1.css_style, "NormalB");

        let n2 = &wrapper.data[2];
        assert_eq!(n2.parent_report_norm_id, Some(4312));
        assert_eq!(n2.calculated_ordering, Some(4));
        assert_eq!(n2.css_style, "Normal");
    }

    // ---- ReportPeriod (BCTT GetListReportData) ----

    #[test]
    fn test_deserialize_report_period() {
        let json = r#"{"data":[
            {
                "RowNumber":1,"ReportDataID":27518,"YearPeriod":2009,"ReportTermID":2,
                "YKienKiemToan":"","CtyKiemToan":"","IsUnited":0,
                "UnitedName":"Hợp nhất","AuditStatusID":4,"AuditStatusName":"Chưa kiểm toán",
                "PeriodBegin":"200901","PeriodEnd":"200903",
                "BasePeriodBegin":"200901","BasePeriodEnd":"200903",
                "IsShowData_Permission":true,
                "Url":"https://static2.vietstock.vn/data/HOSE/VCB_Baocaotaichinh_Q1_2009_Hopnhat_1.pdf"
            },
            {
                "RowNumber":69,"ReportDataID":288951,"YearPeriod":2026,"ReportTermID":2,
                "YKienKiemToan":"","CtyKiemToan":"","IsUnited":0,
                "UnitedName":"Hợp nhất","AuditStatusID":10,"AuditStatusName":"Kiểm toán",
                "PeriodBegin":"202601","PeriodEnd":"202603",
                "BasePeriodBegin":"202601","BasePeriodEnd":"202603",
                "IsShowData_Permission":true,"Url":null
            }
        ]}"#;
        let wrapper: ReportDataWrapper<Vec<ReportPeriod>> = serde_json::from_str(json).unwrap();
        assert_eq!(wrapper.data.len(), 2);

        let p0 = &wrapper.data[0];
        assert_eq!(p0.row_number, 1);
        assert_eq!(p0.report_data_id, 27518);
        assert_eq!(p0.year_period, 2009);
        assert_eq!(p0.report_term_id, 2); // Q1
        assert_eq!(p0.is_united, 0);
        assert_eq!(p0.united_name, "Hợp nhất");
        assert_eq!(p0.audit_status_id, 4);
        assert_eq!(p0.audit_status_name, "Chưa kiểm toán");
        assert_eq!(p0.period_begin, "200901");
        assert_eq!(p0.period_end, "200903");
        assert!(p0.is_show_data_permission);
        assert!(p0.url.is_some());
        assert!(p0.url.as_ref().unwrap().contains("VCB"));

        let p1 = &wrapper.data[1];
        assert_eq!(p1.row_number, 69);
        assert_eq!(p1.report_data_id, 288951);
        assert_eq!(p1.year_period, 2026);
        assert_eq!(p1.audit_status_name, "Kiểm toán");
        assert!(p1.url.is_none()); // older reports may lack a download URL
    }

    // ---- ReportValueRow (BCTT GetReportDataDetailValue) ----

    #[test]
    fn test_deserialize_report_value_row() {
        let json = r#"{"data":[
            {
                "ReportNormId":4302,"ReportTypeCode":"CD",
                "Value1":null,"Value2":null,"Value3":2550963.00,
                "Value4":2441929.00,"Value5":2378186.00,"Value6":2217941.00,
                "Value7":2109261.00,"Value8":2085397.00,"Value9":1932362.00
            },
            {
                "ReportNormId":4385,"ReportTypeCode":"KQ",
                "Value1":17651.00,"Value2":null,"Value3":null,
                "Value4":null,"Value5":null,"Value6":null,
                "Value7":null,"Value8":null,"Value9":null
            }
        ]}"#;
        let wrapper: ReportDataWrapper<Vec<ReportValueRow>> = serde_json::from_str(json).unwrap();
        assert_eq!(wrapper.data.len(), 2);

        let r0 = &wrapper.data[0];
        assert_eq!(r0.report_norm_id, 4302);
        assert_eq!(r0.report_type_code.as_deref(), Some("CD"));
        assert!(r0.value1.is_none());
        assert!(r0.value2.is_none());
        assert!((r0.value3.unwrap() - 2550963.0).abs() < 0.01);
        assert!((r0.value9.unwrap() - 1932362.0).abs() < 0.01);

        let r1 = &wrapper.data[1];
        assert_eq!(r1.report_norm_id, 4385);
        assert_eq!(r1.report_type_code.as_deref(), Some("KQ"));
        assert!((r1.value1.unwrap() - 17651.0).abs() < 0.01);
        assert!(r1.value2.is_none());
    }

    // ---- FinancialTab ----

    #[test]
    fn test_financial_tab_prefixes() {
        assert_eq!(FinancialTab::Bctt.prefix(), "BCTT");
        assert_eq!(FinancialTab::Cdkt.prefix(), "CDKT");
        assert_eq!(FinancialTab::Kqkd.prefix(), "KQKD");
        assert_eq!(FinancialTab::Lctt.prefix(), "LCTT");
        assert_eq!(FinancialTab::Tm.prefix(), "TM");
        assert_eq!(FinancialTab::Cstc.prefix(), "CSTC");
        assert_eq!(FinancialTab::Ctkh.prefix(), "CTKH");
    }
}
