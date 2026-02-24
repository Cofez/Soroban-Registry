# Troubleshooting Guide

## Overview

This guide provides comprehensive troubleshooting steps for common issues encountered when using the Soroban Registry. Each issue includes symptoms, root causes, diagnostic commands, and step-by-step solutions.

> **Quick Tip:** Before diving deep, check the [Quick Diagnostic Flowchart](#quick-diagnostic-flowchart) to identify your issue category fast.

---

## Table of Contents

- [Quick Diagnostic Flowchart](#quick-diagnostic-flowchart)
- [1. Installation and Setup Issues](#1-installation-and-setup-issues)
- [2. Connection and Network Issues](#2-connection-and-network-issues)
- [3. Contract Publishing Issues](#3-contract-publishing-issues)
- [4. Contract Verification Issues](#4-contract-verification-issues)
- [5. API and Integration Issues](#5-api-and-integration-issues)
- [6. Performance Issues](#6-performance-issues)
- [7. Database and Data Consistency Issues](#7-database-and-data-consistency-issues)
- [8. CLI Issues](#8-cli-issues)
- [9. Frontend and UI Issues](#9-frontend-and-ui-issues)
- [10. Docker and Deployment Issues](#10-docker-and-deployment-issues)
- [Recovery Procedures](#recovery-procedures)
- [Diagnostic Commands Reference](#diagnostic-commands-reference)
- [When to Open an Issue](#when-to-open-an-issue)
- [Related Documentation](#related-documentation)

---

## Quick Diagnostic Flowchart

Use this flowchart to quickly identify which section to jump to:

```
Start: What type of issue are you experiencing?
│
├── Can't install or set up? ──────────────▶ Section 1: Installation and Setup
│
├── Can't connect to services? ────────────▶ Section 2: Connection and Network
│
├── Can't publish a contract? ─────────────▶ Section 3: Contract Publishing
│
├── Verification failing? ─────────────────▶ Section 4: Contract Verification
│   (Also see: docs/VERIFICATION_TROUBLESHOOTING.md)
│
├── API returning errors? ─────────────────▶ Section 5: API and Integration
│   (Also see: docs/ERROR_CODES.md)
│
├── Slow responses or timeouts? ───────────▶ Section 6: Performance
│
├── Data looks wrong or inconsistent? ─────▶ Section 7: Database and Data Consistency
│
├── CLI not working? ──────────────────────▶ Section 8: CLI Issues
│
├── Frontend not loading? ─────────────────▶ Section 9: Frontend and UI
│
└── Docker/deployment problems? ───────────▶ Section 10: Docker and Deployment
```

---

## 1. Installation and Setup Issues

### Issue 1.1: Rust Build Fails During Backend Setup

**Symptoms:**
- `cargo build` fails with compilation errors
- Missing system dependencies

**Diagnostic Steps:**
```bash
# Check Rust version (requires 1.75+)
rustc --version

# Check if wasm32 target is installed
rustup target list --installed | grep wasm32

# Verify cargo is functional
cargo --version
```

**Solutions:**

**Rust version too old:**
```bash
rustup update stable
rustup default stable
```

**Missing wasm32 target:**
```bash
rustup target add wasm32-unknown-unknown
```

**Missing system dependencies (Linux):**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Fedora
sudo dnf install gcc openssl-devel
```

---

### Issue 1.2: Database Connection Fails on Setup

**Symptoms:**
- `sqlx migrate run` fails
- "connection refused" errors
- "role does not exist" errors

**Diagnostic Steps:**
```bash
# Check if PostgreSQL is running
pg_isready -h localhost -p 5432

# Verify database exists
psql -U postgres -l | grep soroban_registry

# Test connection string
psql "$DATABASE_URL" -c "SELECT 1;"
```

**Solutions:**

**PostgreSQL not running:**
```bash
# macOS (Homebrew)
brew services start postgresql@16

# Linux (systemd)
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Docker
docker run -d --name postgres -p 5432:5432 \
  -e POSTGRES_PASSWORD=postgres postgres:16
```

**Database does not exist:**
```bash
createdb -U postgres soroban_registry
```

**Wrong DATABASE_URL:**
```bash
# Ensure the variable is set correctly
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/soroban_registry"
```

---

### Issue 1.3: Node.js/Frontend Setup Fails

**Symptoms:**
- `npm install` fails with dependency errors
- Next.js build errors
- Node version incompatibility

**Diagnostic Steps:**
```bash
# Check Node.js version (requires 20+)
node --version

# Check npm version
npm --version

# Clear npm cache if needed
npm cache verify
```

**Solutions:**

**Node.js version too old:**
```bash
# Using nvm
nvm install 20
nvm use 20

# Or download from https://nodejs.org/
```

**Corrupted node_modules:**
```bash
cd frontend
rm -rf node_modules package-lock.json
npm install
```

---

### Issue 1.4: Environment Variables Not Set

**Symptoms:**
- Services fail to start with "missing configuration" errors
- API returns unexpected 500 errors on startup

**Diagnostic Steps:**
```bash
# Check if .env file exists
ls -la .env

# Verify required variables
grep DATABASE_URL .env
grep STELLAR_RPC_URL .env
```

**Solution:**
```bash
# Copy example env and customize
cp .env.example .env

# Required variables:
# DATABASE_URL=postgresql://postgres:postgres@localhost:5432/soroban_registry
# STELLAR_RPC_URL=https://soroban-testnet.stellar.org
# API_PORT=3001
# FRONTEND_URL=http://localhost:3000
```

---

## 2. Connection and Network Issues

### Issue 2.1: Cannot Connect to Stellar RPC

**Symptoms:**
- "Failed to communicate with Stellar RPC" errors (`ERR_RPC_ERROR`)
- Contract fetching/deployment timeouts
- 502 Bad Gateway responses

**Log Example:**
```
[ERROR] soroban_registry::rpc: RPC connection failed: 
  endpoint=https://soroban-testnet.stellar.org 
  error=ConnectTimeout(30s)
  correlation_id=550e8400-e29b-41d4-a716-446655440000
```

**Diagnostic Steps:**
```bash
# Test RPC endpoint connectivity
curl -s https://soroban-testnet.stellar.org/health

# Check network latency
ping soroban-testnet.stellar.org

# Test with a simple RPC call
curl -X POST https://soroban-testnet.stellar.org \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}'

# Check Stellar network status
curl -s https://status.stellar.org/api/v2/status.json | jq '.status'
```

**Solutions:**

1. **Check Stellar network status** at https://status.stellar.org
2. **Try an alternative RPC endpoint:**
   ```bash
   # Update .env
   STELLAR_RPC_URL=https://soroban-rpc.mainnet.stellar.gateway.fm
   ```
3. **If behind a firewall/proxy**, ensure outbound HTTPS (port 443) is allowed
4. **Retry with exponential backoff** — transient network issues often self-resolve

---

### Issue 2.2: API Service Unreachable

**Symptoms:**
- `curl http://localhost:3001/health` returns "connection refused"
- Frontend shows "Cannot connect to API"

**Diagnostic Steps:**
```bash
# Check if API process is running
# Linux/macOS
lsof -i :3001
# Windows
netstat -ano | findstr :3001

# Check service logs
docker-compose logs api

# Test health endpoint
curl -v http://localhost:3001/health
```

**Solutions:**

**API not started:**
```bash
# Start via Docker
docker-compose up -d api

# Or manually
cd backend && cargo run --bin api
```

**Port conflict:**
```bash
# Change API port in .env
API_PORT=3002
```

**Database not reachable from API:**
```bash
# Verify database connection from API container
docker-compose exec api pg_isready -h db -p 5432
```

---

### Issue 2.3: CORS Errors in Browser

**Symptoms:**
- Browser console shows "Access-Control-Allow-Origin" errors
- API requests work from CLI but fail from frontend

**Log Example (Browser Console):**
```
Access to fetch at 'http://localhost:3001/api/contracts' from origin 
'http://localhost:3000' has been blocked by CORS policy
```

**Solutions:**

1. Ensure `FRONTEND_URL` is correctly set in the API configuration:
   ```bash
   FRONTEND_URL=http://localhost:3000
   ```
2. If running frontend on a non-default port, update the allowed origins
3. Clear browser cache and retry

---

### Issue 2.4: WebSocket Connection Drops

**Symptoms:**
- Real-time updates stop working
- "WebSocket connection closed" in browser console

**Solutions:**

1. Check if a reverse proxy (nginx) is configured for WebSocket upgrade:
   ```nginx
   location /ws {
       proxy_pass http://api:3001;
       proxy_http_version 1.1;
       proxy_set_header Upgrade $http_upgrade;
       proxy_set_header Connection "upgrade";
   }
   ```
2. Increase proxy timeout values
3. Implement client-side reconnection logic

---

## 3. Contract Publishing Issues

### Issue 3.1: Publish Fails with "Invalid Contract Source"

**Symptoms:**
- `ERR_INVALID_CONTRACT_SOURCE` error (HTTP 422)
- "Missing Cargo.toml" or "Missing src/lib.rs" messages

**Diagnostic Steps:**
```bash
# Verify project structure
ls -la Cargo.toml src/lib.rs

# Ensure it compiles
cargo build --release --target wasm32-unknown-unknown

# Check WASM output exists
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

**Solutions:**

Ensure your contract directory has the required structure:
```
my-contract/
├── Cargo.toml        # Required
├── Cargo.lock        # Recommended
└── src/
    └── lib.rs        # Required (entry point)
```

---

### Issue 3.2: Publish Fails with "Already Exists"

**Symptoms:**
- `ERR_ALREADY_EXISTS` error (HTTP 409)
- Contract with same ID already registered

**Solutions:**

1. **If updating an existing contract**, use the versioning endpoint:
   ```bash
   soroban-registry publish --contract-path ./my-contract --version 1.1.0
   ```
2. **If it's a different contract**, ensure you're using the correct contract ID
3. Check existing contract:
   ```bash
   soroban-registry info <contract-id>
   ```

---

### Issue 3.3: Publish Fails with Authentication Error

**Symptoms:**
- `ERR_MISSING_AUTH` or `ERR_INVALID_TOKEN` errors (HTTP 401)
- "Authentication required" message

**Solutions:**

1. Ensure your API key is configured:
   ```bash
   soroban-registry config set api-key YOUR_API_KEY
   ```
2. Verify the key is valid:
   ```bash
   curl -H "Authorization: Bearer YOUR_API_KEY" \
     http://localhost:3001/api/publishers/me
   ```
3. If your token expired, generate a new one

---

### Issue 3.4: Large Contract Upload Timeout

**Symptoms:**
- Upload times out for contracts with many dependencies
- `ERR_TIMEOUT` error (HTTP 504)

**Solutions:**

1. **Minimize contract size:**
   ```toml
   [dependencies]
   soroban-sdk = { version = "21.0.0", default-features = false }
   ```
2. **Increase client timeout:**
   ```bash
   soroban-registry publish --contract-path ./my-contract --timeout 120
   ```
3. Strip debug symbols from WASM:
   ```toml
   [profile.release]
   opt-level = "z"
   strip = true
   ```

---

## 4. Contract Verification Issues

> **Note:** For in-depth verification troubleshooting, see the dedicated [Verification Troubleshooting Guide](./VERIFICATION_TROUBLESHOOTING.md).

### Issue 4.1: Bytecode Mismatch

**Symptoms:**
- `ERR_BYTECODE_MISMATCH` error
- "Compiled bytecode hash does not match on-chain hash"

**Quick Fix Checklist:**
1. Pin exact SDK version: `soroban-sdk = "=21.0.0"`
2. Include `Cargo.lock` in source
3. Match optimization level: `opt-level = "z"`
4. Use Docker for reproducible builds

See [Verification Troubleshooting — Bytecode Mismatch](./VERIFICATION_TROUBLESHOOTING.md#1-bytecode-mismatch) for full details.

---

### Issue 4.2: Verification Stuck in "Pending"

**Symptoms:**
- Verification status remains "pending" for more than 10 minutes
- No progress updates

**Diagnostic Steps:**
```bash
# Check verification status
curl -s http://localhost:3001/api/verifications/<verification-id> | jq '.status'

# Check verification queue
curl -s http://localhost:3001/api/stats | jq '.verification_queue'
```

**Solutions:**

1. **Queue may be congested** — wait and check back
2. **Cancel and retry:**
   ```bash
   soroban-registry verify --cancel <verification-id>
   soroban-registry verify <contract-id> --source ./src
   ```
3. Check server logs for worker errors:
   ```bash
   docker-compose logs verifier
   ```

---

### Issue 4.3: Verification Fails with "Unsupported Compiler"

**Symptoms:**
- `ERR_UNSUPPORTED_COMPILER` error (HTTP 422)
- Requested compiler version not available

**Solution:**
```bash
# List supported compiler versions
curl -s http://localhost:3001/api/compilers | jq '.supported_versions'

# Use a supported version in Cargo.toml
[dependencies]
soroban-sdk = "=21.0.0"  # Use a supported version
```

---

## 5. API and Integration Issues

### Issue 5.1: Rate Limiting (HTTP 429)

**Symptoms:**
- `ERR_RATE_LIMIT_EXCEEDED` error
- `Retry-After` header in response

**Diagnostic Steps:**
```bash
# Check your current rate limit status
curl -v http://localhost:3001/api/contracts 2>&1 | grep -i ratelimit
# Look for: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset
```

**Solutions:**

1. **Implement exponential backoff:**
   ```python
   import time
   
   def call_with_backoff(fn, max_retries=5):
       for attempt in range(max_retries):
           response = fn()
           if response.status_code == 429:
               wait = int(response.headers.get('Retry-After', 2 ** attempt))
               print(f"Rate limited. Waiting {wait}s...")
               time.sleep(wait)
               continue
           return response
       raise Exception("Max retries exceeded")
   ```
2. **Batch requests** where possible
3. **Cache responses** client-side to reduce API calls
4. See [API Rate Limiting](./API_RATE_LIMITING.md) for tier details and limits

---

### Issue 5.2: Unexpected 500 Internal Server Error

**Symptoms:**
- Sporadic `ERR_INTERNAL_SERVER_ERROR` responses
- Requests that previously worked now fail

**Diagnostic Steps:**
```bash
# Note the correlation_id from the error response
# Then search server logs
docker-compose logs api | grep "correlation_id=<your-id>"

# Check system health
curl -s http://localhost:3001/health | jq

# Check database connectivity
curl -s http://localhost:3001/health/db | jq
```

**Solutions:**

1. **Retry the request** — may be a transient issue
2. **Check server resources** (CPU, memory, disk):
   ```bash
   docker stats
   ```
3. **Check database connections:**
   ```bash
   psql "$DATABASE_URL" -c "SELECT count(*) FROM pg_stat_activity;"
   ```
4. If persistent, report with the `correlation_id` from the error response

---

### Issue 5.3: Pagination Not Working as Expected

**Symptoms:**
- Missing results in paginated responses
- Duplicate entries across pages
- `ERR_INVALID_PAGINATION` error

**Solutions:**

1. Use cursor-based pagination when available:
   ```bash
   # First page
   curl "http://localhost:3001/api/contracts?limit=20"
   
   # Next page (use cursor from previous response)
   curl "http://localhost:3001/api/contracts?limit=20&cursor=<next_cursor>"
   ```
2. Keep `limit` at or below 1000
3. Don't modify filters between paginated requests

---

### Issue 5.4: Webhook Delivery Failures

**Symptoms:**
- Webhook events not received
- Webhook status shows "failed" deliveries

**Diagnostic Steps:**
```bash
# Check webhook configuration
curl -s -H "Authorization: Bearer $API_KEY" \
  http://localhost:3001/api/webhooks | jq

# Check delivery history
curl -s -H "Authorization: Bearer $API_KEY" \
  http://localhost:3001/api/webhooks/<webhook-id>/deliveries | jq
```

**Solutions:**

1. Verify your webhook endpoint is publicly accessible
2. Ensure your endpoint returns HTTP 200 within 10 seconds
3. Check that your endpoint accepts `POST` requests with JSON body
4. Verify SSL certificate is valid if using HTTPS

---

## 6. Performance Issues

### Issue 6.1: Slow API Responses

**Symptoms:**
- API response times > 2 seconds
- Timeouts on complex queries

**Diagnostic Steps:**
```bash
# Measure response time
curl -w "\nTime: %{time_total}s\n" http://localhost:3001/api/contracts

# Check database query performance
psql "$DATABASE_URL" -c "
  SELECT query, mean_exec_time, calls
  FROM pg_stat_statements
  ORDER BY mean_exec_time DESC
  LIMIT 10;
"

# Check Prometheus metrics
curl -s http://localhost:3001/metrics | grep soroban_http_request_duration
```

**Solutions:**

1. **Add database indexes** for frequently queried fields:
   ```sql
   CREATE INDEX CONCURRENTLY idx_contracts_name ON contracts USING gin(name gin_trgm_ops);
   CREATE INDEX CONCURRENTLY idx_contracts_network ON contracts(network);
   ```
2. **Enable query caching** in API configuration
3. **Scale horizontally** — add more API instances behind a load balancer
4. **Optimize query parameters** — use filters to narrow results

---

### Issue 6.2: High Memory Usage

**Symptoms:**
- OOM (Out of Memory) kills
- Services restarting unexpectedly
- `docker stats` shows high memory consumption

**Diagnostic Steps:**
```bash
# Check container resource usage
docker stats --no-stream

# Check for memory leaks in logs
docker-compose logs api | grep -i "memory\|oom\|heap"
```

**Solutions:**

1. **Set memory limits** in Docker Compose:
   ```yaml
   services:
     api:
       deploy:
         resources:
           limits:
             memory: 512M
   ```
2. **Tune database connection pool size:**
   ```bash
   DATABASE_MAX_CONNECTIONS=20  # Reduce from default if needed
   ```
3. **Enable swap** as a safety net:
   ```bash
   sudo sysctl vm.swappiness=10
   ```

---

### Issue 6.3: Database Connection Pool Exhaustion

**Symptoms:**
- "too many connections" errors from PostgreSQL
- Requests hanging waiting for database connections
- Intermittent 500 errors under load

**Log Example:**
```
[ERROR] sqlx::pool: timed out waiting for a connection after 30s
  pool_size=20 idle=0 waiters=15
```

**Diagnostic Steps:**
```bash
# Check active connections
psql "$DATABASE_URL" -c "
  SELECT state, count(*)
  FROM pg_stat_activity
  WHERE datname = 'soroban_registry'
  GROUP BY state;
"

# Check connection limit
psql "$DATABASE_URL" -c "SHOW max_connections;"
```

**Solutions:**

1. **Increase pool size** (ensure PostgreSQL `max_connections` can handle it):
   ```bash
   DATABASE_MAX_CONNECTIONS=50
   ```
2. **Close idle connections:**
   ```bash
   DATABASE_IDLE_TIMEOUT=300  # seconds
   ```
3. **Use PgBouncer** for connection pooling in production:
   ```yaml
   services:
     pgbouncer:
       image: edoburu/pgbouncer:latest
       environment:
         DATABASE_URL: postgresql://postgres:postgres@db:5432/soroban_registry
         MAX_CLIENT_CONN: 200
         DEFAULT_POOL_SIZE: 25
   ```

---

### Issue 6.4: Slow Contract Search

**Symptoms:**
- Search queries take > 5 seconds
- Timeouts when searching by name or description

**Solutions:**

1. **Use specific search filters** instead of broad text search:
   ```bash
   # Slow (broad text search)
   curl "http://localhost:3001/api/contracts?q=token"
   
   # Faster (filtered)
   curl "http://localhost:3001/api/contracts?category=defi&network=mainnet&q=token"
   ```
2. **Rebuild search indexes:**
   ```sql
   REINDEX INDEX CONCURRENTLY idx_contracts_search;
   ```

---

## 7. Database and Data Consistency Issues

### Issue 7.1: Migration Failures

**Symptoms:**
- `sqlx migrate run` fails
- "relation already exists" or "column not found" errors

**Diagnostic Steps:**
```bash
# Check current migration status
psql "$DATABASE_URL" -c "SELECT * FROM _sqlx_migrations ORDER BY version;"

# Check if tables exist
psql "$DATABASE_URL" -c "\dt"
```

**Solutions:**

1. **If migration partially applied**, fix and re-run:
   ```bash
   # Check which migrations have run
   sqlx migrate info --source database/migrations
   
   # Force re-run if needed (CAUTION: may lose data)
   sqlx migrate revert --source database/migrations
   sqlx migrate run --source database/migrations
   ```
2. **For fresh start** (development only):
   ```bash
   dropdb soroban_registry
   createdb soroban_registry
   sqlx migrate run --source database/migrations
   ```

---

### Issue 7.2: Data Inconsistencies Between Cache and Database

**Symptoms:**
- Stale data shown after updates
- Contract details differ between list and detail views
- Recently published contract not appearing in search

**Solutions:**

1. **Clear the application cache:**
   ```bash
   curl -X POST -H "Authorization: Bearer $ADMIN_KEY" \
     http://localhost:3001/api/admin/cache/clear
   ```
2. **Wait for cache TTL to expire** (default: 5 minutes)
3. **Force refresh** in API requests:
   ```bash
   curl -H "Cache-Control: no-cache" \
     http://localhost:3001/api/contracts/<id>
   ```

---

### Issue 7.3: Verification Records Out of Sync

**Symptoms:**
- Contract shows as "verified" but verification details are missing
- Verification completed but status not updated

**Diagnostic Steps:**
```bash
# Check verification record in database
psql "$DATABASE_URL" -c "
  SELECT id, contract_id, status, created_at, updated_at
  FROM verifications
  WHERE contract_id = '<contract-id>'
  ORDER BY created_at DESC;
"
```

**Solutions:**

1. **Trigger a re-sync:**
   ```bash
   curl -X POST -H "Authorization: Bearer $ADMIN_KEY" \
     http://localhost:3001/api/admin/verifications/sync
   ```
2. Re-submit verification if the record is corrupted

---

### Issue 7.4: Indexer Lag (Blockchain Data Behind)

**Symptoms:**
- Recently deployed contracts not appearing
- On-chain data not reflected in registry
- Indexer health check shows lag

**Diagnostic Steps:**
```bash
# Check indexer status
curl -s http://localhost:3001/health | jq '.indexer'

# Check latest indexed ledger vs network
curl -s http://localhost:3001/api/stats | jq '.latest_indexed_ledger'
```

**Solutions:**

1. **Restart the indexer:**
   ```bash
   docker-compose restart indexer
   ```
2. **Check RPC connection** — indexer depends on Stellar RPC
3. **Increase indexer resources** if it's CPU-bound:
   ```yaml
   services:
     indexer:
       deploy:
         resources:
           limits:
             cpus: "2.0"
             memory: 1G
   ```

---

## 8. CLI Issues

### Issue 8.1: CLI Command Not Found

**Symptoms:**
- `soroban-registry: command not found`
- CLI installed but not in PATH

**Solutions:**

1. **Install the CLI:**
   ```bash
   cargo install --path cli
   ```
2. **Ensure Cargo bin is in PATH:**
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/.cargo/bin:$PATH"
   ```
3. **Verify installation:**
   ```bash
   which soroban-registry
   soroban-registry --version
   ```

---

### Issue 8.2: CLI Configuration Issues

**Symptoms:**
- "Missing configuration" errors
- CLI using wrong API endpoint or network

**Diagnostic Steps:**
```bash
# Check current config
cat ~/.soroban-registry/config.toml

# Check for legacy config
ls ~/.soroban-registry.toml
```

**Solutions:**

1. **Set up configuration:**
   ```bash
   soroban-registry config set api-url http://localhost:3001
   soroban-registry config set network testnet
   soroban-registry config set api-key YOUR_API_KEY
   ```
2. **If legacy config exists**, it should auto-migrate. If not:
   ```bash
   mkdir -p ~/.soroban-registry
   cp ~/.soroban-registry.toml ~/.soroban-registry/config.toml
   ```

---

### Issue 8.3: CLI Authentication Fails

**Symptoms:**
- "Unauthorized" errors from CLI commands
- "Invalid API key" messages

**Solutions:**

1. **Verify API key:**
   ```bash
   soroban-registry config get api-key
   ```
2. **Re-set the key:**
   ```bash
   soroban-registry config set api-key YOUR_NEW_API_KEY
   ```
3. **Test connectivity:**
   ```bash
   soroban-registry health
   ```

---

## 9. Frontend and UI Issues

### Issue 9.1: Frontend Not Loading

**Symptoms:**
- Blank page at `http://localhost:3000`
- JavaScript errors in browser console
- Next.js build errors

**Diagnostic Steps:**
```bash
# Check if dev server is running
curl -s http://localhost:3000 | head -5

# Check build logs
cd frontend && npm run build 2>&1 | tail -20
```

**Solutions:**

1. **Rebuild frontend:**
   ```bash
   cd frontend
   rm -rf .next
   npm run build
   npm run dev
   ```
2. **Check environment variables:**
   ```bash
   # frontend/.env.local
   NEXT_PUBLIC_API_URL=http://localhost:3001
   ```

---

### Issue 9.2: Contract Search Returns No Results

**Symptoms:**
- Search bar returns empty results
- Contracts exist in API but not shown in UI

**Diagnostic Steps:**
```bash
# Verify API returns data
curl "http://localhost:3001/api/contracts?q=<search-term>"

# Check browser network tab for failed requests
```

**Solutions:**

1. Verify the API URL is correctly configured in the frontend
2. Check browser console for CORS or network errors (see [Issue 2.3](#issue-23-cors-errors-in-browser))
3. Ensure the database has been seeded with data:
   ```bash
   cargo run --bin seeder -- --count=50
   ```

---

### Issue 9.3: UI Shows Stale Data

**Symptoms:**
- Published contract not appearing in list
- Updated contract still shows old information

**Solutions:**

1. **Hard refresh the browser:** `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
2. Clear browser local storage/session storage
3. Check if API cache needs clearing (see [Issue 7.2](#issue-72-data-inconsistencies-between-cache-and-database))

---

## 10. Docker and Deployment Issues

### Issue 10.1: Docker Compose Services Won't Start

**Symptoms:**
- `docker-compose up` fails
- Services exit immediately after starting

**Diagnostic Steps:**
```bash
# Check status of all services
docker-compose ps

# Check logs for failing service
docker-compose logs <service-name>

# Verify Docker is running
docker info
```

**Solutions:**

1. **Ensure Docker is running** and has enough resources
2. **Check port conflicts:**
   ```bash
   # Linux/macOS
   lsof -i :3000 -i :3001 -i :5432
   # Windows
   netstat -ano | findstr "3000 3001 5432"
   ```
3. **Rebuild images after code changes:**
   ```bash
   docker-compose build --no-cache
   docker-compose up -d
   ```

---

### Issue 10.2: Docker Container OOM Killed

**Symptoms:**
- Container exits with code 137
- `dmesg` shows OOM killer messages

**Solutions:**

1. **Increase Docker memory limit** in Docker Desktop settings
2. **Set per-container limits:**
   ```yaml
   services:
     api:
       deploy:
         resources:
           limits:
             memory: 1G
           reservations:
             memory: 256M
   ```

---

### Issue 10.3: Volume Permission Issues

**Symptoms:**
- "Permission denied" errors accessing mounted volumes
- Database data directory not writable

**Solutions:**

1. **Fix ownership:**
   ```bash
   sudo chown -R $(id -u):$(id -g) ./data
   ```
2. **Use named volumes** instead of bind mounts in production:
   ```yaml
   volumes:
     postgres_data:
   
   services:
     db:
       volumes:
         - postgres_data:/var/lib/postgresql/data
   ```

---

### Issue 10.4: SSL/TLS Certificate Issues in Production

**Symptoms:**
- HTTPS not working
- Browser shows "insecure connection" warning
- Certificate expired or invalid

**Solutions:**

1. **Use Let's Encrypt for free certificates:**
   ```bash
   certbot certonly --standalone -d registry.yourdomain.com
   ```
2. **Auto-renew certificates:**
   ```bash
   # Add to crontab
   0 0 1 * * certbot renew --quiet
   ```
3. **Verify certificate:**
   ```bash
   openssl s_client -connect registry.yourdomain.com:443 -brief
   ```

---

## Recovery Procedures

### Recovering from a Failed Deployment

```bash
# 1. Check what's running
docker-compose ps

# 2. Stop all services
docker-compose down

# 3. Check for data corruption
psql "$DATABASE_URL" -c "SELECT count(*) FROM contracts;"

# 4. Restore from backup if needed
pg_restore -d soroban_registry backup_latest.dump

# 5. Restart services
docker-compose up -d

# 6. Verify health
curl http://localhost:3001/health
```

### Recovering from Database Corruption

```bash
# 1. Stop the application
docker-compose stop api indexer verifier

# 2. Check database integrity
psql "$DATABASE_URL" -c "
  SELECT schemaname, tablename
  FROM pg_tables
  WHERE schemaname = 'public';
"

# 3. Restore from latest backup
pg_restore --clean --if-exists -d soroban_registry latest_backup.dump

# 4. Re-run pending migrations
sqlx migrate run --source database/migrations

# 5. Restart services
docker-compose start api indexer verifier

# 6. Verify data
curl http://localhost:3001/api/stats
```

### Recovering from Full Disk

```bash
# 1. Check disk usage
df -h

# 2. Find large files
du -sh /var/lib/docker/* | sort -rh | head -10

# 3. Clean up Docker resources
docker system prune -a --volumes

# 4. Clean up old logs
docker-compose logs --no-log-prefix api | wc -l
truncate -s 0 /var/lib/docker/containers/*/*.log

# 5. Restart services
docker-compose restart
```

---

## Diagnostic Commands Reference

### System Health

```bash
# Full health check
curl -s http://localhost:3001/health | jq

# Database health
curl -s http://localhost:3001/health/db | jq

# Registry statistics
curl -s http://localhost:3001/api/stats | jq
```

### Log Analysis

```bash
# View API logs (last 100 lines)
docker-compose logs --tail=100 api

# Filter error logs
docker-compose logs api 2>&1 | grep -i error

# Follow logs in real-time
docker-compose logs -f api

# Search for a specific correlation ID
docker-compose logs api 2>&1 | grep "correlation_id=<your-id>"
```

### Database Diagnostics

```bash
# Check table sizes
psql "$DATABASE_URL" -c "
  SELECT relname AS table, pg_size_pretty(pg_total_relation_size(relid))
  FROM pg_catalog.pg_statio_user_tables
  ORDER BY pg_total_relation_size(relid) DESC;
"

# Check active queries
psql "$DATABASE_URL" -c "
  SELECT pid, state, query, now() - query_start AS duration
  FROM pg_stat_activity
  WHERE datname = 'soroban_registry' AND state != 'idle'
  ORDER BY duration DESC;
"

# Check index usage
psql "$DATABASE_URL" -c "
  SELECT indexrelname, idx_scan, idx_tup_read
  FROM pg_stat_user_indexes
  ORDER BY idx_scan DESC
  LIMIT 10;
"
```

### Network Diagnostics

```bash
# Test Stellar RPC
curl -s https://soroban-testnet.stellar.org/health | jq

# Test API endpoint
curl -w "\n  DNS: %{time_namelookup}s\n  Connect: %{time_connect}s\n  TLS: %{time_appconnect}s\n  Total: %{time_total}s\n" \
  http://localhost:3001/health

# Check open ports
ss -tlnp | grep -E "3000|3001|5432"
```

---

## When to Open an Issue

Open a GitHub issue if:

1. **You've followed this guide** and the problem persists
2. **You've found a bug** — unexpected behavior not covered here
3. **You're experiencing data loss** or corruption
4. **The error includes a `correlation_id`** — include it in the issue
5. **You've identified a security vulnerability** — use responsible disclosure (see [SECURITY.md](./SECURITY.md))

**Do NOT open an issue for:**
- Questions covered in the [FAQ](./FAQ.md)
- Issues resolved by following this guide
- Feature requests (use the Feature Request template instead)

**When reporting, include:**
- Steps to reproduce
- Expected vs actual behavior
- Error messages and `correlation_id` values
- Environment details (OS, Docker version, Rust version, Node.js version)
- Relevant logs (redact sensitive information)

File at: https://github.com/ALIPHATICHYD/Soroban-Registry/issues

---

## Related Documentation

- [FAQ](./FAQ.md) — Frequently asked questions
- [Verification Troubleshooting](./VERIFICATION_TROUBLESHOOTING.md) — Verification-specific issues
- [Error Codes Reference](./ERROR_CODES.md) — All API error codes explained
- [API Rate Limiting](./API_RATE_LIMITING.md) — Rate limit details
- [Observability](./OBSERVABILITY.md) — Monitoring and metrics setup
- [Disaster Recovery Plan](./DISASTER_RECOVERY_PLAN.md) — Recovery procedures
- [Incident Response](./INCIDENT_RESPONSE.md) — Incident handling procedures
- [Security](./SECURITY.md) — Security best practices

## Support

For additional help:
- GitHub Issues: https://github.com/ALIPHATICHYD/Soroban-Registry/issues
- Community Forum: https://community.stellar.org
- Stellar Discord: https://discord.gg/stellar
