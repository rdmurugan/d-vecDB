# pocketvec-sqlite

Lightweight vector search as a SQLite loadable extension, written in Rust. It provides:
- A cosine-similarity scalar function for float32 blobs.
- A virtual table (`pocketvec`) that scans a base table of vectors and returns neighbors with a distance metric (1 − cosine).

Use it directly from the SQLite shell or Python’s `sqlite3` module. Ideal for small/medium datasets, demos, and embedded scenarios.

## Quick Start

Prereqs: Rust toolchain, `sqlite3` CLI, Python 3 (optional).

1) Build the extension
```
cd pocketvec-sqlite
cargo build --release
```
2) Load in the SQLite shell
```
sqlite3 :memory:
.load ./target/release/libpocketvec0
SELECT pocketvec_version();
```
3) Basic usage (SQL)
```
CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL);
-- vec must be a contiguous float32 array stored as a BLOB
CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128);
-- Set the query vector (bind a float32 blob)
SELECT pocketvec_set_q(?);
SELECT rowid, distance FROM ann ORDER BY distance LIMIT 10;
```

Python demo: `python pocketvec-sqlite/python/demo.py`

## Repository Layout
- `pocketvec-sqlite/` Rust cdylib crate (source, tests, Python demos).
- `AGENTS.md` contributor guidelines.

## Documentation
- Installation: see `INSTALL.md`
- User Guide (SQL/Python examples, tips): see `USER_GUIDE.md`

---
Built with Rust 2021 and `sqlite-loadable`. Contributions welcome!
