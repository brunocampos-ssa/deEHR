//! # deEHR Identity / DID Registry
//!
//! Klever KVM smart contract that stores patient and institution DID
//! Documents, key-rotation history, and guardian / recovery signer sets.
//!
//! See [ADR-0001 — Identity & Key Management][adr-0001] for the identity
//! model, and [ADR-0002 — On-chain Registry Design][adr-0002] for the
//! broader on-chain layer.
//!
//! ## Status
//!
//! **Skeleton only.** The `klever-vm-sdk-rs` integration is intentionally
//! deferred — the exact SDK version and contract macro pattern still need
//! to be verified against KVM upgradeability and cross-contract call
//! semantics (see issue #8).
//!
//! [adr-0001]: ../../../docs/architecture/adr-0001-identity-and-key-management.md
//! [adr-0002]: ../../../docs/architecture/adr-0002-on-chain-registry-design.md

/// Marker — to be replaced with the real contract trait once the
/// `klever-vm-sdk-rs` framework is wired in.
pub struct IdentityRegistry;
