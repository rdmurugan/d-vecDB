#!/bin/bash
set -e

# d-vecDB Installation Script
# This script installs d-vecDB and its dependencies

echo "🚀 Installing d-vecDB Vector Database..."
echo

# Check Python version
python_version=$(python3 -c "import sys; print('.'.join(map(str, sys.version_info[:2])))")
required_version="3.8"

if ! python3 -c "import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)"; then
    echo "❌ Error: Python ${required_version}+ required. Found Python ${python_version}"
    exit 1
fi

echo "✅ Python ${python_version} detected"

# Install options
install_type=${1:-"user"}

case $install_type in
    "user")
        echo "📦 Installing d-vecDB for current user..."
        pip3 install --user .
        ;;
    "system")
        echo "📦 Installing d-vecDB system-wide (requires sudo)..."
        sudo pip3 install .
        ;;
    "dev")
        echo "🔧 Installing d-vecDB in development mode..."
        pip3 install -e ".[dev,docs,examples]"
        ;;
    "venv")
        echo "🔧 Creating virtual environment and installing d-vecDB..."
        python3 -m venv venv
        source venv/bin/activate
        pip install --upgrade pip
        pip install -e ".[dev,docs,examples]"
        echo "✅ Virtual environment created. Activate with: source venv/bin/activate"
        ;;
    *)
        echo "Usage: $0 [user|system|dev|venv]"
        echo "  user   - Install for current user (default)"
        echo "  system - Install system-wide"  
        echo "  dev    - Install in development mode with extras"
        echo "  venv   - Create virtual environment and install"
        exit 1
        ;;
esac

echo
echo "🎉 d-vecDB installation completed!"
echo
echo "Quick start:"
echo "  from vectordb_client import VectorDBClient"
echo "  client = VectorDBClient()"
echo
echo "📚 Documentation: https://github.com/rdmurugan/d-vecDB#readme"
echo "🐛 Issues: https://github.com/rdmurugan/d-vecDB/issues"