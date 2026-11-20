//! Vietstock API client — session management, CSRF tokens, and API calls.

use serde::de::DeserializeOwned;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use wreq::{
    Client, StatusCode,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue, REFERER},
};
use wreq_util::{Emulation, Platform, Profile};

use crate::error::ApiError;
use crate::models::{
    AnalysisReport, AuthTokenResponse, BondChartPoint, BondMarketOverview, BondNewsItem,
    BondRelatedResponse, BondTradingItem, CorporateEvent, Document, DocumentType,
    EventCalendarItem, EventTypeLookup, FinanceIndexGroupResponse, FinanceInfoResponse,
    FinancialStatement, FinancialTab, FundamentalRatingResponse, HeaderNewsItem,
    IndustryFinancialIndex, IndustryGics, IndustryHeatmapItem, IndustryIndexPoint,
    IndustryNewsItem, IndustryPerformance, InsiderTradingRecord, LoginResponse, MarketPrice,
    NewsItem, PerformanceResponse, PlannedTargetResponse, RefPriceResponse, ReportDataWrapper,
    ReportNorm, ReportPeriod, ReportValueRow, SectorReport, TargetImplementationResponse,
    TechnicalRatingResponse, TradingDayOverview, TradingInfo, TradingOverviewResponse,
    TransferTypeDef,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const BASE_URL: &str = "https://finance.vietstock.vn";
const REQUEST_TIMEOUT_SECS: u64 = 30;
/// File downloads: a large ZIP/RAR from the CDN can take minutes.
const DOWNLOAD_TIMEOUT_SECS: u64 = 300;
const DEFAULT_RATE_LIMIT_MS: u64 = 1000;

/// `Unit` of [`VietstockClient::get_finance_info`]: values in billions of
/// dong (the unit the site's own summary uses).
pub const FINANCE_INFO_DEFAULT_UNIT: u64 = 1_000_000_000;

/// `Unit` for whole-dong values from
/// [`VietstockClient::get_finance_info_in_unit`].
pub const FINANCE_INFO_UNIT_DONG: u64 = 1;

// ---------------------------------------------------------------------------
// Rate limiter
// ---------------------------------------------------------------------------

/// Spaces the starts of requests at least `interval` apart.
///
/// Each caller reserves the next free slot under the lock and sleeps outside it,
/// so it works through `&self` and concurrent callers queue up one interval apart.
/// Uses tokio's clock, so tests can run it with a paused clock.
#[derive(Debug)]
struct RateLimiter {
    interval: Duration,
    next_slot: Mutex<Option<tokio::time::Instant>>,
}

impl RateLimiter {
    fn new(interval: Duration) -> Self {
        Self {
            interval,
            next_slot: Mutex::new(None),
        }
    }

    /// Waits for this request's slot and records it.
    async fn wait(&self) {
        let wait = {
            let mut next = self
                .next_slot
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let now = tokio::time::Instant::now();
            let slot = match *next {
                Some(t) if t > now => t,
                _ => now,
            };
            *next = Some(slot + self.interval);
            slot.saturating_duration_since(now)
        };
        if !wait.is_zero() {
            debug!(wait_ms = wait.as_millis(), "Rate limiting");
            tokio::time::sleep(wait).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Main client for the Vietstock Finance API (`finance.vietstock.vn`).
///
/// # Session Lifecycle
///
/// 1. Create with [`VietstockClient::new()`]
/// 2. Call [`init_session()`](VietstockClient::init_session) — acquires CSRF token + cookies
/// 3. Make API calls
///
/// # Example
///
/// ```rust,no_run
/// use vietstock::VietstockClient;
///
/// # async fn example() -> Result<(), vietstock::ApiError> {
/// let mut client = VietstockClient::new()?;
/// client.init_session("VCB").await?;
///
/// let perf = client.get_performance("VCB").await?;
/// tracing::info!(close = perf.data[0].close_price, "VCB close (VND)");
///
/// let trading = client.get_trading_info("VCB").await?;
/// tracing::info!(pe = trading.pe, "VCB P/E");
/// # Ok(())
/// # }
/// ```
pub struct VietstockClient {
    client: Client,
    base_url: String,
    csrf_token: Option<String>,
    /// Every request, including the session page and file downloads, waits here.
    limiter: RateLimiter,
    download_timeout: Duration,
    auth_token_refreshed: bool,
    is_logged_in: bool,
}

/// Builder for [`VietstockClient`] with customizable options.
pub struct VietstockClientBuilder {
    rate_limit_delay: Duration,
    timeout: Duration,
    download_timeout: Duration,
}

impl Default for VietstockClientBuilder {
    fn default() -> Self {
        Self {
            rate_limit_delay: Duration::from_millis(DEFAULT_RATE_LIMIT_MS),
            timeout: Duration::from_secs(REQUEST_TIMEOUT_SECS),
            download_timeout: Duration::from_secs(DOWNLOAD_TIMEOUT_SECS),
        }
    }
}

impl VietstockClientBuilder {
    /// Defaults: one request per second, 30 s per API request, 300 s per file download.
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimum gap between the starts of two requests (every request, including
    /// [`init_session`](VietstockClient::init_session) and downloads). Default 1000.
    pub fn rate_limit_ms(mut self, ms: u64) -> Self {
        self.rate_limit_delay = Duration::from_millis(ms);
        self
    }

    /// Timeout of API and page requests, from connecting until the whole response
    /// body is read. Default 30 s. File downloads use
    /// [`download_timeout`](Self::download_timeout) instead.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Timeout of one file download ([`VietstockClient::download_url`]), from
    /// connecting until the whole file is read. Default 300 s, because large
    /// ZIP/RAR filings from the CDN can exceed the 30 s API timeout.
    pub fn download_timeout(mut self, timeout: Duration) -> Self {
        self.download_timeout = timeout;
        self
    }

    pub fn build(self) -> Result<VietstockClient, ApiError> {
        // Chrome on Windows: the TLS, HTTP/2 and header fingerprint (including the
        // User-Agent) all come from one profile, so they stay consistent with each
        // other. This replaces the old headless-Chromium fallback.
        let emulation = Emulation::builder()
            .profile(Profile::Chrome149)
            .platform(Platform::Windows)
            .http2(true)
            .build();
        let client = Client::builder()
            .emulation(emulation)
            .timeout(self.timeout)
            .cookie_store(true)
            .https_only(true)
            .build()?;

        Ok(VietstockClient {
            client,
            base_url: BASE_URL.to_string(),
            csrf_token: None,
            limiter: RateLimiter::new(self.rate_limit_delay),
            download_timeout: self.download_timeout,
            auth_token_refreshed: false,
            is_logged_in: false,
        })
    }
}

impl VietstockClient {
    pub fn new() -> Result<Self, ApiError> {
        VietstockClientBuilder::new().build()
    }

    pub fn builder() -> VietstockClientBuilder {
        VietstockClientBuilder::new()
    }

    pub fn is_session_initialized(&self) -> bool {
        self.csrf_token.is_some()
    }

    pub fn is_authenticated(&self) -> bool {
        self.is_logged_in
    }

    pub fn is_auth_token_refreshed(&self) -> bool {
        self.auth_token_refreshed
    }

    // -----------------------------------------------------------------------
    // URL Builders
    // -----------------------------------------------------------------------

    pub fn get_login_url_google(&self, return_url: &str) -> String {
        format!(
            "{}/Account/LoginGooglePlus?urlReturn={}",
            self.base_url,
            urlencoding(return_url)
        )
    }

    pub fn get_login_url_facebook(&self, return_url: &str) -> String {
        format!(
            "{}/Account/LoginFacebook?urlReturn={}",
            self.base_url,
            urlencoding(return_url)
        )
    }

    // -----------------------------------------------------------------------
    // Session Initialization
    // -----------------------------------------------------------------------

    /// Initialize a session by visiting a stock's document listing page.
    ///
    /// Acquires `ASP.NET_SessionId`, `__RequestVerificationToken`, and then
    /// automatically refreshes the anonymous `AuthRequest` JWT.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn init_session(&mut self, stock_code: &str) -> Result<(), ApiError> {
        let stock_code = validate_stock_code(stock_code)?;

        let url = format!("{}/{}/tai-tai-lieu.htm", self.base_url, stock_code);

        debug!(%url, "Initializing session");

        let request = self
            .client
            .get(&url)
            .header(REFERER, HeaderValue::from_static(BASE_URL));
        let resp = self.send(request).await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!(%status, "Session initialization returned non-200");
            return Err(ApiError::Api { status, body });
        }

        let html = resp.text().await.map_err(ApiError::Http)?;
        self.extract_csrf_from_html(&html)?;

        info!("Session initialized successfully with CSRF token");
        Ok(())
    }

    fn extract_csrf_from_html(&mut self, html: &str) -> Result<(), ApiError> {
        let token = extract_csrf_token(html).ok_or(ApiError::CsrfTokenNotFound)?;
        debug!("CSRF token extracted (len={})", token.len());
        self.csrf_token = Some(token);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Rate Limiting
    // -----------------------------------------------------------------------

    /// Sends a request after waiting for its rate-limit slot. Every request of
    /// this client goes through here, so none can skip the limiter.
    async fn send(&self, request: wreq::RequestBuilder) -> Result<wreq::Response, ApiError> {
        self.limiter.wait().await;
        Ok(request.send().await?)
    }

    // -----------------------------------------------------------------------
    // Common Headers
    // -----------------------------------------------------------------------

    fn api_headers(&self, stock_code: impl AsRef<str>, with_content_type: bool) -> HeaderMap {
        let stock_code = stock_code.as_ref();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-requested-with",
            HeaderValue::from_static("XMLHttpRequest"),
        );

        let referer = format!("{}/{}/tai-tai-lieu.htm", self.base_url, stock_code);
        if let Ok(val) = HeaderValue::from_str(&referer) {
            headers.insert(REFERER, val);
        } else {
            headers.insert(REFERER, HeaderValue::from_static(BASE_URL));
        }

        if with_content_type {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"),
            );
        }
        headers
    }

    // -----------------------------------------------------------------------
    // Internal Helpers
    // -----------------------------------------------------------------------

    fn require_csrf_token(&self) -> Result<String, ApiError> {
        self.csrf_token
            .as_ref()
            .cloned()
            .ok_or(ApiError::SessionNotInitialized)
    }

    /// Build a form-urlencoded body from params + CSRF token.
    fn body_with_params(&self, params: &[(&str, &str)]) -> Result<String, ApiError> {
        let token = self.require_csrf_token()?;
        Ok(encode_form(params.iter().copied().chain(std::iter::once(
            ("__RequestVerificationToken", token.as_str()),
        ))))
    }

    /// POST with arbitrary params + CSRF token. Generic JSON deserialization.
    async fn post_with_params<T: DeserializeOwned>(
        &mut self,
        path: &str,
        stock_code: impl AsRef<str>,
        params: &[(&str, &str)],
    ) -> Result<T, ApiError> {
        let stock_code = stock_code.as_ref();
        let url = format!("{}{}", self.base_url, path);
        let body = self.body_with_params(params)?;

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers(stock_code, true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        debug!(len = response_body.len(), path = %path, "Received response");
        parse_response(response_body)
    }

    /// POST with only the CSRF token (no extra params).
    async fn post_csrf_only<T: DeserializeOwned>(&mut self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.require_csrf_token()?;
        let body = encode_form([("__RequestVerificationToken", token.as_str())]);

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers("VNM", true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        debug!(len = response_body.len(), path = %path, "Received response");
        parse_response(response_body)
    }

    /// POST without CSRF token (for endpoints that don't require it).
    async fn post_no_csrf<T: DeserializeOwned>(
        &self,
        path: &str,
        stock_code: impl AsRef<str>,
        body: String,
    ) -> Result<T, ApiError> {
        let stock_code = stock_code.as_ref();
        let url = format!("{}{}", self.base_url, path);

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers(stock_code, true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        parse_response(response_body)
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        stock_code: &str,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);

        let request = self
            .client
            .get(&url)
            .headers(self.api_headers(stock_code, false));
        let resp = self.send(request).await?;

        let body = Self::check_and_read_text(resp).await?;
        parse_response(body)
    }

    async fn check_and_read_bytes(resp: wreq::Response) -> Result<Vec<u8>, ApiError> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, body });
        }
        Ok(resp.bytes().await?.to_vec())
    }

    async fn check_and_read_text(resp: wreq::Response) -> Result<String, ApiError> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::Api { status, body });
        }
        Ok(resp.text().await?)
    }

    // =======================================================================
    // DOCUMENT API
    // =======================================================================

    /// Fetch all document type categories. No CSRF required.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_document_types(&self) -> Result<Vec<DocumentType>, ApiError> {
        self.get_json("/data/getdocumenttype", "ACV").await
    }

    /// Fetch available years for a stock's documents. No CSRF required.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_years(&self, stock_code: &str) -> Result<Vec<i32>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let body = encode_form([("code", stock_code.as_str())]);
        self.post_no_csrf("/data/getyearreport", &stock_code, body)
            .await
    }

    /// Fetch one page (20 documents) of a stock's document library, newest first.
    ///
    /// `year` filters by document year; `page` starts at 1. Each returned
    /// [`Document`] carries `total_row`, the size of the whole (filtered) list.
    /// Works without logging in (checked 2026-11-19: 424 documents for VNM,
    /// 2003–2026). Unknown extra form fields such as `doctype` are ignored.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_documents_page(
        &mut self,
        stock_code: &str,
        year: Option<i32>,
        page: u32,
    ) -> Result<Vec<Document>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let page = page.max(1).to_string();
        let year = year.map(|y| y.to_string());
        let mut params: Vec<(&str, &str)> = vec![("code", &stock_code), ("page", &page)];
        if let Some(year) = &year {
            params.push(("year", year));
        }
        self.post_with_params("/data/getdocument", &stock_code, &params)
            .await
    }

    /// Fetch the first page (20 newest documents) of a stock's document library.
    ///
    /// Use [`get_all_documents`](Self::get_all_documents) for the full history.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_documents(&mut self, stock_code: &str) -> Result<Vec<Document>, ApiError> {
        let t0 = Instant::now();
        let result = self.get_documents_page(stock_code, None, 1).await?;
        info!(code = %stock_code, count = result.len(), elapsed_ms = t0.elapsed().as_millis(), "documents fetched");
        Ok(result)
    }

    /// Fetch every document in a stock's library by walking the pages until
    /// `total_row` documents have been seen (or a page comes back empty).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_all_documents(
        &mut self,
        stock_code: &str,
        year: Option<i32>,
    ) -> Result<Vec<Document>, ApiError> {
        let t0 = Instant::now();
        let mut all: Vec<Document> = Vec::new();
        for page in 1.. {
            let batch = self.get_documents_page(stock_code, year, page).await?;
            let total = batch
                .first()
                .map_or(0, |d| usize::try_from(d.total_row).unwrap_or(0));
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            if all.len() >= total {
                break;
            }
        }
        // Pages can shift while a filing is being published; keep each file once.
        let mut seen = std::collections::HashSet::new();
        all.retain(|d| seen.insert(d.file_info_id));
        info!(code = %stock_code, count = all.len(), elapsed_ms = t0.elapsed().as_millis(), "all documents fetched");
        Ok(all)
    }

    /// Fetch documents from the **overview page** (paginated via Offset/Limit).
    ///
    /// NOTE: Different from `/data/getdocument` — this endpoint uses `StockCode` param
    /// and supports `PageType`, `Offset`, `Limit`, `LanguageID`.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_documents_overview(
        &mut self,
        stock_code: &str,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<Document>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/data/GetDocument",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("Offset", &offset.to_string()),
                ("Limit", &limit.to_string()),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // TRADING API
    // =======================================================================

    /// Fetch real-time trading information for a stock.
    ///
    /// **Updated (v3):** Now uses `StockCode=` parameter (not `code=`).
    /// Includes extended fields: foreign room, dividends, beta, forward P/E, etc.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_trading_info(&mut self, stock_code: &str) -> Result<TradingInfo, ApiError> {
        let t0 = Instant::now();
        let stock_code = validate_stock_code(stock_code)?;
        let result: TradingInfo = self
            .post_with_params(
                "/company/tradinginfo",
                &stock_code,
                &[("StockCode", &stock_code), ("LanguageID", "1")],
            )
            .await?;
        info!(code = %stock_code, last_price = result.last_price, change_pct = result.per_change, elapsed_ms = t0.elapsed().as_millis(), "trading info fetched");
        Ok(result)
    }

    /// Fetch reference price and recent trading color data.
    ///
    /// **Note:** `RefPrice` in the response is in **thousands of VND**
    /// (e.g., 61.6 = 61,600 VND).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_ref_price(&mut self, stock_code: &str) -> Result<RefPriceResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/company/GetDataRefPriceByCode",
            &stock_code,
            &[("StockCode", &stock_code), ("LanguageID", "1")],
        )
        .await
    }

    /// Download the raw bytes of a document from its CDN URL.
    ///
    /// Same as [`download_url`](Self::download_url) with `document.url`.
    pub async fn download_document(&self, document: &Document) -> Result<Vec<u8>, ApiError> {
        self.download_url(&document.url).await
    }

    /// Download the raw bytes of a file URL (a [`Document::url`] on the CDN).
    ///
    /// Rate limited like every other request, and bounded by the builder's
    /// [`download_timeout`](VietstockClientBuilder::download_timeout) instead of
    /// the API timeout. Needs no session.
    #[tracing::instrument(skip(self), err)]
    pub async fn download_url(&self, url: &str) -> Result<Vec<u8>, ApiError> {
        let t0 = Instant::now();
        let request = self.client.get(url).timeout(self.download_timeout);
        let resp = self.send(request).await?;
        let body = Self::check_and_read_bytes(resp).await?;
        info!(%url, size = body.len(), elapsed_ms = t0.elapsed().as_millis(), "document downloaded");
        Ok(body)
    }

    /// Download a document and save it to a local file.
    #[tracing::instrument(skip(self, document), fields(output_dir = %output_dir.display()), err)]
    pub async fn download_document_to_file(
        &self,
        document: &Document,
        output_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, ApiError> {
        let data = self.download_document(document).await?;
        // Keep only the last URL segment and refuse anything that could escape
        // `output_dir` (`.`, `..`, empty).
        let filename = match document.url.rsplit('/').next().map(str::trim) {
            Some(name) if !name.is_empty() && name != "." && name != ".." => name,
            _ => "download.tmp",
        };

        std::fs::create_dir_all(output_dir)?;
        let filepath = output_dir.join(filename);
        std::fs::write(&filepath, &data)?;

        info!(path = %filepath.display(), size = data.len(), "Document saved");
        Ok(filepath)
    }

    // =======================================================================
    // AUTHENTICATION API
    // =======================================================================

    /// Refresh the anonymous `AuthRequest` JWT token (~3min expiry).
    #[tracing::instrument(skip(self), err)]
    pub async fn refresh_auth_token(&mut self) -> Result<(), ApiError> {
        let t0 = Instant::now();
        let resp: AuthTokenResponse = self.post_csrf_only("/Account/GetAuthToken").await?;

        if resp.is_success {
            self.auth_token_refreshed = true;
            debug!(elapsed_ms = t0.elapsed().as_millis(), "JWT refreshed");
            Ok(())
        } else {
            warn!("JWT refresh returned IsSuccess=false");
            Err(ApiError::Api {
                status: StatusCode::OK,
                body: "GetAuthToken returned IsSuccess=false".into(),
            })
        }
    }

    /// Log in with email + password + reCAPTCHA v2 token.
    #[tracing::instrument(skip(self, email, password, recaptcha_token), fields(email = %email), err)]
    pub async fn login(
        &mut self,
        email: &str,
        password: &str,
        recaptcha_token: &str,
        remember: bool,
    ) -> Result<LoginResponse, ApiError> {
        let token = self.require_csrf_token()?;

        let url = format!("{}/Account/Login", self.base_url);
        let remember = remember.to_string();
        let body = encode_form([
            ("__RequestVerificationToken", token.as_str()),
            ("Email", email),
            ("Password", password),
            ("responseCaptchaLoginPopup", recaptcha_token),
            ("Remember", remember.as_str()),
        ]);

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers("VNM", true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        let login_resp: LoginResponse = serde_json::from_str(&response_body)?;

        if login_resp.is_success {
            self.is_logged_in = true;
            info!("Login successful");
        }

        Ok(login_resp)
    }

    // =======================================================================
    // PERFORMANCE API
    // =======================================================================

    /// Fetch stock price performance across multiple timeframes (1W to All-time).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_performance(
        &mut self,
        stock_code: &str,
    ) -> Result<PerformanceResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/data/GetPerformanceByStockCode",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // FINANCIAL STATEMENTS API
    // =======================================================================

    /// Fetch the financial summary of a company in billions of dong
    /// ([`FINANCE_INFO_DEFAULT_UNIT`]); see
    /// [`VietstockClient::get_finance_info_in_unit`] for other units.
    ///
    /// # Arguments
    ///
    /// * `report_term_type` — `1` for annual, `2` for quarterly
    /// * `report_type` — `"BCTQ"` (quarterly), `"BCTN"` (annual)
    ///
    /// # Returns
    ///
    /// `None` if the API returns an empty array (no data for that combination).
    /// Otherwise, a tuple of `(period_metadata, report_components)` where:
    /// - Period metadata describes each column (Q1/2026, Q4/2025, etc.)
    /// - Report components maps name → array of line items
    pub async fn get_finance_info(
        &mut self,
        stock_code: &str,
        report_term_type: i32,
        report_type: &str,
    ) -> Result<Option<FinanceInfoResponse>, ApiError> {
        self.get_finance_info_in_unit(
            stock_code,
            report_term_type,
            report_type,
            FINANCE_INFO_DEFAULT_UNIT,
        )
        .await
    }

    /// [`VietstockClient::get_finance_info`] with the values in `unit` dong
    /// each (the request's `Unit` field): [`FINANCE_INFO_UNIT_DONG`] for whole
    /// dong, `1_000` thousands, `1_000_000` millions,
    /// [`FINANCE_INFO_DEFAULT_UNIT`] billions. Values come back as `f64`,
    /// exact in whole dong up to 2^53 (about 9·10^15 dong).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_finance_info_in_unit(
        &mut self,
        stock_code: &str,
        report_term_type: i32,
        report_type: &str,
        unit: u64,
    ) -> Result<Option<FinanceInfoResponse>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let url = format!("{}/data/financeinfo", self.base_url);
        let form = finance_info_form(&stock_code, report_term_type, report_type, unit);
        let fields: Vec<(&str, &str)> = form.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let body = self.body_with_params(&fields)?;

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers(&stock_code, true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        debug!(len = response_body.len(), "Received finance info");

        // The API returns `[]` for empty results, or a 2-element tuple array
        if response_body.trim() == "[]" {
            return Ok(None);
        }

        parse_response(response_body).map(Some)
    }

    // =======================================================================
    // FINANCIAL STATEMENT TAB API (BCTT, CDKT, KQKD, LCTT, TM, CSTC, CTKH)
    // =======================================================================
    //
    // Every financial statement tab follows a 3-step pipeline:
    //   1. GetListReportNorm_{TAB}  → Row definitions
    //   2. {TAB}_GetListReportData   → Column/period definitions
    //   3. GetReportDataDetailValue_{TAB}_ByReportDataIds → Actual values (9 columns/batch)

    /// Get report norm/row definitions for a financial tab.
    ///
    /// # Exceptions
    ///
    /// - **CDKT** uses `/data/GetListReportNormByStockCode` (no tab prefix)
    /// - **LCTT** uses `/data/GetListReportNorm_LCTT_ByStockCode`
    /// - All other tabs: `/data/GetListReportNorm_{TAB}_ByStockCode`
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code, tab = ?tab), err)]
    pub async fn get_report_norms(
        &mut self,
        stock_code: &str,
        tab: FinancialTab,
    ) -> Result<Vec<ReportNorm>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let endpoint = match tab {
            FinancialTab::Cdkt => "/data/GetListReportNormByStockCode".to_string(),
            _ => format!("/data/GetListReportNorm_{}_ByStockCode", tab.prefix()),
        };
        let wrapper: ReportDataWrapper<Vec<ReportNorm>> = self
            .post_with_params(&endpoint, &stock_code, &[("stockCode", &stock_code)])
            .await?;
        Ok(wrapper.data)
    }

    /// Get available periods/columns for a financial tab.
    ///
    /// # Arguments
    ///
    /// * `period_type` — `"QUY"` (quarterly) or `"NAM"` (annual)
    /// * `unit` — Display unit: 1000 (Nghìn đồng), 1000000 (Triệu đồng), 1000000000 (Tỷ đồng)
    /// * `united_id` — -1 = All, 0 = Consolidated (Hợp nhất), 1 = Parent (Công ty mẹ)
    /// * `audited_status_id` — -1 = All, 4 = Unaudited, 10 = Audited
    /// * `is_nam_duong_lich` — `false` = Fiscal year, `true` = Calendar year
    /// * `sort_time_type` — `"Time_ASC"` (oldest first) or `"Time_DESC"` (newest first)
    ///
    /// # Exceptions
    ///
    /// - **LCTT** uses `/data/LCTT_GetListReportData_Quarter_VSTTinh`
    /// - All other tabs: `/data/{TAB}_GetListReportData`
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code, tab = ?tab, period_type = %period_type), err)]
    #[allow(clippy::too_many_arguments)]
    pub async fn get_report_periods(
        &mut self,
        stock_code: &str,
        tab: FinancialTab,
        period_type: &str,
        unit: i64,
        united_id: i32,
        audited_status_id: i32,
        is_nam_duong_lich: bool,
        sort_time_type: &str,
    ) -> Result<Vec<ReportPeriod>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let endpoint = match tab {
            FinancialTab::Lctt => "/data/LCTT_GetListReportData_Quarter_VSTTinh".to_string(),
            _ => format!("/data/{}_GetListReportData", tab.prefix()),
        };
        let wrapper: ReportDataWrapper<Vec<ReportPeriod>> = self
            .post_with_params(
                &endpoint,
                &stock_code,
                &[
                    ("StockCode", &stock_code),
                    ("UnitedId", &united_id.to_string()),
                    ("AuditedStatusId", &audited_status_id.to_string()),
                    ("Unit", &unit.to_string()),
                    ("IsNamDuongLich", &is_nam_duong_lich.to_string()),
                    ("PeriodType", period_type),
                    ("SortTimeType", sort_time_type),
                ],
            )
            .await?;
        Ok(wrapper.data)
    }

    /// Get actual financial values for a batch of periods (max 9 per call).
    ///
    /// # Arguments
    ///
    /// * `periods` — Slice of `(ReportDataID, YearPeriod, IsShowData)` tuples.
    ///   Max **9** items per call. These come from the [`ReportPeriod`] list.
    /// * `unit` — Display unit (typically 1000000000 for Tỷ đồng)
    /// * `type_compare` — 1 = No comparison, other values for %YoY / %QoQ
    ///
    /// # Returns
    ///
    /// A vector of [`ReportValueRow`] where each row maps a `report_norm_id`
    /// to up to `periods.len()` values. `Value1` corresponds to `periods[0]`,
    /// `Value2` to `periods[1]`, etc.
    #[tracing::instrument(skip(self, periods), fields(stock_code = %stock_code, tab = ?tab, batch_size = periods.len()), err)]
    pub async fn get_report_values(
        &mut self,
        stock_code: &str,
        tab: FinancialTab,
        periods: &[(i32, i32, bool)],
        unit: i64,
        type_compare: i32,
    ) -> Result<Vec<ReportValueRow>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let endpoint = format!(
            "/data/GetReportDataDetailValue_{}_ByReportDataIds",
            tab.prefix()
        );

        let n = periods.len();
        let mut params: Vec<(String, String)> = vec![
            ("StockCode".to_string(), stock_code.clone()),
            ("Unit".to_string(), unit.to_string()),
            ("TypeCompare".to_string(), type_compare.to_string()),
        ];
        for (i, (report_data_id, year_period, is_show_data)) in periods.iter().enumerate() {
            let idx = i.to_string();
            params.push((format!("listReportDataIds[{}][Index]", idx), idx.clone()));
            params.push((
                format!("listReportDataIds[{}][ReportDataId]", idx),
                report_data_id.to_string(),
            ));
            params.push((
                format!("listReportDataIds[{}][IsShowData]", idx),
                is_show_data.to_string(),
            ));
            params.push((
                format!("listReportDataIds[{}][RowNumber]", idx),
                "0".to_string(),
            ));
            params.push((
                format!("listReportDataIds[{}][YearPeriod]", idx),
                year_period.to_string(),
            ));
            params.push((
                format!("listReportDataIds[{}][TotalCount]", idx),
                n.to_string(),
            ));
            params.push((
                format!("listReportDataIds[{}][SortTimeType]", idx),
                "Time_ASC".to_string(),
            ));
        }

        let refs: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let wrapper: ReportDataWrapper<Vec<ReportValueRow>> =
            self.post_with_params(&endpoint, &stock_code, &refs).await?;
        Ok(wrapper.data)
    }

    /// High-level convenience: fetch ALL financial data for a tab.
    ///
    /// Automatically handles the 3-step pipeline and auto-paginates
    /// the value requests (batches of 9 periods each).
    ///
    /// # Arguments
    ///
    /// * `period_type` — `"QUY"` or `"NAM"`
    /// * `unit` — Display unit (typically 1000000000 for Tỷ đồng)
    ///
    /// # Returns
    ///
    /// Tuple of `(norms, periods, values)` — the complete financial statement.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code, tab = ?tab, period_type = %period_type), err)]
    pub async fn get_full_financial_data(
        &mut self,
        stock_code: &str,
        tab: FinancialTab,
        period_type: &str,
        unit: i64,
    ) -> Result<FinancialStatement, ApiError> {
        let norms = self.get_report_norms(stock_code, tab).await?;
        let periods = self
            .get_report_periods(
                stock_code,
                tab,
                period_type,
                unit,
                -1,
                -1,
                false,
                "Time_ASC",
            )
            .await?;

        let batch_size: usize = 9;
        let mut all_values = Vec::new();
        for chunk in periods.chunks(batch_size) {
            let period_tuples: Vec<(i32, i32, bool)> = chunk
                .iter()
                .map(|p| (p.report_data_id, p.year_period, p.is_show_data_permission))
                .collect();
            let values = self
                .get_report_values(stock_code, tab, &period_tuples, unit, 1)
                .await?;
            all_values.extend(values);
        }

        Ok((norms, periods, all_values))
    }

    // =======================================================================
    // TRADING OVERVIEW API
    // =======================================================================

    /// Fetch 5-day trading overview table.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_trading_overview(
        &mut self,
        stock_code: &str,
    ) -> Result<Vec<TradingDayOverview>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let resp: TradingOverviewResponse = self
            .post_with_params(
                "/data/GetStockDeal_OverviewTrading",
                &stock_code,
                &[
                    ("StockCode", &stock_code),
                    ("PageType", "1"),
                    ("LanguageID", "1"),
                ],
            )
            .await?;
        Ok(resp.data)
    }

    // =======================================================================
    // NEWS API
    // =======================================================================

    /// Fetch company news with offset/limit pagination.
    ///
    /// Each item's `content` field is empty when fetched from the overview;
    /// the `url` field provides the link to the full article.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_news(
        &mut self,
        stock_code: &str,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<NewsItem>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let page = offset / limit + 1;
        self.post_with_params(
            "/data/GetNews",
            &stock_code,
            &[
                ("code", &stock_code),
                ("type", "-1"),
                ("page", &page.to_string()),
                ("pageSize", &limit.to_string()),
            ],
        )
        .await
    }

    /// Fetch company news with category filtering and page-based pagination.
    ///
    /// This is the format used on the dedicated company news page
    /// (`/tin-tuc-su-kien.htm`). Supports filtering by news category.
    ///
    /// # News Categories (`news_type`)
    /// | Value | Vietnamese | English |
    /// |:-----:|-----------|---------|
    /// | -1 | Tất cả | All |
    /// | 3 | KQKD | Financial Results |
    /// | 4 | Cổ tức | Dividends |
    /// | 5 | Tăng vốn - M&A | Capital Increase / M&A |
    /// | 6 | Giao dịch cổ đông lớn | Major Shareholder Trading |
    /// | 7 | Thay đổi nhân sự | Personnel Changes |
    /// | 8 | Trái phiếu | Bonds |
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code, news_type = %news_type), err)]
    pub async fn get_company_news(
        &mut self,
        stock_code: &str,
        news_type: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<NewsItem>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/data/GetNews",
            &stock_code,
            &[
                ("code", &stock_code),
                ("type", &news_type.to_string()),
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
            ],
        )
        .await
    }

    // =======================================================================
    // EVENTS API
    // =======================================================================

    /// Fetch corporate events (dividends, meetings, etc.).
    ///
    /// The underlying API returns `[[event, ...]]` — we flatten to a single vec.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_events(
        &mut self,
        stock_code: &str,
        offset: i32,
        limit: i32,
    ) -> Result<Vec<CorporateEvent>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let page = offset / limit + 1;
        // Response is [events_array, total_count]
        let val: serde_json::Value = self
            .post_with_params(
                "/data/EventsTypeData",
                &stock_code,
                &[
                    ("code", &stock_code),
                    ("eventTypeID", "-1"),
                    ("channelID", "0"),
                    ("catID", "-1"),
                    ("page", &page.to_string()),
                    ("pageSize", &limit.to_string()),
                    ("orderBy", "Date1"),
                    ("orderDir", "DESC"),
                ],
            )
            .await?;

        Ok(serde_json::from_value(val[0].clone()).unwrap_or_default())
    }

    /// Fetch company-specific events with category filtering.
    ///
    /// Used on the company news & events page (`/tin-tuc-su-kien.htm`).
    ///
    /// # Event Categories (`cat_id`)
    /// | Value | Vietnamese | English |
    /// |:-----:|-----------|---------|
    /// | -1 | Tất cả | All |
    /// | 1 | Cổ tức, thưởng và phát hành thêm | Dividends, Bonus, Additional Issue |
    ///
    /// # Event Types (`event_type_id`)
    /// | Value | Vietnamese | English |
    /// |:-----:|-----------|---------|
    /// | 1 | Sự kiện cổ phiếu | Stock Events |
    /// | 2 | Niêm yết | Listing |
    /// | 5 | Đại hội cổ đông | Shareholders Meeting |
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_company_events(
        &mut self,
        stock_code: &str,
        event_type_id: i32,
        cat_id: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<CorporateEvent>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let nested: Vec<Vec<CorporateEvent>> = self
            .post_with_params(
                "/data/EventsTypeData",
                &stock_code,
                &[
                    ("eventTypeID", &event_type_id.to_string()),
                    ("channelID", "0"),
                    ("code", &stock_code),
                    ("catID", &cat_id.to_string()),
                    ("page", &page.to_string()),
                    ("pageSize", &page_size.to_string()),
                    ("orderBy", "Date1"),
                    ("orderDir", "DESC"),
                ],
            )
            .await?;

        Ok(nested.into_iter().flatten().collect())
    }

    /// Fetch market-wide event calendar (paginated).
    ///
    /// # Arguments
    ///
    /// * `event_type_id` — 1=Dividend, 2=Listing, 5=Shareholders Meeting
    /// * `channel_id` — 0=All, or specific channel (17–54). Only meaningful for event_type_id=2 (Listing)
    /// * `page` — 1-based page number
    /// * `page_size` — Items per page (10, 20, or 50)
    /// * `from_date` — Filter start date in YYYY-MM-DD format (optional)
    /// * `to_date` — Filter end date in YYYY-MM-DD format (optional)
    #[tracing::instrument(skip(self), err)]
    pub async fn get_event_calendar(
        &mut self,
        event_type_id: i32,
        channel_id: i32,
        page: i32,
        page_size: i32,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<Vec<EventCalendarItem>, ApiError> {
        let f_date = from_date.unwrap_or("");
        let t_date = to_date.unwrap_or("");

        self.post_with_params(
            "/data/eventstypedata",
            "VNINDEX",
            &[
                ("eventTypeID", &event_type_id.to_string()),
                ("channelID", &channel_id.to_string()),
                ("code", ""),
                ("catID", "-1"),
                ("fDate", f_date),
                ("tDate", t_date),
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
                ("orderBy", "Date1"),
                ("orderDir", "DESC"),
            ],
        )
        .await
    }

    /// Fetch event type and channel type definitions.
    ///
    /// Returns a tuple of `(event_types, channel_types)`.
    /// The event types describe the top-level categories (Listing, Dividend, Shareholders Meeting).
    /// The channel types describe sub-categories within an event type.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_event_type_definitions(&mut self) -> Result<EventTypeLookup, ApiError> {
        self.post_with_params("/data/eventtypebyid", "VNINDEX", &[])
            .await
    }

    /// Fetch insider trading data for a stock (paginated).
    ///
    /// # Arguments
    ///
    /// * `stock_code` — Stock ticker (e.g., "VNM", "VCB")
    /// * `transfer_type_id` — 0=All, 1=Major shareholder, 2=Internal, 3=Related, 5=Treasury
    /// * `page` — 1-based page number
    /// * `page_size` — Items per page
    /// * `from_date` — Filter start date in YYYY-MM-DD format (optional)
    /// * `to_date` — Filter end date in YYYY-MM-DD format (optional)
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_insider_trading(
        &mut self,
        stock_code: &str,
        transfer_type_id: i32,
        page: i32,
        page_size: i32,
        from_date: Option<&str>,
        to_date: Option<&str>,
    ) -> Result<Vec<InsiderTradingRecord>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        let f_date = from_date.unwrap_or("");
        let t_date = to_date.unwrap_or("");

        self.post_with_params(
            "/data/eventstransferdata",
            &stock_code,
            &[
                ("transferTypeID", &transfer_type_id.to_string()),
                ("stockCode", &stock_code),
                ("fDate", f_date),
                ("tDate", t_date),
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
                ("orderBy", "EventID"),
                ("orderDir", "DESC"),
            ],
        )
        .await
    }

    /// Fetch insider trading transaction type definitions.
    ///
    /// This is a GET endpoint — no CSRF token required.
    /// Returns the available filter types for the insider trading page.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_transfer_types(&self) -> Result<Vec<TransferTypeDef>, ApiError> {
        self.get_json("/data/eventstransfertype", "VNINDEX").await
    }

    // =======================================================================
    // ANALYSIS REPORTS API
    // =======================================================================

    /// Fetch analysis reports (brokerage research) for a stock.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_analysis_reports(
        &mut self,
        stock_code: &str,
    ) -> Result<Vec<AnalysisReport>, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/data/Overview_ReportByStockCode",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("ReportType", "RPT"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // RATINGS API
    // =======================================================================

    /// Fetch technical analysis rating (moving averages, oscillators, overall score).
    ///
    /// Returns the first rating in the data array (typically 1 element).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_technical_rating(
        &mut self,
        stock_code: &str,
    ) -> Result<TechnicalRatingResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/TechnicalRating/GetFinalRanking",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    /// Fetch fundamental analysis rating (6 factor groups + overall letter grade).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_fundamental_rating(
        &mut self,
        stock_code: &str,
    ) -> Result<FundamentalRatingResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/Data/FundamentalRating_GetByStock",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // BONDS API
    // =======================================================================

    /// Fetch bond listings related to a company.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_bond_related(
        &mut self,
        stock_code: &str,
        page_index: i32,
        page_size: i32,
    ) -> Result<BondRelatedResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/Data/GetBondRelated",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("PageIndex", &page_index.to_string()),
                ("PageSize", &page_size.to_string()),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // BONDS MARKET API (Corporate Bonds Overview Page)
    // =======================================================================

    /// Fetch corporate bonds market overview statistics.
    ///
    /// Returns aggregate data: total bonds, total face value, trading volumes,
    /// negotiated volumes, and the latest trading date.
    ///
    /// **Note:** The underlying API uses `multipart/form-data` content type.
    /// This client sends `application/x-www-form-urlencoded` which the server
    /// typically accepts as well. If you encounter HTTP 415 errors, switch to
    /// multipart.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_bond_market_overview(&mut self) -> Result<BondMarketOverview, ApiError> {
        self.post_with_params("/Bonds/GetBondMarketGeneralInfo", "VNINDEX", &[])
            .await
    }

    /// Fetch bond market trading chart data for a given timeframe.
    ///
    /// # Arguments
    ///
    /// * `duration` — Timeframe: `"1D"`, `"5D"`, `"1M"`, `"3M"`, `"6M"`, `"YTD"`, `"1Y"`, `"ALL"`.
    ///
    /// Each returned [`BondChartPoint`] represents one trading day's
    /// aggregated volume and value.
    #[tracing::instrument(skip(self), fields(duration = %duration), err)]
    pub async fn get_bond_market_chart(
        &mut self,
        duration: &str,
    ) -> Result<Vec<BondChartPoint>, ApiError> {
        self.post_with_params(
            "/Bonds/MarketOverviewChart",
            "VNINDEX",
            &[("duration", duration)],
        )
        .await
    }

    /// Fetch paginated bond trading history / listing table.
    ///
    /// # Arguments
    ///
    /// * `page` — 1-based page number.
    /// * `page_size` — Items per page (observed: 10).
    /// * `sort` — Sort direction: `"ASC"` or `"DESC"`.
    /// * `column` — Sort column: `"LotVal"`, `"StockCode"`, `"ClosePrice"`, etc.
    ///
    /// Each item contains a `total_record` field useful for determining
    /// the total number of pages.
    #[tracing::instrument(skip(self), fields(page, page_size, sort = %sort, column = %column), err)]
    pub async fn get_bond_trading_history(
        &mut self,
        page: i32,
        page_size: i32,
        sort: &str,
        column: &str,
    ) -> Result<Vec<BondTradingItem>, ApiError> {
        self.post_with_params(
            "/Bonds/GetLastTrBondHistory",
            "VNINDEX",
            &[
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
                ("sort", sort),
                ("column", column),
            ],
        )
        .await
    }

    /// Fetch paginated bond news by news type.
    ///
    /// # Arguments
    ///
    /// * `news_type` — News category:
    ///   - `5` = Đăng ký giao dịch (Registration)
    ///   - `6` = Thay đổi đăng ký giao dịch (Registration Changes)
    ///   - `4` = Tin thị trường (Market News)
    ///   - `0` = Thông tin công bố (Announcements)
    /// * `page` — 1-based page number.
    /// * `page_size` — Items per page (observed: 5 on overview, more on news page).
    ///
    /// Each item contains a `total_record` field for pagination control.
    #[tracing::instrument(skip(self), fields(news_type, page, page_size), err)]
    pub async fn get_bond_news(
        &mut self,
        news_type: i32,
        page: i32,
        page_size: i32,
    ) -> Result<Vec<BondNewsItem>, ApiError> {
        self.post_with_params(
            "/Bonds/GetBondNews",
            "VNINDEX",
            &[
                ("type", &news_type.to_string()),
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
            ],
        )
        .await
    }

    // =======================================================================
    // PLANNING API
    // =======================================================================

    /// Fetch planned business targets (historical and current year plans).
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_planned_targets(
        &mut self,
        stock_code: &str,
    ) -> Result<PlannedTargetResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/Data/GetData_PlannedTarget",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    /// Fetch implementation status against planned targets.
    #[tracing::instrument(skip(self), fields(stock_code = %stock_code), err)]
    pub async fn get_target_implementation(
        &mut self,
        stock_code: &str,
    ) -> Result<TargetImplementationResponse, ApiError> {
        let stock_code = validate_stock_code(stock_code)?;
        self.post_with_params(
            "/Data/GetData_PlannedTarget_ImplementStatus",
            &stock_code,
            &[
                ("StockCode", &stock_code),
                ("PageType", "1"),
                ("LanguageID", "1"),
            ],
        )
        .await
    }

    // =======================================================================
    // MARKET DATA API
    // =======================================================================

    /// Fetch market index prices (VN-Index, HNX-Index, gold, oil, etc.).
    ///
    /// Common codes: `["VNIndex", "VS100", "HNXINDEX", "CL", "XAUUSDO", "XAUUSDVN", "VN30F1M"]`
    #[tracing::instrument(skip(self), err)]
    pub async fn get_market_prices(
        &mut self,
        codes: &[&str],
    ) -> Result<Vec<MarketPrice>, ApiError> {
        let code_str = codes.join(",");
        let stock_code = "VNINDEX"; // generic referer
        self.post_with_params(
            "/data/getmarketprice",
            stock_code,
            &[("Code", &code_str), ("LanguageID", "1")],
        )
        .await
    }

    /// Fetch headline news for the top navigation banner.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_header_news(&mut self) -> Result<Vec<HeaderNewsItem>, ApiError> {
        self.post_with_params("/data/headernews", "VNINDEX", &[("LanguageID", "1")])
            .await
    }

    /// Fetch a page template by name.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_template_by_name(
        &mut self,
        code: &str,
    ) -> Result<Vec<crate::models::TemplateItem>, ApiError> {
        self.post_with_params(
            "/data/GetTemplateByName",
            "VNINDEX",
            &[("code", code), ("PageType", "1"), ("LanguageID", "1")],
        )
        .await
    }

    /// POST with a pre-built body string (no param encoding).
    ///
    /// Used for endpoints that require array parameters or custom URL encoding.
    async fn post_raw_body<T: DeserializeOwned>(
        &mut self,
        path: &str,
        body: String,
    ) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);

        let request = self
            .client
            .post(&url)
            .headers(self.api_headers("VNINDEX", true))
            .body(body);
        let resp = self.send(request).await?;

        let response_body = Self::check_and_read_text(resp).await?;
        debug!(len = response_body.len(), path = %path, "Received response");
        parse_response(response_body)
    }

    // =======================================================================
    // INDUSTRY / SECTOR API
    // =======================================================================

    /// Fetch the complete GICS industry classification hierarchy.
    ///
    /// Returns 285 items across all 4 levels of the GICS classification.
    /// Level 1 has 11 sectors: Energy(10), Materials(15), Industrials(20),
    /// Consumer Disc.(25), Consumer Staples(30), Health Care(35),
    /// Financials(40), IT(45), Communication(50), Utilities(55), Real Estate(60).
    #[tracing::instrument(skip(self), err)]
    pub async fn get_industry_list(&mut self) -> Result<Vec<IndustryGics>, ApiError> {
        self.post_csrf_only("/Data/GetListIndustryGICS").await
    }

    /// Fetch industry heatmap data with market cap, price changes, and stock counts.
    ///
    /// # Arguments
    ///
    /// * `industry_code` — `0` for all industries, or a specific GICS code.
    /// * `detail_industry` — include sub-industry detail when `true`.
    /// * `detail_stock_code` — include individual stock detail when `true` (fills `stock_full_name`).
    #[tracing::instrument(
        skip(self),
        fields(industry_code, detail_industry, detail_stock_code),
        err
    )]
    pub async fn get_industry_heatmap(
        &mut self,
        industry_code: i32,
        detail_industry: bool,
        detail_stock_code: bool,
    ) -> Result<Vec<IndustryHeatmapItem>, ApiError> {
        self.post_with_params(
            "/Data/GetHeatmapGICS",
            "VNINDEX",
            &[
                ("industryCode", &industry_code.to_string()),
                (
                    "detailIndustry",
                    if detail_industry { "true" } else { "false" },
                ),
                (
                    "detailStockCode",
                    if detail_stock_code { "true" } else { "false" },
                ),
            ],
        )
        .await
    }

    /// Fetch historical sector index values for charting.
    ///
    /// # Arguments
    ///
    /// * `start_date` — Start date in `dd/MM/yyyy` format (slashes will be URL-encoded).
    /// * `end_date` — End date in `dd/MM/yyyy` format.
    /// * `sector_codes` — List of GICS Level 1 codes to include (e.g., `[10, 15, 20]`).
    /// * `group_id` — Sector group ID (1 = default).
    /// * `index_id` — Index type (1 = Price/Giá).
    #[tracing::instrument(skip(self), fields(start_date = %start_date, end_date = %end_date, group_id, index_id), err)]
    pub async fn get_industry_index(
        &mut self,
        start_date: &str,
        end_date: &str,
        sector_codes: &[i32],
        group_id: i32,
        index_id: i32,
    ) -> Result<Vec<IndustryIndexPoint>, ApiError> {
        let token = self.require_csrf_token()?;

        let group_id = group_id.to_string();
        let index_id = index_id.to_string();
        let sectors: Vec<String> = sector_codes.iter().map(i32::to_string).collect();
        let body = encode_form(
            [
                ("startDate", start_date),
                ("endDate", end_date),
                ("groupID", group_id.as_str()),
                ("indexID", index_id.as_str()),
            ]
            .into_iter()
            .chain(sectors.iter().map(|code| ("listSector[]", code.as_str())))
            .chain(std::iter::once((
                "__RequestVerificationToken",
                token.as_str(),
            ))),
        );

        self.post_raw_body("/Data/GetGICSIndex", body).await
    }

    /// Fetch industry performance percentages across multiple timeframes.
    ///
    /// # Arguments
    ///
    /// * `level` — Industry level (1, 2, 3, or 4). Level 1 returns 88 items
    ///   (11 industries × 8 timeframes).
    ///
    /// Timeframe codes: 1=Today, 2=1W, 3=1M, 4=6M, 5=YTD, 6=1Y, 7=5Y, 8=10Y.
    #[tracing::instrument(skip(self), fields(level), err)]
    pub async fn get_industry_performance(
        &mut self,
        level: i32,
    ) -> Result<Vec<IndustryPerformance>, ApiError> {
        self.post_with_params(
            "/Data/GetGICSPerformance",
            "VNINDEX",
            &[("level", &level.to_string())],
        )
        .await
    }

    /// Fetch financial index group definitions and individual index definitions.
    ///
    /// Returns 10 groups (Valuation, Profitability, Growth, Liquidity, Efficiency,
    /// Leverage, Cashflow, Cost Structure, Short-term Asset Structure,
    /// Long-term Asset Structure) with their associated individual indices.
    #[tracing::instrument(skip(self), err)]
    pub async fn get_finance_index_groups(
        &mut self,
    ) -> Result<FinanceIndexGroupResponse, ApiError> {
        self.post_csrf_only("/Data/GetFinanceIndexGroupForSector")
            .await
    }

    /// Fetch financial index values for all industries.
    ///
    /// # Arguments
    ///
    /// * `report_type` — `"Y"` for Yearly, `"Q"` for Quarterly.
    #[tracing::instrument(skip(self), fields(report_type = %report_type), err)]
    pub async fn get_industry_financial_index(
        &mut self,
        report_type: &str,
    ) -> Result<Vec<IndustryFinancialIndex>, ApiError> {
        self.post_with_params(
            "/Data/GetGICSFinancialIndex",
            "VNINDEX",
            &[("reportType", report_type)],
        )
        .await
    }

    /// Fetch industry analysis reports from brokerages and research firms.
    ///
    /// Returns 9 most recent reports (no pagination).
    #[tracing::instrument(skip(self), err)]
    pub async fn get_sector_reports(&mut self) -> Result<Vec<SectorReport>, ApiError> {
        self.post_csrf_only("/Data/GetReportSector").await
    }

    /// Fetch paginated industry news.
    ///
    /// # Arguments
    ///
    /// * `page` — 1-based page number.
    /// * `page_size` — Items per page (typically 10).
    /// * `industry_code` — `0` for all industries, or a specific GICS code.
    ///
    /// Returns an empty vec when no more pages are available.
    #[tracing::instrument(skip(self), fields(page, page_size, industry_code), err)]
    pub async fn get_industry_news(
        &mut self,
        page: i32,
        page_size: i32,
        industry_code: i32,
    ) -> Result<Vec<IndustryNewsItem>, ApiError> {
        self.post_with_params(
            "/Data/GetIndustryNews",
            "VNINDEX",
            &[
                ("page", &page.to_string()),
                ("pageSize", &page_size.to_string()),
                ("industryCode", &industry_code.to_string()),
            ],
        )
        .await
    }
}

