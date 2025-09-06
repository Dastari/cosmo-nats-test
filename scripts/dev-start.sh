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

echo "Starting subgraph-fame..."
cargo run --bin subgraph-fame -- --port 9001 &
SUB1_PID=$!
sleep 2

echo "Starting subgraph-jim..."
cargo run --bin subgraph-jim -- --port 9002  &
SUB2_PID=$!
sleep 2

echo "Starting subgraph-jim..."
cargo run --bin subgraph-zorus -- --port 9003 &
SUB3_PID=$!
sleep 2


echo "Composing graph..."
(cd router && DISABLE_TELEMETRY=1 POSTHOG_DISABLED=1 WG_TELEMETRY_DISABLED=1 npx wgc@latest router compose -i graph.yaml -o execution-config.json)

echo "Starting router..."
(cd router && ./bin/router/router) &
ROUTER_PID=$!

echo "✅ All services started!"
echo "subgraph-fame: http://127.0.0.1:9001/graphql"
echo "subgraph-jim: http://127.0.0.1:9002/graphql"
echo "subgraph-zorus: http://127.0.0.1:9003/graphql"
echo "Router:     http://127.0.0.1:3002/graphql"
echo ""
echo "Press Ctrl+C to stop all services"

cleanup() {
  echo "Stopping services..."
  kill $ROUTER_PID $SUB3_PID $SUB2_PID $SUB1_PID $NATS_PID 2>/dev/null || true
  exit 0
}
trap cleanup INT TERM
wait
