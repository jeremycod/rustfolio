# Monitoring Stack - Quick Start

Get the complete Rustfolio monitoring stack running in 5 minutes.

## What You Get

| Tool | Purpose | URL |
|------|---------|-----|
| **Grafana** | Log visualization & dashboards | http://localhost:3001 |
| **Loki** | Log aggregation backend | http://localhost:3100 |
| **Uptime Kuma** | Uptime monitoring & alerts | http://localhost:4000 |

## Quick Setup

### 1. Start All Monitoring Services

```bash
docker-compose up -d loki grafana uptime-kuma
```

Wait ~30 seconds for all services to be ready.

### 2. Configure Backend Logging

Edit `backend/.env`:
```bash
LOKI_ENABLED=true
LOKI_URL=http://localhost:3100
SERVICE_NAME=rustfolio
ENVIRONMENT=development
RUST_LOG=info
```

### 3. Start Backend

```bash
cd backend
cargo run
```

Look for this message: `✅ Loki logging initialized successfully`

### 4. Configure Uptime Kuma

1. Open http://localhost:4000
2. Create admin account (first time only)
3. Click "Add New Monitor"
4. Configure:
   - **Type:** HTTP(s)
   - **Name:** Rustfolio Backend
   - **URL:** `http://host.docker.internal:3000/health`
   - **Interval:** 60 seconds
5. Click "Save"

### 5. View Logs in Grafana

1. Open http://localhost:3001
2. Click "Explore" (compass icon)
3. Select "Loki" datasource
4. Query: `{service="rustfolio"}`
5. Click "Run query"

## Verify Everything Works

Run this test script:
```bash
./test-loki.sh
```

Or manually verify:

```bash
# Check containers are running
docker ps | grep -E "loki|grafana|uptime"

# Test backend health
curl http://localhost:3000/health
# Should return: OK

# Check logs are in Loki
curl -s "http://localhost:3100/loki/api/v1/label/service/values"
# Should include: "rustfolio"
```

## What Each Tool Does

### Grafana (http://localhost:3001)
- **What:** Log visualization and exploration
- **Use for:** Searching logs, analyzing patterns, debugging issues
- **Example query:** `{service="rustfolio"} |= "error"`

### Loki (http://localhost:3100)
- **What:** Backend that stores logs
- **Use for:** Usually not accessed directly; Grafana queries it
- **Note:** API available at `/loki/api/v1/*`

### Uptime Kuma (http://localhost:4000)
- **What:** Uptime monitoring and alerting
- **Use for:** Getting alerts when backend goes down
- **Example:** Email/Slack alert if health check fails

## Common Tasks

### Stop All Monitoring
```bash
docker-compose stop loki grafana uptime-kuma
```

### Start Only Specific Services
```bash
# Just logs
docker-compose up -d loki grafana

# Just uptime monitoring
docker-compose up -d uptime-kuma
```

### View Logs from Docker Services
```bash
docker logs rustfolio-loki
docker logs rustfolio-grafana
docker logs uptime-kuma
```

### Reset Everything
```bash
docker-compose down
docker volume rm rustfolio_loki-data rustfolio_grafana-data rustfolio_uptime-kuma
docker-compose up -d
```

## Troubleshooting

### "No logs in Grafana"

1. Check backend is running with Loki enabled:
   ```bash
   # Should show backend process
   ps aux | grep rustfolio-backend
   ```

2. Verify logs reaching Loki:
   ```bash
   curl -s "http://localhost:3100/loki/api/v1/label/service/values"
   ```

3. Check backend logs for errors:
   ```bash
   # Look for "Loki logging initialized"
   ```

### "Uptime Kuma shows backend as Down"

1. Make sure URL is `http://host.docker.internal:3000/health`
   - **NOT** `http://localhost:3000/health`

2. Verify health endpoint works:
   ```bash
   curl http://localhost:3000/health
   ```

### "Can't access Grafana/Uptime Kuma"

Check containers are running:
```bash
docker-compose ps
```

If not running:
```bash
docker-compose up -d
```

## URLs Reference Card

Print this and keep handy:

```
┌─────────────────────────────────────────────────┐
│  RUSTFOLIO MONITORING STACK                     │
├─────────────────────────────────────────────────┤
│  Grafana (Logs)                                 │
│  → http://localhost:3001                        │
│  → Query: {service="rustfolio"}                 │
│                                                 │
│  Uptime Kuma (Monitoring)                       │
│  → http://localhost:4000                        │
│  → Use: http://host.docker.internal:3000/health │
│                                                 │
│  Backend Health Check                           │
│  → http://localhost:3000/health                 │
│  → Should return: OK                            │
└─────────────────────────────────────────────────┘
```

## Next Steps

1. ✅ Set up notifications in Uptime Kuma ([guide](UPTIME_KUMA_SETUP.md))
2. ✅ Explore Grafana queries ([guide](LOGGING.md))
3. ✅ Configure alerts for critical issues
4. ✅ Set up backups for monitoring data

## Full Documentation

- **Logging:** See [LOGGING.md](LOGGING.md)
- **Uptime Kuma:** See [UPTIME_KUMA_SETUP.md](UPTIME_KUMA_SETUP.md)
- **Quick Reference:** See [QUICKSTART_LOGGING.md](QUICKSTART_LOGGING.md)

## Cost & Resources

All tools are:
- ✅ Free and open source
- ✅ Self-hosted (no external services)
- ✅ Low resource usage (~200-300MB RAM total)
- ✅ Data stored locally

Perfect for development and small production deployments!
