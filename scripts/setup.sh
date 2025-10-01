#!/bin/bash

echo "🚀 Setting up Aura Chain development environment..."

# Install Rust if not exists
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

# Install dependencies
echo "Installing dependencies..."
sudo apt update
sudo apt install -y git curl wget build-essential clang pkg-config libssl-dev

echo "✅ Aura Chain environment ready!"
echo "🔨 Run: cargo build --release"
