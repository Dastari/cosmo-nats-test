#!/bin/bash
set -euo pipefail

if [ $# -lt 2 ]; then
    echo "Usage: $0 <id> <json_payload>"
    echo "Example: $0 1 '{\"id\":\"1\"}'"
    exit 1
fi

ID=$1
PAYLOAD=$2
SUBJECT="demo.endpoint.updated.$ID"
NATS_URL="nats://127.0.0.1:4222"

echo "📢 Publishing to NATS JetStream..."
echo "Subject: $SUBJECT"
echo "Payload: $PAYLOAD"

nats -s "$NATS_URL" pub "$SUBJECT" "$PAYLOAD"

echo "✅ Event published successfully!"
