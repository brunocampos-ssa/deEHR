# ADR-0002: On-chain Registry Design

- **Status:** Proposed
- **Date:** 2026-05-22
- **Deciders:** deEHR maintainers

## Context

deEHR's on-chain layer stores **proofs only — never PHI**. It consists of four
logical registries: Identity / DID, Credential, Consent, and Anchor & Audit.
The contracts are written in Rust and compiled to WebAssembly for the Klever
KVM.

This ADR decides how that on-chain layer is structured: contract decomposition,
the data model, the event model, access control, and upgradeability. The
**Consent Registry is the source of truth for authorization** — the SMART
authorization server reads it before issuing any token — so its design is
especially load-bearing.

This ADR is `Proposed`: several decisions depend on Klever KVM behaviour that
must still be verified (see *Open questions*).

## Decision

1. **Contract decomposition — four separate contracts, one per registry**,
   rather than a single monolith. Each registry is independently auditable and
   upgradeable, has a minimal and distinct write-permission set, and a smaller
   blast radius if compromised.

2. **No-PHI invariant.** On-chain data is limited to: SHA-256 integrity hashes,
   DIDs, IPFS CIDs, status enumerations, **coded** scope and purpose-of-use
   identifiers (never free text), timestamps and expiries. Enforced by code
   review and audit.

3. **Identity / DID Registry.** DID Documents (or pointers to them),
   key-rotation history, and guardian / recovery signer sets.

4. **Credential Registry.** Issuance and revocation **status** of Verifiable
   Credentials — hashes only, never the credential body. Only credentialed
   issuer DIDs may write.

5. **Consent Registry.** Patient-signed consent records: patient DID, grantee
   DID, coded scope set, a resource-filter reference, purpose-of-use, expiry
   and status. Every grant and every revocation emits an event. This registry
   is the authorization source of truth.

6. **Anchor & Audit Registry.** Integrity hashes of encrypted FHIR bundles
   paired with their IPFS CIDs, plus an append-only log of data-access events.

7. **Event model.** Every state-changing call emits a structured event so the
   off-chain indexer and audit pipeline can reconstruct state and serve audit
   queries.

8. **Access control.** Writes are authorized by the calling actor's DID and
   role. Patient-scoped writes are submitted by the Signing & Fee Service
   acting for the patient (see [ADR-0001](adr-0001-identity-and-key-management.md));
   institutional writes use credentialed institution DIDs.

9. **Upgradeability.** Each contract is deployed with an upgrade path
   controlled by a governance multisig, and every upgrade requires an explicit
   data-migration plan. (This depends on Klever KVM upgrade semantics — see
   *Open questions*.)

## Consequences

### Positive

- Clear separation of concerns; least-privilege write permissions per registry.
- Independent audit and upgrade per contract; smaller per-contract attack
  surface.
- The no-PHI invariant is structurally simple to review.
- A consistent event model makes the off-chain indexer and audit trail
  straightforward.

### Negative / risks

- **Cross-contract interaction.** The authorization flow reads both the Consent
  and Credential registries; Klever KVM cross-contract call support and cost
  must be confirmed.
- **Fee cost.** Every consent grant/revoke and every audit event costs network
  fees, paid by the Signing & Fee Service treasury — this couples registry
  activity to the ADR-0001 treasury model.
- **Indexer trust.** Off-chain authorization decisions depend on chain state
  being read correctly and promptly; indexer reliability becomes part of the
  security model.
- Four contracts mean more deployment and upgrade coordination than a monolith.

## Alternatives considered

- **A single monolithic registry contract.** Rejected — a larger blast radius,
  broader write permissions, and harder to audit and upgrade independently.
- **Off-chain consent database with periodic hash anchoring.** Rejected —
  consent must itself be the verifiable on-chain source of truth, not a hash of
  an off-chain record that could diverge.
- **Grouping Identity + Credential into one contract.** Deferred — kept as an
  open question pending the cross-contract-call cost analysis.

## Open questions

These must be resolved (or explicitly deferred) before this ADR can move to
`Accepted`:

- Klever KVM contract **upgradeability** mechanism and data-migration story.
- Klever KVM **cross-contract calls** — supported, and at what cost?
- The **event / log model** and exactly how the off-chain indexer consumes it.
- The **storage / fee cost model** per record, and confirmation that the
  treasury absorbs it.
- Whether to keep four contracts or group some (e.g. Identity + Credential).
- The **coded value sets** for scope and purpose-of-use — these must align with
  SMART scopes and with RNDS (see the planned FHIR profile selection).

## References

- [README.md](../../README.md) — *What Lives on Klever* section.
- [ADR-0001](adr-0001-identity-and-key-management.md) — Identity & Key
  Management (the Signing & Fee Service and treasury).
- Klever KVM capability verification (2026-05-22).
