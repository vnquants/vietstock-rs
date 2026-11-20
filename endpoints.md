# Endpoint Catalog — Vietstock Finance API

Base URL: `https://finance.vietstock.vn`

---

## 1. GET /data/getdocumenttype

Fetch all document type categories.

| Property | Value |
|----------|-------|
| **Method** | GET |
| **Auth** | None |
| **Request Body** | None |
| **Response** | `Vec<DocumentType>` |

### Request Headers
```
x-requested-with: XMLHttpRequest
```

### DocumentType Schema

| Field | Type | Nullable | Description |
|-------|------|:--------:|-------------|
| `DocumentTypeID` | `i32` | No | Unique category identifier |
| `DocumentName` | `string` | No | Vietnamese display name |
| `DocumentNameEn` | `string` | No | English display name |
| `DisplayOrder` | `i32` | No | Sort order (1-based) |

### Example Response
```json
[
  {"DocumentTypeID":1, "DocumentName":"Báo cáo tài chính", "DocumentNameEn":"Financial Statement", "DisplayOrder":1},
  {"DocumentTypeID":23,"DocumentName":"Nghị quyết HĐQT", "DocumentNameEn":"Board of Management", "DisplayOrder":1},
  {"DocumentTypeID":8, "DocumentName":"Giải trình KQKD", "DocumentNameEn":"Explanation for Financial Statement", "DisplayOrder":3},
  {"DocumentTypeID":9, "DocumentName":"Báo cáo quản trị", "DocumentNameEn":"Corporate Governance Report", "DisplayOrder":4},
  {"DocumentTypeID":2, "DocumentName":"Báo cáo thường niên", "DocumentNameEn":"Annual Report", "DisplayOrder":5},
  {"DocumentTypeID":4, "DocumentName":"Nghị quyết ĐHĐCĐ", "DocumentNameEn":"Resolution of Shareholders' Meeting", "DisplayOrder":6},
  {"DocumentTypeID":5, "DocumentName":"Tài liệu ĐHĐCĐ", "DocumentNameEn":"Shareholders' Meeting Documents", "DisplayOrder":7},
  {"DocumentTypeID":3, "DocumentName":"Bản cáo bạch", "DocumentNameEn":"Prospectus", "DisplayOrder":8},
  {"DocumentTypeID":10,"DocumentName":"Tỷ lệ vốn khả dụng", "DocumentNameEn":"Liquidity Ratio Report", "DisplayOrder":9},
  {"DocumentTypeID":31,"DocumentName":"Tỷ lệ an toàn vốn", "DocumentNameEn":"Capital Adequacy Ratio Report", "DisplayOrder":10},
  {"DocumentTypeID":6, "DocumentName":"Khác", "DocumentNameEn":"Regulations", "DisplayOrder":10}
]
```

---

## 2. POST /data/getyearreport

Fetch available years for which documents exist for a stock.

| Property | Value |
|----------|-------|
| **Method** | POST |
| **Auth** | None (no CSRF token needed) |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Response** | `Vec<i32>` (descending order) |

### Request Body (form-urlencoded)

| Parameter | Type | Required | Description |
|-----------|------|:--------:|-------------|
| `code` | string | Yes | Stock ticker (e.g., `ACV`, `VNM`) |

### Example Request
```
POST /data/getyearreport
Content-Type: application/x-www-form-urlencoded

code=ACV
```

### Example Response (ACV)
```json
[2026, 2025, 2024, 2023, 2022, 2021, 2020, 2019, 2018, 2017, 2016, 2015, 2014, 2013, 2012]
```

### Example Response (VNM)
```json
[2026, 2025, 2024, 2023, 2022, 2021, 2020, 2019, 2018, 2017, 2016, 2015, 2014, 2013, 2012, 2011, 2010, 2009, 2008, 2007, 2006, 2005, 2004, 2003]
```

---

## 3. POST /data/getdocument (PRIMARY)

Fetch ALL documents for a stock. Returns complete metadata and download URLs.

| Property | Value |
|----------|-------|
| **Method** | POST |
| **Auth** | **CSRF token required** |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Pagination** | None — returns ALL documents |
| **Response** | `Vec<Document>` |

### Request Body (form-urlencoded)

