#!/bin/bash
set -euo pipefail

echo "🌊 Setting up NATS JetStream configuration..."

NATS_URL="nats://127.0.0.1:4222"

echo "Waiting for NATS server to be ready..."
until nats -s "$NATS_URL" account info &>/dev/null; do
    sleep 1
done

echo "Creating JetStream stream 'demoStream'..."
echo | nats -s "$NATS_URL" stream add demoStream \
    --subjects "demo.endpoint.updated.*" \
    --storage file \
    --retention limits \
    --max-msgs=-1 \
    --max-bytes=-1 \
    --max-age=24h \
    --max-msg-size=-1 \
    --num-replicas=1 \
    --discard old \
    --no-ack \
    --replicas 1 || echo "Stream might already exist"

echo "Creating durable consumer 'demoConsumer'..."
printf "\n\n\n\n\n" | nats -s "$NATS_URL" consumer add demoStream demoConsumer \
    --filter "demo.endpoint.updated.*" \
    --ack none \
    --deliver all \
    --replay instant \
    --inactive-threshold=30s || echo "Consumer might already exist"

echo "✅ NATS JetStream setup complete!"
