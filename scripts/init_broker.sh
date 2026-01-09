#!/bin/bash
set -e

echo ">>> Initializing Redpanda topics..."
max_attempts=30
attempt=0

# Wait for Redpanda Admin API to be ready
until docker compose exec -T redpanda rpk cluster info >/dev/null 2>&1; do
  attempt=$((attempt + 1))
  if [ $attempt -ge $max_attempts ]; then
    echo "ERROR: Redpanda failed to start after 60 seconds"
    exit 1
  fi
  echo "Waiting for Redpanda to be ready (attempt $attempt/$max_attempts)..."
  sleep 2
done

# Define topics in an array
TOPICS=(
  "api.users.email_confirmation"
  "api.users.email_confirmation_dlq"
  "api.users.password_reset"
  "api.users.password_reset_dlq"
  "api.users.user_created"
  "search.users.changes"
  "search.users.changes_dlq"
)

for topic in "${TOPICS[@]}"; do
  echo "Creating topic: $topic"
  # We use 'rpk topic create'. Redpanda won't error if it already exists
  # but we add --partitions and --replicas for explicit dev config.
  docker compose exec -T redpanda rpk topic create "$topic" \
    --partitions 1 \
    --replicas 1 \
    -c cleanup.policy=compact || echo "Topic $topic might already exist, skipping..."
done

echo "✓ Redpanda topics created"
