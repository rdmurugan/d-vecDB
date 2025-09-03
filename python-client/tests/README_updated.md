# d-vecDB Python Client Tests (Updated with Embedded Server)

This directory contains comprehensive tests for the d-vecDB Python client library with automatic server management.

## New Features ✨

- **🚀 Automatic Server Management**: Tests can now automatically start and stop the d-vecDB server
- **📦 Embedded Server Support**: Integration with the `d-vecdb-server` Python package
- **🧪 Zero Configuration Testing**: Run tests without manually starting a server
- **🔧 Flexible Configuration**: Support both embedded and external servers

## Test Structure

```
tests/
├── conftest.py                        # Original pytest configuration (requires manual server)
├── conftest_with_embedded_server.py   # NEW: Auto-managed server configuration  
├── test_sync_client.py                # Synchronous client tests
├── test_async_client.py               # Asynchronous client tests
├── test_types.py                      # Type validation and data model tests
├── test_performance.py                # Performance and benchmark tests
├── test_integration.py                # Integration and workflow tests
└── README.md                          # This file
```

## Quick Start 🏃‍♂️

### Option 1: Zero-Configuration Testing (Recommended)

```bash
# Install the embedded server package
pip install d-vecdb-server

# Run tests - server starts automatically!
pytest --confcutdir=conftest_with_embedded_server.py
```

### Option 2: External Server Testing (Classic)

```bash
# Start server manually
cargo run --bin vectordb-server

# Run tests with original configuration
pytest
```

## Installation

### For Embedded Server Testing (Recommended)
```bash
# Install the complete test environment
pip install d-vecdb-server  # Embedded server
pip install -e .[dev]       # Client with dev dependencies

# Verify embedded server is available
python -c "from d_vecdb_server import DVecDBServer; print('✅ Embedded server available')"
```

### For External Server Testing
```bash
# Just install client dependencies
pip install -e .[dev]

# Start server separately
cargo run --bin vectordb-server
```

## Running Tests

### Embedded Server Mode (Auto-managed) 🤖

```bash
# Use the embedded server configuration
export PYTEST_CONFIG=conftest_with_embedded_server.py

# Run all tests - server starts/stops automatically
pytest

# Run specific test categories
pytest -m integration   # Server auto-starts for integration tests
pytest -m performance   # Server auto-starts for performance tests

# Run with verbose server output
VECTORDB_AUTO_START_SERVER=true pytest -s
```

### External Server Mode (Manual) 👤

```bash
# Start server first
cargo run --bin vectordb-server

# Run tests with original configuration
pytest --confcutdir=conftest.py

# Or set environment variable
export VECTORDB_USE_EMBEDDED_SERVER=false
pytest
```

## Configuration Options

Control test behavior with environment variables:

### Server Management
```bash
# Enable/disable embedded server (default: true)
export VECTORDB_USE_EMBEDDED_SERVER=true

# Auto-start server for tests (default: true)  
export VECTORDB_AUTO_START_SERVER=true

# Server connection settings
export VECTORDB_TEST_HOST=localhost
export VECTORDB_TEST_PORT=8080
export VECTORDB_TEST_GRPC_PORT=9090
```

### Example Configurations

**Full Auto Mode** (Default):
```bash
# Everything automatic - just run pytest!
pytest
```

**External Server Mode**:
```bash
# Use external server on custom ports
export VECTORDB_USE_EMBEDDED_SERVER=false
export VECTORDB_TEST_PORT=8081
export VECTORDB_TEST_GRPC_PORT=9091

# Start your server
cargo run --bin vectordb-server -- --rest-port 8081 --grpc-port 9091

# Run tests
pytest
```

**Development Mode**:
```bash
# Auto-start but with verbose output for debugging
export VECTORDB_AUTO_START_SERVER=true
pytest -s -v
```

## Test Fixtures (Enhanced)

### New Server Management Fixtures
- `vectordb_server`: Session-scoped embedded server instance
- `server_ports`: Auto-allocated free ports for testing
- `test_config`: Adaptive configuration based on server type

### Existing Client Fixtures (Enhanced)
- `client`: Auto-configured synchronous VectorDB client
- `async_client`: Auto-configured asynchronous VectorDB client

