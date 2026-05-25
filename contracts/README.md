# Smart Contracts

This directory contains the deEHR on-chain layer — a set of Rust smart
contracts compiled to **WebAssembly** for the **Klever Virtual Machine
(KVM)**.

The on-chain layer stores **proofs only — never PHI** (see
[ADR-0002](../docs/architecture/adr-0002-on-chain-registry-design.md)).

## Crates

| Crate | Responsibility |
| --- | --- |
| [`identity-registry/`](identity-registry/) | DID Documents, key-rotation history, guardian/recovery signer sets |
| [`credential-registry/`](credential-registry/) | Issuance and revocation **status** of Verifiable Credentials (hashes only) |
| [`consent-registry/`](consent-registry/) | Patient-signed consent grants — the source of truth for authorization |
| [`anchor-registry/`](anchor-registry/) | Integrity hashes of encrypted FHIR bundles + IPFS CIDs; tamper-evident audit log |

## Status

**Skeleton only** (Phase 0). Each crate has a minimal `lib.rs` that documents
intent; the `klever-vm-sdk-rs` integration is intentionally deferred until the
ADR-0002 open questions ([#8](https://github.com/brunocampos-ssa/deEHR/issues/8))
are resolved — specifically Klever KVM upgradeability, cross-contract call
semantics, the fee/event model, and the SDK contract-macro pattern.

## Building

Once the SDK is wired, contracts will build to WASM with:

```bash
cd contracts
cargo build --release --target wasm32-unknown-unknown
```

Until then, `cargo check` validates the workspace structure.

## License

[MIT](../LICENSE).
