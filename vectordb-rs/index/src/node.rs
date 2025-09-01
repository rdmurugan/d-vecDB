use vectordb_common::types::VectorId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node in the HNSW graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswNode {
    pub id: VectorId,
    pub vector: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub layer: usize,
    pub connections: Vec<Vec<VectorId>>, // Connections per layer
}

impl HnswNode {
    pub fn new(
        id: VectorId,
        vector: Vec<f32>,
        metadata: Option<HashMap<String, serde_json::Value>>,
        max_layer: usize,
    ) -> Self {
        let mut connections = Vec::with_capacity(max_layer + 1);
        for _ in 0..=max_layer {
            connections.push(Vec::new());
        }
        
        Self {
            id,
            vector,
            metadata,
            layer: max_layer,
            connections,
        }
    }
    
    /// Add connection at a specific layer
    pub fn add_connection(&mut self, layer: usize, neighbor_id: VectorId) {
        if layer < self.connections.len() {
            if !self.connections[layer].contains(&neighbor_id) {
                self.connections[layer].push(neighbor_id);
            }
        }
    }
    
    /// Remove connection at a specific layer
    pub fn remove_connection(&mut self, layer: usize, neighbor_id: &VectorId) -> bool {
        if layer < self.connections.len() {
            if let Some(pos) = self.connections[layer].iter().position(|&id| id == *neighbor_id) {
                self.connections[layer].remove(pos);
                return true;
            }
        }
        false
    }
    
    /// Get connections at a specific layer
    pub fn get_connections(&self, layer: usize) -> &[VectorId] {
        if layer < self.connections.len() {
            &self.connections[layer]
        } else {
            &[]
        }
    }
    
    /// Get number of connections at a specific layer
    pub fn connection_count(&self, layer: usize) -> usize {
        if layer < self.connections.len() {
            self.connections[layer].len()
        } else {
            0
        }
    }
    
    /// Prune connections at a layer to keep only the closest ones
    pub fn prune_connections(&mut self, layer: usize, max_connections: usize, get_distance: impl Fn(&VectorId) -> Option<f32>) {
        if layer >= self.connections.len() || self.connections[layer].len() <= max_connections {
            return;
        }
        
        // Sort connections by distance and keep only the closest ones
        let mut connections_with_distance: Vec<(VectorId, f32)> = self.connections[layer]
            .iter()
            .filter_map(|&id| get_distance(&id).map(|dist| (id, dist)))
            .collect();
        
        connections_with_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        self.connections[layer] = connections_with_distance
            .into_iter()
            .take(max_connections)
            .map(|(id, _)| id)
            .collect();
    }
    
    /// Calculate memory usage of this node
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vector.len() * std::mem::size_of::<f32>()
            + self.connections.iter().map(|layer| layer.len() * std::mem::size_of::<VectorId>()).sum::<usize>()
            + self.metadata.as_ref().map_or(0, |m| {
                m.iter().map(|(k, v)| {
                    k.len() + match v {
                        serde_json::Value::String(s) => s.len(),
                        _ => std::mem::size_of::<serde_json::Value>(),
                    }
                }).sum::<usize>()
            })
    }
}

/// Priority queue entry for search operations
#[derive(Debug, Clone, PartialEq)]
pub struct SearchCandidate {
    pub id: VectorId,
    pub distance: f32,
}

impl Eq for SearchCandidate {}

impl PartialOrd for SearchCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // For max-heap behavior (farthest first for candidate pruning)
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for SearchCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

/// Min-heap variant for nearest neighbors
#[derive(Debug, Clone, PartialEq)]
pub struct NearestCandidate {
    pub id: VectorId,
    pub distance: f32,
}

impl Eq for NearestCandidate {}

impl PartialOrd for NearestCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // For min-heap behavior (nearest first)
        other.distance.partial_cmp(&self.distance)
    }
}

impl Ord for NearestCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_node_creation() {
        let id = Uuid::new_v4();
        let vector = vec![1.0, 2.0, 3.0];
        let node = HnswNode::new(id, vector.clone(), None, 3);
        
        assert_eq!(node.id, id);
        assert_eq!(node.vector, vector);
        assert_eq!(node.layer, 3);
        assert_eq!(node.connections.len(), 4); // 0-3 inclusive
    }
    
    #[test]
    fn test_connections() {
        let id = Uuid::new_v4();
        let neighbor_id = Uuid::new_v4();
        let mut node = HnswNode::new(id, vec![1.0, 2.0], None, 2);
        
        node.add_connection(0, neighbor_id);
        assert_eq!(node.get_connections(0), &[neighbor_id]);
        assert_eq!(node.connection_count(0), 1);
        
        let removed = node.remove_connection(0, &neighbor_id);
        assert!(removed);
        assert_eq!(node.connection_count(0), 0);
    }
    
    #[test]
    fn test_prune_connections() {
        let id = Uuid::new_v4();
        let mut node = HnswNode::new(id, vec![0.0, 0.0], None, 1);
        
        let neighbors = (0..5).map(|_| Uuid::new_v4()).collect::<Vec<_>>();
        for neighbor in &neighbors {
            node.add_connection(0, *neighbor);
        }
        
        // Mock distance function that returns increasing distances
        let get_distance = |neighbor_id: &VectorId| {
            neighbors.iter().position(|&n| n == *neighbor_id).map(|i| i as f32)
        };
        
        node.prune_connections(0, 3, get_distance);
        assert_eq!(node.connection_count(0), 3);
        
        // Should keep the 3 closest neighbors (indices 0, 1, 2)
        let kept_connections = node.get_connections(0);
        assert!(kept_connections.contains(&neighbors[0]));
        assert!(kept_connections.contains(&neighbors[1]));
        assert!(kept_connections.contains(&neighbors[2]));
    }
}