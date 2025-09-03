# Changelog

All notable changes to the d-vecdb-server package will be documented in this file.

## [0.1.2] - 2025-09-02

### 🚀 Major Features
- **Complete Package**: Now includes embedded d-vecDB Rust binary (4MB) - no external dependencies required
- **Zero Configuration**: Single `pip install d-vecdb-server` provides complete vector database server
- **Fixed Server Startup**: Resolved critical server initialization issues

### 🛠️ Bug Fixes
- **Fixed CLI Error**: Resolved `'Namespace' object has no attribute 'daemon'` error in default command handling
- **Fixed Server Configuration**: Added missing `--metrics-port` parameter for proper server startup
- **Fixed Host Resolution**: Convert localhost to 127.0.0.1 for server compatibility
- **Fixed Build Isolation**: Added proper build dependencies in pyproject.toml

### 🔧 Improvements
- **Enhanced Binary Detection**: Automatically finds embedded binary before falling back to PATH
- **Robust Server Management**: Better error handling and graceful shutdowns
- **Command Line Arguments**: Switched from config files to command line arguments for reliability
- **Port Management**: Automatic metrics port allocation (grpc_port + 1)

### 📦 Package Changes
- **Embedded Binary**: d-vecDB server binary now included in package (`d_vecdb_server/binaries/`)
- **Updated Dependencies**: Added requests>=2.25.0 as build dependency
- **Improved Packaging**: Modern pyproject.toml configuration with proper binary inclusion

### 🧪 Testing Enhancements
- **Enhanced Test Suite**: Python client tests now support embedded server
- **Automatic Server Management**: Tests can automatically start/stop server instances
- **Health Check Workarounds**: Handle API format differences gracefully
- **Isolated Testing**: Support for running tests with dedicated server instances

### 📚 Documentation
- Updated package description to reflect complete functionality
- Enhanced README with installation and usage examples
- Added comprehensive changelog

### ⚡ Breaking Changes
- None - fully backward compatible

### 🔄 Migration Guide
**From 0.1.1 to 0.1.2**: No changes required. The package now works out of the box without needing external server installation.

**Before**: Required manual d-vecDB server installation and startup
```bash
# Previous workflow
cargo install d-vecdb  # Install server separately
d-vecdb &              # Start server manually
python your_script.py  # Use Python package
```

**After**: Zero configuration - everything included
```bash
# New workflow
pip install d-vecdb-server  # Complete package
python your_script.py       # Server starts automatically
```

### 🏗️ Internal Changes
- Refactored server startup logic for better reliability
- Improved error messages and debugging output
- Enhanced binary path resolution
- Better handling of temporary files and cleanup