# Bootstrap the development environment
bootstrap:
    @echo "🚀 Running bootstrap script..."
    ./scripts/bootstrap.sh

# Compose the federated graph schema
compose:
    @echo "🔄 Composing federated graph..."
    cd router && DISABLE_TELEMETRY=1 POSTHOG_DISABLED=1 WG_TELEMETRY_DISABLED=1 npx wgc@latest router compose -i graph.yaml -o execution-config.json

# Start NATS server with JetStream
nats:
    @echo "🌊 Starting NATS server with JetStream..."
    nats-server --jetstream --store_dir ./nats-data --port 4222

# Setup NATS JetStream streams and consumers
nats-setup:
    @echo "⚙️ Setting up NATS JetStream configuration..."
    ./scripts/nats-setup.sh

# Run subgraph-1
subgraph-1:
    @echo "🦀 Starting subgraph-1 on port 8082..."
    cd subgraphs/subgraph-1 && cargo run

# Run subgraph-2  
subgraph-2:
    @echo "🦀 Starting subgraph-2 on port 8083..."
    cd subgraphs/subgraph-2 && cargo run

# Start the Cosmo Router
router:
    @echo "🌐 Starting Cosmo Router on port 8080..."
    cd router && ./bin/router/router

# Start everything in development mode
dev:
    @echo "🚀 Starting development environment..."
    @./scripts/dev-start.sh

# Publish a test event to NATS
publish id payload:
    @echo "📢 Publishing event to NATS..."
    ./scripts/publish-nats.sh {{id}} {{payload}}

# Clean up build artifacts and stop processes
clean:
    @echo "🧹 Cleaning up..."
    -pkill -f "nats-server"
    -pkill -f "subgraph-1"
    -pkill -f "subgraph-2"
    -pkill -f "router"
    -cargo clean
    -rm -rf ./nats-data
    -rm -rf router/execution-config.json
    @echo "✅ Cleanup complete!"

# Build all subgraphs
build:
    @echo "🔨 Building all subgraphs..."
    cargo build

# Check subgraph schemas
check-schemas:
    @echo "🔍 Checking subgraph schemas..."
    cd subgraphs/jim && cargo run --bin schema || echo "Run jim to get schema"
    cd subgraphs/zorus && cargo run --bin schema || echo "Run zorus to get schema"

# Show running processes
ps:
    @echo "📊 Showing running processes..."
    @ps aux | grep -E "(nats-server|jim|zorus|router)" | grep -v grep || echo "No services running"
