# Rustfolio Functional Specification

## 1. Overview

Rustfolio is a full-stack stock portfolio tracker and analytics dashboard. Users can manage their portfolios, record positions and transactions, and visualize performance over time. The backend is implemented in Rust using Axum and SQLx, and the frontend is built with React and TypeScript.

---

## 2. User Roles and Personas

### 2.1 Roles

- **Anonymous user**
  - Can access public documentation or a landing page (optional future feature).
- **Authenticated user** (stretch feature)
  - Can create and manage their own portfolios, positions, and transactions.

For the MVP, you can assume a **single logical user** without authentication to simplify implementation.

---

## 3. Core Domain Concepts

### 3.1 Portfolio

Represents a logical grouping of investments (e.g., "Long-term", "Retirement", "Speculative").

**Fields (MVP):**

- `id` (UUID)
- `name` (string)
- `created_at` (datetime)

### 3.2 Position

Represents the current holdings of a particular stock within a portfolio.

**Fields (MVP):**

- `id` (UUID)
- `portfolio_id` (UUID, FK to Portfolio)
- `ticker` (string, e.g., "AAPL")
- `shares` (decimal / numeric)
- `avg_buy_price` (decimal / numeric)
- `created_at` (datetime)

### 3.3 Transaction

Represents a buy or sell event that affects a portfolio's holdings.

**Fields (MVP):**

- `id` (UUID)
- `portfolio_id` (UUID)
- `ticker` (string)
- `quantity` (decimal)
- `price` (decimal)
- `side` (enum: BUY | SELL)
- `executed_at` (datetime)
- `created_at` (datetime)

The system can derive current positions from the transaction history as a stretch feature.

### 3.4 PricePoint

Represents a historical price for a given ticker.

**Fields (MVP):**

- `id` (UUID)
- `ticker` (string)
- `date` (date)
- `close_price` (decimal)
- `created_at` (datetime)

In early stages, price data can be mock-generated; later, a real price API can populate these records.

---

## 4. Functional Requirements

### 4.1 Portfolio Management

- Create a portfolio with a given name.
- List all portfolios.
- View details of a single portfolio (positions, summary metrics).
- (Stretch) Rename or delete a portfolio.

### 4.2 Position Management

- Add a position to a portfolio:
  - Provide ticker, number of shares, and average buy price.
- List positions for a portfolio.
- (Stretch) Edit or delete a position.
- Compute **per-position metrics**:
  - Current market value = `shares * latest_close_price`
  - Unrealized P&L = `shares * (latest_close_price - avg_buy_price)`
  - Unrealized P&L % = `Unrealized P&L / (shares * avg_buy_price)`

### 4.3 Transaction Management (Stretch)

- Record transactions for a portfolio:
  - BUY or SELL, ticker, quantity, price, date.
- Derive positions from cumulative transactions.
- Show transaction history per portfolio.

### 4.4 Price & Value Tracking

- Maintain a table of historical daily prices per ticker.
- For a given date, compute portfolio value:
  - Sum of `shares * close_price` for all positions using price on that date.
- Generate a **time series of portfolio value** over a date range.

### 4.5 Analytics & Trends

For each ticker (and aggregated portfolio):

- Compute simple moving averages (SMA):
  - 7-day SMA
  - 30-day SMA
- Compute a simple linear regression trendline for the last N days:
  - Use closing prices vs. days index to estimate slope.
  - Label as:
    - Uptrend (slope > threshold)
    - Downtrend (slope < -threshold)
    - Sideways (otherwise)

Metrics are for **educational/visualization purposes only**, not financial advice.

### 4.6 Dashboard & Visualization

- **Portfolio Dashboard**
  - Total current portfolio value.
  - Total unrealized P&L and P&L %.
  - Daily change (optional) based on most recent price changes.
  - Line chart of portfolio value over time.
  - Pie chart of allocation by ticker.

- **Portfolio Detail Page**
  - Table of positions showing:
    - Ticker
    - Shares
    - Avg buy price
    - Latest price
    - Market value
    - Unrealized P&L and P&L %
  - Per-ticker chart:
    - Price over time
    - Optional overlay of SMA lines.

- **Transactions Page (Stretch)**
  - Table showing:
    - Ticker
    - Side (BUY/SELL)
    - Quantity
    - Price
    - Date

- **Analytics Page**
  - Summary of trend status for each ticker.
  - Potential risk/volatility indicators (stretch).

---

## 5. API Design (High-Level)

The backend exposes a REST-style JSON API. All endpoints are prefixed with `/api` (e.g., `/api/portfolios`).

### 5.1 Portfolios

- `GET /api/portfolios`
  - Returns list of portfolios.
- `POST /api/portfolios`
  - Body: `{ "name": "Long-term" }`
  - Creates and returns the portfolio.
- `GET /api/portfolios/{id}`
  - Returns portfolio details, including positions summary.

### 5.2 Positions

- `GET /api/portfolios/{id}/positions`
  - Returns positions for the portfolio.
- `POST /api/portfolios/{id}/positions`
  - Body: `{ "ticker": "AAPL", "shares": 10.0, "avg_buy_price": 150.0 }`

### 5.3 Transactions (Stretch)

- `GET /api/portfolios/{id}/transactions`
- `POST /api/portfolios/{id}/transactions`

### 5.4 Prices

- `GET /api/prices/{ticker}`
  - Query parameters for date range (optional).
- (Stretch) `POST /api/prices/fetch`
  - Trigger an update from an external API.

### 5.5 Analytics

- `GET /api/portfolios/{id}/analytics`
  - Returns aggregated metrics and chart data points.
- `GET /api/tickers/{ticker}/analytics`
  - Returns moving averages and trend info for a single stock.

---

## 6. Frontend Behavior

The frontend is a React SPA that consumes the backend API.

### 6.1 Pages

1. **Dashboard**
   - Calls `/api/portfolios` and `/api/portfolios/{id}/analytics` for a selected portfolio.
   - Renders:
     - Summary cards.
     - Line chart (portfolio value over time).
     - Allocation pie chart.

2. **Portfolio Detail**
   - Calls `/api/portfolios/{id}` & `/api/portfolios/{id}/positions`.
   - Renders positions table and optional per-ticker charts.

3. **Transactions (Stretch)**
   - Calls `/api/portfolios/{id}/transactions`.
   - Displays table and a form to add new transactions.

4. **Analytics**
   - Calls `/api/portfolios/{id}/analytics` and/or `/api/tickers/{ticker}/analytics`.
   - Renders trend summaries and indicators.

### 6.2 Components

- `PortfolioSelector` – dropdown to choose active portfolio.
- `ValueChart` – line chart for portfolio value over time.
- `AllocationPie` – pie chart by ticker weight.
- `PositionsTable` – tabular positions view.
- `TrendBadge` – badge showing uptrend/downtrend/sideways for a given ticker.

---

## 7. Non-Functional Requirements

- **Performance:** Suitable for small personal portfolios (tens to hundreds of positions).
- **Reliability:** Use proper error handling and HTTP status codes.
- **Security (Stretch):**
  - JWT authentication for APIs.
  - Proper CORS configuration.
- **Maintainability:**
  - Separate modules in the backend (routes, models, services).
  - Use TypeScript in the frontend.

---

## 8. Future Enhancements

- Multi-user authentication and authorization.
- Real-time updates via WebSockets or Server-Sent Events.
- More advanced risk metrics (Sharpe ratio, beta, etc.).
- Import from brokers via CSV or APIs.
