# VectorDB-RS

A vector database implementation in Rust - **Currently in Alpha Development**.

⚠️ **This project is experimental and not ready for production use.**

## Current Features (Alpha Stage)

### Working Components ✅
- **Distance Calculations**: Cosine, Euclidean, Dot Product, Manhattan (tested)
- **HNSW Indexing**: Basic approximate nearest neighbor search implementation
- **Storage Framework**: Memory-mapped storage with WAL (Write-Ahead Log) foundation
- **Type System**: Comprehensive Rust type definitions and error handling
- **API Design**: Protocol buffer definitions for gRPC and REST APIs

### In Development 🔄
- **Client-Server Communication**: Protocol mapping and integration 
- **Performance Optimization**: Memory management and query optimization
- **Error Handling**: Comprehensive edge case handling
- **Integration Testing**: End-to-end workflow validation

### Not Yet Implemented ❌
- **Authentication/Security**: No access control
- **Clustering**: Single-node operation only
- **Advanced Queries**: No filtering or aggregations
- **Production Tooling**: Limited operational features
- **Client SDKs**: Only basic framework exists

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Clients     │    │   REST/gRPC     │    │   Monitoring    │
│   (SDK/CLI)     │────│     Server      │────│   (Metrics)     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                   ┌─────────────────┐
                   │   Vector Store  │
                   │    (Engine)     │
                   └─────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│HNSW Index   │       │  Storage    │       │   Common    │
│(ANN Search) │       │(WAL + Mmap) │       │ (Utilities) │
└─────────────┘       └─────────────┘       └─────────────┘
```

## Development Setup

⚠️ **Note**: The build currently has compilation issues that need to be resolved.

```bash
# Clone and build (may have compilation errors)
git clone <repository>
cd d-vecDB
cargo build --release

# Run unit tests for working components
cargo test --lib --package vectordb-common  # Distance calculations
cargo test --lib --package vectordb-index   # HNSW indexing

# Run benchmarks (when working)
cargo bench --package vectordb-common
```

### Current Build Status
- ✅ **vectordb-common**: Compiles and tests pass
- ✅ **vectordb-index**: Compiles with basic functionality
- ❌ **vectordb-client**: Compilation errors (protocol mapping issues)  
- ❌ **vectordb-server**: Compilation errors (missing dependencies)
- ❌ **vectordb-vectorstore**: Type annotation issues

## Performance Status

⚠️ **Important**: Current performance claims are **theoretical and unvalidated**.

### Theoretical Performance (Unverified)
Based on algorithmic complexity analysis:
- **Search Complexity**: O(log N) HNSW vs O(N) linear scan
- **Expected Speedup**: 10-1000x over brute force (dataset dependent)
- **Memory Usage**: ~4 bytes per dimension + index overhead

### Actual Benchmarking Status
- **❌ Not Measured**: No real-world performance testing completed
- **❌ Not Validated**: Claims require empirical verification  
- **❌ Hardware Dependent**: Performance varies significantly by system

### Benchmarking Infrastructure
- ✅ **Framework**: Criterion benchmarking setup available
- ✅ **Test Data**: Scripts to generate synthetic datasets
- ✅ **Baseline Comparison**: Framework for comparing with linear search
- ❌ **Real Data Testing**: No validation with actual embeddings

To run available benchmarks:
```bash
cargo bench --package vectordb-common    # Distance calculations only
# Other benchmarks pending compilation fixes
```

## License

MIT License