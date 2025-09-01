pub mod wal;
pub mod mmap;
pub mod recovery;

use vectordb_common::{Result, VectorDbError};
use vectordb_common::types::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use parking_lot::RwLock;

pub use wal::*;
pub use mmap::*;
pub use recovery::*;

/// Storage engine for persistent vector storage with WAL
pub struct StorageEngine {
    data_dir: PathBuf,
    collections: RwLock<HashMap<CollectionId, CollectionStorage>>,
    wal: WriteAheadLog,
}

impl StorageEngine {
    pub async fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        
        let wal_path = data_dir.join("wal");
        let wal = WriteAheadLog::new(wal_path).await?;
        
        let mut engine = Self {
            data_dir,
            collections: RwLock::new(HashMap::new()),
            wal,
        };
        
        // Recover from WAL on startup
        engine.recover().await?;
        
        Ok(engine)
    }
    
    pub async fn create_collection(&self, config: &CollectionConfig) -> Result<()> {
        let mut collections = self.collections.write();
        
        if collections.contains_key(&config.name) {
            return Err(VectorDbError::CollectionAlreadyExists {
                name: config.name.clone(),
            });
        }
        
        let collection_dir = self.data_dir.join(&config.name);
        let storage = CollectionStorage::new(collection_dir, config.clone()).await?;
        
        // Log the operation
        let op = WALOperation::CreateCollection(config.clone());
        self.wal.append(&op).await?;
        
        collections.insert(config.name.clone(), storage);
        
        tracing::info!("Created collection: {}", config.name);
        Ok(())
    }
    
    pub async fn delete_collection(&self, name: &str) -> Result<()> {
        let mut collections = self.collections.write();
        
        if !collections.contains_key(name) {
            return Err(VectorDbError::CollectionNotFound {
                name: name.to_string(),
            });
        }
        
        // Log the operation
        let op = WALOperation::DeleteCollection(name.to_string());
        self.wal.append(&op).await?;
        
        collections.remove(name);
        
        // Remove collection directory
        let collection_dir = self.data_dir.join(name);
        if collection_dir.exists() {
            std::fs::remove_dir_all(collection_dir)?;
        }
        
        tracing::info!("Deleted collection: {}", name);
        Ok(())
    }
    
    pub async fn insert_vector(&self, collection: &str, vector: &Vector) -> Result<()> {
        let collections = self.collections.read();
        let storage = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound {
                name: collection.to_string(),
            })?;
        
        // Log the operation
        let op = WALOperation::InsertVector {
            collection: collection.to_string(),
            vector: vector.clone(),
        };
        self.wal.append(&op).await?;
        
        storage.insert(vector).await?;
        
        Ok(())
    }
    
    pub async fn batch_insert(&self, collection: &str, vectors: &[Vector]) -> Result<()> {
        let collections = self.collections.read();
        let storage = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound {
                name: collection.to_string(),
            })?;
        
        // Log the operation
        let op = WALOperation::BatchInsert {
            collection: collection.to_string(),
            vectors: vectors.to_vec(),
        };
        self.wal.append(&op).await?;
        
        storage.batch_insert(vectors).await?;
        
        Ok(())
    }
    
    pub async fn get_vector(&self, collection: &str, id: &VectorId) -> Result<Option<Vector>> {
        let collections = self.collections.read();
        let storage = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound {
                name: collection.to_string(),
            })?;
        
        storage.get(id).await
    }
    
    pub async fn delete_vector(&self, collection: &str, id: &VectorId) -> Result<bool> {
        let collections = self.collections.read();
        let storage = collections
            .get(collection)
            .ok_or_else(|| VectorDbError::CollectionNotFound {
                name: collection.to_string(),
            })?;
        
        // Log the operation
        let op = WALOperation::DeleteVector {
            collection: collection.to_string(),
            id: *id,
        };
        self.wal.append(&op).await?;
        
        storage.delete(id).await
    }
    
    pub fn list_collections(&self) -> Vec<CollectionId> {
        let collections = self.collections.read();
        collections.keys().cloned().collect()
    }
    
    pub fn get_collection_config(&self, name: &str) -> Result<Option<CollectionConfig>> {
        let collections = self.collections.read();
        Ok(collections.get(name).map(|s| s.config().clone()))
    }
    
    pub async fn get_collection_stats(&self, name: &str) -> Result<Option<CollectionStats>> {
        let collections = self.collections.read();
        if let Some(storage) = collections.get(name) {
            Ok(Some(storage.stats().await?))
        } else {
            Ok(None)
        }
    }
    
    pub async fn sync(&self) -> Result<()> {
        self.wal.sync().await?;
        
        let collections = self.collections.read();
        for storage in collections.values() {
            storage.sync().await?;
        }
        
        Ok(())
    }
    
    async fn recover(&mut self) -> Result<()> {
        let recovery = RecoveryManager::new(&self.data_dir);
        let operations = recovery.recover_from_wal(&self.wal).await?;
        
        tracing::info!("Recovering {} operations from WAL", operations.len());
        
        for op in operations {
            self.apply_operation(op).await?;
        }
        
        Ok(())
    }
    
    async fn apply_operation(&mut self, op: WALOperation) -> Result<()> {
        match op {
            WALOperation::CreateCollection(config) => {
                let collection_dir = self.data_dir.join(&config.name);
                let storage = CollectionStorage::new(collection_dir, config.clone()).await?;
                self.collections.write().insert(config.name.clone(), storage);
            }
            WALOperation::DeleteCollection(name) => {
                self.collections.write().remove(&name);
            }
            WALOperation::InsertVector { collection, vector } => {
                if let Some(storage) = self.collections.read().get(&collection) {
                    storage.insert(&vector).await?;
                }
            }
            WALOperation::BatchInsert { collection, vectors } => {
                if let Some(storage) = self.collections.read().get(&collection) {
                    storage.batch_insert(&vectors).await?;
                }
            }
            WALOperation::DeleteVector { collection, id } => {
                if let Some(storage) = self.collections.read().get(&collection) {
                    storage.delete(&id).await?;
                }
            }
        }
        
        Ok(())
    }
}

