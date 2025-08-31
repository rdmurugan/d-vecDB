---
name: Roadmap — IVF-PQ indexing
about: Tracking issue for IVF-PQ and lightweight ANN optimizations
title: "Roadmap: IVF-PQ indexing"
labels: ["roadmap", "enhancement", "ANN"]
---

Goal
- Add IVF-PQ indexing to accelerate nearest neighbor search beyond linear scan while keeping a lightweight ANN footprint suitable for local embeddings in SQLite.

Scope
- IVF coarse quantizer, PQ codebooks, training pipeline.
- On-disk index format compatible with the SQLite vector database extension.
- ANN query path integrated with `pocketvec` vtab.

Milestones
- [ ] Design index schema (tables/files, metadata)
- [ ] Training API (SQL/Python) and reproducible seeds
- [ ] Build index from base table vectors
- [ ] Query execution path (coarse probe + re-ranking)
- [ ] Quality/latency benchmarks (N=100k, 1M)

Notes
- Favor incremental landing: IVF first, then PQ.
- Keep memory use predictable; support device-constrained builds.

How to Contribute
- Comment with design proposals, benchmarks, or references.
- Open PRs for individual milestones with test cases and docs.

