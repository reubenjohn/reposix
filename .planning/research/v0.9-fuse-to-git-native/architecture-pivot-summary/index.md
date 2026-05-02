# Architecture Pivot Summary: FUSE to Git-Native Partial Clone

**Date:** 2026-04-24
**Status:** Design confirmed via two independent POCs; ready for implementation planning.
**Supersedes:** FUSE-based architecture (`crates/reposix-fuse`).

This document is the canonical record of the design decision to replace reposix's FUSE virtual filesystem with git's built-in partial clone mechanism. It captures every conclusion from two research sessions (read-path and push-path), the confirmed protocol findings, and the sync/conflict model designed in conversation. Future sessions planning this work should start here.

---

## Chapters

1. [Problem Statement](./section-1-problem-statement.md)
2. [Key Design Decision: Delete FUSE, Use Git's Partial Clone](./section-2-key-design-decision.md)
3. [Confirmed Technical Findings](./section-3-technical-findings.md)
4. [Sync and Conflict Model](./section-4-sync-conflict-model.md)
5. [Architecture: What Changes](./section-5-architecture-what-changes.md)
6. [What Stays the Same](./section-6-what-stays-same.md)
7. [Risks and Open Questions](./section-7-risks-open-questions.md)
8. [POC Artifacts](./section-8-poc-artifacts.md)
9. [Milestone Impact](./section-9-milestone-impact.md)
