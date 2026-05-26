//! # deEHR Anchor & Audit Registry
//!
//! Klever KVM smart contract that stores integrity hashes of encrypted
//! FHIR bundles paired with their IPFS CIDs, plus an **append-only log
//! of data-access events** for tamper-evident audit.
//!
//! See [ADR-0002 — On-chain Registry Design][adr-0002].
//!
//! ## Status
//!
//! **Skeleton only.** The `klever-vm-sdk-rs` integration is intentionally
//! deferred — the exact SDK version and contract macro pattern still need
//! to be verified against KVM upgradeability and cross-contract call
//! semantics (see issue #8).
//!
//! [adr-0002]: ../../../docs/architecture/adr-0002-on-chain-registry-design.md

/// Marker — to be replaced with the real contract trait once the
/// `klever-vm-sdk-rs` framework is wired in.
pub struct AnchorRegistry;
