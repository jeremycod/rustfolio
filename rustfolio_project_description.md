# Rustfolio Project Description

## Overview

Rustfolio is a full-stack portfolio tracking and analytics tool. The
backend is built with Rust (Axum, SQLx) and the frontend with React +
TypeScript. The system allows users to manage stock holdings, track
price history, view analytics, and visualize portfolio performance using
interactive charts.

## Core MVP Features

-   Manage user portfolios\
-   Add/edit/delete stock positions\
-   Track daily price history (mock or real API)\
-   Display portfolio value over time\
-   Show allocation charts and P&L tables\
-   Compute basic trend indicators (moving averages, linear regression
    trendline)\
-   Database persistence using PostgreSQL or SQLite

## Stretch Features

-   User authentication with JWT\
-   Advanced analytics: volatility, max drawdown, risk scores\
-   Watchlists and price alerts\
-   Import/export CSV data\
-   What-if investment simulations

## Backend Architecture (Rust)

-   Framework: Axum\
-   Async runtime: Tokio\
-   Database: PostgreSQL (SQLx)\
-   Models:
    -   User\
    -   Portfolio\
    -   Position\
    -   Transaction\
    -   PricePoint\
-   REST API Endpoints:
    -   /api/portfolios (CRUD)\
    -   /api/positions\
    -   /api/transactions\
    -   /api/prices/{ticker}\
    -   /api/analytics/{portfolio_id}

## Frontend Architecture (React + TypeScript)

-   Pages:
    -   Dashboard: charts, P&L summary, value timeline\
    -   Portfolio detail: positions table, per-stock analysis\
    -   Transactions page\
    -   Analytics page\
-   Libraries:
    -   Recharts or Chart.js for visualizations\
    -   Material UI or Tailwind for layout\
    -   Axios/fetch for API calls

## Project Folder Structure

    rustfolio/
     ├── backend/
     │    ├── Cargo.toml
     │    ├── src/
     │    └── migrations/
     ├── frontend/
     │    ├── package.json
     │    ├── src/
     │    └── public/
     ├── README.md
     └── docker-compose.yml (optional)

## Development Phases

### Phase 1 --- CLI Prototype (Rust)

-   Build core structs\
-   Simulate transactions & price history\
-   Output basic analytics

### Phase 2 --- Backend API

-   Axum server\
-   SQLx models and queries\
-   Implement analytics logic

### Phase 3 --- Frontend Integration

-   React pages + charts\
-   API consumption\
-   Real-time visual dashboards

### Phase 4 --- Enhancements

-   Real stock API integration\
-   Trend prediction using simple regression\
-   Authentication system

This document summarizes all features and architecture decisions
discussed so far.
