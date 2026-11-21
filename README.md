# Vietstock Finance Document API

Reverse-engineered API specification and Rust client for [finance.vietstock.vn](https://finance.vietstock.vn) — a Vietnamese financial data portal.

## Overview

| Property | Value |
|----------|-------|
| **Base URL** | `https://finance.vietstock.vn` |
| **Authentication** | Session-based (ASP.NET session cookies) with CSRF token protection |
| **Content-Type (response)** | `application/json; charset=utf-8` |
| **Content-Type (request)** | `application/x-www-form-urlencoded` (all POST requests) |
| **Login Required** | No for documents and files; yes (upgraded account) for detailed statement tables |
| **Framework** | ASP.NET MVC with server-side rendering (SSR) |
| **Pagination** | `page` form field, 20 documents per page |

## Quickstart (Rust)

```rust
use vietstock::VietstockClient;

#[tokio::main]
async fn main() -> Result<(), vietstock::ApiError> {
    let mut client = VietstockClient::new()?;
    client.init_session("VNM").await?;

    // Whole document library (20 per page), newest first.
    let docs = client.get_all_documents("VNM", None).await?;
    tracing::info!(count = docs.len(), "VNM documents");

    if let Some(doc) = docs.iter().find(|d| d.url.contains("/BCTC/")) {
        let bytes = client.download_document(doc).await?;
        tracing::info!(title = %doc.title, bytes = bytes.len(), "downloaded");
    }
    Ok(())
}
```

Examples: `cargo run -p vietstock-rs --example get_documents -- VNM --all`,
`--example get_financial_statement -- VNM CDKT NAM`, `--example download_document -- VNM ./out`.

## Access pattern (verified 2026-11-19)

How the site behaves today and how this crate deals with it. Re-check this section
when something breaks; the site changes without notice.

**Transport.** Plain HTTP through [`wreq`](https://crates.io/crates/wreq) with a
Chrome 149 / Windows TLS + HTTP/2 profile. No headless browser is needed or used.

- `finance.vietstock.vn` answers `Accept-Encoding: deflate` with **raw deflate** (no
  zlib header). Rust decoders (wreq, and the Obscura headless browser) reject it.
  The crate therefore enables only gzip/br/zstd decoding and does not send
  `deflate`; the server then picks gzip.

**Session.**

1. `GET /{CODE}/tai-tai-lieu.htm` sets `ASP.NET_SessionId` and the
   `__RequestVerificationToken` cookie (both `HttpOnly`, `secure`).
2. The form token is in the HTML. The page is minified with **unquoted attributes**:
   `<form ... id=__CHART_AjaxAntiForgeryForm ...><input name=__RequestVerificationToken type=hidden value=TOKEN>`.
   The same name also appears inside a jQuery selector, so a naive search for
   `name="__RequestVerificationToken"` returns a wrong value. Take the input inside
   `#__CHART_AjaxAntiForgeryForm` (that is what the page's own scripts read).
3. Send the cookie jar plus the form token on every POST.

**AJAX request shape** (recorded from the page's own jQuery calls with Obscura):

| Header | Value |
|---|---|
| `X-Requested-With` | `XMLHttpRequest` |
| `Content-Type` | `application/x-www-form-urlencoded; charset=UTF-8` |
| `Accept` | `*/*` |
| `Referer` | `https://finance.vietstock.vn/{CODE}/tai-tai-lieu.htm` |

Bodies are standard form encoding with the token as a field, for example
`code=VNM&s=0&t=&__RequestVerificationToken=…` (`/company/tradinginfo`) or
`stockCode=VNM&__RequestVerificationToken=…` (`/data/GetIconByStockCodeForIR`).
Array fields are sent jQuery-style (`listSector%5B%5D=10`).

**What an anonymous session can read.**

| Data | Anonymous | Notes |
|---|---|---|
| Document library `POST /data/getdocument` | **Yes, full history** | `code`, `page` (20 per page), optional `year`; each row has `TotalRow`. VNM: 424 files, 2003–2026 |
| Files on `static2.vietstock.vn` | Yes | `.pdf`, `.zip` and `.rar` (older filings) |
| Years `POST /data/getyearreport` | Yes | no token needed |
| Statement line items `GetListReportNorm_*` | Yes | e.g. 143 balance-sheet lines for VNM |
| Statement periods/values `*_GetListReportData`, `GetReportDataDetailValue_*` | **No** | `{"error":{"ErrorCode":"RequestUpgradeAccount_Permission"}}` for every scope/audit filter tried; surfaced as `ApiError::Vietstock` (`is_permission_denied()`) |
| Summary `POST /data/financeinfo` | Yes | 4 recent quarters, ~6 lines per section, with consolidated/audit labels. The annual call (`ReportTermType=1`, `BCTN`) answers an empty body (2026-11-19 and 11-20) |

**Summary unit.** The `Unit` field of `/data/financeinfo` is dong per value.
`get_finance_info` sends the site's own `Unit=1000000000` (billions, unchanged);
`get_finance_info_in_unit(code, term, type, unit)` sends any other, e.g.
`FINANCE_INFO_UNIT_DONG` (`1`). Checked 2026-11-20 on VNM: with `Unit=1` every
money line comes back in whole dong (total assets Q3/2026 `57892388235962.0`
against `57892.0` in billions), equal to CafeF to the dong; ratio lines (EPS,
ROS, P/E) are the same in every unit. Values are `f64`, exact up to 2^53 dong.
The response's own legends (elements 2 and 3) read `AuditedStatus` `KT`
audited, `SX` reviewed, `CKT` unaudited, and `United` `HN` consolidated, `ĐL`
stand-alone, `CTM` parent.

**Point-in-time caveat.** A document's `LastUpdate` is when Vietstock last touched
the file, not when it was published: VNM's audited 2010 parent-company statement
shows 2016. Record your own first-seen time.

**Browsers tried for discovery.** CloakBrowser now needs a license key tied to an
e-mail address, so it was not used. `obscura --stealth serve` loads the page only
after forcing `Accept-Encoding: gzip, br` through CDP, and its XHRs still fail on the
raw deflate, so it can record request shapes but not run the page's data flow.

## Authentication & Session Management

Vietstock uses a two-part authentication mechanism:

1. **`ASP.NET_SessionId`** — Session identifier cookie, required for all requests
2. **`__RequestVerificationToken`** — Anti-CSRF token, stored as a cookie AND submitted in POST request bodies

### Token Acquisition

```
Step 1: GET https://finance.vietstock.vn/{STOCK_CODE}/tai-tai-lieu.htm
        → Extract cookies from Set-Cookie response headers

Step 2: Use both cookies on all subsequent requests
        → For POST endpoints that require CSRF, include the token value
          in the request body as __RequestVerificationToken={VALUE}
```

### CSRF Requirements by Endpoint

| Endpoint | CSRF Required |
|----------|:---:|
| `GET /data/getdocumenttype` | No |
| `POST /data/getyearreport` | No |
| `POST /data/getdocument` | **Yes** |
| `POST /company/tradinginfo` | **Yes** |

## Document Types Available

| TypeID | Vietnamese Name | English Name |
|--------|----------------|--------------|
| 1 | Báo cáo tài chính | Financial Statement |
| 2 | Báo cáo thường niên | Annual Report |
| 3 | Bản cáo bạch | Prospectus |
| 4 | Nghị quyết ĐHĐCĐ | Resolution of Shareholders' Meeting |
| 5 | Tài liệu ĐHĐCĐ | Shareholders' Meeting Documents |
| 6 | Khác | Others/Regulations |
| 8 | Giải trình KQKD | Explanation for Financial Statement |
| 9 | Báo cáo quản trị | Corporate Governance Report |
| 10 | Tỷ lệ vốn khả dụng | Liquidity Ratio Report |
| 23 | Nghị quyết HĐQT | Board of Management |
| 31 | Tỷ lệ an toàn vốn | Capital Adequacy Ratio Report |

## Date Handling

All dates use Microsoft's JSON serialization format:

```
/Date(1780368384730)/
```

Where the number is **Unix time in milliseconds** (UTC). The Rust client includes a custom serde deserializer that handles this format transparently.

## Rate Limiting

No rate-limiting headers were observed in responses. The Rust client enforces a conservative rate limit of **1 request/second** to avoid being blocked. This is configurable via `VietstockClient::builder().rate_limit_ms(...)`.

Every request goes through the limiter: the session page (`init_session`), the
endpoints that need no token (`get_years`, `get_document_types`,
`get_transfer_types`) and file downloads included. The limiter spaces request
starts and works through `&self`, so the first POST after `init_session` waits its
turn too.

## Timeouts

API and page requests time out after 30 s (`builder().timeout(...)`), measured from
connecting until the whole body is read. File downloads (`download_url`,
`download_document`) use a separate, longer timeout, 300 s by default
(`builder().download_timeout(...)`), because large ZIP/RAR filings from the CDN can
take longer than 30 s. `download_url(&str)` needs only the file URL and no session;
`download_document(&Document)` is a thin wrapper over it.

## File Formats

Documents are stored on a CDN (`static2.vietstock.vn`). The library lists older
files (before about 2022) with `http://` URLs; the client is HTTPS-only, so
`download_url` fetches a plain-HTTP link on a `vietstock.vn` host over HTTPS (the
CDN serves the same bytes there) and refuses plain HTTP to any other host. Files are
served as:
- `.zip` — Multi-file archives (may contain PDFs + supplementary documents); **note: trailing spaces on extension** (e.g., `".zip "`)
- `.pdf` — Single PDF documents

Prices are in **VND** as integers (e.g., `45100` = 45,100 VND).

## Files

| File | Description |
|------|-------------|
| `README.md` | This file — overview, authentication, quickstart |
| `endpoints.md` | Full endpoint catalog with request/response schemas |
| `raw_requests.md` | Raw network traces for verification |

## Gaps & Limitations

- **Error responses:** No errors were triggered during exploration. Expected 400 for missing CSRF, 403 for expired session, empty arrays for invalid stocks.
- **CSRF token lifetime:** Unknown — tokens likely expire with the ASP.NET session.
- **Filtered API calls:** The page uses SSR with URL params (`?doctype={ID}`, `?year={YYYY}`) — it's unclear whether the API returns filtered results or all documents with client-side filtering.
- **Premium endpoints:** Login-required features were not explored.
- **`s` and `t` parameters** in `/company/tradinginfo`: purpose unknown (`s=0`, `t=` empty).