// ---------------------------------------------------------------------------
// Utility Functions
// ---------------------------------------------------------------------------

fn validate_stock_code(code: &str) -> Result<String, ApiError> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(ApiError::InvalidStockCode(code.to_string()));
    }
    Ok(trimmed.to_uppercase())
}

#[allow(dead_code)]
/// Find the anti-forgery token the page's own AJAX calls use.
///
/// The page is minified and its attributes are unquoted
/// (`<input name=__RequestVerificationToken type=hidden value=...>`, seen
/// 2026-11-19), and the same name also appears inside a jQuery selector, so a
/// plain string search picks the wrong value. Prefer the input inside
/// `#__CHART_AjaxAntiForgeryForm`, then fall back to the first matching input.
fn extract_csrf_token(html: &str) -> Option<String> {
    const NAME: &str = "__RequestVerificationToken";
    html.find("__CHART_AjaxAntiForgeryForm")
        .and_then(|pos| find_input_value(&html[pos..], NAME))
        .or_else(|| find_input_value(html, NAME))
}

/// Value of the first `<input>` whose `name` attribute equals `name`.
fn find_input_value(html: &str, name: &str) -> Option<String> {
    let mut rest = html;
    while let Some(pos) = rest.find("<input") {
        let tag = &rest[pos..];
        let end = tag.find('>')?;
        let attrs = &tag[..end];
        if attr_value(attrs, "name") == Some(name)
            && let Some(value) = attr_value(attrs, "value").filter(|v| !v.is_empty())
        {
            return Some(value.to_string());
        }
        rest = &tag[end..];
    }
    None
}

