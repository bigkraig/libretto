# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Libretto is a Rust application that serves as an API gateway to the Porsche PCSS (Porsche Component Structure System). It has two modes:
1. **Vehicle Importer**: CLI tool that fetches and caches vehicle data from PCSS API
2. **REST API Server**: Serves the cached data via JSON REST endpoints

## Build Commands

```bash
cargo build --release          # Production build
cargo build                    # Debug build
cargo run -- migrate           # Run database migrations manually
```

## Run Commands

```bash
./target/release/libretto api                                    # Start API server (default: 0.0.0.0:3030)
./target/release/libretto vehicle_importer [-m MODEL] [-y YEAR]  # Import vehicle data from PCSS
```

Migrations run automatically at startup. Set `LIBRETTO_DATABASE_URL` to choose the backend:

```bash
export LIBRETTO_DATABASE_URL="sqlite://content.sqlite3"          # default
export LIBRETTO_DATABASE_URL="postgres://user:pass@host/libretto"     # k8s / production
```

## Docker

```bash
make release   # bump patch version, build amd64 image, push to registry.bigkraig.com
make build     # build without bumping version
```

## Architecture

```
PCSS API (Porsche) → VehicleImporter → cache/ → ContentStore → content DB → API Server
                      (pcss crate)      (HTTP cache)
```

### Crate Structure

- **Main crate (/)**: API server, content store, vehicle importer, CLI
- **pcss crate (/crates/pcss)**: PCSS API client with request caching

### Key Source Files

| File | Purpose |
|------|---------|
| `src/bin/main.rs` | CLI entry point with Api and VehicleImporter subcommands |
| `src/api.rs` | Axum REST API with 20+ endpoints |
| `src/content_store.rs` | sqlx database layer (50+ query methods, AnyPool for SQLite/Postgres) |
| `src/vehicle_importer.rs` | PCSS import orchestration |
| `src/models.rs` | DB models with PCSS type conversions |
| `src/settings.rs` | Config from config.toml + environment |
| `crates/pcss/src/lib.rs` | PCSS HTTP client |
| `crates/pcss/src/api_types.rs` | PCSS API response types |

### Database

Single pool (`AnyPool`) selected at runtime by `LIBRETTO_DATABASE_URL`:
- **SQLite**: `sqlite://content.sqlite3` (default, local dev)
- **Postgres**: `postgres://user:pass@host/libretto` (k8s / production)

Migrations live in `migrations/sqlite/` and `migrations/postgres/` — selected automatically at startup.

## Configuration

**config.toml**:
```toml
# database_url = "sqlite://content.sqlite3"  # default; set to postgres://... for k8s

[importer]
web_cache_database = "web_cache.sqlite3"
cookie = ""  # PCSS auth cookie
vehicle_image_path = "images/vehicles/"

[[vehicle]]
vehicle = "Y1BFH1"  # PCSS model ID
year = 2022
name = "Taycan Cross Turismo Turbo S"

[api]
bind_address = "0.0.0.0:3030"
host = "http://localhost:3030"
```

Environment variables use `LIBRETTO_` prefix to override config settings.

## API Endpoints

- `GET /v1/vehicles` - List vehicles
- `GET /v1/vehicles/:year/:model` - Vehicle details
- `GET /v1/vehicle_component_tree/:year/:model` - Root component tree
- `GET /v1/vehicle_component_tree/nodes/:id` - Component node details
- `GET /v1/vehicle_component_tree/nodes/:id/documents` - Node documents
- `POST /v1/workshop_literature/search` - Search documents
- `GET /v1/workshop_literature/:hkap_id` - Get document
- `GET /v1/content/tool_data/:year/:model/:id` - Tool specifications
