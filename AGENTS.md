# Repository Guidelines

Rust-based SQLite vector database extension for local embeddings and lightweight ANN search.

## Project Structure & Module Organization
- Root: `pocketvec-sqlite/` Rust crate and assets.
- `Cargo.toml`: cdylib extension `pocketvec0` (Rust 2021).
- `src/`: `lib.rs`, `vtab.rs`, `cosine.rs`, `utils.rs`.
- `python/`: demo script `demo.py`.
- `tests/`: end-to-end `test.py` and `smoke.sql`.
- `makefile`: common tasks. Build artifacts land in `target/`.

## Build, Test, and Development Commands
- Build (release): `make build` or `cargo build --release` → `target/release/libpocketvec0.{dylib|so}`.
- Build (debug): `make debug` or `cargo build`.
- Clean: `make clean`.
- Quick smoke: `python pocketvec-sqlite/test_minimal.py`.
- End-to-end: `python pocketvec-sqlite/tests/test.py`.
- Demo DB: `python pocketvec-sqlite/python/demo.py` (runs a local embeddings example).
- Verbose logs: prefix commands with `RUST_LOG=debug`.

## Coding Style & Naming Conventions
- Rust: rustfmt defaults (4-space indent).
- Files/modules: snake_case; keep modules small and focused.
- Public APIs: descriptive names like `pocketvec_set_q`, `pocketvec_cosine`.
- FFI boundaries explicit; add minimal doc comments for SQLite-exposed functions.

## Testing Guidelines
- Tests are Python scripts using `sqlite3.load_extension(...)`.
- Name tests `test_*.py`; keep deterministic seeds where applicable.
- Validate: `pocketvec_version`, `pocketvec_cosine`, and the lightweight ANN vtab query path (`SELECT pocketvec_set_q(?)` then `SELECT rowid, distance FROM ann ORDER BY distance LIMIT 10`).
- Expected artifact name: `libpocketvec0` in `target/...` discoverable by Python tests.
- For debugging, run tests with `RUST_LOG=debug` to trace execution.

## Commit & Pull Request Guidelines
- Commits: imperative, scoped messages (e.g., "add vtab arg parser"); reference issues with `#123` when relevant.
- PRs include: summary, rationale, affected areas, build/run notes (OS, Rust toolchain), and test output. For behavior changes, add before/after SQL or Python snippets.

## Architecture & Tips
- Registers: `pocketvec_version`, `pocketvec_cosine`, `pocketvec_set_q`, and vtab `pocketvec`.
- Typical flow:
  1) `CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL)`
  2) `CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128)`
  3) `SELECT pocketvec_set_q(?)` with a local embeddings vector, then query `ann`.
- Safety: ensure blobs are multiples of 4 (float32) and match the configured dimension.

This extension keeps data and compute inside SQLite for privacy-focused, local embeddings workflows and fast, lightweight ANN retrieval on-device.
