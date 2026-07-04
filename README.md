# Libretto — Technical Service Information

Internal workshop reference system for Porsche, Ferrari, and Audi vehicles. Aggregates factory service documentation into a searchable web interface.

## Structure

```
libretto/
├── api/          # Rust/Axum API server + data importers
└── web/          # Next.js 14 frontend
```

## Architecture

```
PCSS API (Porsche) ─┐
Ferrari PDFs       ─┼─→ Importers → content.sqlite3 → API (:3030) → Web (:3000)
Audi PDFs          ─┘              cache/ (HTTP cache)
```

The API serves normalized vehicle data from `content.sqlite3`. The HTTP cache (`cache/`) avoids re-fetching from upstream sources on reimport.

## Vehicles

| Vehicle | Source | Import method |
|---------|--------|---------------|
| Porsche 991.2 GT3 | PCSS API | `vehicle-importer` |
| Porsche Cayman GT4 | PCSS API | `vehicle-importer` |
| Porsche Taycan Cross Turismo Turbo S | PCSS API | `vehicle-importer` |
| Ferrari GTC4Lusso | PDF (source_files/ferrari) | `load-ferrari` |
| Audi R8 2016–2020 | PDF (source_files/audi) | `load-audi` |

## Development

Requires: `mise`, Rust, Node 20, pnpm 9, python3

```bash
mise run dev          # Start API (:3030) + web (:3000) in parallel
mise run db:reset     # Rebuild content.sqlite3 from scratch (uses cache/)
mise run db:migrate   # Run pending migrations only
```

### First-time setup

```bash
# 1. Set PCSS cookie in api/config.toml (only needed for Porsche reimport; cache hits skip it)
#    Or via env: export LIBRETTO_IMPORTER__COOKIE="..."

# 2. Create web/.env.local (see environment variables below)

# 3. Build the content database
mise run db:reset
```

### Auth bypass (local dev)

The web frontend requires OIDC (Pocket ID). Set `NEXT_PUBLIC_AUTH_BYPASS=true` in `web/.env.local` to skip auth entirely in development.

### Environment variables

**API** — set via `api/config.toml` or `LIBRETTO_` prefixed env vars:

| Variable | Description |
|----------|-------------|
| `LIBRETTO_IMPORTER__COOKIE` | PCSS session cookie (for Porsche vehicle reimport) |
| `LIBRETTO_API__HOST` | Public API base URL (used in cross-document links) |
| `LIBRETTO_DATABASE_URL` | DB connection string (default: `sqlite://content.sqlite3`; set to `postgres://...` for k8s) |

**Web** — set in `web/.env.local`:

| Variable | Description |
|----------|-------------|
| `NEXT_PUBLIC_API_HOST` | API base URL |
| `NEXT_PUBLIC_AUTH_BYPASS` | Set `true` to skip OIDC in dev |
| `NEXTAUTH_URL` | NextAuth callback base URL |
| `NEXTAUTH_SECRET` | Session signing secret |
| `POCKET_ID_ISSUER` | Pocket ID OIDC issuer URL |
| `POCKET_ID_CLIENT_ID` | OIDC client ID |
| `POCKET_ID_CLIENT_SECRET` | OIDC client secret |

## Data

Large files are kept outside git and synced separately:

| Path | Contents | Storage |
|------|----------|---------|
| `api/cache/` | Filesystem HTTP cache (~35K files) | B2 |
| `api/source_files/ferrari/` | Ferrari workshop PDFs | B2 |
| `api/source_files/audi/` | Audi workshop PDFs | B2 |
| `api/images/vehicles/` | Vehicle thumbnail PNGs | B2 |

## Deployment

Docker images are built and pushed to `registry.bigkraig.com`:

```bash
make release        # bump patch version, build + push both images
make build          # build without pushing
```

Images: `registry.bigkraig.com/libretto-api`, `registry.bigkraig.com/libretto-web`

Tagged with semver (`0.1.0`) and git short hash (`abc1234`).
