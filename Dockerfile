# Multi-stage build for optimized production image
FROM rust:1.70-slim as chef
RUN cargo install cargo-chef
WORKDIR /app

# Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependencies (this step is cached when dependencies don't change)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --bin vectordb-server

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r vecdb && useradd -r -g vecdb vecdb

# Create directories
RUN mkdir -p /data /etc/vectordb /var/log/vectordb \
    && chown -R vecdb:vecdb /data /var/log/vectordb

# Copy binary and config
COPY --from=builder /app/target/release/vectordb-server /usr/local/bin/
COPY --from=builder /app/config.toml /etc/vectordb/config.toml

# Make binary executable
RUN chmod +x /usr/local/bin/vectordb-server

# Switch to non-root user
USER vecdb

# Set working directory
WORKDIR /data

# Expose ports
# 8080: REST API
# 9090: gRPC API  
# 9091: Prometheus metrics
EXPOSE 8080 9090 9091

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["vectordb-server", "--config", "/etc/vectordb/config.toml"]

# Labels for better container management
LABEL maintainer="durai@infinidatum.com"
LABEL description="d-vecDB: High-performance vector database written in Rust"
LABEL version="0.1.1"
LABEL org.opencontainers.image.source="https://github.com/rdmurugan/d-vecDB"
LABEL org.opencontainers.image.url="https://github.com/rdmurugan/d-vecDB"
LABEL org.opencontainers.image.documentation="https://github.com/rdmurugan/d-vecDB#readme"
LABEL org.opencontainers.image.title="d-vecDB"
LABEL org.opencontainers.image.description="High-performance vector database written in Rust"