#!/bin/bash
set -e

echo "==> Stopping any existing haven-postgres container..."
docker rm -f haven-postgres 2>/dev/null || true

echo "==> Starting fresh haven-postgres container on 127.0.0.1:5433..."
docker run -d \
  --name haven-postgres \
  -e POSTGRES_USER=user \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=haven_rust \
  -p 127.0.0.1:5433:5432 \
  postgres:15

echo "==> Waiting for postgres to be ready..."
sleep 3

echo "==> Running migrations via sqlx..."
cargo sqlx database setup

echo "==> Preparing SQLx offline cache..."
cargo sqlx prepare --workspace

echo "==> Done! Database is ready."
