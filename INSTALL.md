# Installation

Install the Rust-based SQLite vector database extension for local embeddings and lightweight ANN search.

## Requirements
- Rust toolchain (stable) with `cargo`
- SQLite 3 with loadable extensions enabled (CLI or embedded)
- Optional: Python 3.9+ and NumPy for demos/tests

## Build
```
cd pocketvec-sqlite
cargo build --release
```
Artifacts:
- macOS: `target/release/libpocketvec0.dylib`
- Linux: `target/release/libpocketvec0.so`

Makefile shortcuts:
- `make build` (release), `make debug`, `make clean`

## Load the Extension

SQLite CLI:
```
sqlite3 :memory:
.load /absolute/path/to/pocketvec-sqlite/target/release/libpocketvec0
SELECT pocketvec_version();
```

Python (`sqlite3`):
```python
import sqlite3, os
con = sqlite3.connect(":memory:")
con.enable_load_extension(True)
con.load_extension(os.path.abspath("pocketvec-sqlite/target/release/libpocketvec0"))
print(con.execute("SELECT pocketvec_version()").fetchone()[0])
```

## Troubleshooting
- “file not found”: use an absolute path and the correct file name.
- “no such function pocketvec_version”: extension not loaded—check `.load`/`load_extension` path.
- macOS Gatekeeper: prefer absolute paths; build locally (codesigning not required).
- Dimension errors: ensure vector blob length equals `4 * dim` (float32 bytes).
 - ANN expectations: current vtab performs lightweight ANN via linear scan; ensure dataset sizes fit your device.

## Verify
- Minimal: `python pocketvec-sqlite/test_minimal.py` (checks SQLite vector database functions)
- End-to-end: `python pocketvec-sqlite/tests/test.py` (demonstrates local embeddings and lightweight ANN queries)
