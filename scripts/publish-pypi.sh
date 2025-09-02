#!/bin/bash
set -e

# d-vecDB PyPI Publishing Script
# This script publishes d-vecDB to PyPI

echo "🚀 Publishing d-vecDB to PyPI..."
echo

# Check if build tools are available
if ! command -v twine &> /dev/null; then
    echo "❌ Error: twine not found. Installing build tools..."
    pip install --upgrade build twine
fi

# Publishing mode
mode=${1:-"test"}

case $mode in
    "check")
        echo "🔍 Checking distribution packages..."
        python -m twine check dist/*
        echo "✅ Package check completed"
        ;;
    "test")
        echo "🧪 Publishing to TestPyPI..."
        echo "📦 Building packages..."
        rm -rf dist/ build/ *.egg-info python-client/*.egg-info
        python -m build
        
        echo "🔍 Checking packages..."
        python -m twine check dist/*
        
        echo "📤 Uploading to TestPyPI..."
        python -m twine upload --repository testpypi dist/*
        
        echo "✅ Successfully published to TestPyPI!"
        echo
        echo "Test installation with:"
        echo "  pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ d-vecdb"
        echo
        echo "View package: https://test.pypi.org/project/d-vecdb/"
        ;;
    "prod"|"production")
        echo "🚀 Publishing to production PyPI..."
        
        # Safety check
        read -p "⚠️  Are you sure you want to publish to PRODUCTION PyPI? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "❌ Publication cancelled"
            exit 1
        fi
        
        echo "📦 Building packages..."
        rm -rf dist/ build/ *.egg-info python-client/*.egg-info
        python -m build
        
        echo "🔍 Checking packages..."
        python -m twine check dist/*
        
        echo "📤 Uploading to PyPI..."
        python -m twine upload dist/*
        
        echo "🎉 Successfully published to PyPI!"
        echo
        echo "Users can now install with:"
        echo "  pip install d-vecdb"
        echo
        echo "View package: https://pypi.org/project/d-vecdb/"
        ;;
    "build")
        echo "📦 Building distribution packages only..."
        rm -rf dist/ build/ *.egg-info python-client/*.egg-info
        python -m build
        
        echo "📋 Built packages:"
        ls -la dist/
        
        echo "✅ Build completed. Use 'check' to validate packages."
        ;;
    *)
        echo "Usage: $0 [check|build|test|prod]"
        echo
        echo "Commands:"
        echo "  build - Build distribution packages only"
        echo "  check - Check built packages for errors"
        echo "  test  - Publish to TestPyPI (default)"
        echo "  prod  - Publish to production PyPI"
        echo
        echo "Examples:"
        echo "  $0 build           # Build packages"
        echo "  $0 check           # Check package validity"  
        echo "  $0 test            # Publish to TestPyPI"
        echo "  $0 prod            # Publish to PyPI"
        exit 1
        ;;
esac

echo
echo "📚 Documentation: https://github.com/rdmurugan/d-vecDB/blob/master/PYPI_PUBLISHING.md"