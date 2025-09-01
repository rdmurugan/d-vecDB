# VectorDB-RS Benchmarking Guide

This document provides detailed information about benchmarking VectorDB-RS performance and comparing it with baseline implementations.

## Overview

**Important**: All performance claims should be validated with real benchmarks on your specific hardware and datasets. This document provides the tools and methodology to conduct such testing.

## Benchmark Categories

### 1. Distance Calculations
Tests the core mathematical operations for vector similarity.

```bash
# Run distance calculation benchmarks
cargo bench --package vectordb-common
```

**What it measures:**
- Cosine similarity computation time
- Euclidean distance computation time  
- Dot product computation time
- Manhattan distance computation time

### 2. HNSW Index Operations
Tests the vector indexing performance.

```bash
# Run HNSW benchmarks
cargo bench --package vectordb-index
```

**What it measures:**
- Vector insertion throughput
- Index construction time
- Search query latency
- Memory usage during operations

### 3. Storage Layer Performance
Tests persistence and recovery operations.

```bash
# Run storage benchmarks  
cargo bench --package vectordb-storage
```

**What it measures:**
- WAL write throughput
- Memory-mapped file I/O performance
- Recovery time from WAL
- Storage overhead

### 4. End-to-End Operations
Tests complete vector database operations.

```bash
# Run full system benchmarks
cargo bench --package vectordb-server
```

**What it measures:**
- Collection creation time
- Batch insertion throughput
- Query response time
- Memory usage under load

## Benchmarking Scripts

### Dataset Generation

Create test datasets of various sizes:

```bash
# Generate test datasets
./scripts/generate_datasets.sh
```

This creates:
- `dataset_1k.json` - 1,000 vectors, 128 dimensions
- `dataset_10k.json` - 10,000 vectors, 128 dimensions  
- `dataset_100k.json` - 100,000 vectors, 128 dimensions

### Performance Testing

```bash
# Run comprehensive performance tests
./scripts/run_benchmarks.sh

# Compare with baseline (linear search)
./scripts/compare_baseline.sh

# Generate performance report
./scripts/generate_report.sh
```

## Methodology

### Hardware Requirements

Document your testing environment:

```bash
# System information
cat /proc/cpuinfo | grep "model name" | head -1
cat /proc/meminfo | grep "MemTotal"
df -h /path/to/test/directory

# For macOS
sysctl -n machdep.cpu.brand_string
sysctl -n hw.memsize
df -h /path/to/test/directory
```

### Test Datasets

Generate reproducible test data:

1. **Synthetic Vectors**: Random vectors with known properties
2. **Real-world Data**: Embeddings from actual text/image data  
3. **Pathological Cases**: Vectors designed to test worst-case performance

### Metrics to Collect

#### Performance Metrics
- **Latency**: P50, P95, P99 query response times
- **Throughput**: Queries per second, insertions per second
- **Memory**: Peak and average memory usage
- **Storage**: Disk usage, I/O operations per second

#### Quality Metrics  
- **Recall**: Percentage of true neighbors found
- **Precision**: Accuracy of returned results
- **Index Quality**: Distribution of connections in HNSW graph

## Baseline Comparisons

### Linear Search Baseline

Compare against O(N) brute-force search:

```rust
// Baseline implementation for comparison
fn linear_search(vectors: &[Vec<f32>], query: &[f32], k: usize) -> Vec<SearchResult> {
    let mut results: Vec<_> = vectors
        .iter()
        .enumerate()
        .map(|(id, vec)| SearchResult {
            id: id as u64,
            distance: cosine_distance(query, vec),
        })
        .collect();
    
    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results.truncate(k);
    results
}
```

### Expected Performance Characteristics

**These are theoretical expectations that need validation:**

| Dataset Size | Linear Search | HNSW Search | Expected Speedup |
|-------------|---------------|-------------|------------------|
| 1,000 vectors | ~1ms | ~0.1ms | 10x |
| 10,000 vectors | ~10ms | ~0.2ms | 50x |  
| 100,000 vectors | ~100ms | ~0.5ms | 200x |
| 1,000,000 vectors | ~1000ms | ~1ms | 1000x |

