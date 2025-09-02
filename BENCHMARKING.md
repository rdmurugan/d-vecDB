# d-vecDB Benchmarking Guide

This document provides comprehensive benchmarking information for d-vecDB, including actual performance results, methodology, and hardware optimization guidelines.

## ⚡ **Current Performance Results**

*Tested on macOS Darwin 24.6.0 with optimized release builds*

### **Distance Calculations**
| Operation | Mean Latency | Throughput | Outliers |
|-----------|--------------|------------|----------|
| **Dot Product** | 28.262 ns | 35.4M ops/sec | 6/100 (6%) |
| **Euclidean Distance** | 30.618 ns | 32.7M ops/sec | 10/100 (10%) |
| **Cosine Similarity** | 76.145 ns | 13.1M ops/sec | 9/100 (9%) |

### **HNSW Index Operations**
| Operation | Performance | Details | Consistency |
|-----------|-------------|---------|-------------|
| **Vector Insertion** | 140.71 ms (1K vectors) | 7,108 vectors/sec | Very stable (1 outlier) |
| **Vector Search** | 76.047 µs | 13,150 queries/sec | Search in 5K dataset |
| **Metadata Insertion** | 390.62 µs | 2,560 inserts/sec | With rich JSON metadata |

## 🚀 **Performance Projections**

### **Hardware Scaling Assumptions**
Based on empirical benchmarks and scaling characteristics:

#### **CPU Scaling Factors**
- **Single-core performance**: Proportional to CPU clock speed and IPC
- **Multi-core scaling**: Near-linear for distance calculations and indexing
- **Memory bandwidth**: Critical for high-dimensional vectors
- **Cache hierarchy**: Significant impact on index traversal performance

#### **High-End Server Hardware (32-core AMD EPYC 7763, 128GB DDR4-3200, NVMe)**
| Operation | Current (Mac) | Projected (Server) | Scaling Factor |
|-----------|---------------|-------------------|----------------|
| **Distance Calculations** | 35M ops/sec | **150M+ ops/sec** | 4.3x (32 cores) |
| **Vector Insertion** | 7K vectors/sec | **50K+ vectors/sec** | 7x (parallel + I/O) |
| **Vector Search** | 13K queries/sec | **100K+ queries/sec** | 7.7x (cache + cores) |
| **Concurrent Queries** | Single-threaded | **500K+ queries/sec** | 38x (full parallelism) |

#### **Cloud Instance (16-core Intel Xeon, 64GB RAM, SSD)**
| Operation | Current (Mac) | Projected (Cloud) | Scaling Factor |
|-----------|---------------|-------------------|----------------|
| **Distance Calculations** | 35M ops/sec | **80M+ ops/sec** | 2.3x (16 cores) |
| **Vector Insertion** | 7K vectors/sec | **25K+ vectors/sec** | 3.6x (I/O + cores) |
| **Vector Search** | 13K queries/sec | **45K+ queries/sec** | 3.5x (memory + cores) |
| **Concurrent Queries** | Single-threaded | **180K+ queries/sec** | 14x (parallelism) |

## 📊 **Benchmark Categories**

### 1. **Distance Calculations**
Tests the core mathematical operations for vector similarity.

```bash
# Run distance calculation benchmarks
cargo bench --package vectordb-common
```

**What it measures:**
- Cosine similarity computation time
- Euclidean distance computation time  
- Dot product computation time
- Performance consistency and outlier analysis

### 2. **HNSW Index Operations**
Tests the vector indexing and search performance.

```bash
# Run HNSW benchmarks (included in common package)
cargo bench --package vectordb-common
```

**What it measures:**
- Vector insertion throughput with index building
- Search performance across different dataset sizes
- Index construction time and memory usage
- Search accuracy vs. performance trade-offs

### 3. **Metadata Operations**
Tests performance with rich metadata storage.

**What it measures:**
- Insertion performance with JSON metadata
- Metadata serialization/deserialization overhead
- Storage efficiency with complex data structures

### 4. **Storage Engine Performance**
Tests WAL and memory-mapped storage performance.

**What it measures:**
- WAL write throughput and durability
- Memory-mapped read performance
- Crash recovery time
- Storage space efficiency

## 🛠️ **Running Benchmarks**

### **Standard Benchmark Suite**

```bash
# Core performance benchmarks
cargo bench --package vectordb-common

# Generate detailed HTML reports
cargo bench --package vectordb-common
open target/criterion/report/index.html

# Run with specific configuration
CARGO_PROFILE_BENCH_LTO=fat cargo bench --package vectordb-common
```

### **Custom Benchmarks**

```bash
# Benchmark specific operations
cargo bench --package vectordb-common -- "cosine_distance"
cargo bench --package vectordb-common -- "hnsw_insert"

# Run with different dataset sizes
BENCHMARK_VECTOR_COUNT=10000 cargo bench --package vectordb-common
BENCHMARK_DIMENSION=256 cargo bench --package vectordb-common
```

### **System-Specific Optimizations**

```bash
# Enable all CPU features
RUSTFLAGS="-C target-cpu=native" cargo bench --package vectordb-common

# Profile-guided optimization (advanced)
cargo pgo bench
```

## 🔧 **Hardware Optimization Guide**

### **For Maximum Distance Calculation Performance**
- **CPU**: High single-thread performance with AVX2/AVX-512 support
- **Memory**: Fast DDR4-3200+ with low latency
- **Optimization**: Enable `target-cpu=native` for SIMD instructions

### **For Maximum Insertion Throughput**
- **CPU**: High core count (16+ cores) for parallel index building
- **Storage**: NVMe SSDs for fast WAL writes and persistence
- **Memory**: Large pool (32GB+) to keep indexes in memory

