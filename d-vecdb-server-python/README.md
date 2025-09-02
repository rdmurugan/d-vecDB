# d-vecDB Server Python Package

[![PyPI version](https://badge.fury.io/py/d-vecdb-server.svg)](https://badge.fury.io/py/d-vecdb-server)
[![Python 3.8+](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Python package that provides the d-vecDB server with embedded pre-built binaries. This package allows you to run the high-performance d-vecDB vector database server directly from Python with a single `pip install` command.

## Prerequisites

- **Python**: 3.8 or higher
- **Operating System**: Linux (x86_64), macOS (Intel/Apple Silicon), or Windows (x86_64)
- **Memory**: Minimum 512MB RAM available
- **Disk Space**: At least 100MB for binaries and data storage
- **Network**: Available ports for REST API (default: 8080) and gRPC (default: 9090)

## Installation Options

### Option 1: Standard Installation (Recommended)

```bash
# Install the server package
pip install d-vecdb-server

# Verify installation
d-vecdb-server version
```

> **Note**: The package includes embedded binaries and is ready to use immediately after installation.

### Option 2: Development Installation

```bash
# Clone the repository
git clone https://github.com/rdmurugan/d-vecDB.git
cd d-vecDB/d-vecdb-server-python

# Install in development mode
pip install -e .

# Verify installation
d-vecdb-server version
```

### Option 3: Virtual Environment Installation (Recommended for Development)

```bash
# Create virtual environment
python -m venv d-vecdb-env

# Activate virtual environment
# On Linux/macOS:
source d-vecdb-env/bin/activate
# On Windows:
d-vecdb-env\Scripts\activate

# Install package
pip install d-vecdb-server

# Verify installation
d-vecdb-server version
```

### Option 4: Install with Python Client

```bash
# Install server with Python client for complete functionality  
pip install 'd-vecdb-server[client]'

# Or install separately
pip install d-vecdb-server
pip install d-vecdb  # Python client library
```

## Quick Start

### Command Line Usage

```bash
# Show version (includes binary status)
d-vecdb-server version

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

### Step-by-Step Server Configuration

#### Step 1: Choose Your Configuration Method

You can configure the server in three ways:
1. **Command line arguments** (quick setup)
2. **Python API parameters** (programmatic setup)
3. **Configuration file** (persistent setup)

#### Step 2: Basic Configuration

**Default Configuration:**
- **Host**: 127.0.0.1 (localhost only)
- **REST Port**: 8080
- **gRPC Port**: 9090  
- **Data Directory**: Temporary directory (auto-generated)
- **Log Level**: info

#### Step 3: Create Data Directory (Optional)

```bash
# Create persistent data directory
mkdir -p /path/to/your/vector-data
chmod 755 /path/to/your/vector-data
```

#### Step 4: Configuration Options

**Option A: Command Line Configuration**

```bash
# Basic configuration
d-vecdb-server start --host 0.0.0.0 --port 8081 --data-dir ./data

# Advanced configuration
d-vecdb-server start \
  --host 0.0.0.0 \
  --port 8081 \
  --grpc-port 9091 \
  --data-dir /path/to/data \
  --log-level debug \
  --daemon
```

**Option B: Python API Configuration**

```python
from d_vecdb_server import DVecDBServer

# Basic configuration
server = DVecDBServer(
    host="0.0.0.0",           # Listen on all interfaces
    port=8081,                # Custom REST port
    grpc_port=9091,           # Custom gRPC port
    data_dir="/path/to/data", # Persistent data directory
    log_level="debug"         # Verbose logging
)

server.start()
```

**Option C: Configuration File Setup**

1. Create a configuration file:

```bash
# Create config directory
mkdir -p ~/.config/d-vecdb

# Create configuration file
cat > ~/.config/d-vecdb/server.toml << EOF
[server]
host = "0.0.0.0"
port = 8080
grpc_port = 9090
workers = 8

[storage]
data_dir = "/path/to/your/data"
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
EOF
```

2. Use the configuration file:

```bash
# Start with configuration file
d-vecdb-server start --config ~/.config/d-vecdb/server.toml
```

#### Step 5: Verify Configuration

```bash
# Check server status
d-vecdb-server status

# Test REST API endpoint
curl http://localhost:8080/health

# Check metrics (if enabled)
curl http://localhost:9091/metrics
```

#### Step 6: Production Configuration Tips

```toml
# Production-ready configuration example
[server]
host = "0.0.0.0"          # Accept connections from any IP
port = 8080               # Standard HTTP port
grpc_port = 9090          # Standard gRPC port
workers = 16              # Match CPU cores

[storage]
data_dir = "/var/lib/d-vecdb"  # Persistent storage location
wal_sync_interval = "5s"       # Longer interval for better performance
memory_map_size = "8GB"        # More memory for larger datasets

[index]
hnsw_max_connections = 64      # Higher connections for better recall
hnsw_ef_construction = 800     # Higher construction for better quality
hnsw_max_layer = 16           # Default is usually fine

[monitoring]
enable_metrics = true
prometheus_port = 9091
log_level = "warn"            # Less verbose for production
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

## Advanced Setup

### Environment Variables

You can also configure the server using environment variables:

```bash
# Set environment variables
export DVECDB_HOST="0.0.0.0"
export DVECDB_PORT="8080"
export DVECDB_GRPC_PORT="9090"
export DVECDB_DATA_DIR="/var/lib/d-vecdb"
export DVECDB_LOG_LEVEL="info"
export RUST_LOG="info"

# Start server (will use environment variables)
d-vecdb-server start
```

### Docker Setup (Alternative)

If you prefer Docker deployment:

```bash
# Pull the Docker image
docker pull rdmurugan/d-vecdb:latest

# Run with custom configuration
docker run -d \
  --name d-vecdb-server \
  -p 8080:8080 \
  -p 9090:9090 \
  -v /path/to/data:/data \
  rdmurugan/d-vecdb:latest
```

### Service Configuration (Linux)

Create a systemd service for automatic startup:

```bash
# Create service file
sudo cat > /etc/systemd/system/d-vecdb.service << EOF
[Unit]
Description=d-vecDB Vector Database Server
After=network.target

[Service]
Type=simple
User=d-vecdb
Group=d-vecdb
WorkingDirectory=/var/lib/d-vecdb
ExecStart=/usr/local/bin/d-vecdb-server start --config /etc/d-vecdb/server.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable d-vecdb
sudo systemctl start d-vecdb
sudo systemctl status d-vecdb
```

## Troubleshooting

### Installation Issues

**Binary Not Found (should not happen with pip install):**
```bash
# Check if binary is available
d-vecdb-server version

# If you still get binary not found errors:
# 1. Reinstall the package
pip uninstall d-vecdb-server
pip install d-vecdb-server

# 2. For development installations:
pip install -e . --force-reinstall
```

**Permission Denied:**
```bash
# Install in user directory
pip install --user d-vecdb-server

# Or use virtual environment
python -m venv venv && source venv/bin/activate
pip install d-vecdb-server
```

### Runtime Issues

**Port Already in Use:**
```bash
# Check what's using the port
lsof -i :8080  # On Linux/macOS
netstat -ano | findstr :8080  # On Windows

# Use different port
d-vecdb-server start --port 8081
```

**Server Won't Start:**
```bash
# Check server logs
d-vecdb-server start  # Run in foreground to see errors

# Check disk space
df -h  # On Linux/macOS

# Check permissions on data directory
ls -la /path/to/data/directory
```

**Connection Refused:**
```bash
# Verify server is running
d-vecdb-server status

# Check if ports are accessible
telnet localhost 8080

# For remote connections, ensure host is set to 0.0.0.0
d-vecdb-server start --host 0.0.0.0
```

### Performance Issues

**Slow Performance:**
```toml
# Optimize configuration for better performance
[storage]
memory_map_size = "4GB"  # Increase based on available RAM

[index]
hnsw_max_connections = 64
hnsw_ef_construction = 800

[server]
workers = 16  # Match your CPU cores
```

**High Memory Usage:**
```toml
# Reduce memory usage
[storage]
memory_map_size = "512MB"  # Reduce if needed

[index]
hnsw_max_connections = 16
hnsw_ef_construction = 200
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Using with Python Client

After installing the server, you can use it with the Python client:

```bash
# Install the Python client
pip install d-vecdb
```

```python
from d_vecdb_server import DVecDBServer
from d_vecdb import VectorDBClient
import numpy as np

# Start the server
with DVecDBServer(port=8080) as server:
    # Connect client
    client = VectorDBClient(host=server.host, port=server.port)
    
    # Create collection
    client.create_collection_simple("documents", 128, "cosine")
    
    # Insert vectors
    vector = np.random.random(128)
    client.insert_simple("documents", "doc1", vector)
    
    # Search
    query = np.random.random(128)
    results = client.search_simple("documents", query, limit=5)
    print(f"Found {len(results)} similar vectors")
```

## Next Steps

After installation and configuration:

1. **Start using the REST API**: Visit `http://localhost:8080/docs` for API documentation
2. **Use Python client**: See example above for Python integration
3. **Check examples**: See the main repository for usage examples
4. **Join community**: Report issues and get support

## Links

- **Main Repository**: https://github.com/rdmurugan/d-vecDB
- **Python Client**: https://pypi.org/project/d-vecdb/
- **Docker Hub**: https://hub.docker.com/r/rdmurugan/d-vecdb
- **Issues & Support**: https://github.com/rdmurugan/d-vecDB/issues
- **Documentation**: https://github.com/rdmurugan/d-vecDB#readme