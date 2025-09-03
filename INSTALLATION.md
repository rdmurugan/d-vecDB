# Installation Guide

d-vecDB can be installed in multiple ways depending on your needs and platform. Choose the method that best fits your use case.

## 🐍 Python Package (Recommended for most users)

The easiest way to get started is with the Python package, which includes pre-built binaries:

```bash
pip install d-vecdb-server
```

### Start the server:
```python
from d_vecdb_server import DVecDBServer

# Start server with default settings
server = DVecDBServer()
server.start()

# Or with custom configuration
server = DVecDBServer(
    host="0.0.0.0",
    port=8080,
    grpc_port=9090,
    data_dir="./data"
)
server.start()
```

### Command line usage:
```bash
# Start server directly
d-vecdb-server

# Or with options
vectordb-server --host 0.0.0.0 --rest-port 8080 --grpc-port 9090
```

### Platform Support:
- ✅ macOS (ARM64 & x86_64)
- ✅ Linux (x86_64) 
- ✅ Windows (coming soon)

Perfect for Google Colab, Jupyter notebooks, and development environments!

---

## 🍺 Homebrew (macOS)

For macOS users who prefer Homebrew:

### Option 1: Binary installation (faster)
```bash
brew install rdmurugan/tap/d-vecdb-bin
```

### Option 2: Source installation (latest features)
```bash
brew install rdmurugan/tap/d-vecdb
```

### Usage:
```bash
# Start server
vectordb-server

# Start as background service
brew services start d-vecdb-bin

# Stop service
brew services stop d-vecdb-bin

# View logs
tail -f /usr/local/var/log/d-vecdb.log
```

---

## 🦀 From Source (Rust)

For developers and advanced users:

### Prerequisites:
- Rust 1.75+ (`rustup update`)
- Protocol Buffers compiler (`protoc`)

```bash
# Install protoc
# macOS:
brew install protobuf

# Ubuntu/Debian:
sudo apt-get install protobuf-compiler

# Arch Linux:
sudo pacman -S protobuf
```

### Build and install:
```bash
git clone https://github.com/rdmurugan/d-vecDB.git
cd d-vecDB
cargo build --release --bin vectordb-server

# Install globally
cargo install --path server

# Or run directly
./target/release/vectordb-server
```

---

## 🐳 Docker

Run d-vecDB in a container:

```bash
# Pull and run
docker run -p 8080:8080 -p 9090:9090 rdmurugan/d-vecdb:latest

# Or with persistent data
docker run -p 8080:8080 -p 9090:9090 -v ./data:/data rdmurugan/d-vecdb:latest
```

### Docker Compose:
```yaml
version: '3.8'
services:
  d-vecdb:
    image: rdmurugan/d-vecdb:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./data:/data
    environment:
      - RUST_LOG=info
```

---

## 🌐 Client Libraries

### Python Client:
```bash
pip install d-vecdb
```

```python
from vectordb_client import VectorDBClient

client = VectorDBClient("http://localhost:8080")
```

### JavaScript/Node.js Client:
```bash
npm install d-vecdb-client
```

### Rust Client:
```toml
[dependencies]
d-vecdb-client = "0.1"
```

---

## ⚙️ Configuration

### Default Configuration:
d-vecDB runs with sensible defaults, but you can customize:

```toml
# config.toml
[server]
host = "127.0.0.1"
rest_port = 8080
grpc_port = 9090

[storage]
data_dir = "./data"

[performance]
workers = 4
batch_size = 1000
```

### Environment Variables:
```bash
export DVECDB_HOST="0.0.0.0"
export DVECDB_PORT="8080"
export RUST_LOG="info"
```

---

## 🚀 Quick Start

After installation, verify everything works:

```bash
# Check server is running
curl http://localhost:8080/health

# Create a collection
curl -X POST http://localhost:8080/collections \
  -H "Content-Type: application/json" \
  -d '{"name": "test", "dimension": 128}'

# Add vectors
curl -X POST http://localhost:8080/collections/test/vectors \
  -H "Content-Type: application/json" \
  -d '{"vectors": [{"id": "1", "data": [0.1, 0.2, ...]}]}'
```

---

## 🆘 Troubleshooting

### Common Issues:

**Port already in use:**
```bash
# Kill process using port 8080
lsof -ti:8080 | xargs kill -9

# Or use different port
vectordb-server --rest-port 8081
```

**Binary not found (Linux):**
```bash
# Make sure you have the right architecture
uname -m

# For ARM64 systems, build from source
```

**Python import error:**
```bash
# Update pip and reinstall
pip install --upgrade pip
pip install --force-reinstall d-vecdb-server
```

---

## 📚 What's Next?

1. **Quick Tutorial**: [Getting Started Guide](./docs/getting-started.md)
2. **API Documentation**: [API Reference](./docs/api.md)
3. **Performance Tuning**: [Optimization Guide](./docs/performance.md)
4. **Client Usage**: [Client Examples](./docs/clients.md)

---

## 💡 Need Help?

- 📖 **Documentation**: https://github.com/rdmurugan/d-vecDB#readme
- 🐛 **Issues**: https://github.com/rdmurugan/d-vecDB/issues
- 💬 **Discussions**: https://github.com/rdmurugan/d-vecDB/discussions

---

**Enjoy using d-vecDB! 🎉**