---
name: Integration with SQLite FTS5
about: Tracking issue to combine vector search with full-text search
title: "Integration with SQLite FTS5"
labels: ["enhancement", "integration", "FTS5"]
---

Goal
- Enable hybrid retrieval: FTS5 text search + ANN rerank inside SQLite for local embeddings workflows.

Scope
- Query plan examples combining `MATCH` with `pocketvec` vtab.
- Helper functions or views to streamline hybrid queries.
- Benchmarks: precision@k vs. FTS-only and vector-only baselines.

Milestones
- [ ] Example schema and demo dataset
- [ ] SQL patterns for two-stage retrieval (FTS → vector)
- [ ] Optional Python helper for hybrid queries
- [ ] Metrics + docs in USER_GUIDE.md

Notes
- Keep everything in-process (no external services).
- Ensure predictable performance on modest hardware.

How to Contribute
- Share query patterns, indexes, and measurements.
- PRs welcome for docs, examples, and helper utilities.

