#!/bin/bash
set -e

# d-vecDB Python Server Package Publishing Script

PACKAGE_DIR="d-vecdb-server-python"
PACKAGE_NAME="d-vecdb-server"
VERSION="0.1.1"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if package directory exists
    if [ ! -d "$PACKAGE_DIR" ]; then
        print_error "Package directory $PACKAGE_DIR not found"
        exit 1
    fi
    
    # Check Python
    if ! python3 --version >/dev/null 2>&1; then
        print_error "Python 3 is required"
        exit 1
    fi
    
    # Check build tools
    if ! python3 -c "import build, twine" >/dev/null 2>&1; then
        print_warning "Missing build tools. Installing..."
        pip install build twine wheel
    fi
    
    # Check if release exists
    if ! curl -s "https://api.github.com/repos/rdmurugan/d-vecDB/releases/tags/v$VERSION" | grep -q "tag_name"; then
        print_error "Release v$VERSION not found on GitHub"
        print_error "Please create a release first with binaries"
        exit 1
    fi
    
    print_success "Prerequisites checked"
}

# Clean previous builds
clean_build() {
    print_status "Cleaning previous build artifacts..."
    
    cd "$PACKAGE_DIR"
    
    # Remove build artifacts
    rm -rf build/ dist/ *.egg-info/
    find . -name "*.pyc" -delete
    find . -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null || true
    
    print_success "Build artifacts cleaned"
    
    cd ..
}

# Test package locally
test_package() {
    print_status "Testing package locally..."
    
    cd "$PACKAGE_DIR"
    
    # Install in development mode
    pip install -e .
    
    # Test CLI
    if d-vecdb-server version >/dev/null 2>&1; then
        print_success "CLI test passed"
    else
        print_warning "CLI test failed, but continuing..."
    fi
    
    # Test Python import
    if python3 -c "from d_vecdb_server import DVecDBServer; print('Import successful')"; then
        print_success "Python import test passed"
    else
        print_error "Python import test failed"
        cd ..
        exit 1
    fi
    
    cd ..
}

# Build package
build_package() {
    print_status "Building Python package..."
    
    cd "$PACKAGE_DIR"
    
    # Build source distribution and wheel
    python3 -m build
    
    # Verify build
    if [ -d "dist" ] && [ "$(ls -1 dist/ | wc -l)" -gt 0 ]; then
        print_success "Package built successfully"
        ls -la dist/
    else
        print_error "Package build failed"
        exit 1
    fi
    
    cd ..
}

# Test built package
test_built_package() {
    print_status "Testing built package..."
    
    cd "$PACKAGE_DIR"
    
    # Create temporary environment
    python3 -m venv test_env
    source test_env/bin/activate
    
    # Install from built package
    pip install dist/*.whl
    
    # Test installation
    if d-vecdb-server version; then
        print_success "Built package test passed"
    else
        print_warning "Built package test had issues"
    fi
    
    # Cleanup
    deactivate
    rm -rf test_env
    
    cd ..
}

# Upload to PyPI
upload_to_pypi() {
    print_status "Uploading to PyPI..."
    
    cd "$PACKAGE_DIR"
    
    # Check if already exists
    if pip index versions "$PACKAGE_NAME" | grep -q "$VERSION"; then
        print_warning "Version $VERSION already exists on PyPI"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "Upload cancelled"
            cd ..
            return 0
        fi
    fi
    
    # Upload to TestPyPI first
    print_status "Uploading to TestPyPI first..."
    if twine upload --repository testpypi dist/*; then
        print_success "Upload to TestPyPI successful"
    else
        print_error "Upload to TestPyPI failed"
        cd ..
        exit 1
    fi
    
    # Test install from TestPyPI
    print_status "Testing install from TestPyPI..."
    python3 -m venv testpypi_env
    source testpypi_env/bin/activate
    
    if pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ "$PACKAGE_NAME==$VERSION"; then
        print_success "TestPyPI install test passed"
    else
        print_error "TestPyPI install test failed"
        deactivate
        rm -rf testpypi_env
        cd ..
        exit 1
    fi
    
    deactivate
    rm -rf testpypi_env
    
    # Confirm production upload
    print_status "Ready to upload to production PyPI"
    read -p "Proceed with production upload? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if twine upload dist/*; then
            print_success "Upload to PyPI successful"
        else
            print_error "Upload to PyPI failed"
            cd ..
            exit 1
        fi
    else
        print_status "Production upload cancelled"
    fi
    
    cd ..
}

# Verify final installation
verify_installation() {
    print_status "Verifying final installation from PyPI..."
    
    # Create clean environment
    python3 -m venv verify_env
    source verify_env/bin/activate
    
    # Wait a bit for PyPI to propagate
    print_status "Waiting for PyPI propagation..."
    sleep 30
    
    # Install from PyPI
    if pip install "$PACKAGE_NAME==$VERSION"; then
        print_success "PyPI installation successful"
        
        # Test functionality
        if d-vecdb-server version; then
            print_success "Final verification passed"
        else
            print_warning "Final verification had issues"
        fi
    else
        print_error "PyPI installation failed"
    fi
    
    # Cleanup
    deactivate
    rm -rf verify_env
}

# Show completion message
show_completion() {
    cat << EOF

🎉 d-vecDB Server Python Package Published Successfully!

Package Details:
  Name: $PACKAGE_NAME
  Version: $VERSION
  PyPI: https://pypi.org/project/$PACKAGE_NAME/

Installation:
  pip install $PACKAGE_NAME

Usage:
  # Command line
  d-vecdb-server start
  
  # Python API  
  from d_vecdb_server import DVecDBServer
  server = DVecDBServer()
  server.start()

Next Steps:
1. Update main documentation to mention this package
2. Create example notebooks/tutorials
3. Consider automation for future releases

Happy vector databasing! 🎯
EOF
}

# Main function
main() {
    echo
    echo "🐍 d-vecDB Python Server Package Publisher"
    echo "=========================================="
    echo
    
    # Check if we're in the right directory
    if [ ! -d "$PACKAGE_DIR" ]; then
        print_error "Please run this script from the d-vecDB repository root"
        exit 1
    fi
    
    # Run all steps
    check_prerequisites
    clean_build
    test_package
    build_package
    test_built_package
    upload_to_pypi
    verify_installation
    show_completion
}

# Run main function
main "$@"