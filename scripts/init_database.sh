#!/bin/bash
set -e

echo "=== Starting Core Database Initialization ==="

# 1. Initialize Database and User
echo ">>> Step 1: Initializing CockroachDB database and user..."
max_attempts=30
attempt=0
until docker compose exec -T cockroachdb ./cockroach sql --insecure --user=root -e "SELECT 1" >/dev/null 2>&1; do
  attempt=$((attempt + 1))
  if [ $attempt -ge $max_attempts ]; then
    echo "ERROR: CockroachDB failed to start after 60 seconds"
    exit 1
  fi
  echo "Waiting for CockroachDB to be ready (attempt $attempt/$max_attempts)..."
  sleep 2
done

docker compose exec -T cockroachdb ./cockroach sql --insecure --user=root <<EOF
  CREATE USER IF NOT EXISTS chaty;
  CREATE DATABASE IF NOT EXISTS chaty;
  GRANT ALL ON DATABASE chaty TO chaty;
EOF

echo "✓ Database and user created"
