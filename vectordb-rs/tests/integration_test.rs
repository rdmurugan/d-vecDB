use vectordb_common::types::*;
use uuid::Uuid;

#[tokio::test]
async fn test_basic_vector_operations() {
    // Test basic vector creation and operations
    let vector_id = Uuid::new_v4();
    let vector_data = vec![1.0, 2.0, 3.0];
    
    let vector = Vector {
        id: vector_id,
        data: vector_data.clone(),
        metadata: None,
    };
    
    assert_eq!(vector.id, vector_id);
    assert_eq!(vector.data, vector_data);
    assert!(vector.metadata.is_none());
    
    println!("✅ Basic vector operations test passed");
}

#[test]
fn test_distance_calculations() {
    use vectordb_common::distance::*;
    
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];
    
    // Test cosine distance
    let cos_dist = distance(&a, &b, DistanceMetric::Cosine);
    assert!((cos_dist - 1.0).abs() < 1e-6); // Should be 1.0 for perpendicular vectors
    
    // Test euclidean distance  
    let eucl_dist = distance(&a, &b, DistanceMetric::Euclidean);
    assert!((eucl_dist - 1.414213).abs() < 1e-5); // sqrt(2)
    
    println!("✅ Distance calculations test passed");
}

#[test]
fn test_collection_config() {
    let config = CollectionConfig {
        name: "test_collection".to_string(),
        dimension: 128,
        distance_metric: DistanceMetric::Cosine,
        vector_type: VectorType::Float32,
        index_config: IndexConfig::default(),
    };
    
    assert_eq!(config.name, "test_collection");
    assert_eq!(config.dimension, 128);
    assert_eq!(config.distance_metric, DistanceMetric::Cosine);
    
    println!("✅ Collection config test passed");
}