/// Storage for a single collection
pub struct CollectionStorage {
    config: CollectionConfig,
    data_file: MMapStorage,
    index_file: MMapStorage,
}

impl CollectionStorage {
    async fn new<P: AsRef<Path>>(dir: P, config: CollectionConfig) -> Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        
        let data_path = dir.join("vectors.bin");
        let index_path = dir.join("index.bin");
        
        let data_file = MMapStorage::new(data_path).await?;
        let index_file = MMapStorage::new(index_path).await?;
        
        Ok(Self {
            config,
            data_file,
            index_file,
        })
    }
    
    fn config(&self) -> &CollectionConfig {
        &self.config
    }
    
    async fn insert(&self, vector: &Vector) -> Result<()> {
        if vector.data.len() != self.config.dimension {
            return Err(VectorDbError::InvalidDimension {
                expected: self.config.dimension,
                actual: vector.data.len(),
            });
        }
        
        let serialized = bincode::serialize(vector)
            .map_err(|e| VectorDbError::Serialization(e.to_string()))?;
        
        self.data_file.append(&serialized).await?;
        
        Ok(())
    }
    
    async fn batch_insert(&self, vectors: &[Vector]) -> Result<()> {
        for vector in vectors {
            self.insert(vector).await?;
        }
        Ok(())
    }
    
    async fn get(&self, _id: &VectorId) -> Result<Option<Vector>> {
        // TODO: Implement efficient lookup using index
        // For now, this is a placeholder
        Ok(None)
    }
    
    async fn delete(&self, _id: &VectorId) -> Result<bool> {
        // TODO: Implement deletion with tombstone markers
        // For now, this is a placeholder
        Ok(false)
    }
    
    async fn stats(&self) -> Result<CollectionStats> {
        Ok(CollectionStats {
            name: self.config.name.clone(),
            vector_count: 0, // TODO: Track count
            dimension: self.config.dimension,
            index_size: self.index_file.size().await? as usize,
            memory_usage: (self.data_file.size().await? + self.index_file.size().await?) as usize,
        })
    }
    
    async fn sync(&self) -> Result<()> {
        self.data_file.sync().await?;
        self.index_file.sync().await?;
        Ok(())
    }
}