### Data Fixtures (Unchanged)
- `sample_vectors`: Set of test vectors for basic operations
- `large_sample_vectors`: Larger dataset for performance tests
- `query_vector`: Normalized query vector for search tests
- `test_collection_config`: Standard collection configuration

## Example Usage

### Simple Test with Auto-Server
```python
def test_my_feature(client, clean_collection):
    """Test runs with automatically managed server."""
    # Server is already running - just use the client!
    collection_name = clean_collection
    
    # Your test code here
    client.create_collection_simple(collection_name, 128, "cosine")
    result = client.ping()
    assert result
```

### Test with Server Access
```python  
def test_server_info(vectordb_server, client):
    """Test that can access server instance."""
    if vectordb_server:
        print(f"Using embedded server on port {vectordb_server.port}")
        assert vectordb_server.is_running()
    else:
        print("Using external server")
    
    # Client works with either server type
    assert client.ping()
```

### Performance Test with Auto-Server
```python
@pytest.mark.performance
def test_performance_with_embedded_server(client, clean_collection):
    """Performance tests work seamlessly with embedded server."""
    collection_name = clean_collection
    
    # Server is optimally configured for testing
    # Run your performance tests normally
    pass
```

## Advantages of Embedded Server Testing

### ✅ Benefits
- **Zero Configuration**: No manual server setup required
- **Isolated Testing**: Each test session gets a fresh server
- **Port Management**: Automatic free port allocation
- **Consistent Environment**: Same server version as package
- **CI/CD Friendly**: Works in automated environments
- **Faster Feedback**: No external dependencies

### 📊 Comparison

| Feature | Embedded Server | External Server |
|---------|----------------|-----------------|
| Setup Required | None | Manual startup |
| Port Conflicts | Auto-resolved | Manual management |
| Server Version | Always matches | May differ |
| Test Isolation | Perfect | Shared state |
| CI/CD Setup | Simple | Complex |
| Debug Access | Full control | Limited |

## Troubleshooting

### Common Issues with Embedded Server

**Server won't start:**
```bash
# Check if d-vecdb-server package is installed
python -c "from d_vecdb_server import DVecDBServer; print('OK')"

# Check for port conflicts
export VECTORDB_TEST_PORT=8090
pytest
```

**Tests still looking for external server:**
```bash
# Ensure you're using the new configuration
pytest --confcutdir=conftest_with_embedded_server.py

# Or set environment variable
export VECTORDB_USE_EMBEDDED_SERVER=true
pytest
```

**Performance issues:**
```bash
# Embedded server uses optimized settings for testing
# If you need production settings, use external server mode
export VECTORDB_USE_EMBEDDED_SERVER=false
```

### Migration from Manual Server

To migrate existing test runs:

1. **Install embedded server package**:
   ```bash
   pip install d-vecdb-server
   ```

2. **Update test runner** (choose one):
   ```bash
   # Option A: Use new conftest
   pytest --confcutdir=conftest_with_embedded_server.py
   
   # Option B: Rename files
   mv conftest.py conftest_manual.py  
   mv conftest_with_embedded_server.py conftest.py
   pytest
   ```

3. **Update CI/CD** (GitHub Actions example):
   ```yaml
   steps:
   - name: Install test dependencies
     run: |
       pip install d-vecdb-server  # Add this line
       pip install -e .[dev]
   
   - name: Run tests  
     run: pytest  # Remove manual server startup
   ```

### Getting Help

- **Embedded server issues**: Check d-vecdb-server package documentation
- **Test configuration**: Review environment variables and fixture setup
- **Performance**: Compare embedded vs external server results
- **CI/CD**: Verify d-vecdb-server package is installed in CI environment

## Contributing

When adding new tests with embedded server support:

1. **Use adaptive fixtures**: Tests should work with both server types
2. **Handle server unavailable**: Graceful skipping when server can't start  
3. **Preserve compatibility**: Don't break existing external server workflows
4. **Document assumptions**: Note if test requires specific server features

### Example Adaptive Test
```python
def test_adaptive_feature(test_config, client):
    """Test that works with both embedded and external servers."""
    
    # Get server info
    if test_config["embedded_server"]:
        # Embedded server specific setup
        server = test_config["embedded_server"]
        print(f"Using embedded server: {server.is_running()}")
    else:
        # External server - limited introspection
        print("Using external server")
    
    # Common test logic works with both
    assert client.ping()
    # ... rest of test
```