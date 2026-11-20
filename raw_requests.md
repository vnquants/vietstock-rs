# Raw Network Traces — Vietstock Finance API

Captured: 2026-06-16 ~15:29 UTC+7  
Browser: Chrome 149 via Chrome DevTools MCP  
Site: https://finance.vietstock.vn

---

## Request 1: GET /data/getdocumenttype

```
GET https://finance.vietstock.vn/data/getdocumenttype
Headers:
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/ACV/tai-tai-lieu.htm
  accept: */*
  accept-language: en-US,en;q=0.9,vi;q=0.8

Status: 200 OK
Content-Type: application/json; charset=utf-8

Response:
[{"DocumentTypeID":1,"DocumentName":"Báo cáo tài chính","DocumentNameEn":"Financial Statement","DisplayOrder":1},{"DocumentTypeID":23,"DocumentName":"Nghị quyết HĐQT","DocumentNameEn":"Board of Management","DisplayOrder":1},{"DocumentTypeID":8,"DocumentName":"Giải trình KQKD","DocumentNameEn":"Explanation for Financial Statement","DisplayOrder":3},{"DocumentTypeID":9,"DocumentName":"Báo cáo quản trị","DocumentNameEn":"Corporate Governance Report","DisplayOrder":4},{"DocumentTypeID":2,"DocumentName":"Báo cáo thường niên","DocumentNameEn":"Annual Report","DisplayOrder":5},{"DocumentTypeID":4,"DocumentName":"Nghị quyết ĐHĐCĐ","DocumentNameEn":"Resolution of Shareholders\u0027 Meeting","DisplayOrder":6},{"DocumentTypeID":5,"DocumentName":"Tài liệu ĐHĐCĐ","DocumentNameEn":"Shareholders\u2019Meeting Documents","DisplayOrder":7},{"DocumentTypeID":3,"DocumentName":"Bản cáo bạch","DocumentNameEn":"Prospectus","DisplayOrder":8},{"DocumentTypeID":10,"DocumentName":"Tỷ lệ vốn khả dụng","DocumentNameEn":"Liquidity Ratio Report","DisplayOrder":9},{"DocumentTypeID":31,"DocumentName":"Tỷ lệ an toàn vốn","DocumentNameEn":"Capital Adequacy Ratio Report","DisplayOrder":10},{"DocumentTypeID":6,"DocumentName":"Khác","DocumentNameEn":"Regulations","DisplayOrder":10}]
```

---

## Request 2: POST /data/getyearreport (ACV)

```
POST https://finance.vietstock.vn/data/getyearreport
Headers:
  content-type: application/x-www-form-urlencoded; charset=UTF-8
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/ACV/tai-tai-lieu.htm
  accept: application/json, text/javascript, */*; q=0.01

Body (form-urlencoded):
  code=ACV

Status: 200 OK
Content-Type: application/json; charset=utf-8

Response:
[2026,2025,2024,2023,2022,2021,2020,2019,2018,2017,2016,2015,2014,2013,2012]
```

---

## Request 3: POST /data/getyearreport (VNM)

```
POST https://finance.vietstock.vn/data/getyearreport
Headers:
  content-type: application/x-www-form-urlencoded; charset=UTF-8
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/VNM/tai-tai-lieu.htm

Body (form-urlencoded):
  code=VNM

Status: 200 OK
Content-Type: application/json; charset=utf-8

Response:
[2026,2025,2024,2023,2022,2021,2020,2019,2018,2017,2016,2015,2014,2013,2012,2011,2010,2009,2008,2007,2006,2005,2004,2003]
```

---

## Request 4: POST /data/getdocument (ACV)

