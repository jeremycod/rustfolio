# Rustfolio 📈

**Rustfolio** is a full-stack portfolio management and analytics application built as a learning-driven but production-oriented project.  
It allows users to track investment portfolios, manage stock positions, fetch historical market data, and visualize performance and risk metrics over time.

The project is intentionally designed to exercise **real-world Rust backend patterns**, modern **React frontend architecture**, and **financial analytics concepts**.

---

## ✨ Features

### Portfolio Management
- Create and manage multiple portfolios
- Select active portfolio from the UI
- View portfolio-level summaries and analytics

### Holdings & Positions
- Add stock tickers to a portfolio
- Track:
  - Number of shares
  - Average buy price
  - Market value
  - Unrealized profit/loss (absolute & %)
- Edit or remove positions
- Color-coded gains and losses for quick insight

### Market Data
- Fetch and store historical daily prices per ticker
- Support for:
  - Mock price generation (for development)
  - External market data providers (extensible)
- Centralized price refresh logic in backend

### Analytics & Visualization
- Portfolio value over time
- Technical indicators:
  - Simple Moving Average (SMA)
  - Exponential Moving Average (EMA)
  - Trendline (linear regression)
  - Bollinger Bands (planned)
- Per-ticker profit/loss history since purchase
- Date range filtering and point summaries

### Frontend UX
- Clean dashboard layout
- Interactive charts using Recharts
- Loading & error states powered by React Query
- Modal dialogs for adding positions
- Responsive layout (desktop-first, mobile-ready)

---

## 🏗 Architecture Overview

```
rustfolio/
├── backend/          # Rust + Axum API
│   ├── db/           # SQLx queries and schema
│   ├── services/     # Business logic & analytics
│   ├── routes/       # HTTP handlers
│   ├── external/     # Market data providers
│   └── state.rs      # AppState (DB pool, providers)
│
├── frontend/         # React + TypeScript + Vite
│   ├── components/   # UI components (charts, tables)
│   ├── lib/          # API clients & endpoints
│   ├── pages/        # Dashboard, Holdings, Analytics
│   └── types.ts      # Shared frontend models
│
└── README.md
```

---

## 🧠 Backend (Rust)

### Tech Stack
- **Rust**
- **Axum**
- **SQLx**
- **PostgreSQL**
- **Chrono**
- **Tower-HTTP**

### Key API Endpoints
```
GET    /api/portfolios
POST   /api/portfolios
GET    /api/portfolios/{id}/positions
POST   /api/portfolios/{id}/positions
PUT    /api/portfolios/{id}/positions/{positionId}
DELETE /api/portfolios/{id}/positions/{positionId}
POST   /api/prices/{ticker}/update
POST   /api/prices/{ticker}/mock
GET    /api/analytics/{portfolioId}
GET    /health
```

---

## 🎨 Frontend (React)

### Tech Stack
- React + TypeScript
- Vite
- Axios
- @tanstack/react-query
- Recharts

---

## 🚀 Getting Started

### Backend
```bash
cd backend
cp .env.example .env
cargo run
```

### Frontend
```bash
cd frontend
npm install
npm run dev
```

---

## 🛣 Roadmap
- Transaction-based accounting
- More analytics indicators
- Export features
- Authentication (future)

---

Built with Rust 🦀 and TypeScript ⚛️
