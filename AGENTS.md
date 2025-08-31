# Repository Guidelines

## Project Structure & Modules
- `pocketvec-sqlite/`: main crate and assets.
  - `Cargo.toml`: cdylib extension `pocketvec0` (Rust 2021).
  - `src/`: Rust modules (`lib.rs`, `vtab.rs`, `cosine.rs`, `utils.rs`).
  - `python/`: demo script (`demo.py`).
  - `tests/`: end-to-end script (`test.py`) and SQL (`smoke.sql`).
  - `makefile`: common tasks. Build artifacts land in `target/`.

## Build, Test, and Run
- Build (release): `make build` or `cargo build --release`
  - Produces `target/release/libpocketvec0.{dylib|so}`.
- Build (debug): `make debug` or `cargo build`
- Clean: `make clean`
- Quick smoke (Python): `python pocketvec-sqlite/test_minimal.py`
- End-to-end: `python pocketvec-sqlite/tests/test.py`
- Demo DB: `python pocketvec-sqlite/python/demo.py`
- Verbose logs: prefix with `RUST_LOG=debug`.

## Coding Style & Naming
- Rust: follow rustfmt defaults (4-space indent, snake_case files and modules).
- Public APIs use clear, descriptive names (e.g., `pocketvec_set_q`, `pocketvec_cosine`).
- Prefer small modules with single responsibility; keep FFI boundaries explicit.
- Add minimal doc comments for functions exposed to SQLite.

## Testing Guidelines
- Python scripts exercise the extension via `sqlite3.load_extension(...)`.
- Tests live as `test_*.py`; keep deterministic seeds where applicable.
- Expected artifact name: `libpocketvec0` (macOS `.dylib`, Linux `.so`).
- Validate: version function, cosine scalar, vtab query path (`pocketvec_set_q` then `SELECT ... FROM ann`).

## Commit & Pull Requests
- Commits: imperative, scoped messages (e.g., "add vtab arg parser"). Reference issues with `#123` when relevant.
- PRs must include:
  - Summary, rationale, and affected areas.
  - Build/run notes (OS, Rust toolchain) and test output.
  - For behavior changes: before/after example (SQL or Python snippet).

## Architecture & Tips
- Extension registers: `pocketvec_version`, `pocketvec_cosine`, `pocketvec_set_q`, and vtab `pocketvec`.
- Typical flow:
  1) `CREATE TABLE docs_vecs(rowid INTEGER PRIMARY KEY, vec BLOB NOT NULL)`
  2) `CREATE VIRTUAL TABLE ann USING pocketvec(base_table=docs_vecs, dim=128)`
  3) `SELECT pocketvec_set_q(?)` then `SELECT rowid, distance FROM ann ORDER BY distance LIMIT 10`
- Safety: validate blob sizes are multiples of 4 (float32) and dimension matches.
