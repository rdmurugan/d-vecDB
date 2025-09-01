use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for vectors
pub type VectorId = Uuid;

/// Unique identifier for collections
pub type CollectionId = String;

/// Vector data types supported by the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorType {
    Float32,
    Float16,
    Int8,
}

/// Distance metrics for vector similarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
}

/// Vector data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub id: VectorId,
    pub data: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub name: CollectionId,
    pub dimension: usize,
    pub distance_metric: DistanceMetric,
    pub vector_type: VectorType,
    pub index_config: IndexConfig,
}

/// HNSW index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub max_connections: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub max_layer: usize,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            ef_construction: 200,
            ef_search: 50,
            max_layer: 16,
        }
    }
}

/// Query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub collection: CollectionId,
    pub vector: Vec<f32>,
    pub limit: usize,
    pub ef_search: Option<usize>,
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: VectorId,
    pub distance: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Batch insert request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchInsertRequest {
    pub collection: CollectionId,
    pub vectors: Vec<Vector>,
}

/// Statistics for a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub name: CollectionId,
    pub vector_count: usize,
    pub dimension: usize,
    pub index_size: usize,
    pub memory_usage: usize,
}

// Protocol buffer conversions
impl From<DistanceMetric> for i32 {
    fn from(metric: DistanceMetric) -> Self {
        match metric {
            DistanceMetric::Cosine => 1,
            DistanceMetric::Euclidean => 2,
            DistanceMetric::DotProduct => 3,
            DistanceMetric::Manhattan => 4,
        }
    }
}

impl From<i32> for DistanceMetric {
    fn from(value: i32) -> Self {
        match value {
            1 => DistanceMetric::Cosine,
            2 => DistanceMetric::Euclidean,
            3 => DistanceMetric::DotProduct,
            4 => DistanceMetric::Manhattan,
            _ => DistanceMetric::Cosine, // Default fallback
        }
    }
}

impl From<VectorType> for i32 {
    fn from(vtype: VectorType) -> Self {
        match vtype {
            VectorType::Float32 => 1,
            VectorType::Float16 => 2,
            VectorType::Int8 => 3,
        }
    }
}

impl From<i32> for VectorType {
    fn from(value: i32) -> Self {
        match value {
            1 => VectorType::Float32,
            2 => VectorType::Float16,
            3 => VectorType::Int8,
            _ => VectorType::Float32, // Default fallback
        }
    }
}