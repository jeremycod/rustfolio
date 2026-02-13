# Rustfolio Monitoring & Logging Setup

This document explains how to configure and use the complete monitoring stack for the Rustfolio application.

## Monitoring Stack

Rustfolio includes a complete monitoring setup:

1. **Grafana Loki** - Centralized log aggregation and analysis
2. **Grafana** - Visualization and log exploration dashboard
3. **Uptime Kuma** - Uptime monitoring and alerting

All services run in Docker and are accessible via:
- **Grafana:** http://localhost:3001
- **Loki:** http://localhost:3100
- **Uptime Kuma:** http://localhost:4000

## Logging Overview

Rustfolio supports two logging modes:

1. **Console-only logging** - Logs are printed to stdout (default for local development)
2. **Loki logging** - Logs are sent to Grafana Loki for centralized log aggregation and analysis

## Configuration

All logging configuration is done through environment variables in your `.env` file:

```bash
# Enable/disable Loki logging
LOKI_ENABLED=false          # Set to "true" to enable Loki, "false" for console-only

# Loki endpoint URL (required if LOKI_ENABLED=true)
LOKI_URL=http://loki:3100   # For Docker: http://loki:3100
                             # For local: http://localhost:3100

# Service identification
SERVICE_NAME=rustfolio       # Service name in Loki/Grafana
ENVIRONMENT=development      # Environment tag (development, staging, production)

# Log level
RUST_LOG=info               # Options: trace, debug, info, warn, error
```

## Setup Options

### Option 1: Local Development (Console-only)

For local development, keep Loki disabled:

```bash
# In backend/.env
LOKI_ENABLED=false
RUST_LOG=debug
```

Then run your application normally:

```bash
cd backend
cargo run
```

Logs will appear in your terminal.

### Option 2: Local Development with Loki

1. Start Loki and Grafana with Docker Compose:

```bash
docker-compose up -d loki grafana
```

2. Update your `.env` file:

```bash
# In backend/.env
LOKI_ENABLED=true
LOKI_URL=http://localhost:3100
SERVICE_NAME=rustfolio
ENVIRONMENT=development
RUST_LOG=info
```

3. Build with Loki feature enabled:

```bash
cd backend
cargo build --features loki
cargo run --features loki
```

4. Access Grafana at http://localhost:3001
   - Default login: admin/admin (or as configured)
   - Loki datasource is pre-configured

### Option 3: Production Deployment with Docker

For production deployment, you can run the entire stack in Docker:

1. Create a `.env` file in the root directory:

```bash
# Database
POSTGRES_USER=genie_user
POSTGRES_PASSWORD=your_secure_password
POSTGRES_DB=rustfolio

# Logging
LOKI_ENABLED=true
LOKI_URL=http://loki:3100
SERVICE_NAME=rustfolio
ENVIRONMENT=production
RUST_LOG=info

# API Keys
PRICE_PROVIDER=multi
TWELVEDATA_API_KEY=your_key_here
ALPHAVANTAGE_API_KEY=your_key_here

# Grafana
GRAFANA_PORT=3001
GRAFANA_ADMIN_USER=admin
GRAFANA_ADMIN_PASSWORD=your_secure_password
```

2. Uncomment the `rustfolio` service in `docker-compose.yml`

3. Start all services:

```bash
docker-compose up -d
```

## Viewing Logs in Grafana

1. Open Grafana at http://localhost:3001
2. Navigate to "Explore" from the left menu
3. Select "Loki" as the datasource
4. Use LogQL queries to filter logs:

### Example Queries

View all logs for the rustfolio service:
```logql
{service="rustfolio"}
```

Filter by environment:
```logql
{service="rustfolio", environment="production"}
```

Search for specific text:
```logql
{service="rustfolio"} |= "error"
```

Filter by log level (if using structured logging):
```logql
{service="rustfolio"} | json | level="error"
```

View logs from the last hour with rate:
```logql
rate({service="rustfolio"}[1h])
```

## Building for Production

### Without Loki (console-only)
```bash
cargo build --release
```

### With Loki support
```bash
cargo build --release --features loki
```

### Docker build
The Dockerfile automatically enables Loki support. To build:

```bash
docker build -t rustfolio-backend ./backend
```

## Troubleshooting

### Loki connection errors

If you see errors like "failed to connect to Loki":

1. Check that Loki is running:
   ```bash
   docker ps | grep loki
   ```

2. Verify Loki is accessible:
   ```bash
   curl http://localhost:3100/ready
   ```

3. Check your `LOKI_URL` matches the environment:
   - Docker: `http://loki:3100`
   - Local: `http://localhost:3100`

### Build errors with tracing-loki

If you get compilation errors about `tracing-loki`:

1. Make sure you're building with the `loki` feature:
   ```bash
   cargo build --features loki
   ```

2. Or temporarily disable Loki in your `.env`:
   ```bash
   LOKI_ENABLED=false
   ```

### No logs appearing in Grafana

1. Verify `LOKI_ENABLED=true` in your `.env`
2. Check the application logs for Loki initialization messages
3. Ensure the service is running with Loki feature enabled
4. Try refreshing the Grafana query or adjusting the time range

## Log Levels

Configure the log level with `RUST_LOG`:

- `trace` - Very detailed, includes all debug information
- `debug` - Detailed information for debugging
- `info` - General informational messages (recommended for production)
- `warn` - Warning messages
- `error` - Error messages only

You can also set different levels per module:
```bash
RUST_LOG=info,rustfolio_backend::services=debug
```

## Performance Considerations

- Loki logging adds minimal overhead (logs are sent asynchronously)
- For high-volume applications, consider using `info` or `warn` level in production
- Loki retries failed sends automatically with backoff
- Console logging has no network overhead and is synchronous

## Security

- Never commit API keys or passwords in `.env` files
- Use environment variables or secrets management in production
- Secure your Grafana instance with strong passwords
- Consider using HTTPS for Loki in production environments
