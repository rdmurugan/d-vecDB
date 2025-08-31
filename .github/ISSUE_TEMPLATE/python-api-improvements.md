---
name: Python API improvements
about: Tracking issue for Python ergonomics and examples
title: "Python API improvements"
labels: ["enhancement", "python", "DX"]
---

Goal
- Make the Python experience smoother for loading the SQLite vector database extension, preparing local embeddings, and running lightweight ANN queries.

Scope
- Convenience helpers (e.g., `f32_blob`, normalization, dim checks).
- Clear error messages and typed stubs.
- More examples (demo + tests) and minimal dependencies.

Milestones
- [ ] Thin utility module under `python/`
- [ ] Type hints and docstrings
- [ ] Examples for batch insert + querying
- [ ] CI smoke test invoking `sqlite3.load_extension`

Notes
- Keep API surface small and explicit; avoid heavy frameworks.

How to Contribute
- Propose function signatures and examples.
- Submit PRs with tests and concise docs.

