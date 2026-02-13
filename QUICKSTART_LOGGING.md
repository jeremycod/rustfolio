# Quick Start: Logging Setup

## TL;DR

Your Rustfolio app now supports configurable logging to Grafana Loki. By default, it logs to console. Enable Loki in your `.env` file to send logs to Grafana.

## Quick Setup (Console-only - Default)

No changes needed! Your app logs to console by default:

```bash
cd backend
cargo run
```

## Quick Setup (With Loki)

1. **Start Loki and Grafana:**
   ```bash
   docker-compose up -d
   ```

2. **Update backend/.env:**
   ```bash
   LOKI_ENABLED=true
   LOKI_URL=http://localhost:3100
   SERVICE_NAME=rustfolio
   ENVIRONMENT=development
   RUST_LOG=info
   ```

3. **Run with Loki feature:**
   ```bash
   cd backend
   cargo run --features loki
   ```

4. **View logs in Grafana:**
   - Open http://localhost:3001
   - Go to "Explore" → Select "Loki"
   - Query: `{service="rustfolio"}`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `LOKI_ENABLED` | `false` | Enable/disable Loki logging |
| `LOKI_URL` | - | Loki endpoint (e.g., `http://localhost:3100`) |
| `SERVICE_NAME` | `rustfolio` | Service identifier in logs |
| `ENVIRONMENT` | `development` | Environment tag |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

## Key Files Created

- `backend/src/logging.rs` - Logging configuration module
- `docker-compose.yml` - Loki + Grafana setup
- `loki-config.yaml` - Loki configuration
- `grafana-datasources.yaml` - Auto-configures Loki in Grafana
- `backend/Dockerfile` - Production Docker build (with Loki enabled)
- `LOGGING.md` - Full documentation

## Toggle Between Console and Loki

Just change one line in your `.env`:

```bash
# Console-only (no Docker needed)
LOKI_ENABLED=false

# Send to Loki (requires Docker Compose running)
LOKI_ENABLED=true
```

No code changes needed - it's all configurable!

## Production Build

```bash
# Console-only
cargo build --release

# With Loki
cargo build --release --features loki
```

See `LOGGING.md` for full documentation.
