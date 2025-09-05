#!/bin/bash
set -euo pipefail

echo "🚀 Bootstrapping cosmo-rust-edfs environment..."

echo "📦 Installing system dependencies..."
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev unzip

echo "🦀 Installing Rust via rustup..."
if ! command -v rustup &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source ~/.cargo/env
fi
rustup update stable

echo "⚡ Installing just task runner..."
if ! command -v just &> /dev/null; then
    if apt list --installed 2>/dev/null | grep -q just; then
        sudo apt install -y just
    else
        cargo install just
    fi
fi

echo "📦 Installing Node.js LTS..."
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
    sudo apt install -y nodejs
fi

echo "🌊 Installing NATS server and CLI..."
if ! command -v nats-server &> /dev/null; then
    echo "Downloading NATS server..."
    mkdir -p bin
    curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.18/nats-server-v2.10.18-linux-amd64.zip -o nats-server.zip
    unzip -q nats-server.zip
    sudo mv nats-server-v2.10.18-linux-amd64/nats-server /usr/local/bin/
    rm -rf nats-server-v2.10.18-linux-amd64 nats-server.zip
fi

if ! command -v nats &> /dev/null; then
    echo "Downloading NATS CLI..."
    curl -L https://github.com/nats-io/natscli/releases/download/v0.1.5/nats-0.1.5-linux-amd64.zip -o nats-cli.zip
    unzip -q nats-cli.zip
    sudo mv nats-0.1.5-linux-amd64/nats /usr/local/bin/
    rm -rf nats-0.1.5-linux-amd64 nats-cli.zip
fi

echo "🌐 Downloading Cosmo Router binary..."
mkdir -p router/bin
cd router
# Disable telemetry for offline usage
export DISABLE_TELEMETRY=1
export POSTHOG_DISABLED=1
export WG_TELEMETRY_DISABLED=1
# Remove existing router if it exists and download fresh copy
rm -rf bin/router
npx wgc@latest router download-binary --out bin/router
chmod +x bin/router/router
cd ..

echo "✅ Bootstrap complete! Run 'just dev' to start the environment."
