# Rustfolio 📈

**Rustfolio** is a full-stack intelligent portfolio management and analytics platform built with Rust and React.  
It provides institutional-grade tools for tracking portfolios, analyzing risk, monitoring sentiment, optimizing allocations, and receiving AI-powered investment insights.

The project combines **production-ready Rust backend architecture**, **modern React frontend**, **advanced financial analytics**, and **AI-powered features** to deliver a comprehensive portfolio assistant.

📖 **[View Comprehensive Feature Guide](docs/COMPREHENSIVE_FEATURE_GUIDE.md)** for detailed documentation of all features.

---

## ✨ Key Features

### 📊 Portfolio & Account Management
- Multiple portfolios and brokerage accounts
- Position tracking with real-time market values
- Automatic transaction detection from account snapshots
- CSV import for bulk transaction loading
- True performance metrics (time-weighted & money-weighted returns)

### 📈 Advanced Risk Analytics
- Comprehensive risk metrics: volatility, max drawdown, beta, VaR, Sharpe ratio
- Risk score (0-100) with detailed breakdown and explanations
- Historical risk tracking with snapshots and alerts
- Correlation matrix and heatmap visualization
- Rolling beta analysis with forecasting
- Risk comparison tool for side-by-side ticker analysis
- Configurable risk thresholds with violation alerts
- Downloadable risk reports (PDF & CSV)

### 🎯 Portfolio Optimization
- Concentration risk detection and alerts
- Risk contribution analysis per position
- Diversification scoring (0-10 scale)
- Actionable rebalancing recommendations
- Expected impact projections (before/after metrics)
- Portfolio health assessment

### 📰 Sentiment & News Analysis
- Real-time news fetching for portfolio tickers
- AI-powered thematic clustering of articles
- Sentiment scoring with trend indicators
- Enhanced sentiment combining news, SEC filings, and insider trading
- Bullish/bearish divergence detection
- Portfolio-level sentiment aggregation

### 🤖 AI-Powered Insights
- LLM-generated portfolio narratives and summaries
- Natural language Q&A about portfolio performance
- Contextual recommendations and explanations
- User preference management (risk appetite, narrative tone)
- Configurable AI providers (OpenAI, Claude)

### 🔔 Alerts & Notifications
- Custom alert rules (price, risk, sentiment, portfolio value)
- Multi-channel notifications (email, in-app)
- Alert severity levels and type classification
- Alert history and resolution tracking
- Notification preferences per alert type

### 📉 Market Data & Analytics
- Multi-provider price data (Alpha Vantage, Twelve Data)
- Historical price storage and charting
- Technical indicators (SMA, EMA, Bollinger Bands)
- Portfolio value forecasting with multiple models
- Price history with moving averages and drawdown charts

### 🎨 Modern UI/UX
- Responsive dashboard with interactive charts
- Real-time data updates with React Query
- Color-coded risk badges and sentiment indicators
- Tabbed interfaces for organized data views
- Modal dialogs, tooltips, and contextual help
- Accessibility features (keyboard navigation, ARIA labels)
- Loading states and error handling

---

## 🏗 Architecture Overview

```
rustfolio/
├── backend/              # Rust + Axum API
│   ├── src/
│   │   ├── db/           # SQLx queries and database operations
│   │   ├── services/     # Business logic & analytics
│   │   ├── routes/       # HTTP handlers (portfolios, risk, alerts, etc.)
│   │   ├── external/     # Market data providers integration
│   │   ├── jobs/         # Background job scheduler
│   │   ├── models/       # Data models and types
│   │   ├── state.rs      # AppState (DB pool, providers, caches)
│   │   └── main.rs       # Application entry point
│   ├── migrations/       # Database migrations
│   └── Cargo.toml        # Rust dependencies
│
├── frontend/             # React + TypeScript + Vite
│   ├── src/
│   │   ├── components/   # UI components (50+ components)
│   │   ├── contexts/     # React contexts (preferences, etc.)
│   │   ├── lib/          # API clients & utility functions
│   │   ├── types.ts      # TypeScript type definitions
│   │   └── App.tsx       # Main application component
│   ├── package.json      # Node dependencies
│   └── vite.config.ts    # Vite configuration
│
├── docs/                 # Documentation
│   ├── COMPREHENSIVE_FEATURE_GUIDE.md
│   ├── ENHANCEMENT_ROADMAP.md
│   └── screenshots/      # UI screenshots
│
└── README.md
```