| Parameter | Type | Required | Description |
|-----------|------|:--------:|-------------|
| `code` | string | Yes | Stock ticker (e.g., `ACV`, `VNM`) |
| `__RequestVerificationToken` | string | Yes | CSRF token value |

### Request Headers
```
Content-Type: application/x-www-form-urlencoded; charset=UTF-8
x-requested-with: XMLHttpRequest
Referer: https://finance.vietstock.vn/{STOCK_CODE}/tai-tai-lieu.htm
Cookie: ASP.NET_SessionId=...; __RequestVerificationToken=...
```

### Example Request
```
POST /data/getdocument
Content-Type: application/x-www-form-urlencoded

code=ACV&__RequestVerificationToken=<redacted>
```

### Document Schema

| Field | Type | Nullable | Description |
|-------|------|:--------:|-------------|
| `FileExt` | `string` | No | File extension (`.zip`, `.pdf`). **Trailing spaces** may be present. |
| `UpdateTime` | `null` | Always | Always `null` in observed data |
| `TotalRow` | `i32` | No | Total document count (redundant on each item) |
| `FileInfoID` | `i64` | No | Unique document identifier |
| `Url` | `string` | No | Direct download URL on `static2.vietstock.vn` CDN |
| `Title` | `string` | No | Short title (Vietnamese) |
| `FullName` | `string` | No | Full descriptive name (Vietnamese) |
| `LastUpdate` | `string` | No | Microsoft `/Date(ms)/` format (Unix milliseconds) |

### Example Response (2 of 223 items for ACV)
```json
[
  {
    "FileExt": ".zip ",
    "UpdateTime": null,
    "TotalRow": 223,
    "FileInfoID": 548794,
    "Url": "https://static2.vietstock.vn/data/UPCOM/2026/TAI LIEU DHCD/VN/ACV_TAILIEU_DHDCD thuong nien_2026.zip",
    "Title": "Tài liệu đại hội đồng cổ đông thường niên năm 2026",
    "FullName": "Tài liệu đại hội đồng cổ đông thường niên năm 2026",
    "LastUpdate": "/Date(1780368384730)/"
  },
  {
    "FileExt": ".zip ",
    "UpdateTime": null,
    "TotalRow": 223,
    "FileInfoID": 546047,
    "Url": "https://static2.vietstock.vn/data/UPCOM/2026/NGHI QUYET HDQT/VN/4_acv_2026_5_6_9221f8a_en_announcement(NhieuFile).zip",
    "Title": "Nghị quyết Hội đồng quản trị số 248/NQ-HĐQT tháng 05/2026...",
    "FullName": "Nghị quyết Hội đồng quản trị số 248/NQ-HĐQT tháng 05/2026...",
    "LastUpdate": "/Date(1778207635757)/"
  }
]
```

### Filtering Behavior

The API itself does NOT accept filtering parameters. The Vietstock website filters documents via the page URL:
- `?doctype={DocumentTypeID}` — filter by document type
- `?year={YYYY}` — filter by year

When the page reloads with these query parameters, the server renders a page that either:
1. Calls the API with server-side filtered results, OR
2. Returns all documents with client-side filtering

**Recommendation:** Fetch all documents and filter client-side by year and document type (inferable from the `Url` path pattern).

### Download URL Pattern

```
https://static2.vietstock.vn/data/{EXCHANGE}/{YEAR}/{FOLDER}/{LANG}/{FILENAME}
```

| Segment | Values |
|---------|--------|
| `{EXCHANGE}` | `UPCOM`, `HNX`, `HOSE` |
| `{YEAR}` | Document year |
| `{FOLDER}` | `BCTC`, `TAI LIEU DHCD`, `NGHI QUYET HDQT`, `NGHI QUYET DHCD`, `BCTN`, `BCTHQT`, `KHAC`, etc. |
| `{LANG}` | `VN` (Vietnamese) |
| `{FILENAME}` | Full filename including stock code and description |

### Response Sizes

| Stock | Documents |
|-------|:---------:|
| ACV | 223 |
| VNM | 415 |

---

## 4. POST /company/tradinginfo

Fetch real-time trading information for a stock.

| Property | Value |
|----------|-------|
| **Method** | POST |
| **Auth** | CSRF token required |
| **Content-Type** | `application/x-www-form-urlencoded` |
| **Response** | `TradingInfo` (single object) |

