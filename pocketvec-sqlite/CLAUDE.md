# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **pocketvec-sqlite**, a SQLite extension written in Rust that provides vector similarity search capabilities using cosine distance. It implements a virtual table interface that enables efficient nearest neighbor queries on float32 vector embeddings stored as BLOBs.

## Build Commands

- **Build release**: `make build` or `cargo build --release`
- **Build debug**: `make debug` or `cargo build`
- **Clean**: `make clean` (removes target/ and demo.db)
- **Python demo**: `make py` (builds release then runs `python3 python/demo.py`)

## Testing

- **End-to-end test**: `python3 tests/test.py` (comprehensive smoke test)
- **SQL smoke test**: Available at `tests/smoke.sql`

The test expects the release build to exist at `target/release/libpocketvec0.dylib` (macOS) or equivalent platform extension.

## Core Architecture

### Extension Entry Point (`lib.rs`)
- Registers 3 scalar functions: `pocketvec_version()`, `pocketvec_cosine(blob, blob)`, `pocketvec_set_q(blob)`
- Registers virtual table module `pocketvec`
- Uses thread-local storage for query vector state via `QVEC` RefCell

### Virtual Table Implementation (`vtab.rs`)
- **PocketVecTable**: Per-table state storing base table name and vector dimension
- **PocketVecCursor**: Query execution state with sorted results by distance
- Argument parsing supports: `base_table=table_name, dim=N` or space-separated format
- Auto-infers dimension from first row if not provided
- Returns schema: `(id INTEGER, distance REAL)` sorted by cosine distance ascending

### Vector Operations (`cosine.rs`)
- `cosine_blob_f32()`: Computes cosine similarity between two float32 BLOB vectors
- Uses little-endian byte order via `byteorder` crate
- Handles vector normalization and edge cases

## Usage Pattern

1. **Load extension**: `con.load_extension("target/release/libpocketvec0")`
2. **Create base table**: Table with `rowid` and `vec BLOB` (float32 arrays as bytes)
3. **Create virtual table**: `CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128)`
4. **Set query vector**: `SELECT pocketvec_set_q(?)`  
5. **Query**: `SELECT rowid, distance FROM ann ORDER BY distance LIMIT k`

## Key Dependencies

- `sqlite-loadable`: SQLite extension framework (alpha version 0.0.6-alpha.6)
- `byteorder`: Little-endian float32 BLOB parsing
- `thiserror`: Error handling

## Important Notes

- Vector BLOBs must be exactly `4 * dimension` bytes (float32 arrays)
- Query vector is stored in thread-local state between `pocketvec_set_q()` and vtab queries
- Virtual table performs full table scan with in-memory sorting by cosine distance
- Distance metric: `1.0 - cosine_similarity` (lower = more similar)

## Known Issues & Workarounds

- **sqlite-loadable VTab State Loss**: There's a bug in sqlite-loadable 0.0.6-alpha.6 where VTab instance fields are corrupted between `connect()` and `open()`. The code includes a workaround using thread-local storage to preserve `base_table`, `dim`, and `db` handle.
- **Argument Parsing**: Virtual table arguments don't follow the typical 3-element skip pattern, so the code dynamically adjusts the parsing logic.