/// Read one attribute from the inside of a tag, accepting `a="v"`, `a='v'` and `a=v`.
fn attr_value<'a>(attrs: &'a str, attr: &str) -> Option<&'a str> {
    let mut from = 0;
    while let Some(rel) = attrs[from..].find(attr) {
        let idx = from + rel;
        let after = idx + attr.len();
        let starts_attr = attrs[..idx]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
        if starts_attr && let Some(raw) = attrs[after..].strip_prefix('=') {
            return Some(match raw.chars().next() {
                Some(quote @ ('"' | '\'')) => raw[1..].split(quote).next().unwrap_or(""),
                _ => raw
                    .split(char::is_whitespace)
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('/'),
            });
        }
        from = after;
    }
    None
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "%20".into(),
            '!' => "%21".into(),
            '"' => "%22".into(),
            '#' => "%23".into(),
            '$' => "%24".into(),
            '%' => "%25".into(),
            '&' => "%26".into(),
            '\'' => "%27".into(),
            '(' => "%28".into(),
            ')' => "%29".into(),
            '*' => "%2A".into(),
            '+' => "%2B".into(),
            ',' => "%2C".into(),
            '/' => "%2F".into(),
            ':' => "%3A".into(),
            ';' => "%3B".into(),
            '=' => "%3D".into(),
            '?' => "%3F".into(),
            '@' => "%40".into(),
            '[' => "%5B".into(),
            ']' => "%5D".into(),
            _ => c.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Deserialize a JSON body, turning Vietstock's own error envelope
