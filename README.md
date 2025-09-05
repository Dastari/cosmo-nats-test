# Cosmo Rust EDFS Test Environment

A complete local development environment for testing WunderGraph Cosmo Router with Rust subgraphs, native WebSocket subscriptions, and Event-Driven Federation via NATS JetStream.

## Architecture

- **Two Rust Subgraphs**: `subgraph-1` and `subgraph-2` using async-graphql + axum
- **WunderGraph Cosmo Router**: Federates subgraphs with WebSocket subscription passthrough
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
- **Subgraph-1**: http://127.0.0.1:8082/graphql (HTTP + WebSocket)
- **Subgraph-2**: http://127.0.0.1:8083/graphql (HTTP + WebSocket)

## Testing WebSocket Subscriptions

Native subgraph subscriptions are exposed and federated through the router via WebSocket passthrough. The composed schema will not list these native subscription fields; passthrough happens at runtime.

Use a client that supports `graphql-transport-ws` (recommended) or `graphql-ws`.

Quick test with websocat (install via your package manager):

```bash
# Connect to the router
websocat -H="Sec-WebSocket-Protocol: graphql-transport-ws" ws://127.0.0.1:8080/graphql

# Then send frames:
{"type":"connection_init"}
{"id":"1","type":"subscribe","payload":{"query":"subscription{ subgraph1OnChangeValue }"}}

# In a separate terminal, trigger an update via HTTP:
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ subgraph1IncrementValue(by:1) }"}'
```

Alternative subscription from subgraph-2:

```json
{"id":"1","type":"subscribe","payload":{"query":"subscription{ subgraph2OnChangeValue }"}}
```

### Event-Driven Federation via NATS

Start an EDFS subscription:

```json
{"id":"3","type":"subscribe","payload":{"query":"subscription{ endpointUpdated(id:\"1\"){ id subgraph1Value subgraph2Count } }"}}
```

Publish a test event:

```bash
just publish 1 '{"id":"1"}'
```

Expected response: An Endpoint entity with data resolved from both subgraphs.

## HTTP Testing

### Queries

```bash
# Test subgraph-1 query
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ subgraph1QueryValue }"}'

# Test subgraph-2 query  
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ subgraph2QueryValue }"}'

# Test federated entity
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"{ endpoint(id:\"1\"){ id subgraph1Value subgraph2Count } }"}'
```

### Mutations

```bash
# subgraph-1 mutation
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ subgraph1IncrementValue(by:5) }"}'

# subgraph-2 mutation
curl -sS -X POST http://127.0.0.1:8080/graphql \
  -H 'content-type: application/json' \
  -d '{"query":"mutation{ subgraph2IncrementValue(by:2) }"}'
```

## GraphQL Schema

### Subgraph-1

```graphql
type Query {
  subgraph1QueryValue: Int!
}

type Mutation {
  subgraph1IncrementValue(by: Int): Int!
}

type Endpoint @key(fields: "id") {
  id: ID!
  subgraph1Value: String
}
```

### Subgraph-2

```graphql
type Query {
  subgraph2QueryValue: Int!
}

type Mutation {
  subgraph2IncrementValue(by: Int): Int!
}

type Endpoint @key(fields: "id") {
  id: ID!
  subgraph2Count: Int
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
just subgraph-1       # Start subgraph-1 only
just subgraph-2       # Start subgraph-2 only  
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
cosmo-nats-test/
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
    ├── subgraph-1/
    │   ├── Cargo.toml
    │   └── src/main.rs
    └── subgraph-2/
        ├── Cargo.toml
        └── src/main.rs
```

## Development Notes

- Router runs in dev mode with file watching enabled
- NATS data is stored in `./nats-data/` (gitignored)
- All services log to stdout for easy debugging
- Use `just clean` to reset the environment completely
- Ensure NATS is running before starting the router; otherwise, the router will fail to start

## Testing Checklist

✅ HTTP queries work via router  
✅ HTTP mutations work via router  
✅ Native WebSocket subscriptions (passthrough via router)  
✅ EDFS subscriptions receive NATS events  
✅ Federated entity resolution works across subgraphs  
✅ Router playground accessible in dev mode  

## Implementation Notes

This is a **working basic implementation** with the following status:

- ✅ **Complete**: Rust subgraphs with federation, queries, mutations, and native WS subscriptions on `/graphql`
- ✅ **Complete**: WunderGraph Cosmo Router setup and configuration  
- ✅ **Complete**: NATS JetStream with Event-Driven Federation
- ✅ **Complete**: Full orchestration via justfile

Note: Native subscription fields are not shown in the composed schema; Cosmo Router forwards WebSocket subscription traffic in passthrough mode.
