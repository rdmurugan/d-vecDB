use vectordb_common::{Result, VectorDbError};
use crate::wal::{WriteAheadLog, WALOperation};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use tokio::fs;
use tracing::{info, warn};

/// Recovery manager for crash recovery and consistency
pub struct RecoveryManager {
    data_dir: PathBuf,
}

impl RecoveryManager {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Recover from WAL after a crash
    pub async fn recover_from_wal(&self, wal: &WriteAheadLog) -> Result<Vec<WALOperation>> {
        info!("Starting crash recovery from WAL");
        
        // Read all operations from WAL
        let operations = wal.read_all().await?;
        
        if operations.is_empty() {
            info!("No operations found in WAL, recovery complete");
            return Ok(operations);
        }
        
        // Validate operations and check for consistency
        let validated_ops = self.validate_operations(&operations).await?;
        
        info!("Recovered {} valid operations from WAL", validated_ops.len());
        Ok(validated_ops)
    }
    
    /// Validate operations for consistency
    async fn validate_operations(&self, operations: &[WALOperation]) -> Result<Vec<WALOperation>> {
        let mut valid_ops = Vec::new();
        let mut existing_collections = self.discover_existing_collections().await?;
        
        for (i, op) in operations.iter().enumerate() {
            match self.validate_operation(op, &mut existing_collections).await {
                Ok(()) => {
                    valid_ops.push(op.clone());
                }
                Err(e) => {
                    warn!("Invalid operation at position {}: {:?} - {}", i, op, e);
                    // Continue with remaining operations
                }
            }
        }
        
        Ok(valid_ops)
    }
    
