#!/bin/bash
set -e

# d-vecDB Server Build Script
# This script builds the Rust server components

echo "🔨 Building d-vecDB server components..."
echo

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust/Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust $(rustc --version) detected"

# Build configuration
build_type=${1:-"release"}

case $build_type in
    "debug")
        echo "🔧 Building debug version..."
        cargo build
        echo "✅ Debug build completed: ./target/debug/"
        ;;
    "release")
        echo "⚡ Building optimized release version..."
        cargo build --release
        echo "✅ Release build completed: ./target/release/"
        ;;
    "server-only")
        echo "🌐 Building server component only..."
        cargo build --release --bin vectordb-server
        echo "✅ Server build completed: ./target/release/vectordb-server"
        ;;
    "cli-only")
        echo "⌨️  Building CLI component only..."
        cargo build --release --bin vectordb-cli
        echo "✅ CLI build completed: ./target/release/vectordb-cli"
        ;;
    *)
        echo "Usage: $0 [debug|release|server-only|cli-only]"
        echo "  debug      - Build debug version"
        echo "  release    - Build optimized release (default)"
        echo "  server-only- Build only server binary"
        echo "  cli-only   - Build only CLI binary"
        exit 1
        ;;
esac

echo
echo "🎉 Build completed successfully!"
echo
echo "Start the server:"
if [ "$build_type" = "debug" ]; then
    echo "  ./target/debug/vectordb-server"
else
    echo "  ./target/release/vectordb-server"
fi
echo
echo "📚 Documentation: https://github.com/rdmurugan/d-vecDB#readme"