from vectordb_client import VectorDBClient
import numpy as np

client = VectorDBClient(host="localhost", port=8080)


client.create_collection_simple("my_vectors", 128, "cosine")


vector_data = np.random.random(128)
client.insert_simple("my_vectors", "vector_1", vector_data)


query_vector = np.random.random(128)
results = client.search_simple("my_vectors", query_vector, limit=5)
print(f"Found {len(results)} similar vectors")

