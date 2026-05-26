//! # deEHR Consent Registry
//!
//! Klever KVM smart contract that records patient-signed consent grants:
//! patient DID, grantee DID, coded scope set, resource-filter reference,
//! purpose-of-use, expiry, status. Every grant and revocation emits an
//! event.
//!
//! **This is the source of truth for authorization** — the SMART
//! authorization server consults this contract before issuing any
//! OAuth2 token.
//!
//! See [ADR-0002 — On-chain Registry Design][adr-0002].
//!
//! ## Status
//!
//! **Skeleton only.** The `klever-vm-sdk-rs` integration is intentionally
//! deferred — the exact SDK version and contract macro pattern still need
//! to be verified against KVM upgradeability and cross-contract call
//! semantics (see issue #8). Coded value sets for scope and purpose-of-use
//! depend on the FHIR profile selection ([#6](https://github.com/brunocampos-ssa/deEHR/issues/6)).
//!
//! [adr-0002]: ../../../docs/architecture/adr-0002-on-chain-registry-design.md

/// Marker — to be replaced with the real contract trait once the
/// `klever-vm-sdk-rs` framework is wired in.
pub struct ConsentRegistry;
