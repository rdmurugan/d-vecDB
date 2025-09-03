#!/bin/bash

set -e

echo "Building multiple platform binaries..."

# Create binaries directory
mkdir -p d-vecdb-server-python/d_vecdb_server/binaries

# Build for current platform (macOS ARM64)
echo "Building for current platform..."
cargo build --release --bin vectordb-server
cp target/release/vectordb-server d-vecdb-server-python/d_vecdb_server/binaries/vectordb-server-darwin-arm64

# Build for macOS x86_64
echo "Building for macOS x86_64..."
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin --bin vectordb-server
cp target/x86_64-apple-darwin/release/vectordb-server d-vecdb-server-python/d_vecdb_server/binaries/vectordb-server-darwin-x64

# For Linux, we'll need to use Docker or download pre-built binaries
echo "Linux binary will need to be built separately using Docker or CI"

echo "Built binaries:"
ls -la d-vecdb-server-python/d_vecdb_server/binaries/