#!/bin/bash
set -e

# d-vecDB Crate Publishing Script
# Publishes the d-vecdb-server crate to crates.io

echo "📦 Publishing d-vecdb-server to crates.io..."
echo

# Check if user is logged in to crates.io
if ! cargo login --help >/dev/null 2>&1; then
    echo "❌ Error: cargo not found. Please install Rust."
    exit 1
fi

# Navigate to the standalone crate directory
CRATE_DIR="d-vecdb-server"
if [ ! -d "$CRATE_DIR" ]; then
    echo "❌ Error: $CRATE_DIR directory not found."
    echo "Please run this script from the d-vecDB repository root."
    exit 1
fi

cd "$CRATE_DIR"

# Verify crate structure
echo "🔍 Verifying crate structure..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found in $CRATE_DIR"
    exit 1
fi

if [ ! -f "README.md" ]; then
    echo "❌ Error: README.md not found in $CRATE_DIR"
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "📋 Version: $VERSION"

# Check if version already exists on crates.io
echo "🔍 Checking if version $VERSION already exists..."
if cargo search d-vecdb-server | grep -q "d-vecdb-server.*$VERSION"; then
    echo "⚠️  Warning: Version $VERSION already exists on crates.io"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Cancelled"
        exit 1
    fi
fi

# Dry run first
echo "🧪 Running dry-run publish..."
if ! cargo publish --dry-run; then
    echo "❌ Dry run failed. Please fix the errors above."
    exit 1
fi

echo "✅ Dry run successful!"
echo

# Confirm publication
echo "📋 Ready to publish d-vecdb-server v$VERSION to crates.io"
echo "This will:"
echo "  - Upload the crate to crates.io"
echo "  - Make it available via 'cargo install d-vecdb-server'"
echo "  - Create a permanent public record"
echo
read -p "Proceed with publication? (y/N): " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled"
    exit 1
fi

# Actual publication
echo "🚀 Publishing to crates.io..."
if cargo publish; then
    echo
    echo "🎉 Successfully published d-vecdb-server v$VERSION!"
    echo
    echo "📦 Users can now install with:"
    echo "    cargo install d-vecdb-server"
    echo
    echo "📖 View on crates.io:"
    echo "    https://crates.io/crates/d-vecdb-server"
    echo
    echo "📚 Documentation will be available at:"
    echo "    https://docs.rs/d-vecdb-server"
    
    # Wait for indexing
    echo "⏳ Waiting for crates.io indexing..."
    sleep 10
    
    # Verify installation works
    echo "🧪 Testing installation..."
    if cargo install d-vecdb-server --force; then
        echo "✅ Installation test successful!"
    else
        echo "⚠️  Installation test failed, but publication succeeded"
    fi
else
    echo "❌ Publication failed!"
    exit 1
fi

echo
echo "🎯 Next steps:"
echo "1. Update documentation to mention 'cargo install d-vecdb-server'"
echo "2. Create a GitHub release with binaries"
echo "3. Update Docker Hub images"
echo "4. Announce the release"