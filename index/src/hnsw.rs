use crate::{VectorIndex, SearchResult, IndexStats};
use crate::node::{HnswNode, SearchCandidate, NearestCandidate};
use vectordb_common::{Result, VectorDbError, distance, types::*};
use std::collections::{HashMap, BinaryHeap, HashSet};
use parking_lot::RwLock;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// HNSW (Hierarchical Navigable Small World) index implementation
#[derive(Debug)]
pub struct HnswIndex {
    nodes: RwLock<HashMap<VectorId, HnswNode>>,
    entry_point: RwLock<Option<VectorId>>,
    config: IndexConfig,
    distance_metric: DistanceMetric,
    dimension: usize,
    rng: RwLock<StdRng>,
}

impl HnswIndex {
    pub fn new(config: IndexConfig, distance_metric: DistanceMetric, dimension: usize) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            entry_point: RwLock::new(None),
            config,
            distance_metric,
            dimension,
            rng: RwLock::new(StdRng::from_entropy()),
        }
    }
    
    /// Select layer for a new node using exponential decay distribution
    fn select_layer(&self) -> usize {
        let mut rng = self.rng.write();
        let mut layer = 0;
        let _ml = 1.0 / (2.0_f64).ln(); // Normalization factor
        
        while rng.gen::<f64>() < 0.5 && layer < self.config.max_layer {
            layer += 1;
        }
        
        layer
    }
    
    /// Calculate distance between two vectors
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        distance(a, b, self.distance_metric)
    }
    
    /// Search for entry points at the given layer
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[VectorId],
        num_closest: usize,
        layer: usize,
    ) -> Result<Vec<SearchCandidate>> {
        let nodes = self.nodes.read();
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new(); // Max-heap for farthest candidates
        let mut dynamic_list = BinaryHeap::new(); // Min-heap for nearest candidates
        
        // Initialize with entry points
        for &entry_id in entry_points {
            if let Some(entry_node) = nodes.get(&entry_id) {
                let dist = self.distance(query, &entry_node.vector);
                candidates.push(SearchCandidate {
                    id: entry_id,
                    distance: dist,
                });
                dynamic_list.push(NearestCandidate {
                    id: entry_id,
                    distance: dist,
                });
                visited.insert(entry_id);
            }
        }
        
        while let Some(current_candidate) = dynamic_list.pop() {
            // If current candidate is farther than the worst in candidates, stop
            if let Some(worst) = candidates.peek() {
                if current_candidate.distance > worst.distance && candidates.len() >= num_closest {
                    break;
                }
            }
            
            if let Some(current_node) = nodes.get(&current_candidate.id) {
                // Explore neighbors
                for &neighbor_id in current_node.get_connections(layer) {
                    if visited.contains(&neighbor_id) {
                        continue;
                    }
                    
                    visited.insert(neighbor_id);
                    
                    if let Some(neighbor_node) = nodes.get(&neighbor_id) {
                        let dist = self.distance(query, &neighbor_node.vector);
                        
                        let should_add = if candidates.len() < num_closest {
                            true
                        } else if let Some(worst) = candidates.peek() {
                            dist < worst.distance
                        } else {
                            false
                        };
                        
                        if should_add {
                            candidates.push(SearchCandidate {
                                id: neighbor_id,
                                distance: dist,
                            });
                            dynamic_list.push(NearestCandidate {
                                id: neighbor_id,
                                distance: dist,
                            });
                            
                            // Prune candidates if too many
                            if candidates.len() > num_closest {
                                candidates.pop();
                            }
                        }
                    }
                }
            }
        }
        
        // Convert to sorted vector (closest first)
        let mut result: Vec<SearchCandidate> = candidates.into_sorted_vec();
        result.reverse(); // Min distance first
        result.truncate(num_closest);
        
        Ok(result)
    }
    
    /// Select M neighbors using heuristic
    fn select_neighbors(&self, candidates: &[SearchCandidate], m: usize) -> Vec<VectorId> {
        if candidates.len() <= m {
            return candidates.iter().map(|c| c.id).collect();
        }
        
        // Simple heuristic: select M closest candidates
        // More sophisticated heuristics could consider diversity
        candidates.iter().take(m).map(|c| c.id).collect()
    }
    
    /// Get M and max_M values for a layer
    fn get_m_values(&self, layer: usize) -> (usize, usize) {
        if layer == 0 {
            (self.config.max_connections, self.config.max_connections * 2)
        } else {
            (self.config.max_connections, self.config.max_connections)
        }
    }
}