### Request Body (form-urlencoded)

| Parameter | Type | Required | Description |
|-----------|------|:--------:|-------------|
| `code` | string | Yes | Stock ticker |
| `s` | int | [UNKNOWN] | Always `0` in observed calls |
| `t` | string | [UNKNOWN] | Always empty in observed calls |
| `__RequestVerificationToken` | string | Yes | CSRF token |

### TradingInfo Schema

| Field | Type | Description |
|-------|------|-------------|
| `StockCode` | `string` | Stock ticker |
| `TradingDate` | `/Date(ms)/` | Trading date |
| `KLCPLH` | `i64` | [INFERRED] Listed shares outstanding |
| `KLCPNY` | `i64` | [INFERRED] Listed shares (total) |
| `PriorClosePrice` | `i64` | Previous close (VND) |
| `CeilingPrice` | `i64` | Ceiling price (VND) |
| `FloorPrice` | `i64` | Floor price (VND) |
| `TotalVol` | `i64` | Total trading volume |
| `TotalVal` | `i64` | Total trading value (VND) |
| `MarketCapital` | `i64` | Market capitalization (VND) |
| `HighestPrice` | `i64` | Day high (VND) |
| `LowestPrice` | `i64` | Day low (VND) |
| `OpenPrice` | `i64` | Opening price (VND) |
| `LastPrice` | `i64` | Last traded price (VND) |
| `AvrPrice` | `i64` | VWAP (VND) |
| `Change` | `i64` | Price change (VND) |
| `PerChange` | `f64` | Percent change |
| `Min52W` | `i64` | 52-week low (VND) |
| `Max52W` | `i64` | 52-week high (VND) |
| `EPS` | `f64` | Earnings per share (VND) |
| `PE` | `f64` | P/E ratio |
| `BVPS` | `f64` | Book value per share (VND) |
| `PB` | `f64` | P/B ratio |

### Example Response (ACV)
```json
{
  "StockCode": "ACV",
  "TradingDate": "/Date(1781596800673)/",
  "KLCPLH": 3582324023,
  "KLCPNY": 3582847523,
  "PriorClosePrice": 44800,
  "CeilingPrice": 51500,
  "FloorPrice": 38100,
  "TotalVol": 759200,
  "TotalVal": 34381960000,
  "MarketCapital": 161562813437300,
  "HighestPrice": 46000,
  "LowestPrice": 44900,
  "OpenPrice": 45400,
  "LastPrice": 45100,
  "AvrPrice": 45300,
  "Change": 300,
  "PerChange": 0.67,
  "Min52W": 42900,
  "Max52W": 67100,
  "EPS": 4281.88,
  "PE": 10.46,
  "BVPS": 20233.00,
  "PB": 2.21
}
```

---

## Additional Endpoints (Homepage/Ancillary)

These endpoints were observed on homepage/stock pages but are not fully documented:

| Endpoint | Method | Auth | Purpose |
|----------|--------|:----:|---------|
| `/company/GetDataRefPriceByCode` | POST | CSRF | Reference price data |
| `/data/getmarketprice` | POST | None | Market index prices (VN-Index, HNX-Index) |
| `/data/headernews` | POST | None | News headlines |
| `/data/getstockdealdetailbytime` | POST | CSRF | Trade details over time |
| `/Home/InitTheme` | POST | None | UI theme initialization |
| `/data/GetTemplateByName` | POST | None | Page template fetch |
| `/Account/GetAuthToken` | POST | None | Auth token |
| `/Data/GeneralMarket_GetMarketIndexVNData` | POST | None | VN market index |
| `/Data/GetListIndustryGICS` | POST | None | Industry classification |
| `/Data/GetInfoFinancialChartHome` | POST | None | Homepage financial chart |
| `/Data/heatmapInfo_v2` | POST | None | Market heatmap |
| `/Data/GetCurrency` | POST | None | Currency rates |
| `/Data/GetCommedities` | POST | None | Commodity prices |

## Error Responses

No error responses were observed during exploration. Expected behaviors [INFERRED]:
- Missing CSRF token → HTTP 400 or 403
- Expired session → HTTP redirect to login
- Invalid stock code → Empty array `[]`
