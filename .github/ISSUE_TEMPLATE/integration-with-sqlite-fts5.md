---
name: Hybrid Search Integration
about: Tracking issue to combine vector search with full-text search capabilities
title: "Hybrid Search Integration"
labels: ["enhancement", "integration", "search"]
---

Goal
- Enable hybrid retrieval: Full-text search + vector similarity reranking for comprehensive search workflows.

Scope
- API endpoints for combining text-based and vector-based search
- Query planning for multi-stage retrieval strategies
- Benchmarks: precision@k vs. text-only and vector-only baselines

Milestones
- [ ] Hybrid search API design and implementation
- [ ] Query patterns for two-stage retrieval (text → vector)
- [ ] Client SDK helpers for hybrid queries
- [ ] Performance metrics and documentation

Notes
- Focus on efficient in-memory processing
- Ensure predictable performance across different hardware configurations
- Maintain API simplicity for common use cases

How to Contribute
- Share search patterns, use cases, and performance measurements
- PRs welcome for docs, examples, and client utilities

