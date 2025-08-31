# User Guide

## Overview
`pocketvec-sqlite` adds cosine similarity and a virtual table for nearest-neighbor style queries. Distances reported are `1.0 - cosine`, so lower is better.

## Data Model
- Base table: one row per vector, e.g. `docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL)`.
- `vec` stores a contiguous float32 array as raw bytes (`4 * dim` bytes).

## Scalar Functions
- `pocketvec_version()` → TEXT: extension version string.
- `pocketvec_cosine(a BLOB, b BLOB)` → REAL: cosine similarity of two float32 blobs (same length).
- `pocketvec_set_q(q BLOB)` → INT: set the thread-local query vector used by the vtab.

## Virtual Table `pocketvec`
Create a search view over your base table:
```
CREATE VIRTUAL TABLE ann
USING pocketvec(base_table=docs_vecs, dim=128);
-- If dim is omitted and base_table has at least one row, it is inferred from vec blob length.
```
Query with a previously set query vector:
```
SELECT pocketvec_set_q(?);                 -- bind a float32 blob of length 4*dim
SELECT rowid, distance FROM ann
ORDER BY distance
LIMIT 10;
```

## Python Example
```python
import sqlite3, numpy as np, os
con = sqlite3.connect(":memory:")
con.enable_load_extension(True)
con.load_extension(os.path.abspath("pocketvec-sqlite/target/release/libpocketvec0"))
con.execute("CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL)")
# make 128D unit vectors
rng = np.random.default_rng(0)
vecs = rng.normal(size=(1000,128)).astype(np.float32)
vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)
con.executemany("INSERT INTO docs_vecs(rowid, vec) VALUES(?, ?)",
                [(i+1, v.tobytes()) for i,v in enumerate(vecs)])
con.execute("CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128)")
q = rng.normal(size=(128,)).astype(np.float32); q /= np.linalg.norm(q)
con.execute("SELECT pocketvec_set_q(?)", (q.tobytes(),))
rows = con.execute("SELECT rowid, distance FROM ann ORDER BY distance LIMIT 5").fetchall()
```

## Notes & Limits
- Full scan: the vtab linearly scans `base_table` and sorts by distance.
- Only float32 cosine is supported; ensure blob size equals `4 * dim`.
- One query vector per thread via `pocketvec_set_q`.
- Enable logs with `RUST_LOG=debug` for troubleshooting.
