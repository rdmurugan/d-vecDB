use std::net::SocketAddr;
use metrics_exporter_prometheus::PrometheusBuilder;
use axum::{routing::get, Router, response::Html};
use tracing::info;

/// Start the metrics server
pub async fn start_metrics_server(addr: SocketAddr) -> anyhow::Result<()> {
    info!("Starting metrics server on {}", addr);
    
    // Install Prometheus metrics exporter
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()?;
    
    // Create metrics endpoint
    let app = Router::new()
        .route("/metrics", get(move || async move {
            prometheus_handle.render()
        }))
        .route("/", get(metrics_index));
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Metrics index page
async fn metrics_index() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>VectorDB Metrics</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; }
            h1 { color: #333; }
            .metric-link { 
                display: inline-block; 
                padding: 10px 20px; 
                background: #007bff; 
                color: white; 
                text-decoration: none; 
                border-radius: 5px; 
                margin: 10px 0;
            }
            .metric-link:hover { background: #0056b3; }
            .description { margin: 20px 0; color: #666; }
        </style>
    </head>
    <body>
        <h1>VectorDB Metrics</h1>
        <div class="description">
            <p>VectorDB metrics are exposed in Prometheus format.</p>
        </div>
        <a href="/metrics" class="metric-link">View Metrics</a>
        
        <h2>Available Metrics</h2>
        <ul>
            <li><strong>vectorstore_collections_created_total</strong> - Total collections created</li>
            <li><strong>vectorstore_collections_deleted_total</strong> - Total collections deleted</li>
            <li><strong>vectorstore_vectors_inserted_total</strong> - Total vectors inserted</li>
            <li><strong>vectorstore_vectors_batch_inserted_total</strong> - Total vectors batch inserted</li>
            <li><strong>vectorstore_queries_total</strong> - Total queries executed</li>
            <li><strong>vectorstore_vectors_deleted_total</strong> - Total vectors deleted</li>
            <li><strong>vectorstore_vectors_updated_total</strong> - Total vectors updated</li>
            <li><strong>vectorstore_insert_duration_seconds</strong> - Vector insertion duration</li>
            <li><strong>vectorstore_batch_insert_duration_seconds</strong> - Batch insertion duration</li>
            <li><strong>vectorstore_query_duration_seconds</strong> - Query duration</li>
            <li><strong>vectorstore_query_results</strong> - Number of results per query</li>
            <li><strong>vectorstore_collections_total</strong> - Total number of collections</li>
            <li><strong>vectorstore_vectors_total</strong> - Total number of vectors</li>
            <li><strong>vectorstore_memory_usage</strong> - Memory usage in bytes</li>
        </ul>
        
        <h2>Usage</h2>
        <p>Configure Prometheus to scrape metrics from this endpoint:</p>
        <pre style="background: #f4f4f4; padding: 10px; border-radius: 5px;">
scrape_configs:
  - job_name: 'vectordb'
    static_configs:
      - targets: ['localhost:9091']
        </pre>
    </body>
    </html>
    "#)
}

/// Initialize metrics
pub fn init_metrics() {
    // Initialize metrics that we want to track
    metrics::describe_counter!(
        "vectorstore.collections.created",
        "Number of collections created"
    );
    metrics::describe_counter!(
        "vectorstore.collections.deleted", 
        "Number of collections deleted"
    );
    metrics::describe_counter!(
        "vectorstore.vectors.inserted",
        "Number of vectors inserted"
    );
    metrics::describe_counter!(
        "vectorstore.vectors.batch_inserted",
        "Number of vectors batch inserted"
    );
    metrics::describe_counter!(
        "vectorstore.queries",
        "Number of queries executed"
    );
    metrics::describe_counter!(
        "vectorstore.vectors.deleted",
        "Number of vectors deleted"
    );
    metrics::describe_counter!(
        "vectorstore.vectors.updated",
        "Number of vectors updated"
    );
    
    metrics::describe_histogram!(
        "vectorstore.insert.duration",
        "Vector insertion duration"
    );
    metrics::describe_histogram!(
        "vectorstore.batch_insert.duration",
        "Batch insertion duration"
    );
    metrics::describe_histogram!(
        "vectorstore.query.duration",
        "Query duration"
    );
    metrics::describe_histogram!(
        "vectorstore.query.results",
        "Number of results per query"
    );
    
    metrics::describe_gauge!(
        "vectorstore.collections.total",
        "Total number of collections"
    );
    metrics::describe_gauge!(
        "vectorstore.vectors.total",
        "Total number of vectors"
    );
    metrics::describe_gauge!(
        "vectorstore.memory.usage",
        "Memory usage in bytes"
    );
}