#!/usr/bin/env bash
set -e
cargo build \
  --bin chaty-api

trap 'pkill -f chaty-' SIGINT
# cargo run --bin chat-database &
cargo run --bin chaty-api
