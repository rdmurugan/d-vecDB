# d-vecDB Server

[![Crates.io](https://img.shields.io/crates/v/d-vecdb-server.svg)](https://crates.io/crates/d-vecdb-server)
[![Documentation](https://docs.rs/d-vecdb-server/badge.svg)](https://docs.rs/d-vecdb-server)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/rdmurugan/d-vecDB#license)

A high-performance vector database server written in Rust. This is the standalone server binary for [d-vecDB](https://github.com/rdmurugan/d-vecDB).

## Features

- **Ultra-fast vector operations** with sub-microsecond latency
- **HNSW indexing** for efficient similarity search
- **REST and gRPC APIs** for universal client compatibility
- **Write-Ahead Logging** for durability and crash recovery
- **Prometheus metrics** for monitoring and observability
- **Concurrent processing** with Rust's fearless concurrency

## Installation

### Install from Crates.io

```bash
cargo install d-vecdb-server
```

### Quick Start

```bash
# Start the server with default settings
vectordb-server

# Start with custom configuration
vectordb-server --host 0.0.0.0 --port 8080 --data-dir ./data
```

### Configuration

Create a `config.toml` file:

```toml
[server]
host = "0.0.0.0"
port = 8080
grpc_port = 9090
workers = 8

[storage]
data_dir = "./data"
wal_sync_interval = "1s"
memory_map_size = "1GB"

[index]
hnsw_max_connections = 16
hnsw_ef_construction = 200

[monitoring]
enable_metrics = true
prometheus_port = 9091
```

Then run:
```bash
vectordb-server --config config.toml
```

## Usage

### REST API

```bash
# Create a collection
curl -X POST http://localhost:8080/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "documents", "dimension": 128, "distance_metric": "cosine"}'

# Insert vectors
curl -X POST http://localhost:8080/collections/documents/vectors \
  -H "Content-Type: application/json" \
  -d '{"id": "doc1", "data": [0.1, 0.2, 0.3, ...], "metadata": {"title": "Example"}}'

# Search for similar vectors
curl -X POST http://localhost:8080/collections/documents/search \
  -H "Content-Type: application/json" \
  -d '{"query_vector": [0.1, 0.2, 0.3, ...], "limit": 10}'
```

### Python Client

Install the Python client:
```bash
pip install d-vecdb
```

```python
from vectordb_client import VectorDBClient

client = VectorDBClient("localhost", 8080)
client.create_collection_simple("documents", 128, "cosine")
client.insert_simple("documents", "doc1", vector_data)
results = client.search_simple("documents", query_vector, limit=10)
```

## Command Line Options

```
vectordb-server [OPTIONS]

OPTIONS:
    -h, --host <HOST>               Server host [default: 127.0.0.1]
    -p, --port <PORT>               REST API port [default: 8080]
    -g, --grpc-port <GRPC_PORT>     gRPC port [default: 9090]
    -d, --data-dir <DATA_DIR>       Data directory [default: ./data]
    -c, --config <CONFIG>           Configuration file
    -l, --log-level <LOG_LEVEL>     Log level [default: info]
    -w, --workers <WORKERS>         Number of worker threads
        --help                      Print help information
        --version                   Print version information
```

## Performance

d-vecDB delivers exceptional performance:

- **Distance Calculations**: 35M+ operations/second
- **Vector Insertion**: 7K+ vectors/second  
- **Vector Search**: 13K+ queries/second
- **Sub-microsecond latency** for vector operations

## Monitoring

d-vecDB exposes Prometheus metrics on port 9091 by default:

```
# Vector operations
vectordb_operations_total
vectordb_operation_duration_seconds

# Collection statistics  
vectordb_collections_total
vectordb_vectors_total

# System metrics
vectordb_memory_usage_bytes
vectordb_disk_usage_bytes
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Links

- [Main Repository](https://github.com/rdmurugan/d-vecDB)
- [Python Client](https://pypi.org/project/d-vecdb/)
- [Docker Hub](https://hub.docker.com/r/rdmurugan/d-vecdb)
- [Documentation](https://github.com/rdmurugan/d-vecDB#readme)