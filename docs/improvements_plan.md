# Rustfolio Frontend Improvement Plan

This document outlines improvements for the Rustfolio frontend and proposes new features requested by the user. Points1–4 from the previous discussion (UI/UX upgrades, ticker forms, editing positions, and better React Query usage) are assumed to be accepted. The new focus is on tracking the history of individual tickers—specifically, computing profit or loss since purchase—and integrating these metrics into the UI.

## 1 — Database & Backend Changes

To support per‑ticker profit/loss calculations, the backend must know when each position was purchased. Two options:

- **Purchase date in positions table** – Add a `purchase_date` column to the positions table. This works if each position corresponds to a single buy.

- **Transactions table (recommended)** – Introduce a transactions table so positions can be derived from multiple buy/sell events. Each transaction record should include `ticker`, `quantity`, `price`, `executed_at` and `side`. This enables accurate realized/unrealized gain calculation later.

Once a purchase date is available, the backend can join positions (or aggregated transactions) with the `price_points` table to compute profit/loss:

```
(close_price − avg_buy_price) × shares
```

for each date after purchase. SQLx queries or Rust services should return both a summary (current profit/loss and percentage) and a time‑series for charting.

### New endpoints:

- `GET /api/positions/{id}/profit` → returns current profit, loss and percentage for that position.
- `GET /api/positions/{id}/profit/history` → returns a list of `{ date, profit }` points for the selected ticker.

These endpoints call the analytics service to compute profit/loss. Use optional query parameters (`start`, `end`) to limit the date range.

## 2 — Frontend Enhancements

### 2.1 Add Ticker Form (from accepted points)

Provide a modal or page with inputs for **Ticker Symbol**, **Shares** and **Average Buy Price** (plus **Purchase Date** if transactions are adopted). Use client‑side validation.

On submit, call `POST /api/portfolios/{id}/positions` or `POST /api/transactions`, then refresh positions and analytics.

### 2.2 Display Profit/Loss in Holdings Table

Extend the holdings table to include columns for **Current Profit/Loss** and **Return %**. Fetch this data via `GET /api/positions/{id}/profit` after loading positions. Use color coding (green for gains, red for losses).

### 2.3 Ticker Profit/Loss History

When a user clicks a ticker in the holdings list, navigate to `/analytics/ticker/{ticker}` or open a drawer. Display:

- A line chart of profit/loss over time using the history endpoint.
- Statistics like total gain/loss, return percentage and max drawdown.
- Optional toggles to overlay the price series and moving averages.

### 2.4 Bulk Price Refresh

Implement a "Refresh All" button that triggers price updates for all tickers in the selected portfolio and recomputes profit/loss. Show a loading spinner and handle rate‑limit errors gracefully.

### 2.5 Persist User Preferences

Store selected portfolio ID, UI theme and indicator toggles in local storage so the app restores the previous state on reload.

### 2.6 Avoid Authentication Work for Now

Since authentication is not a priority, keep the API calls open. When you later add JWT auth (Chapter 11), ensure existing requests can easily attach a token via an Axios interceptor.

## 3 — Implementation Steps

1. **Database migration** – Use SQLx migrations to add `purchase_date` or create the transactions table.
2. **Service and queries** – Implement functions that compute current profit/loss and time‑series profit by joining positions or transactions with price history.
3. **Endpoints** – Add the `/profit` and `/profit/history` routes in Axum. Return JSON with summary and series.
4. **React types and API wrappers** – Update TypeScript types to include profit fields and write Axios functions to call the new endpoints.
5. **UI forms and tables** – Build forms for adding/editing tickers. Extend the holdings table to show profit metrics.
6. **Ticker detail view** – Build a chart component (using Recharts) to show the profit history. Attach row click events in the table to open this view.
7. **Testing** – Write unit tests for profit calculations and service logic. Add integration tests for new endpoints. In the UI, verify that profit numbers update when prices change.

## 4 — Future Considerations

- **Realized vs. unrealized gains** – Once a transactions table exists, compute realized gains from closed positions separately from unrealized gains.
- **Tax and cost‑basis methods** – Support FIFO/LIFO cost basis and capital‑gains reporting.
- **Performance** – For large portfolios or long date ranges, add pagination or limit the number of points returned by the history endpoint.
- **Authentication** – When adding JWT auth, protect all data‑modifying routes and store tokens securely in memory or local storage.