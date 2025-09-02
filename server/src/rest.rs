use vectordb_vectorstore::VectorStore;
use vectordb_common::types::*;
use std::sync::Arc;
use std::collections::HashMap;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error, instrument};
use uuid::Uuid;
use std::net::SocketAddr;

/// REST API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Collection creation request
#[derive(Deserialize, Debug)]
struct CreateCollectionRequest {
    name: String,
    dimension: usize,
    distance_metric: DistanceMetric,
    vector_type: VectorType,
    index_config: Option<IndexConfig>,
}

/// Vector insertion request
#[derive(Deserialize, Debug)]
struct InsertVectorRequest {
    id: Option<String>,
    data: Vec<f32>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Batch vector insertion request
#[derive(Deserialize, Debug)]
struct BatchInsertRequest {
    vectors: Vec<InsertVectorRequest>,
}

/// Query request
#[derive(Deserialize, Debug)]
struct QueryVectorsRequest {
    vector: Vec<f32>,
    limit: Option<usize>,
    ef_search: Option<usize>,
    filter: Option<HashMap<String, serde_json::Value>>,
}

/// Query parameters for search
#[derive(Deserialize, Debug)]
struct QueryParams {
    limit: Option<usize>,
    ef_search: Option<usize>,
}

type AppState = Arc<VectorStore>;

/// Create collection
#[instrument(skip(state))]
async fn create_collection(
    State(state): State<AppState>,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let config = CollectionConfig {
        name: payload.name,
        dimension: payload.dimension,
        distance_metric: payload.distance_metric,
        vector_type: payload.vector_type,
        index_config: payload.index_config.unwrap_or_default(),
    };
    
    match state.create_collection(&config).await {
        Ok(()) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            error!("Failed to create collection: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// List collections
#[instrument(skip(state))]
async fn list_collections(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let collections = state.list_collections();
    Ok(Json(ApiResponse::success(collections)))
}

/// Get collection info
#[instrument(skip(state))]
async fn get_collection_info(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
) -> Result<Json<ApiResponse<(CollectionConfig, CollectionStats)>>, StatusCode> {
    let config = match state.get_collection_config(&collection_name) {
        Ok(Some(config)) => config,
        Ok(None) => return Ok(Json(ApiResponse::error("Collection not found".to_string()))),
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };
    
    let stats = match state.get_collection_stats(&collection_name).await {
        Ok(Some(stats)) => stats,
        Ok(None) => return Ok(Json(ApiResponse::error("Collection not found".to_string()))),
        Err(e) => return Ok(Json(ApiResponse::error(e.to_string()))),
    };
    
    Ok(Json(ApiResponse::success((config, stats))))
}

/// Delete collection
#[instrument(skip(state))]
async fn delete_collection(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.delete_collection(&collection_name).await {
        Ok(()) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            error!("Failed to delete collection: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Insert vector
#[instrument(skip(state))]
async fn insert_vector(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(payload): Json<InsertVectorRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let vector_id = if let Some(id_str) = payload.id {
        Uuid::parse_str(&id_str)
            .map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        Uuid::new_v4()
    };
    
    let vector = Vector {
        id: vector_id,
        data: payload.data,
        metadata: payload.metadata,
    };
    
    match state.insert(&collection_name, &vector).await {
        Ok(()) => Ok(Json(ApiResponse::success(vector_id.to_string()))),
        Err(e) => {
            error!("Failed to insert vector: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Batch insert vectors
#[instrument(skip(state))]
async fn batch_insert_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Json(payload): Json<BatchInsertRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    let mut vectors = Vec::new();
    let mut vector_ids = Vec::new();
    
    for vector_req in payload.vectors {
        let vector_id = if let Some(id_str) = vector_req.id {
            Uuid::parse_str(&id_str)
                .map_err(|_| StatusCode::BAD_REQUEST)?
        } else {
            Uuid::new_v4()
        };
        
        vector_ids.push(vector_id.to_string());
        vectors.push(Vector {
            id: vector_id,
            data: vector_req.data,
            metadata: vector_req.metadata,
        });
    }
    
    match state.batch_insert(&collection_name, &vectors).await {
        Ok(()) => Ok(Json(ApiResponse::success(vector_ids))),
        Err(e) => {
            error!("Failed to batch insert vectors: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Query vectors
#[instrument(skip(state))]
async fn query_vectors(
    State(state): State<AppState>,
    Path(collection_name): Path<String>,
    Query(params): Query<QueryParams>,
    Json(payload): Json<QueryVectorsRequest>,
) -> Result<Json<ApiResponse<Vec<QueryResult>>>, StatusCode> {
    let query_request = QueryRequest {
        collection: collection_name,
        vector: payload.vector,
        limit: payload.limit.or(params.limit).unwrap_or(10),
        ef_search: payload.ef_search.or(params.ef_search),
        filter: payload.filter,
    };
    
    match state.query(&query_request).await {
        Ok(results) => Ok(Json(ApiResponse::success(results))),
        Err(e) => {
            error!("Failed to query vectors: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get vector by ID
#[instrument(skip(state))]
async fn get_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Option<Vector>>>, StatusCode> {
    let uuid = Uuid::parse_str(&vector_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.get(&collection_name, &uuid).await {
        Ok(vector) => Ok(Json(ApiResponse::success(vector))),
        Err(e) => {
            error!("Failed to get vector: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Delete vector
#[instrument(skip(state))]
async fn delete_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let uuid = Uuid::parse_str(&vector_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    match state.delete(&collection_name, &uuid).await {
        Ok(deleted) => Ok(Json(ApiResponse::success(deleted))),
        Err(e) => {
            error!("Failed to delete vector: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Update vector
#[instrument(skip(state))]
async fn update_vector(
    State(state): State<AppState>,
    Path((collection_name, vector_id)): Path<(String, String)>,
    Json(payload): Json<InsertVectorRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let uuid = Uuid::parse_str(&vector_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let vector = Vector {
        id: uuid,
        data: payload.data,
        metadata: payload.metadata,
    };
    
    match state.update(&collection_name, &vector).await {
        Ok(()) => Ok(Json(ApiResponse::success(()))),
        Err(e) => {
            error!("Failed to update vector: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Get server stats
#[instrument(skip(state))]
async fn get_stats(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<vectordb_vectorstore::ServerStats>>, StatusCode> {
    match state.get_server_stats().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            error!("Failed to get server stats: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

/// Health check
#[instrument]
async fn health() -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("OK".to_string())))
}

/// Create REST API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Collection management
        .route("/collections", post(create_collection))
        .route("/collections", get(list_collections))
        .route("/collections/:collection", get(get_collection_info))
        .route("/collections/:collection", delete(delete_collection))
        
        // Vector operations
        .route("/collections/:collection/vectors", post(insert_vector))
        .route("/collections/:collection/vectors/batch", post(batch_insert_vectors))
        .route("/collections/:collection/search", post(query_vectors))
        .route("/collections/:collection/vectors/:vector_id", get(get_vector))
        .route("/collections/:collection/vectors/:vector_id", put(update_vector))
        .route("/collections/:collection/vectors/:vector_id", delete(delete_vector))
        
        // Server operations
        .route("/stats", get(get_stats))
        .route("/health", get(health))
        
        .with_state(state)
}

/// Start the REST server
pub async fn start_rest_server(addr: SocketAddr, store: Arc<VectorStore>) -> anyhow::Result<()> {
    let app = create_router(store);
    
    info!("Starting REST server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}