import sqlite3, os, struct, random
import numpy as np

DB = "demo.db"
if os.path.exists(DB): os.remove(DB)

con = sqlite3.connect(DB)
con.enable_load_extension(True)
con.load_extension(os.path.abspath(os.path.join("..", "target", "release", "libpocketvec0")))  # .so/.dylib

con.executescript(
"""
PRAGMA journal_mode=WAL;
CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL);
"""
)

# helpers
def to_blob_f32(x: np.ndarray) -> bytes:
    assert x.dtype == np.float32 and x.flags['C_CONTIGUOUS']
    return x.tobytes()

# ingest random unit vectors
N, D = 5000, 128
vecs = np.random.randn(N, D).astype(np.float32)
vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)

with con:
    con.executemany("INSERT INTO docs_vecs(rowid, vec) VALUES(?, ?)",
                    [(i+1, to_blob_f32(v)) for i, v in enumerate(vecs)])

# Create virtual table after inserting data (for dimension inference)
con.execute("CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128)")

q = np.random.randn(D).astype(np.float32)
q /= np.linalg.norm(q)

# Set query vector (required for vtab)
con.execute("SELECT pocketvec_set_q(?)", (to_blob_f32(q),))

rows = con.execute(
    "SELECT rowid, distance FROM ann ORDER BY distance LIMIT 10"
).fetchall()

print("Top10:", rows)
print(con.execute("SELECT pocketvec_version()").fetchone())