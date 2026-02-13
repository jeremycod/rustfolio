# Uptime Kuma Setup Guide

Uptime Kuma is a self-hosted monitoring tool that tracks the uptime and performance of your Rustfolio application.

## Quick Start

1. **Start Uptime Kuma with Docker Compose:**
   ```bash
   docker-compose up -d uptime-kuma
   ```

2. **Access Uptime Kuma:**
   - Open http://localhost:4000
   - On first visit, create an admin account

3. **Add Your First Monitor:**
   - Click "Add New Monitor"
   - Configure as shown below

## Setting Up Rustfolio Backend Monitor

### Basic HTTP(s) Monitor

1. **Monitor Type:** `HTTP(s)`
2. **Friendly Name:** `Rustfolio Backend`
3. **URL:**
   - Local development: `http://host.docker.internal:3000/health`
   - Production: `http://your-domain.com/health`
4. **Heartbeat Interval:** `60` seconds
5. **Retries:** `3`
6. **Heartbeat Retry Interval:** `60` seconds
7. **Request Timeout:** `48` seconds

### Advanced Settings

- **Expected Status Code:** `200`
- **Expected Response:** `OK`
- **Follow Redirect:** ✅ Enabled
- **Max Redirects:** `10`
- **Accept Invalid SSL Certificates:** ❌ Disabled (enable only for development)

### Why `host.docker.internal`?

When Uptime Kuma runs in Docker and your backend runs locally:
- **DON'T use** `http://localhost:3000` (points to the container itself)
- **DO use** `http://host.docker.internal:3000` (points to your host machine)

If both run in Docker (same network):
- Use the service name: `http://rustfolio-backend:3000/health`

## Setting Up Additional Monitors

### Database Connection Monitor

Monitor your PostgreSQL database:

1. **Monitor Type:** `PostgreSQL`
2. **Friendly Name:** `Rustfolio Database`
3. **Connection String:** `postgresql://genie_user:genie_user@host.docker.internal:5432/rustfolio`
4. **Heartbeat Interval:** `300` seconds (5 minutes)

### Frontend Monitor

Monitor your frontend application:

1. **Monitor Type:** `HTTP(s)`
2. **Friendly Name:** `Rustfolio Frontend`
3. **URL:** `http://localhost:5173` (or your frontend URL)
4. **Expected Status Code:** `200`

### API Endpoint Monitors

Monitor specific API endpoints:

#### Portfolios API
- **URL:** `http://host.docker.internal:3000/api/portfolios`
- **Method:** `GET`
- **Expected Status Code:** `200` or `401` (if auth required)

#### Prices API
- **URL:** `http://host.docker.internal:3000/api/prices`
- **Expected Status Code:** `200` or `401`

## Notification Setup

Uptime Kuma supports many notification channels:

### Email Notifications

1. Click "Settings" → "Notifications"
2. Select "SMTP (Email)"
3. Configure your SMTP settings:
   - **Host:** `smtp.gmail.com` (for Gmail)
   - **Port:** `587`
   - **Security:** `TLS`
   - **Username:** Your email
   - **Password:** App password (not your regular password)
   - **From Email:** Your email
   - **To Email:** Where to receive alerts

### Slack Notifications

1. Create a Slack webhook URL
2. In Uptime Kuma: "Settings" → "Notifications" → "Slack"
3. Paste webhook URL
4. Test notification

### Discord Notifications

1. Create a Discord webhook in your channel settings
2. In Uptime Kuma: "Settings" → "Notifications" → "Discord"
3. Paste webhook URL

### Other Supported Channels

- Telegram
- Microsoft Teams
- PagerDuty
- Pushover
- Webhooks (custom)
- And many more...

## Status Page (Optional)

Create a public status page:

1. Click "Status Pages" in sidebar
2. Click "Add New Status Page"
3. Choose monitors to display
4. Customize appearance
5. Share the public URL with your team/users

## Integration with Grafana

You can display Uptime Kuma data in Grafana:

1. **Export metrics** from Uptime Kuma
2. **Use Prometheus exporter** (if needed)
3. **Import into Grafana** dashboard

Alternatively, just use both tools side-by-side:
- **Grafana** → Logs and application metrics
- **Uptime Kuma** → Uptime monitoring and alerts

## Docker Compose Configuration

Your current setup:

```yaml
uptime-kuma:
  image: louislam/uptime-kuma:1
  container_name: uptime-kuma
  volumes:
    - uptime-kuma:/app/data
  ports:
    - "4000:3001"
  restart: always
  networks:
    - monitoring
```

Data is persisted in the `uptime-kuma` Docker volume.

## Backup and Restore

### Backup

1. In Uptime Kuma UI: "Settings" → "Backup"
2. Click "Export" to download JSON backup
3. Or backup the Docker volume:
   ```bash
   docker run --rm -v uptime-kuma:/data -v $(pwd):/backup alpine tar czf /backup/uptime-kuma-backup.tar.gz -C /data .
   ```

### Restore

1. In Uptime Kuma UI: "Settings" → "Backup"
2. Click "Import" and select backup file
3. Or restore from volume backup:
   ```bash
   docker run --rm -v uptime-kuma:/data -v $(pwd):/backup alpine tar xzf /backup/uptime-kuma-backup.tar.gz -C /data
   ```

## Accessing Uptime Kuma

- **Local:** http://localhost:4000
- **Default credentials:** Set on first login (remember them!)

## Troubleshooting

### Can't access localhost:4000

Check if container is running:
```bash
docker ps | grep uptime-kuma
```

Start if not running:
```bash
docker-compose up -d uptime-kuma
```

### Monitor shows "Down" for local backend

**Problem:** Using `http://localhost:3000` in monitor URL

**Solution:** Use `http://host.docker.internal:3000` instead

### "Connection Refused" errors

1. Verify backend is running: `curl http://localhost:3000/health`
2. Verify URL in monitor uses `host.docker.internal`
3. Check backend logs for errors

### Password Reset

If you forget your admin password:
```bash
docker exec -it uptime-kuma npm run reset-password
```

## Performance Tips

1. **Don't over-monitor:** Use reasonable intervals (60s minimum for most checks)
2. **Group related monitors:** Use tags to organize
3. **Set up maintenance windows:** Prevent false alerts during deployments
4. **Use notification rate limiting:** Avoid alert fatigue

## Security Recommendations

1. **Change default port** if exposing to internet
2. **Use strong password** for admin account
3. **Enable 2FA** in settings
4. **Restrict access** with firewall rules
5. **Use HTTPS** with reverse proxy (nginx, caddy)

## Monitoring Best Practices

1. **Health Check Endpoint:** Already set up at `/health`
2. **Response Time Alerts:** Set thresholds for slow responses
3. **Certificate Monitoring:** Track SSL certificate expiration
4. **Maintenance Windows:** Schedule known downtime
5. **Status Page:** Keep stakeholders informed

## URLs Reference

| Service | URL | Purpose |
|---------|-----|---------|
| Uptime Kuma | http://localhost:4000 | Monitoring dashboard |
| Backend Health | http://localhost:3000/health | Health check endpoint |
| Grafana | http://localhost:3001 | Logs and metrics |
| Loki | http://localhost:3100 | Log aggregation |

## Next Steps

1. Set up at least one notification channel
2. Configure alerts for critical monitors
3. Create a status page (optional)
4. Set up weekly backup automation
5. Review monitor history regularly

For more information, visit the [Uptime Kuma documentation](https://github.com/louislam/uptime-kuma).
