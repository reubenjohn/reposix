//! `reposix-swarm` — adversarial swarm harness.
//!
//! N concurrent simulated "agents" hammer the simulator (HTTP via
//! [`SimBackend`](reposix_core::backend::sim::SimBackend)) or a real
//! Confluence tenant for a fixed duration. Each client runs a realistic
//! workload loop (list + reads + patch) and records per-operation
//! latencies. At run end the binary emits a markdown summary with
//! P50/P95/P99 per op type, total requests, error rate, and an audit-row
//! invariant check.
//!
//! Motivation: the "10k agent QA team" pattern from the `StrongDM` playbook
//! (see `AgenticEngineeringReference.md` §1). This is a miniature credible
//! version — validates the simulator under concurrent load while
//! asserting the SG-06 append-only audit invariant holds.

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]

pub mod confluence_direct;
pub mod contention;
pub mod driver;
pub mod metrics;
pub mod sim_direct;
pub mod workload;
