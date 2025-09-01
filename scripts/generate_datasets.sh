#!/bin/bash

# Dataset Generation Script for VectorDB-RS Benchmarking
# Creates test datasets of various sizes for performance testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_ROOT/benchmark_data"

echo "Generating benchmark datasets..."
mkdir -p "$DATA_DIR"

# Function to generate a dataset
generate_dataset() {
    local size=$1
    local dimensions=$2
    local filename=$3
    
    echo "Generating $filename ($size vectors, $dimensions dimensions)"
    
    python3 << EOF
import json
import random
import math

def generate_vector(dim):
    # Generate random vector and normalize
    vector = [random.gauss(0, 1) for _ in range(dim)]
    magnitude = math.sqrt(sum(x*x for x in vector))
    return [x/magnitude for x in vector] if magnitude > 0 else vector

def generate_dataset(size, dim):
    vectors = []
    for i in range(size):
        vector_data = generate_vector(dim)
        vectors.append({
            "id": f"vec_{i:06d}",
            "vector": vector_data,
            "metadata": {
                "index": i,
                "category": f"cat_{i % 10}",
                "timestamp": i * 1000
            }
        })
    return vectors

# Generate dataset
dataset = generate_dataset($size, $dimensions)

# Write to file
with open('$DATA_DIR/$filename', 'w') as f:
    json.dump({
        "metadata": {
            "size": $size,
            "dimensions": $dimensions,
            "data_type": "synthetic_normalized",
            "distribution": "gaussian"
        },
        "vectors": dataset
    }, f, indent=2)

print(f"Generated $filename with $size vectors")
EOF
}

# Generate datasets of different sizes
generate_dataset 1000 128 "dataset_1k.json"
generate_dataset 10000 128 "dataset_10k.json" 
generate_dataset 100000 128 "dataset_100k.json"

# Generate datasets with different dimensions
generate_dataset 10000 64 "dataset_10k_64d.json"
generate_dataset 10000 256 "dataset_10k_256d.json"
generate_dataset 10000 512 "dataset_10k_512d.json"

# Generate clustered dataset (more realistic)
echo "Generating clustered dataset..."
python3 << 'EOF'
import json
import random
import math
import numpy as np

def generate_clustered_dataset(size, dim, num_clusters=50):
    # Generate cluster centers
    centers = []
    for _ in range(num_clusters):
        center = [random.gauss(0, 2) for _ in range(dim)]
        magnitude = math.sqrt(sum(x*x for x in center))
        centers.append([x/magnitude for x in center] if magnitude > 0 else center)
    
    vectors = []
    for i in range(size):
        # Choose a random cluster
        cluster_idx = random.randint(0, num_clusters - 1)
        center = centers[cluster_idx]
        
        # Generate vector near cluster center
        noise_scale = 0.3
        vector = [c + random.gauss(0, noise_scale) for c in center]
        magnitude = math.sqrt(sum(x*x for x in vector))
        vector = [x/magnitude for x in vector] if magnitude > 0 else vector
        
        vectors.append({
            "id": f"vec_{i:06d}",
            "vector": vector,
            "metadata": {
                "index": i,
                "cluster": cluster_idx,
                "category": f"cluster_{cluster_idx}"
            }
        })
    
    return vectors

# Generate clustered dataset
dataset = generate_clustered_dataset(50000, 128)

with open('benchmark_data/dataset_50k_clustered.json', 'w') as f:
    json.dump({
        "metadata": {
            "size": 50000,
            "dimensions": 128,
            "data_type": "synthetic_clustered",
            "distribution": "gaussian_clusters",
            "num_clusters": 50
        },
        "vectors": dataset
    }, f, indent=2)

print("Generated clustered dataset with 50k vectors")
EOF

# Generate pathological dataset (all vectors very similar)
echo "Generating pathological dataset..."
python3 << 'EOF'
import json
import random
import math

def generate_pathological_dataset(size, dim):
    # Base vector
    base = [1.0] + [0.0] * (dim - 1)
    
    vectors = []
    for i in range(size):
        # Add tiny random noise
        noise_scale = 0.001
        vector = [base[j] + random.gauss(0, noise_scale) for j in range(dim)]
        magnitude = math.sqrt(sum(x*x for x in vector))
        vector = [x/magnitude for x in vector] if magnitude > 0 else vector
        
        vectors.append({
            "id": f"vec_{i:06d}",
            "vector": vector,
            "metadata": {
                "index": i,
                "similarity_to_base": sum(base[j] * vector[j] for j in range(dim))
            }
        })
    
    return vectors

dataset = generate_pathological_dataset(10000, 128)

with open('benchmark_data/dataset_10k_pathological.json', 'w') as f:
    json.dump({
        "metadata": {
            "size": 10000,
            "dimensions": 128,
            "data_type": "synthetic_pathological",
            "distribution": "nearly_identical",
            "description": "All vectors are very similar to test worst-case performance"
        },
        "vectors": dataset
    }, f, indent=2)

print("Generated pathological dataset")
EOF

echo "Dataset generation complete!"
echo "Generated datasets:"
ls -la "$DATA_DIR"/*.json

echo ""
echo "To use these datasets in benchmarks:"
echo "  ./scripts/run_benchmarks.sh"
echo "  ./scripts/compare_baseline.sh"