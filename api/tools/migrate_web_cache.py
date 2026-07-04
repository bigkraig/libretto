#!/usr/bin/env python3
"""
Migrate web_cache.sqlite3 → cache/ filesystem layout.

  cache/web/<sha256(url)>              — GET responses
  cache/web/<sha256(url + "\n" + params)>  — POST responses (params = request body)
  cache/media/<id>                     — media blobs

Run from api/:
  python3 tools/migrate_web_cache.py
"""
import hashlib
import os
import sqlite3
import sys

DB = os.environ.get("WEB_CACHE_DB", "web_cache.sqlite3")
CACHE_DIR = os.environ.get("CACHE_DIR", "cache")


def sha256(data: str) -> str:
    return hashlib.sha256(data.encode()).hexdigest()


def main():
    if not os.path.exists(DB):
        print(f"error: {DB} not found (run from api/)", file=sys.stderr)
        sys.exit(1)

    web_dir = os.path.join(CACHE_DIR, "web")
    media_dir = os.path.join(CACHE_DIR, "media")
    os.makedirs(web_dir, exist_ok=True)
    os.makedirs(media_dir, exist_ok=True)

    con = sqlite3.connect(DB)

    # ── web_cache table ───────────────────────────────────────────────────────
    rows = con.execute("SELECT url, params, content FROM web_cache").fetchall()
    print(f"Migrating {len(rows)} web_cache rows...")
    skipped = written = 0
    for url, params, content in rows:
        key = sha256(url) if not params else sha256(f"{url}\n{params}")
        path = os.path.join(web_dir, key)
        if os.path.exists(path):
            skipped += 1
            continue
        with open(path, "wb") as f:
            f.write(content)
        written += 1
        if written % 1000 == 0:
            print(f"  {written}/{len(rows)} written...")
    print(f"  done: {written} written, {skipped} already existed")

    # ── media table ───────────────────────────────────────────────────────────
    rows = con.execute("SELECT id, content FROM media").fetchall()
    print(f"Migrating {len(rows)} media rows...")
    skipped = written = 0
    for media_id, content in rows:
        path = os.path.join(media_dir, media_id)
        if os.path.exists(path):
            skipped += 1
            continue
        with open(path, "wb") as f:
            f.write(content)
        written += 1
    print(f"  done: {written} written, {skipped} already existed")

    con.close()
    print(f"\nCache written to {os.path.abspath(CACHE_DIR)}/")
    print("You can now delete web_cache.sqlite3 once you've verified the importer works.")


if __name__ == "__main__":
    main()
