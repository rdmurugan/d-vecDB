# d-vecDB Server Python Package

[![PyPI version](https://badge.fury.io/py/d-vecdb-server.svg)](https://badge.fury.io/py/d-vecdb-server)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Python package that provides the d-vecDB server with embedded pre-built binaries for multiple platforms. This package allows you to run the high-performance d-vecDB vector database server directly from Python without requiring Rust toolchain or manual compilation.

## Installation

```bash
pip install d-vecdb-server
```

## Quick Start

### Command Line Usage

```bash
# Start the server (foreground)
d-vecdb-server start

# Start in background
d-vecdb-server start --daemon

# Start with custom settings
d-vecdb-server start --host 0.0.0.0 --port 8081 --data-dir ./my-data

# Stop the server
d-vecdb-server stop

# Check server status
d-vecdb-server status

# Show version
d-vecdb-server version
```

### Python API

```python
from d_vecdb_server import DVecDBServer

# Create and start server
server = DVecDBServer(
    host="127.0.0.1",
    port=8080,
    data_dir="./vector-data"
)

# Start the server
server.start()

print(f"Server running: {server.is_running()}")
print(f"REST API: http://{server.host}:{server.port}")
print(f"gRPC API: {server.host}:{server.grpc_port}")

# Stop the server
server.stop()
```

### Context Manager

```python
from d_vecdb_server import DVecDBServer

# Automatically start and stop server
with DVecDBServer(port=8080) as server:
    print(f"Server is running on port {server.port}")
    # Server will be automatically stopped when exiting the context
```

## Configuration

### Default Configuration

The server uses these defaults:
- **Host**: 127.0.0.1
- **REST Port**: 8080  
- **gRPC Port**: 9090
- **Data Directory**: Temporary directory (auto-generated)
- **Log Level**: info

### Custom Configuration

#### Via Python API

```python
server = DVecDBServer(
    host="0.0.0.0",           # Listen on all interfaces
    port=8081,                # Custom port
    grpc_port=9091,           # Custom gRPC port  
    data_dir="/path/to/data", # Persistent data directory
    log_level="debug",        # Verbose logging
    config_file="custom.toml" # Use external config file
)
```

#### Via Configuration File

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
hnsw_max_connections = 32
hnsw_ef_construction = 400
hnsw_max_layer = 16

[monitoring]
enable_metrics = true
prometheus_port = 9091
log_level = "info"
```

Then use it:

```bash
d-vecdb-server start --config config.toml
```

## API Reference

### DVecDBServer Class

#### Constructor

```python
DVecDBServer(
    host: str = "127.0.0.1",
    port: int = 8080,
    grpc_port: int = 9090,
    data_dir: Optional[str] = None,
    log_level: str = "info",
    config_file: Optional[str] = None
)
```

#### Methods

- **`start(background: bool = True, timeout: int = 30) -> bool`**
  - Start the server process
  - Returns `True` if successful

- **`stop(timeout: int = 10) -> bool`**
  - Stop the server process
  - Returns `True` if successful

- **`restart(timeout: int = 30) -> bool`**
  - Restart the server
  - Returns `True` if successful

- **`is_running() -> bool`**
  - Check if server is running
  - Returns `True` if running

- **`get_status() -> Dict[str, Any]`**
  - Get detailed server status
  - Returns status dictionary

### Command Line Interface

```bash
d-vecdb-server [OPTIONS] COMMAND

Commands:
  start    Start the server
  stop     Stop the server  
  status   Check server status
  version  Show version information

Options:
  --host HOST           Server host (default: 127.0.0.1)
  --port PORT           REST API port (default: 8080)
  --grpc-port PORT      gRPC port (default: 9090)
  --data-dir DIR        Data directory
  --config FILE         Configuration file
  --log-level LEVEL     Log level (debug/info/warn/error)

Start Options:
  --daemon              Run in background
```

## Platform Support

- **Linux**: x86_64 (with musl for better compatibility)
- **macOS**: Intel (x86_64) and Apple Silicon (ARM64)
- **Windows**: x86_64

## Development

### Installing from Source

```bash
git clone https://github.com/rdmurugan/d-vecDB.git
cd d-vecDB/d-vecdb-server-python

# Install in development mode
pip install -e .
```

## Troubleshooting

### Port Already in Use

```python
# Use different port
server = DVecDBServer(port=8081)
```

### Permission Denied

```bash
# Install in user directory
pip install --user d-vecdb-server
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- **Main Repository**: https://github.com/rdmurugan/d-vecDB
- **Python Client**: https://pypi.org/project/d-vecdb/
- **Issues**: https://github.com/rdmurugan/d-vecDB/issues