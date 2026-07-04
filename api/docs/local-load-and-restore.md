# Fast DB population: load locally, then dump/restore to the cluster

The importer writes one row at a time. Against the remote Postgres (through a
`kubectl port-forward`), every `INSERT`/upsert is a round-trip over the tunnel —
tens of thousands of sequential statements dominated by network latency. The PCSS
scrape itself is *not* the bottleneck once `cache/` is warm (responses are read
from local disk).

This process loads into a **local** Postgres (near-zero write latency), then ships
the finished database to the cluster as a single bulk `pg_dump`/`pg_restore`. The
dump includes the schema *with* all constraints, so it also avoids the missing-PK
corruption that a hand-rolled/partial restore caused before.

All commands are `fish`.

## Prereqs

- `cache/` populated (warm) and `config.toml`'s `[importer].cookie` set (only needed
  if the cache is cold and Porsche data must be re-scraped).
- `docker`, `psql`, `pg_dump`, `pg_restore` installed locally.

## 1. Start a local Postgres

Use port **5433** so it doesn't collide with the cluster port-forward on 5432.

```fish
docker run -d --name libretto-pg \
  -e POSTGRES_USER=libretto -e POSTGRES_PASSWORD=libretto -e POSTGRES_DB=libretto \
  -p 5433:5432 postgres:16
```

## 2. Point the loader at the local DB and run it

```fish
set -x LIBRETTO_DATABASE_URL "postgres://libretto:libretto@localhost:5433/libretto?sslmode=disable"

# Migrations run automatically on the first loader invocation and create the full
# schema (all PK/UNIQUE constraints) — nothing to do manually.
make load-porsche
make load-ferrari
make load-audi
```

This is fast: local writes have no tunnel latency. To start over, just
`docker rm -f libretto-pg` and repeat from step 1.

## 3. Dump the finished local database

Custom format (`-Fc`): compressed, includes schema + constraints + data + the
`_sqlx_migrations` bookkeeping (so the cluster API sees migrations as already
applied — checksums match because both use the same migration files).

```fish
pg_dump "$LIBRETTO_DATABASE_URL" -Fc -f libretto.dump
```

## 4. Restore into the cluster

Port-forward the cluster Postgres (separate terminal), then reset its schema and
restore. **Do not** restore an old/broken dump — only the one just produced.

```fish
# terminal A: port-forward
KUBECONFIG=~/.kube/vc-bigkraig.yaml kubectl -n libretto port-forward svc/libretto-db-rw 5432:5432
```

```fish
# terminal B: creds come from the libretto-db-app secret (host swapped to localhost)
set REMOTE (KUBECONFIG=~/.kube/vc-bigkraig.yaml kubectl -n libretto get secret libretto-db-app \
  -o jsonpath='{.data.uri}' | base64 -d | sed 's/@libretto-db-rw.libretto:5432/@localhost:5432/')
set REMOTE "$REMOTE?sslmode=disable"

# wipe and rebuild from the dump (schema + constraints + data in one shot)
psql "$REMOTE" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
pg_restore --no-owner --no-privileges -d "$REMOTE" libretto.dump
```

`--no-owner --no-privileges` because the local role (`libretto`/`libretto`) differs
from the CNPG-managed cluster role.

## 5. Verify

```fish
psql "$REMOTE" -c "SELECT vehicle, year, length(image) AS img_bytes FROM vehicles ORDER BY vehicle, year;"
psql "$REMOTE" -c "SELECT count(*) FROM documents;"
```

Restart the `libretto-api` deployment if you want it to reconnect to a
freshly-restored DB immediately (it also picks it up on its next connection):

```fish
KUBECONFIG=~/.kube/vc-bigkraig.yaml kubectl -n libretto rollout restart deploy/libretto-api
```

## Cleanup

```fish
docker rm -f libretto-pg
rm libretto.dump
```

## Notes

- The dump can be large (documents/media are BYTEA). `-Fc` compresses it; the single
  transfer is still far faster than per-row writes over the port-forward.
- Because the dump carries the constraints, this is the canonical "clean restore" —
  it will never reproduce the missing-PK / duplicate-row state that came from the
  earlier partial restores.
