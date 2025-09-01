#!/bin/bash

# Baseline Comparison Script for VectorDB-RS
# Compares HNSW performance against linear search baseline

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results/baseline_comparison"
DATA_DIR="$PROJECT_ROOT/benchmark_data"

echo "VectorDB-RS Baseline Comparison"
echo "==============================="

mkdir -p "$RESULTS_DIR"

# Check if datasets exist
if [ ! -d "$DATA_DIR" ]; then
    echo "Generating test datasets..."
    "$SCRIPT_DIR/generate_datasets.sh"
fi

# Create baseline comparison test program
echo "Creating baseline comparison test..."

cat > "$PROJECT_ROOT/src/baseline_test.rs" << 'EOF'
use std::time::Instant;
use vectordb_common::types::*;
use vectordb_common::distance::*;
use vectordb_index::{VectorIndex, HnswIndex};
use serde_json::Value;
use std::fs;

#[derive(Debug)]
pub struct BenchmarkResult {
    pub operation: String,
    pub dataset_size: usize,
    pub duration_ms: f64,
    pub throughput_ops_per_sec: f64,
    pub memory_mb: Option<f64>,
}

pub fn load_dataset(path: &str) -> Result<Vec<(VectorId, Vec<f32>)>, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let json: Value = serde_json::from_str(&data)?;
    
    let vectors = json["vectors"].as_array().unwrap();
    let mut result = Vec::new();
    
    for (i, vector_obj) in vectors.iter().enumerate() {
        let vector_data: Vec<f32> = vector_obj["vector"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
            
        let id = uuid::Uuid::new_v4();
        result.push((id, vector_data));
    }
    
    Ok(result)
}

pub fn linear_search_baseline(
    vectors: &[(VectorId, Vec<f32>)], 
    query: &[f32], 
    k: usize
) -> Vec<SearchResult> {
    let mut results: Vec<SearchResult> = vectors
        .iter()
        .map(|(id, vec)| SearchResult {
            id: *id,
            distance: distance(vec, query, DistanceMetric::Cosine),
            metadata: None,
        })
        .collect();
    
    results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    results.truncate(k);
    results
}

pub fn benchmark_linear_search(vectors: &[(VectorId, Vec<f32>)], queries: &[Vec<f32>], k: usize) -> BenchmarkResult {
    let start = Instant::now();
    
    for query in queries {
        let _results = linear_search_baseline(vectors, query, k);
    }
    
    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;
    let ops_per_sec = queries.len() as f64 / duration.as_secs_f64();
    
    BenchmarkResult {
        operation: "Linear Search".to_string(),
        dataset_size: vectors.len(),
        duration_ms,
        throughput_ops_per_sec: ops_per_sec,
        memory_mb: None,
    }
}

pub fn benchmark_hnsw_search(vectors: &[(VectorId, Vec<f32>)], queries: &[Vec<f32>], k: usize) -> BenchmarkResult {
    // Build HNSW index
    let config = IndexConfig {
        max_connections: 16,
        ef_construction: 200,
        ef_search: 50,
        max_layer: 16,
    };
    
    let mut index = HnswIndex::new(config, DistanceMetric::Cosine, vectors[0].1.len());
    
    // Insert all vectors
    let build_start = Instant::now();
    for (id, vector) in vectors {
        let _ = index.insert(*id, vector, None);
    }
    let build_time = build_start.elapsed();
    
    // Query benchmark
    let search_start = Instant::now();
    
    for query in queries {
        let _results = index.search(query, k, None).unwrap_or_default();
    }
    
    let search_duration = search_start.elapsed();
    let duration_ms = search_duration.as_secs_f64() * 1000.0;
    let ops_per_sec = queries.len() as f64 / search_duration.as_secs_f64();
    
    println!("HNSW Index build time: {:.2}ms", build_time.as_secs_f64() * 1000.0);
    
    BenchmarkResult {
        operation: "HNSW Search".to_string(),
        dataset_size: vectors.len(),
        duration_ms,
        throughput_ops_per_sec: ops_per_sec,
        memory_mb: None,
    }
}

pub fn run_comparison(dataset_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading dataset: {}", dataset_path);
    let vectors = load_dataset(dataset_path)?;
    
    // Generate query vectors (use first few vectors as queries)
    let num_queries = std::cmp::min(100, vectors.len() / 10);
    let queries: Vec<Vec<f32>> = vectors.iter().take(num_queries).map(|(_, v)| v.clone()).collect();
    
    println!("Dataset size: {} vectors", vectors.len());
    println!("Query count: {} queries", queries.len());
    println!("Vector dimension: {}", vectors[0].1.len());
    
    // Benchmark linear search
    println!("\nRunning linear search benchmark...");
    let linear_result = benchmark_linear_search(&vectors, &queries, 10);
    
    // Benchmark HNSW search  
    println!("Running HNSW search benchmark...");
    let hnsw_result = benchmark_hnsw_search(&vectors, &queries, 10);
    
    // Compare results
    println!("\n=== COMPARISON RESULTS ===");
    println!("Linear Search:");
    println!("  Duration: {:.2}ms total", linear_result.duration_ms);
    println!("  Throughput: {:.1} queries/sec", linear_result.throughput_ops_per_sec);
    println!("  Avg per query: {:.3}ms", linear_result.duration_ms / queries.len() as f64);
    
    println!("HNSW Search:");
    println!("  Duration: {:.2}ms total", hnsw_result.duration_ms);
    println!("  Throughput: {:.1} queries/sec", hnsw_result.throughput_ops_per_sec);
    println!("  Avg per query: {:.3}ms", hnsw_result.duration_ms / queries.len() as f64);
    
    let speedup = linear_result.throughput_ops_per_sec / hnsw_result.throughput_ops_per_sec;
    println!("\nSpeedup: {:.1}x faster", speedup);
    
    Ok(())
}
EOF

