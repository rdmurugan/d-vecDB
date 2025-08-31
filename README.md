x# pocketvec-sqlite

**Pocket-sized vector search inside SQLite** — a Rust extension that turns **SQLite** into a lightweight **vector database** for local embeddings, with Python bindings.

Unlike heavyweight vector DBs, `pocketvec-sqlite` runs entirely in-process: no servers, no extra dependencies, minimal memory overhead. Perfect for **edge, desktop, or cost-sensitive environments** where running Milvus, Pinecone, or Weaviate is overkill.

---

## ✨ Features

* **Native SQLite extension** (`.dylib`/`.so`): load with `.load` or via `sqlite3` Python driver.
* **Vector functions**: cosine similarity (`pocketvec_cosine`), normalize in-place, etc.
* **ANN Virtual Table**: query embeddings with `SELECT rowid, distance FROM ann …`.
* **Local-first**: no network or external service required.
* **Drop-in Python support** via `sqlite3` module.

---

## 🚀 Quickstart

### 1. Build the extension

```bash
git clone https://github.com/YOUR-USERNAME/pocketvec-sqlite.git
cd pocketvec-sqlite
cargo build --release
```

Artifacts will be under `target/release/`:

* `libpocketvec0.dylib` (macOS)
* `libpocketvec0.so` (Linux)

---

### 2. Python demo

```python
import sqlite3, os, numpy as np

# Path to extension
EXT = os.path.abspath("target/release/libpocketvec0.dylib")

def f32_blob(x: np.ndarray) -> bytes:
    x = np.asarray(x, dtype=np.float32, order="C")
    return x.tobytes()

con = sqlite3.connect("demo.db")
con.enable_load_extension(True)
con.load_extension(EXT)

# Base table
con.execute("CREATE TABLE IF NOT EXISTS docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL)")

# Insert random vectors
D, N = 128, 1000
vecs = np.random.randn(N, D).astype(np.float32)
vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
con.executemany("INSERT INTO docs_vecs(rowid, vec) VALUES(?, ?)", [(i+1, f32_blob(v)) for i,v in enumerate(vecs)])

# Create ANN vtab
con.execute("CREATE VIRTUAL TABLE IF NOT EXISTS ann USING pocketvec(base_table=docs_vecs, dim=128)")

# Query
q = np.random.randn(D).astype(np.float32)
q /= np.linalg.norm(q)
con.execute("SELECT pocketvec_set_q(?)", (f32_blob(q),))
rows = con.execute("SELECT rowid, distance FROM ann ORDER BY distance LIMIT 5").fetchall()
print(rows)
```

Expected: Top-5 rowids with cosine distances.

---

## 🔍 Why pocketvec-sqlite?

* **Zero infra cost**: ships as a single shared library, no server.
* **Perfect for notebooks, mobile, and embedded use**.
* **SQLite ecosystem**: works with FTS5, JSON1, and any existing SQL queries.
* **Hackable**: written in Rust, contributions welcome.

---

## 🚣️ Roadmap

* [ ] IVF-PQ indexing for sub-linear ANN search.
* [ ] Heap-based top-K (no full sort).
* [ ] Hybrid queries with FTS5.
* [ ] SIMD optimizations.
* [ ] Benchmarks at 100k+ vectors.

---

## 🤝 Contributing

We welcome contributions — code, docs, issues, or benchmarks.

* Open a PR or issue on [GitHub](https://github.com/YOUR-USERNAME/pocketvec-sqlite).
* Contact maintainer: **Durai Rajamanickam** ([durai@infinidatum.com](mailto:durai@infinidatum.com)).

If you’re into Rust, IR/ANN, or embedded AI, this is a great way to shape the **next generation of lightweight vector search**.

---

## 📣 Community

* Join discussions via GitHub Issues / PRs.
* Share use cases (edge AI, mobile RAG, local LLM fine-tuning).
* Cite this repo in blogs or benchmarks: *“SQLite as a vector DB with pocketvec-sqlite.”*

---

## 📜 License

MIT License © 2025 Durai Rajamanickam
