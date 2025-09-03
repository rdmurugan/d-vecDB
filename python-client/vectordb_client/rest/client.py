"""
Synchronous REST API client for d-vecDB.
"""

import json
from typing import List, Optional, Dict, Any
from urllib.parse import urljoin
import httpx

from ..types import (
    CollectionConfig, Vector, QueryResult, SearchRequest, SearchResponse,
    CollectionStats, ServerStats, HealthResponse, InsertResponse,
    ListCollectionsResponse, CollectionResponse, VectorData
)
from ..exceptions import (
    VectorDBError, ConnectionError, CollectionNotFoundError, 
    VectorNotFoundError, create_exception_from_response
)


class RestClient:
    """Synchronous REST API client for d-vecDB."""
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 8080,
        ssl: bool = False,
        timeout: float = 30.0,
        retries: int = 3,
        headers: Optional[Dict[str, str]] = None,
        auth: Optional[httpx.Auth] = None
    ):
        """
        Initialize REST client.
        
        Args:
            host: Server hostname or IP address
            port: Server port number
            ssl: Use HTTPS if True, HTTP if False
            timeout: Request timeout in seconds
            retries: Number of retry attempts
            headers: Additional HTTP headers
            auth: HTTP authentication
        """
        self.host = host
        self.port = port
        self.ssl = ssl
        self.timeout = timeout
        self.retries = retries
        
        # Build base URL
        protocol = "https" if ssl else "http"
        self.base_url = f"{protocol}://{host}:{port}"
        
        # Setup HTTP client
        default_headers = {
            "Content-Type": "application/json",
            "Accept": "application/json",
            "User-Agent": "vectordb-client-python/0.1.0"
        }
        if headers:
            default_headers.update(headers)
            
        self.client = httpx.Client(
            base_url=self.base_url,
            headers=default_headers,
            timeout=timeout,
            auth=auth,
            follow_redirects=True
        )
        
        # Client-side ID mapping: user_id -> (server_id, vector_data, metadata)
        self._id_mapping: Dict[str, tuple] = {}
    
    def __enter__(self):
        """Context manager entry."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
    
    def close(self):
        """Close the HTTP client."""
        if hasattr(self, 'client'):
            self.client.close()
    
    def _make_request(
        self, 
        method: str, 
        endpoint: str, 
        json_data: Optional[Dict[str, Any]] = None,
        params: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """Make an HTTP request with error handling."""
        try:
            response = self.client.request(
                method=method,
                url=endpoint,
                json=json_data,
                params=params
            )
            
            # Check for HTTP errors
            if response.status_code >= 400:
                try:
                    error_data = response.json()
                    message = error_data.get("message", f"HTTP {response.status_code}")
                    details = error_data.get("details", {})
                except (json.JSONDecodeError, KeyError):
                    message = f"HTTP {response.status_code}: {response.text}"
                    details = {}
                
                raise create_exception_from_response(
                    response.status_code, message, details
                )
            
            # Parse JSON response
            try:
                return response.json()
            except json.JSONDecodeError as e:
                raise VectorDBError(f"Invalid JSON response: {e}")
                
        except httpx.RequestError as e:
            raise ConnectionError(f"Request failed: {e}")
        except httpx.TimeoutException:
            raise VectorDBError("Request timed out")
    
    # Collection Management
    def create_collection(self, config: CollectionConfig) -> CollectionResponse:
        """Create a new vector collection."""
        response_data = self._make_request(
            "POST", 
            "/collections",
            json_data=config.model_dump()
        )
        return CollectionResponse(**response_data)
    
    def delete_collection(self, name: str) -> CollectionResponse:
        """Delete a vector collection."""
        response_data = self._make_request("DELETE", f"/collections/{name}")
        return CollectionResponse(**response_data)
    
    def get_collection(self, name: str) -> CollectionResponse:
        """Get collection information."""
        response_data = self._make_request("GET", f"/collections/{name}")
        return CollectionResponse(**response_data)
    
    def list_collections(self) -> ListCollectionsResponse:
        """List all collections."""
        response_data = self._make_request("GET", "/collections")
        return ListCollectionsResponse(**response_data)
    
    def get_collection_stats(self, name: str) -> CollectionStats:
        """Get collection statistics."""
        response_data = self._make_request("GET", f"/collections/{name}/stats")
        return CollectionStats(**response_data["data"])
    
    # Vector Operations
    def insert_vector(self, collection_name: str, vector: Vector) -> InsertResponse:
        """Insert a single vector."""
        # Server expects {"data": [...]} and generates its own ID
        request_data = {"data": vector.data}
        if vector.metadata:
            request_data["metadata"] = vector.metadata
            
        response_data = self._make_request(
            "POST",
            f"/collections/{collection_name}/vectors",
            json_data=request_data
        )
        
        # Create response with the server-generated ID
        result = InsertResponse(**response_data)
        if response_data.get("success") and response_data.get("data"):
            # The server returns the generated ID in the data field
            server_id = response_data["data"]
            result.generated_id = server_id
            
            # Store mapping: user_id -> (server_id, vector_data, metadata)
            self._id_mapping[vector.id] = (server_id, vector.data, vector.metadata)
        
        return result
    
    def insert_vectors(self, collection_name: str, vectors: List[Vector]) -> InsertResponse:
        """Insert multiple vectors."""
        # Server expects {"vectors": [{"data": [...]}, ...]} format
        vector_data = []
        for v in vectors:
            vec_data = {"data": v.data}
            if v.metadata:
                vec_data["metadata"] = v.metadata
            vector_data.append(vec_data)
            
        response_data = self._make_request(
            "POST",
            f"/collections/{collection_name}/vectors/batch",
            json_data={"vectors": vector_data}
        )
        
        # Server returns array of generated IDs in data field
        result = InsertResponse(**response_data)
        if response_data.get("success") and isinstance(response_data.get("data"), list):
            server_ids = response_data["data"]
            result.generated_id = server_ids  # Array of IDs
            
            # Store mappings for each vector: user_id -> (server_id, vector_data, metadata)
            for i, vector in enumerate(vectors):
                if i < len(server_ids):
                    server_id = server_ids[i]
                    self._id_mapping[vector.id] = (server_id, vector.data, vector.metadata)
        
        return result
    
    def get_vector(self, collection_name: str, vector_id: str) -> Vector:
        """Retrieve a vector by ID."""
        # First check if this is a user-provided ID in our mapping
        if vector_id in self._id_mapping:
            server_id, vector_data, metadata = self._id_mapping[vector_id]
            return Vector(id=vector_id, data=vector_data, metadata=metadata)
        
        # If not in mapping, try direct server retrieval (assuming it's a server ID)
        response_data = self._make_request(
            "GET", 
            f"/collections/{collection_name}/vectors/{vector_id}"
        )
        # Server returns data in "data" field, not "vector" field
        if response_data.get("data"):
            return Vector(**response_data["data"])
        else:
            # If server doesn't return vector data, we need to handle this case
            from ..exceptions import VectorNotFoundError
            raise VectorNotFoundError(f"Vector {vector_id} not found or server doesn't return vector data")
    
    def update_vector(self, collection_name: str, vector: Vector) -> InsertResponse:
        """Update an existing vector."""
        response_data = self._make_request(
            "PUT",
            f"/collections/{collection_name}/vectors/{vector.id}",
            json_data=vector.model_dump()
        )
        return InsertResponse(**response_data)
    
    def delete_vector(self, collection_name: str, vector_id: str) -> InsertResponse:
        """Delete a vector by ID."""
        response_data = self._make_request(
            "DELETE",
            f"/collections/{collection_name}/vectors/{vector_id}"
        )
        return InsertResponse(**response_data)
    
    # Search Operations  
    def search(
        self, 
        collection_name: str,
        query_vector: VectorData,
        limit: int = 10,
        ef_search: Optional[int] = None,
        filter: Optional[Dict[str, Any]] = None
    ) -> SearchResponse:
        """Search for similar vectors."""
        if hasattr(query_vector, 'tolist'):
            query_vector = query_vector.tolist()
        
        # Server expects {"vector": [...], "limit": N}
        search_data = {
            "vector": query_vector,
            "limit": limit
        }
        if ef_search is not None:
            search_data["ef_search"] = ef_search
        if filter is not None:
            search_data["filter"] = filter
        
        response_data = self._make_request(
            "POST",
            f"/collections/{collection_name}/search",
            json_data=search_data
        )
        
        # Convert response data to QueryResult list
        results = [QueryResult(**item) for item in response_data.get("data", [])]
        return SearchResponse(
            success=response_data["success"],
            data=results,
            error=response_data.get("error")
        )
    
    # Server Operations
    def get_server_stats(self) -> ServerStats:
        """Get server statistics."""
        response_data = self._make_request("GET", "/stats")
        return ServerStats(**response_data["data"])
    
    def health_check(self) -> HealthResponse:
        """Check server health."""
        response_data = self._make_request("GET", "/health")
        return HealthResponse(**response_data)
    
    # Convenience methods
    def ping(self) -> bool:
        """Check if server is reachable."""
        try:
            self.health_check()
            return True
        except VectorDBError:
            return False
    
    def create_collection_simple(
        self,
        name: str,
        dimension: int,
        distance_metric: str = "cosine"
    ) -> CollectionResponse:
        """Create a collection with minimal configuration."""
        from ..types import DistanceMetric
        
        # Map lowercase to proper enum values
        metric_mapping = {
            "cosine": DistanceMetric.COSINE,
            "euclidean": DistanceMetric.EUCLIDEAN,
            "dot_product": DistanceMetric.DOT_PRODUCT,
            "manhattan": DistanceMetric.MANHATTAN
        }
        
        if distance_metric.lower() in metric_mapping:
            metric = metric_mapping[distance_metric.lower()]
        else:
            try:
                metric = DistanceMetric(distance_metric)
            except ValueError:
                raise ValueError(f"Invalid distance metric: {distance_metric}. Valid options: {list(metric_mapping.keys())}")
        
        config = CollectionConfig(
            name=name,
            dimension=dimension,
            distance_metric=metric
        )
        return self.create_collection(config)
    
    def insert_simple(
        self,
        collection_name: str,
        vector_id: str,
        vector_data: VectorData,
        metadata: Optional[Dict[str, Any]] = None
    ) -> InsertResponse:
        """Insert a vector with simple parameters."""
        if hasattr(vector_data, 'tolist'):
            vector_data = vector_data.tolist()
            
        vector = Vector(id=vector_id, data=vector_data, metadata=metadata)
        return self.insert_vector(collection_name, vector)
    
    def search_simple(
        self,
        collection_name: str,
        query_vector: VectorData,
        limit: int = 10
    ) -> List[QueryResult]:
        """Simple vector search returning just results."""
        response = self.search(collection_name, query_vector, limit)
        return response.results