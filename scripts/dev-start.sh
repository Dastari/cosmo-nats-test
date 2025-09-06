#!/bin/bash
set -euo pipefail

echo "🚀 Starting development environment..."

pkill -f "nats-server" || true
pkill -f "target/debug/manager" || true
pkill -f "target/debug/subgraph" || true
sleep 1

echo "Starting NATS server..."
nats-server --jetstream --store_dir ./nats-data --port 4222 &
NATS_PID=$!
sleep 3

echo "Setting up NATS JetStream..."
./scripts/nats-simple-setup.sh

echo "Building binaries..."
cargo build --bin manager --bin subgraph

echo "Starting subgraph-manager..."
cargo run --bin manager &
MANAGER_PID=$!
sleep 2

echo "Starting subgraph-1..."
cargo run --bin subgraph -- --number 1 --profile subgraph1 &
SUB1_PID=$!
sleep 2

echo "Starting subgraph-2..."
cargo run --bin subgraph -- --number 2 --profile subgraph2 &
SUB2_PID=$!
sleep 2

echo "Starting subgraph-3..."
cargo run --bin subgraph -- --number 3 --profile subgraph3 &
SUB3_PID=$!
sleep 2

echo "Starting subgraph-4..."
cargo run --bin subgraph -- --number 4 --profile subgraph4 &
SUB4_PID=$!
sleep 2

echo "Starting subgraph-5..."
cargo run --bin subgraph -- --number 5 --profile subgraph5 &
SUB5_PID=$!
sleep 2

echo "Composing graph..."
(cd router && DISABLE_TELEMETRY=1 POSTHOG_DISABLED=1 WG_TELEMETRY_DISABLED=1 npx wgc@latest router compose -i graph.yaml -o execution-config.json)

echo "Starting router..."
(cd router && ./bin/router/router) &
ROUTER_PID=$!

echo "✅ All services started!"
echo "Manager:      http://127.0.0.1:9000/graphql"
echo "Subgraph-1: http://127.0.0.1:9001/graphql"
echo "Subgraph-2: http://127.0.0.1:9002/graphql"
echo "Subgraph-3: http://127.0.0.1:9003/graphql"
echo "Subgraph-4: http://127.0.0.1:9004/graphql"
echo "Subgraph-5: http://127.0.0.1:9005/graphql"
echo "Router:     http://127.0.0.1:3002/graphql"
echo ""
echo "Press Ctrl+C to stop all services"

cleanup() {
  echo "Stopping services..."
  kill $ROUTER_PID $SUB5_PID $SUB4_PID $SUB3_PID $SUB2_PID $SUB1_PID $MANAGER_PID $NATS_PID 2>/dev/null || true
  exit 0
}
trap cleanup INT TERM
wait
