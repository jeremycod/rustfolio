# Rustfolio Learning & Development Plan

This plan ties your Rust learning journey directly to building the Rustfolio project. It is structured as a **multi-week roadmap**. Timelines are suggestions; move at your own pace.

---

## Week 1–2: Rust Foundations + CLI Prototype

**Goals:**

- Get comfortable with Rust syntax and tooling.
- Understand ownership, borrowing, lifetimes at a basic level.
- Build a small CLI prototype that models portfolios and positions.

**Study:**

- Rust Book:
  - Chapters 1–6 (Basics, ownership, structs, enums).
- Rustlings:
  - Exercises on variables, functions, ownership, borrowing, structs, enums.

**Project Tasks:**

1. Initialize a simple Rust binary (separate from the backend for now).
2. Define core structs:
   - `Portfolio`
   - `Position`
   - `PricePoint`
3. Hard-code some example data.
4. Implement functions to:
   - Compute portfolio value.
   - Compute basic P&L for each position.
5. Print a simple text report to the console.

**By the end of Week 2, you should:**

- Be comfortable reading basic Rust code.
- Understand how to structure simple domain models.
- Have a CLI tool that prints portfolio stats.

---

## Week 3–4: Async Rust & Backend Skeleton (Axum)

**Goals:**

- Learn basic async programming in Rust.
- Set up the Rustfolio backend server with Axum.
- Implement health check and basic portfolio endpoints with in-memory storage.

**Study:**

- Axum documentation (basic examples).
- Tokio basics.
- Serde for JSON serialization/deserialization.

**Project Tasks:**

1. Use the scaffolded backend in `backend/`:
   - Run the existing `/health` and root endpoints.
2. Create basic route modules for:
   - `/api/portfolios`
3. Implement in-memory storage (e.g., `Arc<Mutex<Vec<Portfolio>>>`) for development.
4. Implement endpoints:
   - `GET /api/portfolios`
   - `POST /api/portfolios`

**By the end of Week 4, you should:**

- Understand how to build a minimal Rust web server.
- Be able to define routes and handlers.
- Be comfortable returning JSON from Rust.

---

## Week 5–6: Database Integration with SQLx

**Goals:**

- Learn how to interact with a database from Rust.
- Persist portfolios and positions using SQLx.
- Replace in-memory storage with a real database.

**Study:**

- SQLx documentation (Postgres or SQLite).
- Basic SQL: CREATE TABLE, SELECT, INSERT, UPDATE, DELETE.

**Project Tasks:**

1. Set up a local database (SQLite or Postgres).
2. Create migration files for tables:
   - `portfolios`
   - `positions`
   - `price_points`
3. Connect the Axum app to the database using SQLx connection pool.
4. Implement CRUD operations for portfolios and positions using SQLx.
5. Add basic error handling and map errors to HTTP status codes.

**By the end of Week 6, you should:**

- Know how to configure SQLx and manage migrations.
- Be able to write simple SQL queries from Rust.
- Have persistent data for portfolios and positions.

---

## Week 7–8: Frontend (React + TypeScript) & Integration

**Goals:**

- Learn React + TypeScript basics.
- Consume the Rust backend API from the frontend.
- Build initial dashboard and portfolio views.

**Study:**

- React fundamentals (components, props, state, effects).
- TypeScript basics (types, interfaces, generics).
- Fetching data with `fetch` or Axios.

**Project Tasks:**

1. Use the scaffolded frontend in `frontend/`:
   - Run the dev server and see the placeholder app.
   - Confirm backend health via the API indicator.
2. Create basic pages/components:
   - `Dashboard` page with a list of portfolios.
   - `PortfolioDetail` page with positions table.
3. Implement API calls:
   - `GET /api/portfolios`
   - `POST /api/portfolios`
   - `GET /api/portfolios/{id}/positions`
4. Display data in tables and simple lists.

**By the end of Week 8, you should:**

- Understand how the frontend and backend communicate.
- Be able to render backend data in a React UI.
- Have a basic but working full-stack flow.

---

## Week 9–10: Charts, Analytics & Price History

**Goals:**

- Introduce charting library on the frontend.
- Implement analytics endpoints on the backend.
- Generate and visualize price history and portfolio value over time.

**Study:**

- A charting library such as Recharts or Chart.js.
- Basic time-series concepts (moving average, trendline).

**Project Tasks:**

1. Backend:
   - Define endpoints to return:
     - Portfolio value over time.
     - Price history per ticker.
   - Implement simple moving averages (7-day and 30-day).
   - Implement linear regression to compute trend slope.
2. Frontend:
   - Create a `ValueChart` component showing portfolio value vs. time.
   - Add SMA overlays to ticker charts (stretch).
   - Add a simple `TrendBadge` indicating uptrend/downtrend/sideways.

**By the end of Week 10, you should:**

- Have a visually informative dashboard.
- Understand how to compute and visualize simple analytics.

---

## Week 11–12: Stretch Features & Refinement

**Goals:**

- Add optional advanced features as time permits.
- Refine code structure and documentation.
- Improve UX and polish.

**Stretch Features:**

- **Authentication (JWT):**
  - Implement signup/login endpoints.
  - Protect portfolio endpoints per user.
- **Transactions:**
  - Add endpoints for recording buys/sells.
  - Derive positions from transactions.
- **Price API Integration:**
  - Fetch real price data from a public API (if available).
  - Store it in `price_points` table.
- **UX Improvements:**
  - Loading states, error messages in UI.
  - Better layout and styling.

**By the end of Week 12, you should:**

- Have a robust, personal portfolio dashboard.
- Be comfortable working across Rust backend and a JS frontend.
- Be able to explain and modify the system confidently.

---

## Ongoing: Deepening Rust Skills

While working on Rustfolio, you can keep deepening your Rust knowledge by exploring:

- Traits and generics in your domain models.
- Error handling patterns (`thiserror`, `anyhow`).
- Testing:
  - Unit tests for analytics functions.
  - Integration tests for API endpoints.
- Performance tuning (profiling, reducing allocations).

---

## How to Use This Plan

- Treat each week’s goals as a **theme**, not a rigid deadline.
- Ask for help here whenever you:
  - Design new endpoints.
  - Need code review.
  - Want to refactor or extend the project.
- We can turn any week’s tasks into detailed, step-by-step coding sessions together.
