# Rustfolio

Full-stack portfolio manager and analytics dashboard.

## Structure

- `backend/` — Rust (Axum + SQLx) API server
- `frontend/` — React + TypeScript single-page application

## Getting Started

### Backend

```bash
cd backend
cargo run
```

The backend runs on `http://127.0.0.1:3000`.

### Frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend runs on `http://127.0.0.1:5173` and proxies `/api` calls to the backend.
