#!/usr/bin/env python3
"""
Example usage of the d-vecDB server Python package
Demonstrates basic server management and usage with the Python client
"""

import numpy as np
from d_vecdb_server import DVecDBServer

def main():
    print("🚀 d-vecDB Server Example")
    print("=" * 40)
    
    # Start the server using context manager
    with DVecDBServer(port=8080) as server:
        print(f"✅ Server started on port {server.port}")
        print(f"   REST API: http://{server.host}:{server.port}")
        print(f"   gRPC API: {server.host}:{server.grpc_port}")
        
        # Optional: Use with Python client if available
        try:
            from d_vecdb import VectorDBClient
            
            print("\n📡 Connecting with Python client...")
            client = VectorDBClient(host=server.host, port=server.port)
            
            print("📦 Creating collection...")
            client.create_collection_simple("example", 128, "cosine")
            
            print("📝 Inserting vectors...")
            for i in range(5):
                vector = np.random.random(128).astype(np.float32)
                client.insert_simple("example", f"doc_{i}", vector)
            
            print("🔍 Searching vectors...")
            query = np.random.random(128).astype(np.float32)
            results = client.search_simple("example", query, limit=3)
            
            print(f"✅ Found {len(results)} similar vectors")
            for i, result in enumerate(results):
                print(f"   {i+1}. ID: {result.id}, Score: {result.score:.4f}")
                
        except ImportError:
            print("\n💡 To use the Python client, install it with:")
            print("   pip install d-vecdb")
            print("\n🌐 You can also use the REST API directly:")
            print(f"   curl http://localhost:{server.port}/health")
        
        print(f"\n🛑 Server will stop automatically when exiting context")
    
    print("✅ Example completed successfully!")

if __name__ == "__main__":
    main()