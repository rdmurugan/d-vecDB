use vectordb_common::{Result, VectorDbError};
use vectordb_common::types::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use uuid::Uuid;

/// Write-Ahead Log operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALOperation {
    CreateCollection(CollectionConfig),
    DeleteCollection(CollectionId),
    InsertVector {
        collection: CollectionId,
        vector: Vector,
    },
    BatchInsert {
        collection: CollectionId,
        vectors: Vec<Vector>,
    },
    DeleteVector {
        collection: CollectionId,
        id: VectorId,
    },
}

/// WAL entry with metadata
#[derive(Debug, Serialize, Deserialize)]
struct WALEntry {
    id: Uuid,
    timestamp: u64,
    checksum: u32,
    operation: WALOperation,
}

/// Write-Ahead Log for durability
pub struct WriteAheadLog {
    path: PathBuf,
}

impl WriteAheadLog {
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Create file if it doesn't exist
        if !path.exists() {
            tokio::fs::File::create(&path).await?;
        }
        
        Ok(Self { path })
    }
    
    /// Append an operation to the WAL
    pub async fn append(&self, operation: &WALOperation) -> Result<()> {
        let entry = WALEntry {
            id: Uuid::new_v4(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            checksum: 0, // TODO: Implement proper checksumming
            operation: operation.clone(),
        };
        
        let serialized = bincode::serialize(&entry)
            .map_err(|e| VectorDbError::Serialization(e.to_string()))?;
        
        // Write length prefix followed by data
        let length = serialized.len() as u32;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;
        
        file.write_all(&length.to_le_bytes()).await?;
        file.write_all(&serialized).await?;
        file.sync_all().await?;
        
        tracing::debug!("Appended WAL entry: {:?}", entry.id);
        Ok(())
    }
    
    /// Read all entries from the WAL
    pub async fn read_all(&self) -> Result<Vec<WALOperation>> {
        let file = File::open(&self.path).await?;
        let mut reader = BufReader::new(file);
        let mut operations = Vec::new();
        
        loop {
            // Read length prefix
            let mut length_bytes = [0u8; 4];
            match reader.read_exact(&mut length_bytes).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            
            let length = u32::from_le_bytes(length_bytes) as usize;
            
            // Read entry data
            let mut entry_data = vec![0u8; length];
            reader.read_exact(&mut entry_data).await?;
            
            // Deserialize entry
            let entry: WALEntry = bincode::deserialize(&entry_data)
                .map_err(|e| VectorDbError::Serialization(e.to_string()))?;
            
            // TODO: Verify checksum
            
            operations.push(entry.operation);
        }
        
        tracing::info!("Read {} operations from WAL", operations.len());
        Ok(operations)
    }
    
    /// Sync the WAL to disk
    pub async fn sync(&self) -> Result<()> {
        // WAL is synced on every append, so this is a no-op
        Ok(())
    }
    
    /// Truncate the WAL (after successful checkpoint)
    pub async fn truncate(&mut self) -> Result<()> {
        // Truncate the file
        let _file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .await?;
        
        tracing::info!("Truncated WAL");
        Ok(())
    }
    
    /// Get WAL file size
    pub async fn size(&self) -> Result<u64> {
        let metadata = tokio::fs::metadata(&self.path).await?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_wal_operations() {
        let temp_dir = tempdir().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let mut wal = WriteAheadLog::new(&wal_path).await.unwrap();
        
        let config = CollectionConfig {
            name: "test".to_string(),
            dimension: 128,
            distance_metric: DistanceMetric::Cosine,
            vector_type: VectorType::Float32,
            index_config: IndexConfig::default(),
        };
        
        let op = WALOperation::CreateCollection(config);
        wal.append(&op).await.unwrap();
        
        let operations = wal.read_all().await.unwrap();
        assert_eq!(operations.len(), 1);
        
        match &operations[0] {
            WALOperation::CreateCollection(c) => {
                assert_eq!(c.name, "test");
            }
            _ => panic!("Unexpected operation type"),
        }
    }
}