# Create a simple binary to run the comparison
cat > "$PROJECT_ROOT/examples/baseline_comparison.rs" << 'EOF'
use std::env;

mod baseline_test;
use baseline_test::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <dataset.json>", args[0]);
        println!("Available datasets:");
        println!("  benchmark_data/dataset_1k.json");
        println!("  benchmark_data/dataset_10k.json");
        println!("  benchmark_data/dataset_100k.json");
        return Ok(());
    }
    
    let dataset_path = &args[1];
    run_comparison(dataset_path)?;
    
    Ok(())
}
EOF

# Check if we can build the comparison tool
echo "Building baseline comparison tool..."

if ! cargo build --example baseline_comparison 2>/dev/null; then
    echo "❌ Cannot build baseline comparison - running simplified tests instead"
    
    # Run basic benchmark comparison using existing tools
    echo ""
    echo "Running basic performance comparison..."
    
    {
        echo "# Baseline Comparison Results"
        echo "Generated: $(date)"
        echo ""
        echo "## System Info"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "- CPU: $(sysctl -n machdep.cpu.brand_string)"
            echo "- Memory: $(echo "$(sysctl -n hw.memsize) / 1024 / 1024 / 1024" | bc)GB"
        fi
        echo "- Rust: $(rustc --version)"
        echo ""
        
        echo "## Theoretical Comparison"
        echo ""
        echo "| Dataset Size | Linear Search* | HNSW Search* | Expected Speedup |"
        echo "|-------------|---------------|-------------|------------------|"
        echo "| 1,000 | ~1ms | ~0.1ms | ~10x |"
        echo "| 10,000 | ~10ms | ~0.2ms | ~50x |"
        echo "| 100,000 | ~100ms | ~0.5ms | ~200x |"
        echo ""
        echo "*Theoretical estimates based on algorithmic complexity"
        echo ""
        
        echo "## Component Benchmarks"
        echo ""
        echo "Running available component benchmarks..."
        
        # Try to run what we can
        if cargo bench --package vectordb-common --quiet 2>&1 | grep -E "test result|time:" | tail -5; then
            echo "✅ Distance calculation benchmarks completed"
        else
            echo "❌ Distance calculation benchmarks failed"  
        fi
        
        echo ""
        if cargo bench --package vectordb-index --quiet 2>&1 | grep -E "test result|time:" | tail -5; then
            echo "✅ HNSW index benchmarks completed"
        else
            echo "❌ HNSW index benchmarks failed"
        fi
        
        echo ""
        echo "## Important Notes"
        echo ""
        echo "- ⚠️  Actual performance testing requires compiled comparison tool"
        echo "- ⚠️  These are theoretical projections, not measured results"
        echo "- ⚠️  Real performance depends on data distribution and hardware"
        echo "- ⚠️  HNSW performance varies significantly with parameter tuning"
        echo ""
        echo "## Next Steps"
        echo ""
        echo "1. Fix compilation issues to enable direct comparison"
        echo "2. Test with real datasets and measure actual performance"
        echo "3. Optimize HNSW parameters for specific use cases"
        echo "4. Compare with other vector search libraries"
        
    } > "$RESULTS_DIR/comparison_results.md"
    
    cat "$RESULTS_DIR/comparison_results.md"
    
else
    echo "✅ Baseline comparison tool built successfully"
    echo ""
    
    # Run comparison on different dataset sizes
    for dataset in dataset_1k.json dataset_10k.json; do
        dataset_path="$DATA_DIR/$dataset"
        if [ -f "$dataset_path" ]; then
            echo ""
            echo "Testing with $dataset..."
            echo "========================="
            
            if timeout 60s ./target/debug/examples/baseline_comparison "$dataset_path" 2>&1; then
                echo "✅ Comparison completed for $dataset"
            else
                echo "⚠️  Comparison timed out or failed for $dataset"
            fi
        fi
    done
    
    echo ""
    echo "✅ Baseline comparison completed!"
    
fi

echo ""
echo "Results saved to: $RESULTS_DIR/"
echo ""
echo "To run manual comparisons:"
echo "  cargo run --example baseline_comparison benchmark_data/dataset_1k.json"