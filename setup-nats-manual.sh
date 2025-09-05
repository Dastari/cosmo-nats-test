#!/bin/bash
echo "Setting up NATS streams manually..."

# Create stream using config approach
cat > /tmp/stream.conf << 'EOF'
{
  "name": "demoStream",
  "subjects": ["demo.endpoint.updated.*"],
  "storage": "file",
  "retention": "limits",
  "max_msgs": -1,
  "max_bytes": -1,
  "max_age": 86400000000000,
  "discard": "old",
  "replicas": 1
}
EOF

echo "Creating stream with config file..."
nats stream add demoStream --config /tmp/stream.conf

echo "Creating consumer..."
cat > /tmp/consumer.conf << 'EOF'
{
  "durable_name": "demoConsumer", 
  "filter_subject": "demo.endpoint.updated.*",
  "ack_policy": "none",
  "deliver_policy": "all",
  "replay_policy": "instant"
}
EOF

nats consumer add demoStream demoConsumer --config /tmp/consumer.conf

echo "Listing streams and consumers..."
nats stream ls
nats consumer ls demoStream

# Cleanup
rm -f /tmp/stream.conf /tmp/consumer.conf

echo "NATS setup complete!"
