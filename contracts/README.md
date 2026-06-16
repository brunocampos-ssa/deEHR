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

| Crate | Status |
| --- | --- |
| [`identity-registry/`](identity-registry/) | **Implemented** (Phase 1, #27) — `klever-sc` wired; builds to WASM; tested. See its [README](identity-registry/README.md). |
| `credential-registry/` | Skeleton |
| `consent-registry/` | Skeleton |
| `anchor-registry/` | Skeleton |

The skeleton crates carry a minimal `lib.rs` documenting intent; their
`klever-sc` integration lands with their respective Phase 1 issues.

## Building

Contracts build to WASM with the Klever SDK build tool (`ksc`):

```bash
cd contracts
ksc all build --path identity-registry   # build one contract
cargo test -p deehr-identity-registry     # run its tests (build first)
```

`cargo check --workspace` validates the workspace without producing WASM.

## License

[MIT](../LICENSE).
