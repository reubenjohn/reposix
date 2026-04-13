//! `reposix-swarm` — adversarial swarm harness.
//!
//! N concurrent simulated "agents" hammer either the simulator (HTTP via
//! [`SimBackend`](reposix_core::backend::sim::SimBackend)) or a mounted FUSE
//! tree for a fixed duration. Each client runs a realistic workload loop
//! (list + reads + patch) and records per-operation latencies. At run end the
//! binary emits a markdown summary with P50/P95/P99 per op type, total
//! requests, error rate, and an audit-row invariant check.
//!
//! Motivation: the "10k agent QA team" pattern from the `StrongDM` playbook
//! (see `AgenticEngineeringReference.md` §1). This is a miniature credible
//! version — validates the simulator (and FUSE) under concurrent load while
//! asserting the SG-06 append-only audit invariant holds.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod metrics;
pub mod workload;
