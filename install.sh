#!/bin/bash
set -e

# d-vecDB Installation Script
# Automatically detects platform and installs the appropriate binary

REPO="rdmurugan/d-vecDB"
BINARY_NAME="vectordb-server"
INSTALL_DIR="/usr/local/bin"
TEMP_DIR="/tmp/d-vecdb-install"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform
detect_platform() {
    local os arch platform_suffix

    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            os="windows"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)
            arch="x64"
            ;;
        arm64|aarch64)
            arch="arm64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            print_error "Supported architectures: x86_64, arm64"
            exit 1
            ;;
    esac

    if [ "$os" = "linux" ]; then
        # Use musl version for better compatibility
        platform_suffix="${os}-musl-${arch}"
    else
        platform_suffix="${os}-${arch}"
    fi

    if [ "$os" = "windows" ]; then
        BINARY_NAME="${BINARY_NAME}.exe"
        platform_suffix="${platform_suffix}.exe"
    fi

    echo "${platform_suffix}"
}

# Get the latest release version
get_latest_version() {
    print_status "Fetching latest release version..."
    
    if command -v curl >/dev/null 2>&1; then
        local version=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    elif command -v wget >/dev/null 2>&1; then
        local version=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
    else
        print_error "Neither curl nor wget is available. Please install one of them."
        exit 1
    fi
    
    if [ -z "$version" ]; then
        print_error "Failed to get latest version. Using fallback version v0.1.1"
        echo "v0.1.1"
    else
        echo "$version"
    fi
}

# Download binary
download_binary() {
    local version=$1
    local platform=$2
    local asset_name="${BINARY_NAME}-${platform}"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"
    
    print_status "Downloading ${asset_name} from ${download_url}"
    
    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"
    
    if command -v curl >/dev/null 2>&1; then
        if ! curl -L -o "$BINARY_NAME" "$download_url"; then
            print_error "Failed to download binary"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -O "$BINARY_NAME" "$download_url"; then
            print_error "Failed to download binary"
            exit 1
        fi
    else
        print_error "Neither curl nor wget is available"
        exit 1
    fi
    
    # Verify download
    if [ ! -f "$BINARY_NAME" ] || [ ! -s "$BINARY_NAME" ]; then
        print_error "Downloaded file is empty or doesn't exist"
        exit 1
    fi
    
    print_success "Binary downloaded successfully"
}

# Install binary
install_binary() {
    print_status "Installing binary to $INSTALL_DIR"
    
    # Check if we have write permission
    if [ ! -w "$INSTALL_DIR" ]; then
        if command -v sudo >/dev/null 2>&1; then
            print_warning "Need sudo permission to install to $INSTALL_DIR"
            sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
            sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
        else
            print_error "No write permission to $INSTALL_DIR and sudo not available"
            print_error "Please run as root or choose a different install directory"
            exit 1
        fi
    else
        mv "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    print_success "Binary installed successfully"
}

# Verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local installed_version=$("$BINARY_NAME" --version 2>/dev/null | head -n1 || echo "version unknown")
        print_success "d-vecDB server installed successfully!"
        print_success "Version: $installed_version"
        print_success "Location: $(which $BINARY_NAME)"
    else
        print_warning "Binary installed but not found in PATH"
        print_warning "You may need to add $INSTALL_DIR to your PATH"
        print_warning "Or run: export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

# Cleanup
cleanup() {
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Show usage info
show_usage() {
    cat << EOF
🚀 d-vecDB Installation Complete!

Quick Start:
  1. Start the server:
     $BINARY_NAME

  2. Start with custom settings:
     $BINARY_NAME --host 0.0.0.0 --port 8080

  3. Use configuration file:
     $BINARY_NAME --config config.toml

  4. Get help:
     $BINARY_NAME --help

Python Client:
  Install: pip install d-vecdb
  
  Usage:
    from vectordb_client import VectorDBClient
    client = VectorDBClient("localhost", 8080)

Documentation: https://github.com/rdmurugan/d-vecDB#readme
Support: https://github.com/rdmurugan/d-vecDB/issues

Happy vector searching! 🎯
EOF
}

# Main installation process
main() {
    echo
    echo "🚀 d-vecDB Installation Script"
    echo "=============================="
    echo
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --install-dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --version VERSION    Install specific version (default: latest)"
                echo "  --install-dir DIR    Install directory (default: /usr/local/bin)"
                echo "  --help              Show this help message"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Detect platform
    PLATFORM=$(detect_platform)
    print_status "Detected platform: $PLATFORM"
    
    # Get version
    if [ -z "$VERSION" ]; then
        VERSION=$(get_latest_version)
    fi
    print_status "Installing version: $VERSION"
    
    # Check for existing installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        print_warning "d-vecDB server is already installed at $(which $BINARY_NAME)"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_status "Installation cancelled"
            exit 0
        fi
    fi
    
    # Download and install
    download_binary "$VERSION" "$PLATFORM"
    install_binary
    verify_installation
    
    echo
    show_usage
}

# Run main function
main "$@"