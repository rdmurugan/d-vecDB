# pocketvec-sqlite

Pocket-sized SQLite vector database extension for local embeddings and lightweight ANN search. Runs fully in-process: no servers, minimal overhead.

## Features
- Native SQLite extension (`.dylib`/`.so`) — load via `.load` or Python `sqlite3`.
- Scalars: `pocketvec_cosine(a,b)` for float32 blobs; `pocketvec_version()`.
- Virtual table `pocketvec`: lightweight ANN via linear scan over a base table; returns `(rowid, distance)` with `distance = 1 - cosine`.

## Quick Start
Prereqs: Rust toolchain, SQLite CLI, Python 3 (optional).

1) Build
```
cd pocketvec-sqlite
cargo build --release
```
Artifacts: `target/release/libpocketvec0.dylib` (macOS) or `.so` (Linux).

2) SQLite CLI
```
sqlite3 :memory:
.load ./target/release/libpocketvec0
SELECT pocketvec_version();
```

3) SQL usage
```
CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL);
CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128);
SELECT pocketvec_set_q(?);  -- bind a float32 blob (4*dim bytes)
SELECT rowid, distance FROM ann ORDER BY distance LIMIT 10;
```

Python demo: `python pocketvec-sqlite/python/demo.py` (local embeddings example)

## Docs
- Installation: `INSTALL.md`
- User Guide (SQL/Python examples): `USER_GUIDE.md`
- Contributing: `AGENTS.md`

## Get Involved
- Issues: use the templates under `.github/ISSUE_TEMPLATE/`:
  - Roadmap — IVF-PQ indexing: `.github/ISSUE_TEMPLATE/roadmap-ivf-pq-indexing.md`
  - Integration with SQLite FTS5: `.github/ISSUE_TEMPLATE/integration-with-sqlite-fts5.md`
  - Python API improvements: `.github/ISSUE_TEMPLATE/python-api-improvements.md`
- Discussions: propose ideas, ask questions, and share results via GitHub Discussions (if enabled) or open an issue using the templates above.
