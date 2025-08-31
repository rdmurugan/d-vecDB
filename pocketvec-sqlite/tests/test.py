#!/usr/bin/env python3
"""
End-to-end smoke test for pocketvec-sqlite

Usage:
  python3 tests/test_end_to_end.py
"""

import os
import sys
import sqlite3
import time
import numpy as np
from pathlib import Path

# -----------------------------
# Config
# -----------------------------
D = 128           # embedding dimension
N = 5000          # rows to insert
TOPK = 10
DB_PATH = Path("demo.db")

# Path to the compiled extension (.dylib on macOS)
# Adjust if your build dir differs.
REPO_ROOT = Path(__file__).resolve().parents[1]
EXT_PATH = REPO_ROOT / "target" / "release" / "libpocketvec0.dylib"


def f32_blob(x: np.ndarray) -> bytes:
    """numpy float32 array -> raw bytes (row-major)"""
    x = np.asarray(x, dtype=np.float32, order="C")
    return x.tobytes()


def ensure_extension(con: sqlite3.Connection) -> None:
    """Enable and load the pocketvec extension."""
    if not EXT_PATH.exists():
        print(f"[ERROR] Extension not found: {EXT_PATH}")
        print("Did you run `cargo build --release`?")
        sys.exit(1)
    con.enable_load_extension(True)
    try:
        con.load_extension(str(EXT_PATH))  # default entrypoint sqlite3_extension_init
    except sqlite3.OperationalError as e:
        print(f"[ERROR] Failed to load extension: {e}")
        sys.exit(1)

    ver = con.execute("SELECT pocketvec_version()").fetchone()[0]
    print(f"[OK] Extension loaded: {ver}")


def init_schema(con: sqlite3.Connection) -> None:
    con.executescript("""
    PRAGMA journal_mode=WAL;
    CREATE TABLE IF NOT EXISTS docs_vecs(
      rowid INTEGER PRIMARY KEY,
      vec   BLOB NOT NULL
    );
    """)
    print("[OK] Base table ensured (docs_vecs).")


def seed_vectors(con: sqlite3.Connection, n: int, d: int) -> None:
    """Insert n normalized d-dim random vectors if table is empty."""
    cur_count = con.execute("SELECT COUNT(*) FROM docs_vecs").fetchone()[0]
    if cur_count >= n:
        print(f"[SKIP] Table already has {cur_count} rows (>= {n}).")
        return

    rng = np.random.default_rng(42)
    vecs = rng.normal(size=(n, d)).astype(np.float32)
    vecs /= np.linalg.norm(vecs, axis=1, keepdims=True)

    print(f"[..] Inserting {n} vectors of dim {d}...")
    t0 = time.time()
    with con:
        con.executemany(
            "INSERT INTO docs_vecs(rowid, vec) VALUES(?, ?)",
            ((i + 1, f32_blob(v)) for i, v in enumerate(vecs)),
        )
    print(f"[OK] Inserted {n} vectors in {time.time()-t0:.2f}s.")


def create_vtab(con: sqlite3.Connection, d: int) -> None:
    """Create the virtual table. If you prefer inference, omit dim=..."""
    # You can also omit dim if table already has at least one row:
    # con.execute("CREATE VIRTUAL TABLE IF NOT EXISTS ann USING pocketvec(base_table=docs_vecs)")
    sql = f"CREATE VIRTUAL TABLE IF NOT EXISTS ann USING pocketvec(base_table=docs_vecs, dim={d})"
    try:
        con.execute(sql)
    except sqlite3.OperationalError as e:
        print(f"[ERROR] Creating vtab failed: {e}\nSQL: {sql}")
        # Helpful hint if dim parser failed:
        print("Tip: ensure docs_vecs has at least one row; or try the space-separated form:")
        print("CREATE VIRTUAL TABLE ann USING pocketvec(base_table docs_vecs, dim {d});")
        sys.exit(1)
    print("[OK] Virtual table created (ann).")


def query_topk(con: sqlite3.Connection, d: int, k: int) -> list[tuple[int, float]]:
    """Set a random query vector and return top-k (rowid, distance)."""
    q = np.random.randn(d).astype(np.float32)
    q /= np.linalg.norm(q)

    con.execute("SELECT pocketvec_set_q(?)", (f32_blob(q),))

    t0 = time.time()
    rows = con.execute(
        "SELECT rowid, distance FROM ann ORDER BY distance LIMIT ?",
        (k,),
    ).fetchall()
    dt = (time.time() - t0) * 1000.0
    print(f"[OK] Query returned {len(rows)} rows in {dt:.1f} ms.")
    return rows


def main() -> None:
    if DB_PATH.exists():
        DB_PATH.unlink()  # start fresh for a clean run
    con = sqlite3.connect(DB_PATH)

    ensure_extension(con)
    init_schema(con)
    seed_vectors(con, N, D)
    create_vtab(con, D)
    top = query_topk(con, D, TOPK)

    print(f"Top-{TOPK} neighbors (rowid, distance):")
    for rid, dist in top:
        print(f"  id={rid:<6} distance={dist:.6f}")

    # sanity: self-match ≈ 0.0 distance (cosine == 1.0)
    # Insert the last query vector and check distance ~ 0
    # (optional, since the vtab uses distance=1-cosine)
    print("\n[Sanity] Checking a self-match distance ~ 0.0 ...")
    q = np.random.randn(D).astype(np.float32)
    q /= np.linalg.norm(q)
    con.execute("INSERT INTO docs_vecs(rowid, vec) VALUES(?, ?)", (999_999, f32_blob(q)))
    con.execute("SELECT pocketvec_set_q(?)", (f32_blob(q),))
    dself = con.execute("SELECT distance FROM ann WHERE rowid = 999999").fetchone()[0]
    print(f"  self distance = {dself:.8f}  (lower is better; ~0.0 is perfect)")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print("[FATAL]", e)
        sys.exit(1)
