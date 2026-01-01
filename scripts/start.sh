#!/usr/bin/env bash
set -e
cargo build \
  --bin chaty-api

trap 'pkill -f chaty-' SIGINT
cargo run --bin chaty-auth &
cargo run --bin chaty-api
