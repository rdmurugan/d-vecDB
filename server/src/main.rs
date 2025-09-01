use vectordb_server::{VectorDbServer, ServerConfig};
use clap::{Arg, Command};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("vectordb-server")
        .version("0.1.0")
        .about("High-performance standalone vector database server")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("DIR")
                .help("Data directory path")
                .default_value("./data")
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Server host address")
                .default_value("127.0.0.1")
        )
        .arg(
            Arg::new("grpc-port")
                .long("grpc-port")
                .value_name("PORT")
                .help("gRPC server port")
                .default_value("9090")
        )
        .arg(
            Arg::new("rest-port")
                .long("rest-port")
                .value_name("PORT")
                .help("REST server port")
                .default_value("8080")
        )
        .arg(
            Arg::new("metrics-port")
                .long("metrics-port")
                .value_name("PORT")
                .help("Metrics server port")
                .default_value("9091")
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (trace, debug, info, warn, error)")
                .default_value("info")
        )
        .get_matches();

    // Load configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        ServerConfig::from_file(config_path)?
    } else {
        let mut config = ServerConfig::default();
        
        // Override with command line arguments
        if let Some(data_dir) = matches.get_one::<String>("data-dir") {
            config.data_dir = data_dir.into();
        }
        if let Some(host) = matches.get_one::<String>("host") {
            config.host = host.clone();
        }
        if let Some(grpc_port) = matches.get_one::<String>("grpc-port") {
            config.grpc_port = grpc_port.parse()?;
        }
        if let Some(rest_port) = matches.get_one::<String>("rest-port") {
            config.rest_port = rest_port.parse()?;
        }
        if let Some(metrics_port) = matches.get_one::<String>("metrics-port") {
            config.metrics_port = metrics_port.parse()?;
        }
        if let Some(log_level) = matches.get_one::<String>("log-level") {
            config.log_level = log_level.clone();
        }
        
        config
    };

    // Validate configuration
    config.validate()?;

    // Initialize logging
    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    format!("vectordb_server={},vectordb_vectorstore={},vectordb_storage={},vectordb_index={}", 
                            log_level, log_level, log_level, log_level).into()
                })
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize metrics
    vectordb_server::init_metrics();

    // Print startup banner
    print_banner(&config);

    // Create and start server
    match VectorDbServer::new(config).await {
        Ok(server) => {
            info!("Server initialized successfully");
            
            // Handle graceful shutdown
            let shutdown_signal = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for ctrl-c signal");
                info!("Shutdown signal received");
            };
            
            tokio::select! {
                result = server.start() => {
                    if let Err(e) = result {
                        error!("Server error: {}", e);
                        std::process::exit(1);
                    }
                }
                _ = shutdown_signal => {
                    info!("Graceful shutdown initiated");
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize server: {}", e);
            std::process::exit(1);
        }
    }

    info!("Server shutdown complete");
    Ok(())
}

fn print_banner(config: &ServerConfig) {
    println!(r#"
 _    _           _             ____  ____        ____   _____ 
| |  | |         | |           |  _ \|  _ \      |  _ \ / ____|
| |  | | ___  ___| |_ ___  _ __| | | | |_) |_____| |_) | (___  
| |  | |/ _ \/ __| __/ _ \| '__|  | | |  _ <______|  _ < \___ \ 
| |__| |  __/ (__| || (_) | |  | |_| | |_) |     | |_) |____) |
 \____/ \___|\___|\__\___/|_|  |____/|____/      |____/|_____/ 
                                                               
    Production-Ready Standalone Vector Database
    Version: 0.1.0
    
    Configuration:
    - Data Directory: {}
    - gRPC Server:    {}:{}
    - REST API:       {}:{}
    - Metrics:        {}:{}
    - Log Level:      {}
    
    Starting server...
"#, 
        config.data_dir.display(),
        config.host, config.grpc_port,
        config.host, config.rest_port,
        config.host, config.metrics_port,
        config.log_level
    );
}