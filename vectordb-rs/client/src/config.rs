use serde::{Deserialize, Serialize};

/// Client protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientProtocol {
    Grpc,
    Rest,
}

/// Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Protocol to use (gRPC or REST)
    pub protocol: ClientProtocol,
    
    /// Server endpoint
    pub endpoint: String,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// Maximum number of retries
    pub max_retries: u32,
    
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Connection pool size (gRPC only)
    pub connection_pool_size: usize,
    
    /// User agent string
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            protocol: ClientProtocol::Grpc,
            endpoint: "http://localhost:9090".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_compression: true,
            connection_pool_size: 10,
            user_agent: format!("vectordb-client/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl ClientConfig {
    /// Create gRPC client configuration
    pub fn grpc<S: Into<String>>(endpoint: S) -> Self {
        Self {
            protocol: ClientProtocol::Grpc,
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }
    
    /// Create REST client configuration
    pub fn rest<S: Into<String>>(endpoint: S) -> Self {
        Self {
            protocol: ClientProtocol::Rest,
            endpoint: endpoint.into(),
            ..Default::default()
        }
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }
    
    /// Set max retries
    pub fn with_retries(mut self, max_retries: u32, delay_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_delay_ms = delay_ms;
        self
    }
    
    /// Enable/disable compression
    pub fn with_compression(mut self, enable: bool) -> Self {
        self.enable_compression = enable;
        self
    }
    
    /// Set connection pool size
    pub fn with_pool_size(mut self, size: usize) -> Self {
        self.connection_pool_size = size;
        self
    }
    
    /// Set user agent
    pub fn with_user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }
    
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.endpoint.is_empty() {
            return Err(anyhow::anyhow!("Endpoint cannot be empty"));
        }
        
        if self.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Timeout must be greater than 0"));
        }
        
        if self.connection_pool_size == 0 {
            return Err(anyhow::anyhow!("Connection pool size must be greater than 0"));
        }
        
        // Validate endpoint format based on protocol
        match self.protocol {
            ClientProtocol::Grpc => {
                if !self.endpoint.starts_with("http://") && !self.endpoint.starts_with("https://") {
                    return Err(anyhow::anyhow!("gRPC endpoint must start with http:// or https://"));
                }
            }
            ClientProtocol::Rest => {
                if !self.endpoint.starts_with("http://") && !self.endpoint.starts_with("https://") {
                    return Err(anyhow::anyhow!("REST endpoint must start with http:// or https://"));
                }
            }
        }
        
        Ok(())
    }
}