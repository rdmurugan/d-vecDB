#!/bin/bash

# Build Linux binary using Docker

echo "Building Linux binary using Docker..."

# Create a temporary Dockerfile for building
cat << 'EOF' > Dockerfile.linux-build
FROM rust:1.78-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy the source code
COPY . .

# Build the binary
RUN cargo build --release --bin vectordb-server

# Create the output directory
RUN mkdir -p /output

# Copy the binary to output
RUN cp target/release/vectordb-server /output/vectordb-server-linux-x64
EOF

# Build the Docker image and extract the binary
docker build -f Dockerfile.linux-build -t vecdb-linux-builder .

# Create a temporary container and copy the binary out
docker create --name temp-container vecdb-linux-builder
docker cp temp-container:/output/vectordb-server-linux-x64 d-vecdb-server-python/d_vecdb_server/binaries/

# Clean up
docker rm temp-container
docker rmi vecdb-linux-builder
rm Dockerfile.linux-build

echo "Linux binary built successfully!"
ls -la d-vecdb-server-python/d_vecdb_server/binaries/vectordb-server-linux-x64