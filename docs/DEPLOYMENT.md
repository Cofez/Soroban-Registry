# Soroban Registry — Deployment Guide

> **Audience:** DevOps engineers, platform teams, and developers running the system locally or in production.  
> **Goal:** Provide clear, reproducible steps for every deployment scenario.

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Environment Variables](#2-environment-variables)
3. [Local Development (Manual)](#3-local-development-manual)
4. [Docker Compose Setup](#4-docker-compose-setup)
5. [Container Architecture](#5-container-architecture)
6. [Production Deployment Checklist](#6-production-deployment-checklist)
7. [Kubernetes Readiness](#7-kubernetes-readiness)
8. [Database Migrations](#8-database-migrations)
9. [Observability Stack](#9-observability-stack)
10. [Scaling Considerations](#10-scaling-considerations)
11. [Disaster Recovery](#11-disaster-recovery)

---

## 1. Prerequisites

| Tool | Minimum Version | Purpose |
|---|---|---|
| Rust | 1.75 | Build `api`, `indexer`, `verifier`, `cli` |
| Node.js | 20 | Build / run frontend and tagging-service |
| pnpm | 8 | Frontend package manager |
| PostgreSQL | 16 | Primary data store |
| Docker | 24 | Containerised deployment |
| Docker Compose | v2 | Local multi-service orchestration |
| sqlx-cli | latest | Running database migrations |

**Install `sqlx-cli`:**

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

---

## 2. Environment Variables

### 2.1 API Server (`backend/api`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `DATABASE_URL` | — | **Yes** | PostgreSQL connection string |
| `RUST_LOG` | `info` | No | Tracing log level (`debug`, `info`, `warn`, `error`) |
| `OTLP_ENDPOINT` | — | No | OpenTelemetry collector endpoint (e.g. `http://jaeger:4317`) |
| `CACHE_ENABLED` | `true` | No | Enable in-process Moka cache |
| `CACHE_MAX_CAPACITY` | `10000` | No | Max weighted entries per cache |
| `PORT` | `3001` | No | HTTP listen port |

### 2.2 Blockchain Indexer (`backend/indexer`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `DATABASE_URL` | — | **Yes** | PostgreSQL connection string |
| `STELLAR_NETWORK` | `testnet` | No | `mainnet` \| `testnet` \| `futurenet` |
| `STELLAR_RPC_MAINNET` | `https://rpc-mainnet.stellar.org` | No | Mainnet RPC endpoint |
| `STELLAR_RPC_TESTNET` | `https://rpc-testnet.stellar.org` | No | Testnet RPC endpoint |
| `STELLAR_RPC_FUTURENET` | `https://rpc-futurenet.stellar.org` | No | Futurenet RPC endpoint |
| `STELLAR_POLL_INTERVAL_SECS` | `30` | No | Ledger poll interval (1–300 s) |
| `RUST_LOG` | `info` | No | Tracing log level |

### 2.3 Tagging Service (`tagging-service`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `DATABASE_URL` | — | **Yes** | PostgreSQL connection string |
| `PORT` | `3002` | No | HTTP listen port |

### 2.4 Frontend (`frontend`)

| Variable | Default | Required | Description |
|---|---|---|---|
| `NEXT_PUBLIC_API_URL` | `http://localhost:3001` | **Yes** | Base URL of the API server |

### 2.5 Observability (Alertmanager)

| Variable | Default | Required | Description |
|---|---|---|---|
| `SLACK_WEBHOOK_URL` | — | No | Slack incoming webhook for alerts |
| `PAGERDUTY_SERVICE_KEY` | — | No | PagerDuty integration key |

### Creating a `.env` file

```bash
cp .env.example .env
# Edit .env with your values
```

Minimum `.env` for local development:

```dotenv
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/soroban_registry
NEXT_PUBLIC_API_URL=http://localhost:3001
STELLAR_NETWORK=testnet
RUST_LOG=debug
CACHE_ENABLED=true
```

---

## 3. Local Development (Manual)

### 3.1 Database

```bash
# Create database
createdb soroban_registry

export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/soroban_registry"

# Apply all migrations
sqlx migrate run --source database/migrations
```

### 3.2 Backend API

```bash
cd backend
cargo build                          # debug build, faster iteration
cargo run --bin api                  # starts on :3001
```

### 3.3 Blockchain Indexer

Run in a second terminal:

```bash
cd backend
export STELLAR_NETWORK=testnet
cargo run --bin indexer
```

### 3.4 Tagging Service

```bash
cd tagging-service
npm install
npm run dev                          # or: node dist/index.js after build
```

### 3.5 Frontend

```bash
cd frontend
pnpm install
pnpm dev                             # starts on :3000
```

### 3.6 Database Seeding (optional)

Populate the database with realistic test data:

```bash
cd backend
cargo run --bin seeder -- --count=50
# Reproducible run:
cargo run --bin seeder -- --count=50 --seed=12345
```

---

## 4. Docker Compose Setup

Docker Compose is the **recommended** way to run the full stack locally and in staging.

```bash
# 1. Clone and enter the repository
git clone https://github.com/yourusername/soroban-registry.git
cd soroban-registry

# 2. Configure environment
cp .env.example .env
# Edit .env as needed

# 3. Start all services
docker compose up -d

# 4. Check service health
docker compose ps

# 5. Stream logs from all services
docker compose logs -f

# 6. Tear down (preserves volumes)
docker compose down

# 7. Tear down and remove all data
docker compose down -v
```

### Useful individual operations

```bash
# Rebuild a single service after code changes
docker compose build api
docker compose up -d --no-deps api

# Open a psql session inside the running database container
docker compose exec postgres psql -U postgres soroban_registry

# Tail API logs
docker compose logs -f api

# Manual health check
curl -f http://localhost:3001/health
```

### Service Endpoints (Docker Compose)

| Service | Local URL | Notes |
|---|---|---|
| API | `http://localhost:3001` | REST API |
| Frontend | `http://localhost:3000` | Web UI |
| Tagging Service | `http://localhost:3002` | Internal service |
| Grafana | `http://localhost:3003` | Dashboards (admin/admin) |
| Prometheus | `http://localhost:9090` | Metrics |
| Jaeger UI | `http://localhost:16686` | Distributed traces |
| Alertmanager | `http://localhost:9093` | Alert routing |
| Loki | `http://localhost:3100` | Log queries (via Grafana) |

---

## 5. Container Architecture

The production Docker image (`Dockerfile`) is a **4-stage multi-stage build**:

```mermaid
graph LR
    S1["Stage 1\nrust-builder\nrust:1-slim\nCompile API binary (release)"]
    S2["Stage 2\nnode-builder\nnode:20-slim\npnpm build frontend"]
    S3["Stage 3\nhealthcheck-builder\nbusybox:1.36\nExtract wget binary"]
    S4["Stage 4\nRuntime\ndistroless/cc-debian12:nonroot\nFinal minimal image"]

    S1 -->|/app/backend/target/release/api| S4
    S2 -->|/app/frontend/.next| S4
    S3 -->|/bin/wget| S4
```

**Security properties of the runtime image:**

- Based on `gcr.io/distroless/cc-debian12:nonroot` — no shell, no package manager, minimal attack surface.
- Runs as `nonroot:nonroot` (non-root user enforced at build time).
- Health check uses the injected `wget` binary (not `curl`, which is not present in distroless).
- Only ports `3001` exposed.

**Health check configuration:**

```
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3
    CMD ["/bin/wget", "-q", "-O", "-", "http://localhost:3001/health"]
```

---

## 6. Production Deployment Checklist

### 6.1 Pre-deployment

- [ ] All environment variables set and validated in the target environment.
- [ ] `DATABASE_URL` points to a managed PostgreSQL instance (not the dev container).
- [ ] `SECRET_KEY` / JWT signing key rotated from default values.
- [ ] `SLACK_WEBHOOK_URL` and/or `PAGERDUTY_SERVICE_KEY` configured for alerts.
- [ ] TLS termination configured upstream (load balancer or ingress controller).
- [ ] `NEXT_PUBLIC_API_URL` set to the **public** API URL (not `localhost`).
- [ ] Database backup verified and tested.
- [ ] All database migrations applied (`sqlx migrate run`).

### 6.2 Deployment

- [ ] Pull the latest image or run `docker compose build`.
- [ ] Roll out with zero-downtime strategy (see §7 for Kubernetes).
- [ ] Smoke-test `GET /health` returns `200 OK`.
- [ ] Verify Prometheus is scraping `/metrics` at the expected interval.
- [ ] Confirm traces appear in Jaeger.
- [ ] Contract list endpoint returns expected data (`GET /api/contracts`).

### 6.3 Post-deployment

- [ ] Alert routing tested (send a test alert via Alertmanager).
- [ ] Grafana dashboards populated.
- [ ] Indexer logs show successful ledger polling.
- [ ] Run smoke tests or a subset of the integration test suite.
- [ ] Tag release in Git (`git tag vX.Y.Z`).

---

## 7. Kubernetes Readiness

The application is designed to run in Kubernetes. Below are the recommended resource and probe configurations.

### API Server Deployment (example snippet)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: soroban-registry-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: soroban-registry-api
  template:
    metadata:
      labels:
        app: soroban-registry-api
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port:   "3001"
        prometheus.io/path:   "/metrics"
    spec:
      containers:
        - name: api
          image: soroban-registry-api:latest
          ports:
            - containerPort: 3001
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: soroban-secrets
                  key: database-url
            - name: RUST_LOG
              value: info
            - name: OTLP_ENDPOINT
              value: http://jaeger-collector:4317
          readinessProbe:
            httpGet:
              path: /health
              port: 3001
            initialDelaySeconds: 10
            periodSeconds: 15
          livenessProbe:
            httpGet:
              path: /health
              port: 3001
            initialDelaySeconds: 30
            periodSeconds: 30
          resources:
            requests:
              cpu:    "250m"
              memory: "256Mi"
            limits:
              cpu:    "1000m"
              memory: "512Mi"
```

### Indexer Deployment (example snippet)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: soroban-registry-indexer
spec:
  replicas: 1          # Single instance — stateful (tracks ledger position)
  selector:
    matchLabels:
      app: soroban-registry-indexer
  template:
    spec:
      containers:
        - name: indexer
          image: soroban-registry-indexer:latest
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: soroban-secrets
                  key: database-url
            - name: STELLAR_NETWORK
              value: mainnet
            - name: STELLAR_POLL_INTERVAL_SECS
              value: "30"
          resources:
            requests:
              cpu:    "100m"
              memory: "128Mi"
            limits:
              cpu:    "500m"
              memory: "256Mi"
```

### Secrets Management

Store all secrets in Kubernetes `Secret` objects (or an external secrets manager such as HashiCorp Vault / AWS Secrets Manager):

```bash
kubectl create secret generic soroban-secrets \
  --from-literal=database-url='postgresql://user:pass@db:5432/soroban_registry' \
  --from-literal=jwt-secret='<strong-random-value>'
```

---

## 8. Database Migrations

Migrations are plain SQL files in `database/migrations/` and are applied with `sqlx-cli`.

```bash
# Apply all pending migrations
sqlx migrate run --source database/migrations

# Check migration status
sqlx migrate info --source database/migrations

# Revert the last migration (development only)
sqlx migrate revert --source database/migrations
```

**In Docker Compose**, migrations can be run as a one-off container before the API starts:

```bash
docker compose run --rm api sqlx migrate run --source /migrations
```

**Migration naming convention:**

```
NNN_description.sql          # e.g. 001_initial.sql, 026_full_text_search.sql
YYYYMMDDHHMMSS_description.sql  # timestamp-based for concurrent feature branches
```

---

## 9. Observability Stack

The observability stack is included in `docker-compose.yml` and pre-configured with provisioned Grafana datasources.

### Starting only the observability services

```bash
docker compose up -d prometheus alertmanager loki promtail grafana jaeger node-exporter
```

### Grafana access

- URL: `http://localhost:3003`
- Default credentials: `admin` / `admin` (change on first login)
- Datasources provisioned automatically from `observability/grafana/provisioning/`

### Prometheus alerting

Alert rules are defined in `observability/prometheus/alert_rules.yml`. Notifications route via Alertmanager to Slack or PagerDuty:

```bash
# Test Alertmanager routing
curl -X POST http://localhost:9093/api/v1/alerts \
  -H 'Content-Type: application/json' \
  -d '[{"labels":{"alertname":"TestAlert","severity":"warning"}}]'
```

### Accessing traces

Open Jaeger UI at `http://localhost:16686` and search by service name `soroban-registry-api`.

---

## 10. Scaling Considerations

### API Server

The API is **stateless** — the only shared state is `AppState` which holds:
- A `PgPool` (connection pooled, safe for multiple replicas)
- A `CacheLayer` (per-process, in-memory — not shared across replicas)

**Horizontal scaling:**  
Multiple API replicas can be placed behind a load balancer. The in-process cache is local to each replica; cache warm-up happens organically through TTL-based population. This is acceptable for the current workload. If cache consistency across replicas becomes critical, migrate to a shared Redis cache.

**Connection pool sizing:**  
PostgreSQL max connections = `replica_count × PgPool.max_connections`. Size `max_connections` accordingly:

```
# backend/api env
DB_MAX_CONNECTIONS=20    # per replica; default 5
```

### Indexer

The indexer is **stateful** (tracks `last_indexed_ledger`) and should run as a **single replica**. Running multiple replicas would result in duplicate ledger processing. Ensure only one pod is active (`replicas: 1`, no HPA).

### Tagging Service

Stateless — horizontally scalable. Multiple replicas can share the same PostgreSQL connection string and run independent cron cycles (duplicate tag assignment is idempotent).

### Database

- Use a managed PostgreSQL service (AWS RDS, Google Cloud SQL, Supabase) in production.
- Enable **connection pooling** via PgBouncer in front of the database.
- Add **read replicas** for analytics/reporting queries (point `GET /api/analytics/*` endpoints at the replica URL).
- Enable **automated backups** with a minimum 7-day retention window.

### Caching

| Scenario | Recommendation |
|---|---|
| ≤ 3 API replicas | Moka in-process cache is sufficient |
| > 3 API replicas | Add Redis; replace `CacheLayer` with a Redis-backed adapter |
| High read traffic on ABI/OpenAPI endpoints | Increase `CACHE_MAX_CAPACITY` or add a CDN layer in front |

### Rate Limiting

The API implements per-IP rate limiting (`rate_limit.rs`). In a multi-replica setup, rate limits are per-process. For cluster-wide limits, introduce a shared Redis-backed rate limiter (e.g. `tower-governor` with Redis store).

---

## 11. Disaster Recovery

Full disaster recovery procedures are documented in [docs/DISASTER_RECOVERY_PLAN.md](DISASTER_RECOVERY_PLAN.md).

Quick reference:

```bash
# Run the full DR automation drill
./scripts/drill_automation.sh

# Verify DR acceptance criteria
./scripts/verify_drp_acceptance_criteria.sh

# Manual disaster recovery
./scripts/disaster_recovery.sh
```

**Recovery Time Objective (RTO):** target < 1 hour for full-service restoration.  
**Recovery Point Objective (RPO):** target < 24 hours data loss (daily backup cadence).