impl VectorIndex for HnswIndex {
    fn insert(
        &mut self,
        id: VectorId,
        vector: &[f32],
        metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(VectorDbError::InvalidDimension {
                expected: self.dimension,
                actual: vector.len(),
            });
        }
        
        let layer = self.select_layer();
        let mut new_node = HnswNode::new(id, vector.to_vec(), metadata, layer);
        
        let mut nodes = self.nodes.write();
        let mut entry_point = self.entry_point.write();
        
        // If this is the first node, make it the entry point
        if entry_point.is_none() {
            *entry_point = Some(id);
            nodes.insert(id, new_node);
            return Ok(());
        }
        
        let entry_id = entry_point.unwrap();
        let entry_layer = nodes.get(&entry_id).unwrap().layer;
        
        // Search from top layer down to layer+1
        let mut current_closest = vec![entry_id];
        for lc in (layer + 1..=entry_layer).rev() {
            drop(nodes); // Release write lock temporarily
            let candidates = self.search_layer(vector, &current_closest, 1, lc)?;
            nodes = self.nodes.write(); // Reacquire write lock
            current_closest = candidates.into_iter().map(|c| c.id).collect();
        }
        
        // Search and connect from layer down to 0
        for lc in (0..=layer).rev() {
            drop(nodes); // Release write lock temporarily
            let ef = if lc == 0 {
                std::cmp::max(self.config.ef_construction, self.config.max_connections)
            } else {
                self.config.ef_construction
            };
            let candidates = self.search_layer(vector, &current_closest, ef, lc)?;
            nodes = self.nodes.write(); // Reacquire write lock
            
            let (m, max_m) = self.get_m_values(lc);
            let selected = self.select_neighbors(&candidates, m);
            
            // Add connections from new node to selected neighbors
            for &neighbor_id in &selected {
                new_node.add_connection(lc, neighbor_id);
                
                // Add reverse connection
                if let Some(neighbor_node) = nodes.get_mut(&neighbor_id) {
                    neighbor_node.add_connection(lc, id);
                    
                    // Prune connections if needed
                    if neighbor_node.connection_count(lc) > max_m {
                        let neighbor_vector = neighbor_node.vector.clone();
                        drop(nodes); // Release write lock temporarily
                        let neighbor_candidates = self.search_layer(
                            &neighbor_vector,
                            &[id],
                            max_m + 1,
                            lc,
                        )?;
                        nodes = self.nodes.write(); // Reacquire write lock
                        
                        if let Some(neighbor_node) = nodes.get_mut(&neighbor_id) {
                            let new_connections = self.select_neighbors(&neighbor_candidates, max_m);
                            neighbor_node.connections[lc] = new_connections;
                        }
                    }
                }
            }
            
            current_closest = selected;
        }
        
        // Update entry point if necessary
        if layer > entry_layer {
            *entry_point = Some(id);
        }
        
