use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vectordb_common::types::*;
use vectordb_common::distance::*;
use vectordb_index::{VectorIndex, HnswIndex};
use uuid::Uuid;
use std::collections::HashMap;

fn generate_test_vectors(count: usize, dim: usize) -> Vec<(VectorId, Vec<f32>)> {
    (0..count)
        .map(|i| {
            let id = Uuid::new_v4();
            let vector: Vec<f32> = (0..dim)
                .map(|j| ((i * dim + j) as f32).sin())
                .collect();
            (id, vector)
        })
        .collect()
}

fn bench_distance_calculations(c: &mut Criterion) {
    let dim = 128;
    let a = vec![1.0; dim];
    let b = vec![0.5; dim];
    
    c.bench_function("cosine_distance", |bench| {
        bench.iter(|| {
            black_box(distance(&a, &b, DistanceMetric::Cosine))
        })
    });
    
    c.bench_function("euclidean_distance", |bench| {
        bench.iter(|| {
            black_box(distance(&a, &b, DistanceMetric::Euclidean))
        })
    });
    
    c.bench_function("dot_product", |bench| {
        bench.iter(|| {
            black_box(distance(&a, &b, DistanceMetric::DotProduct))
        })
    });
}

fn bench_hnsw_operations(c: &mut Criterion) {
    let dim = 128;
    let config = IndexConfig {
        max_connections: 16,
        ef_construction: 200,
        ef_search: 50,
        max_layer: 16,
    };
    
    // Benchmark vector insertion
    c.bench_function("hnsw_insert_1000", |bench| {
        bench.iter(|| {
            let mut index = HnswIndex::new(config.clone(), DistanceMetric::Cosine, dim);
            let vectors = generate_test_vectors(1000, dim);
            
            for (id, vector) in vectors {
                let _ = index.insert(id, &vector, None);
            }
        })
    });
    
    // Benchmark search performance
    let mut index = HnswIndex::new(config.clone(), DistanceMetric::Cosine, dim);
    let vectors = generate_test_vectors(5000, dim);
    
    // Pre-populate index
    for (id, vector) in &vectors {
        let _ = index.insert(*id, vector, None);
    }
    
    let query_vector = vec![0.7; dim];
    
    c.bench_function("hnsw_search_5000", |bench| {
        bench.iter(|| {
            black_box(index.search(&query_vector, 10, None).unwrap())
        })
    });
}

fn bench_metadata_operations(c: &mut Criterion) {
    let dim = 128;
    let config = IndexConfig::default();
    let mut index = HnswIndex::new(config, DistanceMetric::Cosine, dim);
    
    // Test with metadata
    c.bench_function("insert_with_metadata", |bench| {
        bench.iter(|| {
            let id = Uuid::new_v4();
            let vector = vec![0.5; dim];
            let mut metadata = HashMap::new();
            metadata.insert("category".to_string(), serde_json::json!("test"));
            metadata.insert("score".to_string(), serde_json::json!(0.95));
            
            let _ = index.insert(id, &vector, Some(metadata));
        })
    });
}

criterion_group!(
    vector_benches,
    bench_distance_calculations,
    bench_hnsw_operations,
    bench_metadata_operations
);
criterion_main!(vector_benches);