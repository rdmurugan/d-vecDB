#!/bin/bash
set -e

# d-vecDB Homebrew Setup Script
# Creates and manages a Homebrew tap for d-vecDB

TAP_REPO="rdmurugan/homebrew-d-vecdb"
TAP_DIR="homebrew-d-vecdb"
FORMULA_NAME="d-vecdb"
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

# Calculate SHA256 for release assets
calculate_shas() {
    print_status "Calculating SHA256 hashes for release assets..."
    
    local base_url="https://github.com/rdmurugan/d-vecDB/releases/download/v${VERSION}"
    local temp_dir=$(mktemp -d)
    
    declare -A assets=(
        ["macos-arm64"]="${base_url}/vectordb-server-macos-arm64"
        ["macos-x64"]="${base_url}/vectordb-server-macos-x64"
        ["linux-musl-x64"]="${base_url}/vectordb-server-linux-musl-x64"
    )
    
    declare -A shas
    
    for platform in "${!assets[@]}"; do
        local url="${assets[$platform]}"
        local filename="vectordb-server-${platform}"
        
        print_status "Downloading ${filename} to calculate SHA256..."
        
        if curl -L -o "$temp_dir/$filename" "$url" 2>/dev/null; then
            local sha=$(shasum -a 256 "$temp_dir/$filename" | cut -d' ' -f1)
            shas[$platform]="$sha"
            print_success "SHA256 for $platform: $sha"
        else
            print_warning "Failed to download $filename, using placeholder"
            shas[$platform]="PLACEHOLDER_${platform^^}_SHA256"
        fi
    done
    
    rm -rf "$temp_dir"
    
    # Return the SHAs
    echo "ARM64_SHA=${shas[macos-arm64]}"
    echo "X64_SHA=${shas[macos-x64]}"
    echo "LINUX_SHA=${shas[linux-musl-x64]}"
}

# Update formula with real SHA values
update_formula() {
    local arm64_sha=$1
    local x64_sha=$2
    local linux_sha=$3
    
    print_status "Updating formula with calculated SHA values..."
    
    local formula_file="$TAP_DIR/Formula/${FORMULA_NAME}-bin.rb"
    
    # Replace placeholders with actual SHA values
    sed -i.bak \
        -e "s/PLACEHOLDER_ARM64_SHA256/$arm64_sha/g" \
        -e "s/PLACEHOLDER_X64_SHA256/$x64_sha/g" \
        -e "s/PLACEHOLDER_LINUX_SHA256/$linux_sha/g" \
        "$formula_file"
    
    # Also update version if needed
    sed -i.bak2 "s/version \".*\"/version \"$VERSION\"/g" "$formula_file"
    sed -i.bak3 "s/v0\.1\.1/v$VERSION/g" "$formula_file"
    
    rm -f "$formula_file".bak*
    
    print_success "Formula updated with SHA values"
}

