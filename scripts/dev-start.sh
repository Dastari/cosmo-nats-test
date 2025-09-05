#!/bin/bash
set -euo pipefail

echo "🚀 Starting development environment..."

# Kill any existing processes
pkill -f "nats-server" || true
pkill -f "jim" || true
pkill -f "zorus" || true
pkill -f "router" || true
sleep 1

echo "Starting NATS server..."
nats-server --jetstream --store_dir ./nats-data --port 4222 &
NATS_PID=$!
sleep 3

echo "Setting up NATS JetStream..."
./scripts/nats-simple-setup.sh

echo "Building subgraphs..."
cargo build

echo "Starting subgraph-1..."
(cd subgraphs/subgraph-1 && cargo run) &
SUBGRAPH1_PID=$!
sleep 3

echo "Starting subgraph-2..."
(cd subgraphs/subgraph-2 && cargo run) &
SUBGRAPH2_PID=$!
sleep 3

echo "Composing graph..."
(cd router && DISABLE_TELEMETRY=1 POSTHOG_DISABLED=1 WG_TELEMETRY_DISABLED=1 npx wgc@latest router compose -i graph.yaml -o execution-config.json)

echo "Starting router..."
(cd router && ./bin/router/router) &
ROUTER_PID=$!

echo "✅ All services started!"
echo "Router: http://127.0.0.1:3002/graphql"
echo "Subgraph-1: http://127.0.0.1:8082/graphql"  
echo "Subgraph-2: http://127.0.0.1:8083/graphql"
echo ""
echo "Press Ctrl+C to stop all services"

# Handle cleanup on exit
cleanup() {
    echo "Stopping services..."
    kill $NATS_PID $SUBGRAPH1_PID $SUBGRAPH2_PID $ROUTER_PID 2>/dev/null || true
    exit 0
}

trap cleanup INT TERM

# Wait for user interrupt
wait
