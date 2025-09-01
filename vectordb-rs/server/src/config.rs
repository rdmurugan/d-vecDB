use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    
    /// gRPC server port
    pub grpc_port: u16,
    
    /// REST server port
    pub rest_port: u16,
    
    /// Metrics server port
    pub metrics_port: u16,
    
    /// Data directory for storage
    pub data_dir: PathBuf,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in seconds
    pub request_timeout: u64,
    
    /// Enable request logging
    pub enable_logging: bool,
    
    /// Log level
    pub log_level: String,
    
    /// Enable CORS for REST API
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            grpc_port: 9090,
            rest_port: 8080,
            metrics_port: 9091,
            data_dir: PathBuf::from("./data"),
            max_connections: 1000,
            request_timeout: 30,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_cors: true,
        }
    }
}

impl ServerConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.grpc_port == self.rest_port || self.grpc_port == self.metrics_port || self.rest_port == self.metrics_port {
            return Err(anyhow::anyhow!("Ports must be unique"));
        }
        
        if self.max_connections == 0 {
            return Err(anyhow::anyhow!("max_connections must be greater than 0"));
        }
        
        if self.request_timeout == 0 {
            return Err(anyhow::anyhow!("request_timeout must be greater than 0"));
        }
        
        // Validate log level
        match self.log_level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => return Err(anyhow::anyhow!("Invalid log level: {}", self.log_level)),
        }
        
        Ok(())
    }
}