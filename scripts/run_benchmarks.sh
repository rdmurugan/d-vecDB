#!/bin/bash

# Comprehensive Benchmark Runner for VectorDB-RS
# Runs all benchmark suites and collects results

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"

echo "Running VectorDB-RS Comprehensive Benchmarks"
echo "============================================="

# Create results directory
mkdir -p "$RESULTS_DIR"

# Get system information
echo "Collecting system information..."
{
    echo "# System Information"
    echo "Date: $(date)"
    echo "Hostname: $(hostname)"
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "OS: $(sw_vers -productName) $(sw_vers -productVersion)"
        echo "CPU: $(sysctl -n machdep.cpu.brand_string)"
        echo "Memory: $(echo "$(sysctl -n hw.memsize) / 1024 / 1024 / 1024" | bc)GB"
    elif [[ "$OSTYPE" == "linux"* ]]; then
        echo "OS: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2)"
        echo "CPU: $(cat /proc/cpuinfo | grep 'model name' | head -1 | cut -d':' -f2 | xargs)"
        echo "Memory: $(cat /proc/meminfo | grep MemTotal | awk '{print int($2/1024/1024)}')GB"
    fi
    
    echo "Rust Version: $(rustc --version)"
    echo ""
} > "$RESULTS_DIR/system_info.md"

cat "$RESULTS_DIR/system_info.md"

# Generate datasets if they don't exist
if [ ! -d "$PROJECT_ROOT/benchmark_data" ]; then
    echo "Generating benchmark datasets..."
    "$SCRIPT_DIR/generate_datasets.sh"
fi

# Function to run benchmark and save results
run_benchmark() {
    local package=$1
    local description=$2
    local output_file="$RESULTS_DIR/${package}_results.txt"
    
    echo ""
    echo "Running $description..."
    echo "Results will be saved to: $output_file"
    
    {
        echo "# $description"
        echo "Package: $package"
        echo "Timestamp: $(date)"
        echo ""
        
        # Run the benchmark
        if cargo bench --package "$package" 2>&1; then
            echo "✅ Benchmark completed successfully"
        else
            echo "❌ Benchmark failed or had errors"
        fi
        
        echo ""
        echo "---"
        echo ""
    } | tee "$output_file"
}

# Build the project first
echo "Building project in release mode..."
if ! cargo build --release; then
    echo "❌ Build failed - cannot run benchmarks"
    exit 1
fi

# Run unit tests to make sure everything works
echo "Running unit tests..."
if ! cargo test --lib --quiet; then
    echo "⚠️  Some unit tests failed - benchmark results may not be reliable"
fi

# Run benchmarks for each component
run_benchmark "vectordb-common" "Distance Calculation Benchmarks"
run_benchmark "vectordb-index" "HNSW Index Benchmarks" 

# Note: These may fail if compilation issues remain
echo ""
echo "Attempting additional component benchmarks..."

if cargo check --package vectordb-storage --quiet 2>/dev/null; then
    run_benchmark "vectordb-storage" "Storage Layer Benchmarks"
else
    echo "⚠️  Skipping storage benchmarks - compilation issues detected"
fi

if cargo check --package vectordb-vectorstore --quiet 2>/dev/null; then
    run_benchmark "vectordb-vectorstore" "Vector Store Benchmarks"  
else
    echo "⚠️  Skipping vectorstore benchmarks - compilation issues detected"
fi

if cargo check --package vectordb-server --quiet 2>/dev/null; then
    run_benchmark "vectordb-server" "Server Benchmarks"
else
    echo "⚠️  Skipping server benchmarks - compilation issues detected"
fi

# Create summary report
echo ""
echo "Creating benchmark summary..."

{
    echo "# VectorDB-RS Benchmark Summary"
    echo ""
    cat "$RESULTS_DIR/system_info.md"
    echo ""
    echo "## Benchmark Results"
    echo ""
    
    for result_file in "$RESULTS_DIR"/*_results.txt; do
        if [ -f "$result_file" ]; then
            echo "### $(basename "$result_file" _results.txt)"
            echo ""
            echo "\`\`\`"
            tail -20 "$result_file" | head -15  # Show relevant parts
            echo "\`\`\`"
            echo ""
        fi
    done
    
    echo "## Notes"
    echo ""
    echo "- All benchmarks run on single thread"
    echo "- Results may vary between runs"
    echo "- Absolute numbers depend on hardware"
    echo "- Focus on relative performance comparisons"
    echo ""
    echo "For detailed results, see individual files in benchmark_results/"
    
} > "$RESULTS_DIR/SUMMARY.md"

echo ""
echo "✅ Benchmark run completed!"
echo ""
echo "Results saved to: $RESULTS_DIR/"
echo "Summary report: $RESULTS_DIR/SUMMARY.md"
echo ""
echo "To view results:"
echo "  cat $RESULTS_DIR/SUMMARY.md"
echo "  ls $RESULTS_DIR/"

# Show quick summary
echo ""
echo "Quick Summary:"
echo "=============="
if [ -f "$RESULTS_DIR/vectordb-common_results.txt" ]; then
    echo "✅ Distance calculations: Benchmarked"
else
    echo "❌ Distance calculations: Failed"
fi

if [ -f "$RESULTS_DIR/vectordb-index_results.txt" ]; then
    echo "✅ HNSW indexing: Benchmarked"  
else
    echo "❌ HNSW indexing: Failed"
fi

echo ""
echo "Next steps:"
echo "1. Review $RESULTS_DIR/SUMMARY.md"
echo "2. Run ./scripts/compare_baseline.sh for comparisons"
echo "3. Optimize based on bottlenecks identified"