# Create the tap repository
create_tap_repo() {
    print_status "Creating Homebrew tap repository..."
    
    # Create directory structure
    mkdir -p "$TAP_DIR"/{Formula,Casks}
    
    # Copy formula files
    cp homebrew/${FORMULA_NAME}.rb "$TAP_DIR/Formula/"
    cp homebrew/${FORMULA_NAME}-bin.rb "$TAP_DIR/Formula/"
    
    # Create README
    cat > "$TAP_DIR/README.md" << EOF
# d-vecDB Homebrew Tap

This tap provides Homebrew formulas for [d-vecDB](https://github.com/rdmurugan/d-vecDB), a high-performance vector database written in Rust.

## Installation

### Install pre-built binary (recommended)
\`\`\`bash
brew tap rdmurugan/d-vecdb
brew install d-vecdb-bin
\`\`\`

### Install from source
\`\`\`bash
brew tap rdmurugan/d-vecdb  
brew install d-vecdb
\`\`\`

## Usage

### Start the server
\`\`\`bash
# Start manually
vectordb-server --config /usr/local/etc/d-vecdb/config.toml

# Or start as a service
brew services start d-vecdb-bin
\`\`\`

### Python Client
\`\`\`bash
pip install d-vecdb
\`\`\`

### Configuration

Configuration file: \`/usr/local/etc/d-vecdb/config.toml\`
Data directory: \`/usr/local/var/d-vecdb/\`
Log files: \`/usr/local/var/log/d-vecdb.log\`

## Available Formulas

- **d-vecdb-bin**: Pre-built binary (fast installation)
- **d-vecdb**: Build from source (requires Rust toolchain)

## Support

- [Documentation](https://github.com/rdmurugan/d-vecDB#readme)
- [Issues](https://github.com/rdmurugan/d-vecDB/issues)
- [Discussions](https://github.com/rdmurugan/d-vecDB/discussions)
EOF

    # Initialize git repo
    cd "$TAP_DIR"
    git init
    git add .
    git commit -m "Initial commit: Add d-vecDB formulas"
    
    print_success "Tap repository created in $TAP_DIR"
    cd ..
}

# Validate formulas
validate_formulas() {
    print_status "Validating Homebrew formulas..."
    
    if command -v brew >/dev/null 2>&1; then
        print_status "Running brew audit..."
        
        cd "$TAP_DIR"
        
        # Audit the formulas
        for formula in Formula/*.rb; do
            if brew audit --strict "$formula"; then
                print_success "✅ $(basename "$formula") passed audit"
            else
                print_warning "⚠️  $(basename "$formula") has audit warnings"
            fi
        done
        
        cd ..
    else
        print_warning "Homebrew not found, skipping formula validation"
    fi
}

# Create GitHub repository
create_github_repo() {
    print_status "Instructions to create GitHub repository:"
    
    cat << EOF

To complete the tap setup, create a GitHub repository:

1. Create a new repository at: https://github.com/new
   - Name: homebrew-d-vecdb
   - Description: "Homebrew tap for d-vecDB vector database"
   - Public repository
   
2. Push the tap:
   cd $TAP_DIR
   git remote add origin git@github.com:$TAP_REPO.git
   git branch -M main
   git push -u origin main

3. Users can then install with:
   brew tap $TAP_REPO
   brew install d-vecdb-bin

EOF
}

# Test installation locally
test_installation() {
    print_status "Testing local installation..."
    
    if command -v brew >/dev/null 2>&1; then
        print_status "Testing formula syntax..."
        
        cd "$TAP_DIR"
        
        # Test formula parsing
        for formula in Formula/*.rb; do
            if brew ruby -e "require_relative '$PWD/$formula'"; then
                print_success "✅ $(basename "$formula") syntax OK"
            else
                print_error "❌ $(basename "$formula") syntax error"
            fi
        done
        
        cd ..
        
        print_status "To test installation locally:"
        print_status "  brew install --build-from-source $PWD/$TAP_DIR/Formula/${FORMULA_NAME}.rb"
        print_status "  brew install $PWD/$TAP_DIR/Formula/${FORMULA_NAME}-bin.rb"
    fi
}

# Main function
main() {
    echo
    echo "🍺 d-vecDB Homebrew Setup"
    echo "========================="
    echo
    
    # Check if release exists
    if ! curl -s "https://api.github.com/repos/rdmurugan/d-vecDB/releases/tags/v$VERSION" | grep -q "tag_name"; then
        print_error "Release v$VERSION not found on GitHub"
        print_error "Please create a release first with the GitHub Actions workflow"
        exit 1
    fi
    
    # Calculate SHA values
    eval $(calculate_shas)
    
    # Create tap repository
    create_tap_repo
    
    # Update formula with SHA values
    update_formula "$ARM64_SHA" "$X64_SHA" "$LINUX_SHA"
    
    # Validate formulas
    validate_formulas
    
    # Test installation
    test_installation
    
    # Show next steps
    create_github_repo
    
    print_success "Homebrew tap setup complete!"
    print_status "Next: Create the GitHub repository and push the tap"
}

# Run main function
main "$@"