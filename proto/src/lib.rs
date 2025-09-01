pub mod vectordb {
    tonic::include_proto!("vectordb.v1");
}

pub use vectordb::*;

use vectordb_common::types;

// Conversion functions between protobuf and common types
impl From<types::DistanceMetric> for DistanceMetric {
    fn from(metric: types::DistanceMetric) -> Self {
        match metric {
            types::DistanceMetric::Cosine => DistanceMetric::Cosine,
            types::DistanceMetric::Euclidean => DistanceMetric::Euclidean,
            types::DistanceMetric::DotProduct => DistanceMetric::DotProduct,
            types::DistanceMetric::Manhattan => DistanceMetric::Manhattan,
        }
    }
}

impl From<DistanceMetric> for types::DistanceMetric {
    fn from(metric: DistanceMetric) -> Self {
        match metric {
            DistanceMetric::Cosine => types::DistanceMetric::Cosine,
            DistanceMetric::Euclidean => types::DistanceMetric::Euclidean,
            DistanceMetric::DotProduct => types::DistanceMetric::DotProduct,
            DistanceMetric::Manhattan => types::DistanceMetric::Manhattan,
            _ => types::DistanceMetric::Cosine, // Default fallback
        }
    }
}

impl From<types::VectorType> for VectorType {
    fn from(vector_type: types::VectorType) -> Self {
        match vector_type {
            types::VectorType::Float32 => VectorType::Float32,
            types::VectorType::Float16 => VectorType::Float16,
            types::VectorType::Int8 => VectorType::Int8,
        }
    }
}

impl From<VectorType> for types::VectorType {
    fn from(vector_type: VectorType) -> Self {
        match vector_type {
            VectorType::Float32 => types::VectorType::Float32,
            VectorType::Float16 => types::VectorType::Float16,
            VectorType::Int8 => types::VectorType::Int8,
            _ => types::VectorType::Float32, // Default fallback
        }
    }
}