---

## 🧠 Backend (Rust)

### Tech Stack
- **Rust** - Systems programming language
- **Axum** - Web framework
- **SQLx** - Async SQL toolkit
- **PostgreSQL** - Primary database
- **Tokio** - Async runtime
- **Tower-HTTP** - HTTP middleware
- **Serde** - Serialization framework
- **Reqwest** - HTTP client for external APIs

### API Categories
- **Portfolios & Positions** - CRUD operations for portfolios and holdings
- **Accounts & Transactions** - Account management and transaction tracking
- **Risk Analytics** - Risk metrics, history, snapshots, and alerts
- **Optimization** - Portfolio optimization recommendations
- **Market Data** - Price fetching, updates, and ticker search
- **Sentiment & News** - News analysis and sentiment scoring
- **AI/LLM** - Narratives, Q&A, and user preferences
- **Alerts** - Alert rules, notifications, and history
- **Jobs & Admin** - Background jobs, cache management, system health

---

## 🎨 Frontend (React)

### Tech Stack
- **React 18** + **TypeScript** - UI framework
- **Vite** - Build tool and dev server
- **Material-UI (MUI)** - Component library
- **@tanstack/react-query** - Data fetching and caching
- **Axios** - HTTP client
- **Recharts** - Data visualization
- **jsPDF** - PDF report generation
- **React Router** - Client-side routing

---

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+ and Cargo
- Node.js 18+ and npm
- PostgreSQL 14+
- API keys for market data providers (Alpha Vantage, Twelve Data)
- Optional: OpenAI API key for AI features

### Backend Setup
```bash
cd backend
cp .env.example .env
# Edit .env with your database URL and API keys
cargo run
```

### Frontend Setup
```bash
cd frontend
npm install
npm run dev
```

### Database Setup
```bash
# Run migrations
cd backend
sqlx migrate run
```

The application will be available at:
- Frontend: http://localhost:5173
- Backend API: http://localhost:3000

---

## 📚 Documentation

- **[Comprehensive Feature Guide](docs/COMPREHENSIVE_FEATURE_GUIDE.md)** - Detailed documentation of all features
- **[Enhancement Roadmap](docs/ENHANCEMENT_ROADMAP.md)** - Future enhancements and vision
- **[Phase 3 Enhancements](docs/PHASE3_ENHANCEMENTS.md)** - Recent feature additions
- **[Portfolio Optimization Spec](docs/PORTFOLIO_OPTIMIZATION_SPEC.md)** - Optimization implementation details

## 🛣 Roadmap

### Completed ✅
- Portfolio and position management
- Advanced risk analytics with history tracking
- Portfolio optimization recommendations
- Sentiment and news analysis
- AI-powered narratives and Q&A
- Alerts and notifications system
- Correlation analysis and heatmaps
- Rolling beta and forecasting
- Export features (PDF & CSV)

### In Progress 🔄
- Enhanced forecasting models
- Additional technical indicators
- Broker integrations for auto-import

### Planned 📋
- User authentication and multi-tenancy
- Watchlists for non-held securities
- Real-time WebSocket updates
- Tax-loss harvesting suggestions
- Mobile application
- Third-party API for integrations

## 🤝 Contributing

This is a learning project, but contributions are welcome! Please feel free to:
- Report bugs or issues
- Suggest new features
- Submit pull requests
- Improve documentation

## 📄 License

MIT License - See LICENSE file for details

---

**Built with Rust 🦀 and TypeScript ⚛️**  
*An intelligent portfolio assistant for modern investors*
