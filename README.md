# Cosmo Rust EDFS Test Environment

A complete local development environment for testing WunderGraph Cosmo Router with Rust subgraphs, native WebSocket subscriptions, and Event-Driven Federation via NATS JetStream.

## Architecture

- **Two Rust Subgraphs**: `jim` and `zorus` using async-graphql + axum
- **WunderGraph Cosmo Router**: Federates subgraphs with WebSocket subscription support
- **Event-Driven Graph (EDG)**: NATS-powered subscriptions via EDFS
- **NATS JetStream**: Message streaming for event-driven subscriptions

## Quick Start

### 1. One-Time Setup

```bash
just bootstrap
```

This installs all dependencies:
- Build tools (build-essential, curl, git, pkg-config, libssl-dev)
- Rust (via rustup)
- Node.js LTS (for wgc)
- NATS server and CLI
- Cosmo Router binary

### 2. Start Everything

```bash
just dev
```

This starts all services:
- NATS server with JetStream on port 4222
- Creates JetStream stream and consumer
- Builds and starts both subgraphs
- Composes the federated graph
- Starts the Cosmo Router

## Endpoints

- **Router**: http://127.0.0.1:8080/graphql (HTTP + WebSocket)
- **Jim Subgraph**: http://127.0.0.1:8082/graphql
- **Zorus Subgraph**: http://127.0.0.1:8083/graphql

## Testing WebSocket Subscriptions

> **Note**: Native subgraph subscriptions are currently disabled in this basic implementation due to async-graphql-axum API compatibility issues with the latest versions. The subgraphs expose HTTP-only GraphQL endpoints for queries and mutations. WebSocket subscription support can be added by using the correct async-graphql-axum API once the compatibility issues are resolved.

### Event-Driven Federation via NATS

Start an EDFS subscription:

```json
{"id":"3","type":"subscribe","payload":{"query":"subscription{ endpointUpdated(id:\"1\"){ id jimValue zorusCount } }"}}
```

Publish a test event:

```bash
just publish 1 '{"id":"1"}'
```

Expected response: An Endpoint entity with data resolved from both subgraphs.

## HTTP Testing

### Queries

```bash
# Test jim
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ jimPing }"}'

# Test zorus  
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ zorusPing }"}'

# Test federated entity
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ endpoint(id:\"1\"){ id jimValue zorusCount } }"}'
```

### Mutations

```bash
# Jim mutation
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ jimIncrement(by:5) }"}'

# Zorus mutation
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ zorusIncrement(by:2) }"}'
```

## GraphQL Schema

### Jim Subgraph

```graphql
type Query {
  jimPing: String!
}

type Mutation {
  jimIncrement(by: Int): Int!
}

type Endpoint @key(fields: "id") {
  id: ID!
  jimValue: String
}
```

### Zorus Subgraph

```graphql
type Query {
  zorusPing: String!
}

type Mutation {
  zorusIncrement(by: Int): Int!
}

type Endpoint @key(fields: "id") {
  id: ID!
  zorusCount: Int
}
```

### Event-Driven Graph (EDG)

```graphql
type Subscription {
  endpointUpdated(id: ID!): Endpoint!
}

type Endpoint @key(fields: "id", resolvable: false) {
  id: ID! @external
}
```

## Available Commands

```bash
just bootstrap        # Install all dependencies
just dev              # Start all services
just compose          # Compose federated graph
just nats             # Start NATS server only
just nats-setup       # Setup JetStream streams
just jim              # Start jim subgraph only
just zorus            # Start zorus subgraph only  
just router           # Start router only
just publish <id> <payload>  # Publish NATS event
just clean            # Stop all services and clean up
just build            # Build all subgraphs
just ps               # Show running processes
```

## Troubleshooting

### WebSocket Subscriptions Not Working

1. Ensure `subscription.protocol: "ws"` is set in `router/graph.yaml`
2. Verify subgraphs expose WebSocket on `/graphql`
3. Check router logs for connection errors

### EDFS Events Not Delivered

1. Verify NATS is running: `just ps`
2. Check JetStream stream exists: `nats str ls`
3. Verify stream configuration matches EDG schema
4. Test manual publish: `just publish 1 '{"id":"1"}'`

### Router Composition Fails

1. Ensure subgraphs are built: `just build`
2. Check subgraph endpoints are accessible
3. Verify federation directives in subgraph schemas

### Port Conflicts

If ports are in use, update these files:
- `subgraphs/jim/src/main.rs` (port 8082)
- `subgraphs/zorus/src/main.rs` (port 8083)  
- `router/config.yaml` (port 8080)
- `router/graph.yaml` (routing URLs)

## Project Structure

```
cosmo-rust-edfs/
├── README.md
├── justfile
├── Cargo.toml
├── scripts/
│   ├── bootstrap.sh      # Dependency installation
│   ├── nats-setup.sh     # JetStream configuration  
│   └── publish-nats.sh   # Event publishing
├── router/
│   ├── config.yaml       # Router configuration
│   ├── graph.yaml        # Subgraph composition
│   ├── execution-config.json  # Generated execution config
│   └── edg.graphqls      # Event-Driven Graph schema
└── subgraphs/
    ├── jim/
    │   ├── Cargo.toml
    │   └── src/main.rs
    └── zorus/
        ├── Cargo.toml
        └── src/main.rs
```

## Development Notes

- Router runs in dev mode with file watching enabled
- NATS data is stored in `./nats-data/` (gitignored)
- All services log to stdout for easy debugging
- Use `just clean` to reset the environment completely

## Testing Checklist

✅ HTTP queries work via router  
✅ HTTP mutations work via router  
⚠️ Native WebSocket subscriptions (disabled in current implementation)  
✅ EDFS subscriptions receive NATS events  
✅ Federated entity resolution works across subgraphs  
✅ Router playground accessible in dev mode  

## Implementation Notes

This is a **working basic implementation** with the following status:

- ✅ **Complete**: Rust subgraphs with federation, queries, and mutations
- ✅ **Complete**: WunderGraph Cosmo Router setup and configuration  
- ✅ **Complete**: NATS JetStream with Event-Driven Federation
- ✅ **Complete**: Full orchestration via justfile
- ⚠️ **Partial**: Native WebSocket subscriptions (API compatibility issues with async-graphql-axum 7.0 + axum 0.8)

The core federation and EDFS functionality works perfectly. Native subgraph subscriptions can be added once the async-graphql-axum WebSocket API is properly integrated.