/// (`{"error":{"ErrorCode":...,"ErrorMessage":...}}`) into [`ApiError::Vietstock`].
fn parse_response<T: DeserializeOwned>(body: String) -> Result<T, ApiError> {
    #[derive(serde::Deserialize)]
    struct Envelope {
        error: VietstockError,
    }
    #[derive(serde::Deserialize)]
    struct VietstockError {
        #[serde(rename = "ErrorCode", default)]
        code: String,
        #[serde(rename = "ErrorMessage", default)]
        message: String,
    }

    match serde_json::from_str(&body) {
        Ok(value) => Ok(value),
        Err(parse_error) => match serde_json::from_str::<Envelope>(&body) {
            Ok(Envelope { error }) => Err(ApiError::Vietstock {
                code: error.code,
                message: error.message,
            }),
            Err(_) => Err(ApiError::UnexpectedResponse {
                parse_error: parse_error.to_string(),
                body,
            }),
        },
    }
}

/// Encode `key=value` pairs as `application/x-www-form-urlencoded`, the way a
/// browser form or jQuery `$.param` does (`[`/`]` and `/` are percent-encoded).
/// Form fields of `POST /data/financeinfo`, without the CSRF token.
fn finance_info_form(
    stock_code: &str,
    report_term_type: i32,
    report_type: &str,
    unit: u64,
) -> [(&'static str, String); 6] {
    [
        ("Code", stock_code.to_string()),
        ("Page", "1".to_string()),
        ("PageSize", "4".to_string()),
        ("ReportTermType", report_term_type.to_string()),
        ("ReportType", report_type.to_string()),
        ("Unit", unit.to_string()),
    ]
}

fn encode_form<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_response_maps_vietstock_error_envelope() {
        let body = r#"{"error":{"ErrorCode":"RequestUpgradeAccount_Permission","ErrorMessage":"Yêu cầu nâng cấp tài khoản","IsLimitData":false}}"#;
        let err = super::parse_response::<Vec<i32>>(body.to_string()).unwrap_err();
        assert!(matches!(
            err,
            super::ApiError::Vietstock { ref code, .. } if code == "RequestUpgradeAccount_Permission"
        ));
        assert!(err.is_permission_denied());
    }

    #[test]
    fn csrf_token_from_minified_unquoted_markup() {
        // Shape of the page on 2026-11-19: unquoted attributes, and the token name
        // also appears in a jQuery selector before any quoted `value="..."`.
        let html = r##"<body><form action=/VNM/tai-tai-lieu.htm id=__CHART_AjaxAntiForgeryForm method=post><input name=__RequestVerificationToken type=hidden value=AJAX_tok-1_x><script>_tokenString=$('input[name="__RequestVerificationToken"]',$("#__CHART_AjaxAntiForgeryForm")).val();</script></form><form action=/Account/Login id=form0><input name=__RequestVerificationToken type=hidden value=LOGIN_tok><input id="x" value="decoy"></form>"##;
        assert_eq!(
            super::extract_csrf_token(html).as_deref(),
            Some("AJAX_tok-1_x")
        );
    }

    #[test]
    fn csrf_token_from_quoted_markup() {
        let html = r#"<form><input type="hidden" name="__RequestVerificationToken" value="quoted_tok" /></form>"#;
        assert_eq!(
            super::extract_csrf_token(html).as_deref(),
            Some("quoted_tok")
        );
    }

    #[test]
    fn csrf_token_missing() {
        assert_eq!(
            super::extract_csrf_token("<html><input name=other value=x></html>"),
            None
        );
    }

    #[test]
    fn encode_form_escapes_reserved_characters() {
        let body = super::encode_form([("Email", "a+b@example.com"), ("Password", "p&ss=w0rd /x")]);
        assert_eq!(
            body,
            "Email=a%2Bb%40example.com&Password=p%26ss%3Dw0rd+%2Fx"
        );
    }

    #[test]
    fn encode_form_keeps_repeated_array_keys_in_order() {
        let body = super::encode_form([
            ("startDate", "01/01/2026"),
            ("listSector[]", "10"),
            ("listSector[]", "15"),
        ]);
        assert_eq!(
            body,
            "startDate=01%2F01%2F2026&listSector%5B%5D=10&listSector%5B%5D=15"
        );
    }

    use super::*;

    /// The summary's `Unit` is a parameter; the default request is unchanged
    /// (billions of dong, the body sent before the parameter existed).
    #[test]
    fn finance_info_form_carries_the_unit() {
        let body = |unit| {
            let form = finance_info_form("VNM", 2, "BCTQ", unit);
            encode_form(form.iter().map(|(k, v)| (*k, v.as_str())))
        };
        assert_eq!(
            body(FINANCE_INFO_DEFAULT_UNIT),
            "Code=VNM&Page=1&PageSize=4&ReportTermType=2&ReportType=BCTQ&Unit=1000000000"
        );
        assert_eq!(
            body(FINANCE_INFO_UNIT_DONG),
            "Code=VNM&Page=1&PageSize=4&ReportTermType=2&ReportType=BCTQ&Unit=1"
        );
    }

    #[test]
    fn test_validate_stock_code() {
        assert_eq!(validate_stock_code("VNM").unwrap(), "VNM");
        assert_eq!(validate_stock_code(" vcb ").unwrap(), "VCB");
        assert!(validate_stock_code("").is_err());
    }

    #[test]
    fn test_builder_defaults() {
        let client = VietstockClient::builder().build().unwrap();
        assert!(!client.is_session_initialized());
        assert!(!client.is_authenticated());
        assert_eq!(
            client.limiter.interval,
            Duration::from_millis(DEFAULT_RATE_LIMIT_MS)
        );
        assert_eq!(
            client.download_timeout,
            Duration::from_secs(DOWNLOAD_TIMEOUT_SECS)
        );
        let client = VietstockClient::builder()
            .download_timeout(Duration::from_secs(900))
            .build()
            .unwrap();
        assert_eq!(client.download_timeout, Duration::from_secs(900));
    }

    #[tokio::test(start_paused = true)]
    async fn limiter_spaces_request_starts() {
        let limiter = RateLimiter::new(Duration::from_secs(1));
        let t0 = tokio::time::Instant::now();
        let mut starts = Vec::new();
        for _ in 0..4 {
            limiter.wait().await;
            starts.push(t0.elapsed());
        }
        // The first request goes at once (the first one after a new session included).
        assert_eq!(
            starts,
            [0, 1, 2, 3].map(Duration::from_secs).to_vec(),
            "starts: {starts:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn limiter_does_not_wait_after_a_long_gap() {
        let limiter = RateLimiter::new(Duration::from_secs(1));
        limiter.wait().await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        let t0 = tokio::time::Instant::now();
        limiter.wait().await;
        assert_eq!(t0.elapsed(), Duration::ZERO);
        // ...but the request right after it does.
        limiter.wait().await;
        assert_eq!(t0.elapsed(), Duration::from_secs(1));
    }

    #[tokio::test(start_paused = true)]
    async fn limiter_queues_concurrent_callers() {
        let limiter = std::sync::Arc::new(RateLimiter::new(Duration::from_millis(500)));
        let t0 = tokio::time::Instant::now();
        let tasks: Vec<_> = (0..3)
            .map(|_| {
                let limiter = limiter.clone();
                tokio::spawn(async move {
                    limiter.wait().await;
                    t0.elapsed()
                })
            })
            .collect();
        let mut starts = Vec::new();
        for task in tasks {
            starts.push(task.await.unwrap());
        }
        starts.sort();
        assert_eq!(starts, [0, 500, 1000].map(Duration::from_millis).to_vec());
    }

    /// Requests that need no session (`get_years`, `get_document_types`, file
    /// downloads) share the session's limiter: a failing request still takes its
    /// slot, so the next one waits.
    #[tokio::test(start_paused = true)]
    async fn no_token_paths_take_a_limiter_slot() {
        let client = VietstockClient::builder()
            .rate_limit_ms(1000)
            .build()
            .unwrap();
        // https_only: a plain-HTTP URL fails before any network I/O, after the slot.
        assert!(
            client
                .download_url("http://127.0.0.1:9/x.pdf")
                .await
                .is_err()
        );
        let t0 = tokio::time::Instant::now();
        assert!(
            client
                .download_url("http://127.0.0.1:9/y.pdf")
                .await
                .is_err()
        );
        assert_eq!(t0.elapsed(), Duration::from_secs(1));
    }

    #[test]
    fn test_get_login_url_google() {
        let client = VietstockClient::new().unwrap();
        let url = client.get_login_url_google("https://finance.vietstock.vn");
        assert!(url.contains("LoginGooglePlus"));
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello"), "hello");
        assert_eq!(urlencoding("hello world"), "hello%20world");
    }
}
