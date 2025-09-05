#!/bin/bash
set -euo pipefail

echo "🌊 Setting up NATS JetStream configuration (simple mode)..."

NATS_URL="nats://127.0.0.1:4222"

echo "Waiting for NATS server to be ready..."
until nats -s "$NATS_URL" account info &>/dev/null; do
    sleep 1
done

echo "Creating/updating JetStream stream 'demoStream'..."
cat > /tmp/stream-config.json << 'EOF'
{
  "name": "demoStream",
  "subjects": ["demo.endpoint.updated.*"],
  "storage": "file",
  "retention": "limits",
  "max_msgs": -1,
  "max_bytes": -1,
  "max_age": 86400000000000,
  "max_msg_size": -1,
  "discard": "old",
  "ack": false,
  "replicas": 1
}
EOF

nats -s "$NATS_URL" stream add demoStream --config /tmp/stream-config.json || echo "Stream already exists"

echo "Creating/updating durable consumer 'demoConsumer'..."
cat > /tmp/consumer-config.json << 'EOF'
{
  "durable_name": "demoConsumer",
  "filter_subject": "demo.endpoint.updated.*",
  "ack_policy": "none",
  "deliver_policy": "all",
  "replay_policy": "instant",
  "inactive_threshold": 30000000000
}
EOF

nats -s "$NATS_URL" consumer add demoStream demoConsumer --config /tmp/consumer-config.json || echo "Consumer already exists"

# Cleanup temp files
rm -f /tmp/stream-config.json /tmp/consumer-config.json

echo "✅ NATS JetStream setup complete!"