**⚠️ These numbers are theoretical and need empirical validation.**

## Running Real Benchmarks

### Quick Performance Check

```bash
# Install and run basic benchmarks
cargo install --path .
./scripts/quick_bench.sh
```

### Comprehensive Testing

```bash
# Full benchmark suite (may take 30+ minutes)
./scripts/comprehensive_bench.sh

# Generate HTML report
cargo bench --package vectordb-common -- --output-format html
```

### Custom Dataset Testing

```bash
# Test with your own data
./scripts/custom_bench.sh /path/to/your/vectors.json
```

## Interpreting Results

### Performance Analysis

1. **Scaling Behavior**: How performance changes with dataset size
2. **Memory Efficiency**: Peak memory usage vs dataset size  
3. **Query Latency Distribution**: P50, P95, P99 response times
4. **Throughput Limits**: Maximum sustainable operation rate

### Quality Analysis

1. **Recall vs Speed Tradeoff**: Effect of index parameters on accuracy
2. **Index Overhead**: Storage space used by HNSW structure
3. **Build Time**: How long index construction takes

## Comparison Framework

### Against Other Solutions

To fairly compare with other vector databases:

1. **Same Hardware**: Use identical test machines
2. **Same Datasets**: Use standardized benchmark datasets  
3. **Same Queries**: Identical query patterns and parameters
4. **Same Metrics**: Consistent measurement methodology

### Reference Implementations

Compare against:
- **Faiss**: Facebook's similarity search library
- **Annoy**: Spotify's approximate nearest neighbor library
- **HNSWlib**: Reference HNSW implementation

```bash
# Install reference implementations
pip install faiss-cpu annoy
git clone https://github.com/nmslib/hnswlib

# Run comparison benchmarks
./scripts/compare_reference.sh
```

## Reporting Results

### Benchmark Report Template

```
## VectorDB-RS Performance Report

### Test Environment
- CPU: [Your CPU model]
- RAM: [Your RAM amount]  
- Storage: [SSD/HDD type]
- OS: [Operating system]
- Rust Version: [rustc version]

### Dataset Characteristics
- Size: [Number of vectors]
- Dimensions: [Vector dimension]
- Data Type: [Synthetic/Real-world]
- Distribution: [Random/Clustered]

### Results
| Operation | Latency (P95) | Throughput | Memory Usage |
|-----------|---------------|------------|--------------|
| Insert | X ms | Y ops/sec | Z MB |
| Search | X ms | Y ops/sec | Z MB |
| Batch Insert | X ms | Y ops/sec | Z MB |

### Comparison with Baseline
- Search Speedup: Xx faster than linear search
- Memory Overhead: X% additional memory for index
- Build Time: X seconds for Y vectors

### Observations
- [Key findings]
- [Performance bottlenecks identified]  
- [Recommendations for optimization]
```

## Limitations and Caveats

### Current Limitations

1. **Single-threaded**: Current benchmarks don't test concurrent operations
2. **Memory-only**: Tests don't include disk I/O simulation
3. **Synthetic Data**: Limited testing with real-world vector distributions
4. **Parameter Sensitivity**: Index parameters not optimized per dataset

### Benchmark Accuracy

- Results vary significantly based on data distribution
- Hardware differences affect absolute numbers
- HNSW parameter tuning affects performance substantially
- Cold vs warm cache performance differs

## Future Benchmarking Work

1. **Multi-threaded Testing**: Concurrent operation benchmarks  
2. **Distributed Testing**: Network latency and protocol overhead
3. **Real Dataset Library**: Standard benchmark datasets
4. **Automated Regression**: Performance monitoring over time
5. **Parameter Optimization**: Automated HNSW tuning

## Contributing Benchmarks

To contribute benchmark improvements:

1. Add new test scenarios in `benches/`
2. Document methodology in this file
3. Include reference results for your hardware
4. Test edge cases and failure modes

---

**Remember**: Benchmark results are only meaningful when:
1. Methodology is clearly documented
2. Test environment is specified  
3. Results are reproducible
4. Limitations are acknowledged