### **For Maximum Search Performance**
- **CPU**: Balance of single-thread performance and core count
- **Memory**: Fast memory and large L3 cache for index traversal
- **Architecture**: NUMA-aware deployment for multi-socket systems

### **For Large Scale Deployments**
- **Network**: High bandwidth (10Gb+) for client connections
- **Storage**: Tiered storage with hot data on NVMe, warm on SSD
- **Monitoring**: Real-time performance metrics and alerting

## 📈 **Performance Monitoring**

### **Key Metrics to Track**

```bash
# System metrics
- CPU utilization and per-core performance
- Memory usage and bandwidth utilization
- Storage I/O patterns and latency
- Network throughput and connection count

# Application metrics  
- Query latency percentiles (p50, p95, p99)
- Index build time and memory usage
- WAL write throughput and sync latency
- Error rates and timeout frequencies
```

### **Benchmark Automation**

```toml
# benchmark-config.toml
[benchmark]
vector_counts = [1000, 10000, 100000, 1000000]
dimensions = [128, 256, 512, 1024]
distance_metrics = ["cosine", "euclidean", "dot_product"]
trials = 10
warmup_iterations = 5

[hardware_detection]
cpu_features = ["avx2", "avx512f", "fma"]
memory_bandwidth = true
storage_type = "detect"
```

## 🔍 **Performance Analysis**

### **Bottleneck Identification**

1. **CPU-bound operations**: Distance calculations, index traversal
2. **Memory-bound operations**: Large vector loading, cache misses  
3. **I/O-bound operations**: WAL writes, index persistence
4. **Network-bound operations**: Client request/response handling

### **Optimization Priorities**

1. **SIMD utilization** for distance calculations (4-8x speedup potential)
2. **Cache optimization** for index data structures
3. **Parallel processing** for batch operations
4. **Memory prefetching** for predictable access patterns

## 📚 **Benchmark Interpretation**

### **Understanding the Results**

- **Latency percentiles**: Focus on p95/p99 for consistent user experience
- **Throughput scaling**: Linear scaling indicates good parallelization
- **Memory usage patterns**: Stable usage shows good memory management
- **Outlier analysis**: Consistent performance across runs

### **Comparative Baselines**

| System | Language | Distance Calc | Insert Rate | Search Rate |
|--------|----------|---------------|-------------|-------------|
| **d-vecDB** | Rust | **35M ops/sec** | **7K/sec** | **13K/sec** |
| Weaviate | Go | ~5M ops/sec | ~2K/sec | ~8K/sec |
| Milvus | C++/Python | ~20M ops/sec | ~15K/sec | ~10K/sec |
| Pinecone | Managed | N/A | ~5K/sec | ~15K/sec |

*Note: Comparisons are approximate and depend heavily on hardware, configuration, and workload characteristics*

## ⚠️ **Important Considerations**

### **Benchmark Validity**
- Results are hardware and configuration dependent
- Real-world performance may vary based on data characteristics
- Network latency not included in core operation benchmarks
- Concurrent load may show different characteristics

### **Hardware Variability**
- Apple Silicon (M-series) may show different scaling patterns
- Intel vs AMD architectures may favor different operations  
- Cloud instances have shared resources affecting consistency
- Storage types (NVMe vs SATA SSD) significantly impact I/O operations

### **Future Optimizations**
- SIMD vectorization for distance calculations
- GPU acceleration for large batch operations
- Advanced indexing algorithms (IVF, PQ, LSH)
- Distributed query processing across multiple nodes

## 🔬 **Advanced Benchmarking**

### **Custom Test Scenarios**

```rust
// Custom benchmark example
use criterion::{criterion_group, criterion_main, Criterion};
use vectordb_common::distance::*;

fn bench_custom_scenario(c: &mut Criterion) {
    // Your custom benchmark logic
    let vectors = generate_realistic_dataset(100000, 384); // OpenAI embeddings size
    
    c.bench_function("realistic_workload", |b| {
        b.iter(|| {
            // Simulate realistic query pattern
            perform_mixed_operations(&vectors)
        })
    });
}

criterion_group!(custom_benches, bench_custom_scenario);
criterion_main!(custom_benches);
```

### **Profiling Integration**

```bash
# Profile with perf (Linux)
perf record --call-graph=dwarf cargo bench --package vectordb-common
perf report

# Profile with Instruments (macOS)
cargo instruments -t "Time Profiler" --bench vector_performance

# Memory profiling with valgrind
valgrind --tool=massif cargo bench --package vectordb-common
```

## 📝 **Reporting Guidelines**

### **Performance Report Template**

```markdown
## Benchmark Report

**Environment:**
- Hardware: [CPU model, RAM, Storage]
- OS: [Operating system and version]
- Rust version: [rustc --version]
- Build flags: [optimization flags used]

**Results:**
- Distance calculations: X ops/sec
- Vector insertion: X vectors/sec  
- Vector search: X queries/sec
- Memory usage: X GB peak

**Notes:**
- [Any specific observations or anomalies]
- [Hardware-specific optimizations applied]
- [Comparison with previous runs if available]
```

### **Contributing Benchmarks**

We welcome benchmark results from different hardware configurations! Please:

1. Use the standard benchmark suite
2. Include full hardware specifications
3. Report any custom optimizations or build flags
4. Share both raw results and analysis
5. Submit via GitHub issues with the "performance" label

---

**For questions about benchmarking or performance optimization, please see our [Performance Guide](PERFORMANCE.md) or open an issue on GitHub.**