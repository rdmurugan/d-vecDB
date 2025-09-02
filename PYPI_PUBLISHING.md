# 📦 PyPI Publishing Guide for d-vecDB

This guide shows how to publish d-vecDB to PyPI so users can install it with `pip install d-vecdb`.

## 🚀 Quick Publishing Steps

### 1. Prerequisites

```bash
# Install required tools (already done)
pip install --upgrade build twine

# Create PyPI accounts (if you haven't):
# - https://test.pypi.org/account/register/ (TestPyPI)
# - https://pypi.org/account/register/ (Production PyPI)
```

### 2. Build Distribution Packages

```bash
# Clean previous builds
rm -rf dist/ build/ *.egg-info python-client/*.egg-info

# Build packages (already done)
python -m build
```

This creates:
- `dist/d_vecdb-0.1.0.tar.gz` (source distribution)  
- `dist/d_vecdb-0.1.0-py3-none-any.whl` (wheel)

### 3. Upload to TestPyPI (Recommended First)

```bash
# Upload to TestPyPI for testing
python -m twine upload --repository testpypi dist/*

# Test installation from TestPyPI
pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ d-vecdb
```

### 4. Upload to Production PyPI

```bash
# Upload to PyPI (production)
python -m twine upload dist/*
```

### 5. Test Installation

```bash
# Install from PyPI
pip install d-vecdb

# Test import
python -c "import vectordb_client; print('✅ Successfully installed from PyPI!')"
```

## 🔐 Authentication Setup

### Option 1: API Tokens (Recommended)

1. **Create API tokens:**
   - TestPyPI: https://test.pypi.org/manage/account/token/
   - PyPI: https://pypi.org/manage/account/token/

2. **Configure credentials:**
```bash
# Create/edit ~/.pypirc
[distutils]
index-servers = 
  pypi
  testpypi

[pypi]
username = __token__
password = pypi-your-api-token-here

[testpypi]
repository = https://test.pypi.org/legacy/
username = __token__
password = pypi-your-testpypi-token-here
```

### Option 2: Username/Password

```bash
# Twine will prompt for credentials
python -m twine upload dist/*
```

## 📋 Complete Publishing Commands

```bash
#!/bin/bash
# Complete PyPI publishing script

echo "🚀 Publishing d-vecDB to PyPI..."

# 1. Clean and build
echo "📦 Building distribution packages..."
rm -rf dist/ build/ *.egg-info python-client/*.egg-info
python -m build

# 2. Check packages
echo "🔍 Checking packages..."
python -m twine check dist/*

# 3. Upload to TestPyPI first
echo "🧪 Uploading to TestPyPI..."
python -m twine upload --repository testpypi dist/*

echo "✅ Uploaded to TestPyPI!"
echo "Test with: pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple/ d-vecdb"

# 4. Upload to PyPI (uncomment when ready)
# echo "🚀 Uploading to PyPI..."
# python -m twine upload dist/*
# echo "✅ Published to PyPI! Users can now: pip install d-vecdb"
```

## 🎯 After Publishing

### Update Version for Next Release

1. **Update version in `pyproject.toml`:**
```toml
[project]
version = "0.1.1"  # or next version
```

2. **Update version in `setup.py`:**
```python
version="0.1.1",
```

3. **Tag the release:**
```bash
git tag v0.1.0
git push origin v0.1.0
```

## 🔄 Automated Publishing with GitHub Actions

Create `.github/workflows/publish-pypi.yml`:

```yaml
name: Publish to PyPI

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.8'
        
    - name: Install build tools
      run: |
        python -m pip install --upgrade pip
        pip install build twine
        
    - name: Build packages
      run: python -m build
      
    - name: Publish to PyPI
      env:
        TWINE_USERNAME: __token__
        TWINE_PASSWORD: \${{ secrets.PYPI_API_TOKEN }}
      run: twine upload dist/*
```

## ⚠️ Important Notes

1. **Package Name**: Using `d-vecdb` (with hyphen) - PyPI converts to `d_vecdb` internally
2. **Dependencies**: All Python dependencies are properly specified
3. **Platform**: Built as universal wheel (`py3-none-any`)
4. **Entry Points**: CLI command `d-vecdb` will be available after install

## 🎉 Result

After publishing, users can install with:

```bash
# Install the package
pip install d-vecdb

# Use the Python client
from vectordb_client import VectorDBClient
client = VectorDBClient()

# Use the CLI
d-vecdb --help
```

## 📚 Resources

- [PyPI Package Publishing](https://packaging.python.org/en/latest/tutorials/packaging-projects/)
- [TestPyPI](https://test.pypi.org/) - Test your packages safely
- [Twine Documentation](https://twine.readthedocs.io/)
- [Python Packaging User Guide](https://packaging.python.org/)