        nodes.insert(id, new_node);
        Ok(())
    }
    
    fn search(&self, query: &[f32], limit: usize, ef: Option<usize>) -> Result<Vec<SearchResult>> {
        if query.len() != self.dimension {
            return Err(VectorDbError::InvalidDimension {
                expected: self.dimension,
                actual: query.len(),
            });
        }
        
        let entry_point = self.entry_point.read();
        
        let entry_id = match *entry_point {
            Some(id) => id,
            None => return Ok(Vec::new()), // Empty index
        };
        
        let entry_layer = {
            let nodes = self.nodes.read();
            nodes.get(&entry_id).unwrap().layer
        };
        let ef_search = ef.unwrap_or(self.config.ef_search);
        
        // Search from top layer down to layer 1
        let mut current_closest = vec![entry_id];
        for lc in (1..=entry_layer).rev() {
            let candidates = self.search_layer(query, &current_closest, 1, lc)?;
            current_closest = candidates.into_iter().map(|c| c.id).collect();
        }
        
        // Search layer 0 with ef parameter
        let candidates = self.search_layer(query, &current_closest, std::cmp::max(ef_search, limit), 0)?;
        
        // Convert to SearchResult and limit results
        let mut results = Vec::new();
        {
            let nodes = self.nodes.read();
            for candidate in candidates.into_iter().take(limit) {
                if let Some(node) = nodes.get(&candidate.id) {
                    results.push(SearchResult {
                        id: candidate.id,
                        distance: candidate.distance,
                        metadata: node.metadata.clone(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    fn delete(&mut self, id: &VectorId) -> Result<bool> {
        let mut nodes = self.nodes.write();
        let mut entry_point = self.entry_point.write();
        
        let _node = match nodes.remove(id) {
            Some(node) => node,
            None => return Ok(false),
        };
        
        // Remove all connections to this node
        for (_, other_node) in nodes.iter_mut() {
            for layer in 0..other_node.connections.len() {
                other_node.remove_connection(layer, id);
            }
        }
        
        // Update entry point if necessary
        if entry_point.as_ref() == Some(id) {
            // Find new entry point (node with highest layer)
            let new_entry = nodes
                .iter()
                .max_by_key(|(_, node)| node.layer)
                .map(|(&id, _)| id);
            *entry_point = new_entry;
        }
        
        Ok(true)
    }
    
    fn stats(&self) -> IndexStats {
        let nodes = self.nodes.read();
        let vector_count = nodes.len();
        
        let memory_usage = nodes
            .values()
            .map(|node| node.memory_usage())
            .sum::<usize>();
        
        let max_layer = nodes
            .values()
            .map(|node| node.layer)
            .max()
            .unwrap_or(0);
        
        let avg_connections = if vector_count > 0 {
            let total_connections: usize = nodes
                .values()
                .map(|node| node.connections.iter().map(|layer| layer.len()).sum::<usize>())
                .sum();
            total_connections as f32 / vector_count as f32
        } else {
            0.0
        };
        
        IndexStats {
            vector_count,
            memory_usage,
            dimension: self.dimension,
            max_layer,
            avg_connections,
        }
    }
    
    fn serialize(&self) -> Result<Vec<u8>> {
        #[derive(Serialize)]
        struct SerializedIndex {
            nodes: HashMap<VectorId, HnswNode>,
            entry_point: Option<VectorId>,
            config: IndexConfig,
            distance_metric: DistanceMetric,
            dimension: usize,
        }
        
        let nodes = self.nodes.read();
        let entry_point = self.entry_point.read();
        
        let serialized = SerializedIndex {
            nodes: nodes.clone(),
            entry_point: *entry_point,
            config: self.config.clone(),
            distance_metric: self.distance_metric,
            dimension: self.dimension,
        };
        
        bincode::serialize(&serialized)
            .map_err(|e| VectorDbError::Serialization(e.to_string()))
    }
    
    fn deserialize(&mut self, data: &[u8]) -> Result<()> {
        #[derive(Deserialize)]
        struct SerializedIndex {
            nodes: HashMap<VectorId, HnswNode>,
            entry_point: Option<VectorId>,
            config: IndexConfig,
            distance_metric: DistanceMetric,
            dimension: usize,
        }
        
        let serialized: SerializedIndex = bincode::deserialize(data)
            .map_err(|e| VectorDbError::Serialization(e.to_string()))?;
        
        *self.nodes.write() = serialized.nodes;
        *self.entry_point.write() = serialized.entry_point;
        self.config = serialized.config;
        self.distance_metric = serialized.distance_metric;
        self.dimension = serialized.dimension;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    fn create_test_index() -> HnswIndex {
        let config = IndexConfig {
            max_connections: 16,
            ef_construction: 200,
            ef_search: 50,
            max_layer: 16,
        };
        HnswIndex::new(config, DistanceMetric::Cosine, 3)
    }
    
    #[test]
    fn test_insert_and_search() {
        let mut index = create_test_index();
        
        // Insert some vectors
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        
        let ids: Vec<VectorId> = (0..3).map(|_| Uuid::new_v4()).collect();
        
        for (i, vector) in vectors.iter().enumerate() {
            index.insert(ids[i], vector, None).unwrap();
        }
        
        // Search for nearest to [1, 0, 0]
        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 2, None).unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].id, ids[0]); // Should find the exact match first
    }
    
    #[test]
    fn test_delete() {
        let mut index = create_test_index();
        
        let id = Uuid::new_v4();
        let vector = vec![1.0, 0.0, 0.0];
        
        index.insert(id, &vector, None).unwrap();
        assert!(index.delete(&id).unwrap());
        assert!(!index.delete(&id).unwrap()); // Should return false for non-existent
        
        let results = index.search(&vector, 1, None).unwrap();
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_layer_selection() {
        let index = create_test_index();
        
        // Layer selection should produce valid layers
        for _ in 0..100 {
            let layer = index.select_layer();
            assert!(layer <= index.config.max_layer);
        }
    }
}