    /// Validate a single operation
    async fn validate_operation(
        &self,
        op: &WALOperation,
        existing_collections: &mut HashSet<String>,
    ) -> Result<()> {
        match op {
            WALOperation::CreateCollection(config) => {
                if existing_collections.contains(&config.name) {
                    return Err(VectorDbError::Internal {
                        message: format!("Collection {} already exists", config.name),
                    });
                }
                existing_collections.insert(config.name.clone());
            }
            WALOperation::DeleteCollection(name) => {
                if !existing_collections.contains(name) {
                    return Err(VectorDbError::Internal {
                        message: format!("Collection {} does not exist", name),
                    });
                }
                existing_collections.remove(name);
            }
            WALOperation::InsertVector { collection, vector: _ } => {
                if !existing_collections.contains(collection) {
                    return Err(VectorDbError::Internal {
                        message: format!("Collection {} does not exist", collection),
                    });
                }
                // Additional vector validation could go here
            }
            WALOperation::BatchInsert { collection, vectors } => {
                if !existing_collections.contains(collection) {
                    return Err(VectorDbError::Internal {
                        message: format!("Collection {} does not exist", collection),
                    });
                }
                // Validate all vectors in the batch
                for vector in vectors {
                    if vector.data.is_empty() {
                        return Err(VectorDbError::Internal {
                            message: "Empty vector in batch".to_string(),
                        });
                    }
                }
            }
            WALOperation::DeleteVector { collection, .. } => {
                if !existing_collections.contains(collection) {
                    return Err(VectorDbError::Internal {
                        message: format!("Collection {} does not exist", collection),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Discover collections that exist on disk
    async fn discover_existing_collections(&self) -> Result<HashSet<String>> {
        let mut collections = HashSet::new();
        
        if !self.data_dir.exists() {
            return Ok(collections);
        }
        
        let mut entries = fs::read_dir(&self.data_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                // Check if it looks like a collection directory
                let vectors_file = path.join("vectors.bin");
                let index_file = path.join("index.bin");
                
                if vectors_file.exists() || index_file.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        collections.insert(name.to_string());
                    }
                }
            }
        }
        
        info!("Discovered {} existing collections on disk", collections.len());
        Ok(collections)
    }
    
    /// Perform consistency check on collections
    pub async fn check_consistency(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        let collections = self.discover_existing_collections().await?;
        
        for collection in &collections {
            if let Err(e) = self.check_collection_consistency(collection).await {
                let issue = format!("Collection {}: {}", collection, e);
                issues.push(issue);
            }
        }
        
        if issues.is_empty() {
            info!("All collections passed consistency check");
        } else {
            warn!("Found {} consistency issues", issues.len());
            for issue in &issues {
                warn!("{}", issue);
            }
        }
        
        Ok(issues)
    }
    
    /// Check consistency of a single collection
    async fn check_collection_consistency(&self, collection: &str) -> Result<()> {
        let collection_dir = self.data_dir.join(collection);
        
        // Check if required files exist
        let vectors_file = collection_dir.join("vectors.bin");
        let index_file = collection_dir.join("index.bin");
        
        if !vectors_file.exists() && !index_file.exists() {
            return Err(VectorDbError::StorageError {
                message: "No data files found".to_string(),
            });
        }
        
        // Check file sizes and basic integrity
        if vectors_file.exists() {
            let metadata = fs::metadata(&vectors_file).await?;
            if metadata.len() == 0 {
                return Err(VectorDbError::StorageError {
                    message: "Empty vectors file".to_string(),
                });
            }
        }
        
        // Additional checks could include:
        // - Vector count consistency between data and index
        // - Index structure validation
        // - Data format validation
        
        Ok(())
    }
    
    /// Create backup of data directory
    pub async fn create_backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();
        
        info!("Creating backup at: {}", backup_path.display());
        
        // Create backup directory
        fs::create_dir_all(backup_path).await?;
        
        // Copy all collection directories
        let collections = self.discover_existing_collections().await?;
        
        for collection in &collections {
            let src_dir = self.data_dir.join(collection);
            let dst_dir = backup_path.join(collection);
            
            self.copy_dir_recursive(&src_dir, &dst_dir).await?;
        }
        
        info!("Backup completed successfully");
        Ok(())
    }
    
    /// Recursively copy directory
    async fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        queue.push_back((src.to_path_buf(), dst.to_path_buf()));
        
        while let Some((src_path, dst_path)) = queue.pop_front() {
            fs::create_dir_all(&dst_path).await?;
            
            let mut entries = fs::read_dir(&src_path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let entry_src = entry.path();
                let entry_dst = dst_path.join(entry.file_name());
                
                if entry_src.is_dir() {
                    queue.push_back((entry_src, entry_dst));
                } else {
                    fs::copy(&entry_src, &entry_dst).await?;
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use vectordb_common::types::*;
    
    #[tokio::test]
    async fn test_discover_collections() {
        let temp_dir = tempdir().unwrap();
        let recovery = RecoveryManager::new(temp_dir.path());
        
        // Create a mock collection directory
        let collection_dir = temp_dir.path().join("test_collection");
        fs::create_dir_all(&collection_dir).await.unwrap();
        fs::File::create(collection_dir.join("vectors.bin")).await.unwrap();
        
        let collections = recovery.discover_existing_collections().await.unwrap();
        assert!(collections.contains("test_collection"));
    }
    
    #[tokio::test]
    async fn test_validate_operations() {
        let temp_dir = tempdir().unwrap();
        let recovery = RecoveryManager::new(temp_dir.path());
        
        let config = CollectionConfig {
            name: "test".to_string(),
            dimension: 128,
            distance_metric: DistanceMetric::Cosine,
            vector_type: VectorType::Float32,
            index_config: IndexConfig::default(),
        };
        
        let operations = vec![
            WALOperation::CreateCollection(config.clone()),
            WALOperation::InsertVector {
                collection: "test".to_string(),
                vector: Vector {
                    id: uuid::Uuid::new_v4(),
                    data: vec![0.1; 128],
                    metadata: None,
                },
            },
        ];
        
        let validated = recovery.validate_operations(&operations).await.unwrap();
        assert_eq!(validated.len(), 2);
    }
}