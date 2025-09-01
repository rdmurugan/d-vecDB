use crate::types::DistanceMetric;
use crate::simd;

/// Calculate distance between two vectors using the specified metric
pub fn distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector dimensions must match");

    match metric {
        DistanceMetric::Cosine => 1.0 - cosine_similarity(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => -dot_product(a, b), // Negative for similarity ordering
        DistanceMetric::Manhattan => manhattan_distance(a, b),
    }
}

/// Calculate cosine similarity between two vectors
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot = simd::dot_product(a, b);
    let norm_a = simd::magnitude(a);
    let norm_b = simd::magnitude(b);
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    simd::euclidean_distance(a, b)
}

/// Calculate dot product between two vectors
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    simd::dot_product(a, b)
}

/// Calculate Manhattan (L1) distance between two vectors
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    simd::manhattan_distance(a, b)
}

/// Normalize a vector to unit length
pub fn normalize(vector: &mut [f32]) {
    let magnitude = simd::magnitude(vector);
    if magnitude > 0.0 {
        for v in vector.iter_mut() {
            *v /= magnitude;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
    }
}