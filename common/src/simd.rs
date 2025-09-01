/// SIMD-optimized vector operations
/// For now, using standard implementations - SIMD optimization can be added later

/// Calculate dot product between two vectors
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x * y)
        .sum()
}

/// Calculate magnitude of a vector
pub fn magnitude(vector: &[f32]) -> f32 {
    vector.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt()
}

/// Calculate Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = x - y;
            diff * diff
        })
        .sum::<f32>()
        .sqrt()
}

/// Calculate Manhattan distance between two vectors
pub fn manhattan_distance(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).abs())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![2.0, 3.0, 4.0, 5.0];
        let result = dot_product(&a, &b);
        assert!((result - 40.0).abs() < 1e-6);
    }

    #[test]
    fn test_magnitude() {
        let v = vec![3.0, 4.0];
        let result = magnitude(&v);
        assert!((result - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let result = euclidean_distance(&a, &b);
        assert!((result - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = vec![1.0, 2.0];
        let b = vec![4.0, 6.0];
        let result = manhattan_distance(&a, &b);
        assert!((result - 7.0).abs() < 1e-6);
    }
}