```
POST https://finance.vietstock.vn/data/getdocument
Headers:
  content-type: application/x-www-form-urlencoded; charset=UTF-8
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/ACV/tai-tai-lieu.htm
  cookie: ASP.NET_SessionId=<redacted>; __RequestVerificationToken=<redacted>
  accept: application/json, text/javascript, */*; q=0.01

Body (form-urlencoded):
  code=ACV&__RequestVerificationToken=<redacted>

Status: 200 OK
Content-Type: application/json; charset=utf-8
Response Size: 223 documents

Response (first 2 of 223 items):
[
  {
    "FileExt": ".zip ",
    "UpdateTime": null,
    "TotalRow": 223,
    "FileInfoID": 548794,
    "Url": "https://static2.vietstock.vn/data/UPCOM/2026/TAI LIEU DHCD/VN/ACV_TAILIEU_DHDCD thuong nien_2026.zip",
    "Title": "Tài liệu đại hội đồng cổ đông thường niên năm 2026 ",
    "FullName": "Tài liệu đại hội đồng cổ đông thường niên năm 2026 ",
    "LastUpdate": "/Date(1780368384730)/"
  },
  {
    "FileExt": ".zip ",
    "UpdateTime": null,
    "TotalRow": 223,
    "FileInfoID": 546047,
    "Url": "https://static2.vietstock.vn/data/UPCOM/2026/NGHI QUYET HDQT/VN/4_acv_2026_5_6_9221f8a_en_announcement(NhieuFile).zip",
    "Title": "Nghị quyết Hội đồng quản trị số 248/NQ-HĐQT tháng 05/2026",
    "FullName": "Nghị quyết Hội đồng quản trị số 248/NQ-HĐQT tháng 05/2026",
    "LastUpdate": "/Date(1778207635757)/"
  }
]
```

**LastUpdate timestamps decoded:**
- `1780368384730` → 2026-06-03T09:06:24.730Z
- `1778207635757` → 2026-05-09T07:53:55.757Z

---

## Request 5: POST /data/getdocument (VNM)

```
POST https://finance.vietstock.vn/data/getdocument
Headers:
  content-type: application/x-www-form-urlencoded; charset=UTF-8
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/VNM/tai-tai-lieu.htm
  cookie: ASP.NET_SessionId=<redacted>; __RequestVerificationToken=<redacted>

Body (form-urlencoded):
  code=VNM&__RequestVerificationToken=<redacted>

Status: 200 OK
Content-Type: application/json; charset=utf-8
Response Size: 415 documents

Response (first item):
{
  "FileExt": ".pdf",
  "UpdateTime": null,
  "TotalRow": 415,
  "FileInfoID": 543493,
  "Url": "https://static2.vietstock.vn/data/HOSE/2026/KHAC/VN/VNM_DIEULE_2026.pdf",
  "Title": "Điều lệ năm 2026",
  "FullName": "Điều lệ năm 2026",
  "LastUpdate": "/Date(1777011268370)/"
}
```

**LastUpdate decoded:** `1777011268370` → 2026-04-25T11:34:28.370Z

---

## Request 6: POST /company/tradinginfo (ACV)

```
POST https://finance.vietstock.vn/company/tradinginfo
Headers:
  content-type: application/x-www-form-urlencoded; charset=UTF-8
  x-requested-with: XMLHttpRequest
  referer: https://finance.vietstock.vn/ACV/tai-tai-lieu.htm
  cookie: ASP.NET_SessionId=<redacted>; __RequestVerificationToken=<redacted>

Body (form-urlencoded):
  code=ACV&s=0&t=&__RequestVerificationToken=<redacted>

Status: 200 OK
Content-Type: application/json; charset=utf-8

Response:
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

## Document Download URL Pattern Summary

Observed CDN URLs follow this structure:

```
https://static2.vietstock.vn/data/{EXCHANGE}/{YEAR}/{FOLDER}/{LANG}/{FILENAME}
```

### Exchange Codes
| Code | Exchange |
|------|----------|
| UPCOM | UPCoM |
| HNX | Hanoi Stock Exchange |
| HOSE | Ho Chi Minh Stock Exchange |

### Document Type Folders (observed)
| Folder | Documents |
|--------|-----------|
| `BCTC` | Financial Statements |
| `BCTN` | Annual Reports |
| `BCTHQT` | Governance Reports |
| `NGHI QUYET HDQT` | Board Resolutions |
| `NGHI QUYET DHCD` | Shareholder Resolutions |
| `TAI LIEU DHCD` | Shareholder Meeting Docs |
| `KHAC` | Others |

### File Formats
- `.zip` — Multi-file archives; may contain `.pdf` files + supplementary documents
- `.pdf` — Single PDF document

### Observed Edge Cases
- Trailing spaces on `FileExt` values (e.g., `".zip "`) — must be trimmed
- URLs may contain spaces (e.g., `TAI LIEU DHCD`, `NGHI QUYET HDQT`)
- URLs may contain parentheses (e.g., `(NhieuFile)`)
- URLs use non-ASCII characters in Vietnamese folder names
