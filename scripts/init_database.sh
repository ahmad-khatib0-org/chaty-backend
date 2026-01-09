#!/bin/bash
set -e

echo "=== Starting Core Database Initialization ==="

# 1. Initialize CockroachDB (SQL)
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
  SET CLUSTER SETTING kv.rangefeed.enabled = true;
EOF

echo "✓ CockroachDB database, user created, and rangefeed enabled"

# 2. Initialize ScyllaDB (NoSQL)
echo ">>> Step 2: Initializing ScyllaDB keyspace..."
max_attempts=30
attempt=0

# DYNAMICALLY get the internal IP that Scylla is actually using
SCYLLA_IP=$(docker exec scylladb hostname -i | awk '{print $1}')
echo "Scylla detected at IP: $SCYLLA_IP"

until docker compose exec -T scylladb cqlsh "$SCYLLA_IP" -e "DESCRIBE KEYSPACES" >/dev/null 2>&1; do
  attempt=$((attempt + 1))
  if [ $attempt -ge $max_attempts ]; then
    echo "ERROR: ScyllaDB failed to respond to CQLSH at $SCYLLA_IP after 60 seconds"
    docker compose logs scylladb --tail 20
    exit 1
  fi
  echo "Waiting for ScyllaDB CQL interface at $SCYLLA_IP... (attempt $attempt/$max_attempts)"
  sleep 2
done

echo "✓ ScyllaDB is alive"

# Execute the Keyspace creation using the explicit IP
docker compose exec -T scylladb cqlsh "$SCYLLA_IP" -e "CREATE KEYSPACE IF NOT EXISTS chaty WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};"

echo "✓ ScyllaDB keyspace 'chaty' created"
