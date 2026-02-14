#!/bin/bash
set -e

echo "=== Running Client Tests (expect AppScope::Splash) ==="
cargo test -p states --features client --test startup_flow

echo "=== Running Dedicated Server Tests (expect AppScope::Session) ==="
cargo test -p states --features server --test startup_flow

echo "All tests passed!"
