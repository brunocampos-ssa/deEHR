# ADR-0002: On-chain Registry Design

- **Status:** Accepted
- **Date:** 2026-05-22
- **Last updated:** 2026-05-27 (open questions Q1-Q5 resolved; promoted to Accepted)
- **Deciders:** deEHR maintainers

## Context

deEHR's on-chain layer stores **proofs only — never PHI**. It consists of four
logical registries: Identity / DID, Credential, Consent, and Anchor & Audit.
The contracts are written in Rust and compiled to WebAssembly for the Klever
KVM.

This ADR decides how that on-chain layer is structured: contract decomposition,
the data model, the event model, access control, upgradeability, and the
per-record fee posture. The **Consent Registry is the source of truth for
authorization** — the SMART authorization server reads it before issuing any
token — so its design is especially load-bearing.

The ADR was originally marked `Proposed` because several decisions depended on
Klever KVM behaviour that needed to be verified. Source-cited research on
2026-05-27 against `klever-sc 0.45.1`, `klever-go`, the official docs, and
mainnet transaction observation resolved questions Q1-Q5; the remaining open
items are tracked below.

## Decision

1. **Contract decomposition — four separate contracts, one per registry**,
   rather than a single monolith. Each registry is independently auditable and
   upgradeable, has a minimal and distinct write-permission set, and a smaller
   blast radius if compromised. Because the SMART authorization server can
   read both Consent and Credential via free, gasless **off-chain VM queries**
   (Q2 resolution), separation imposes no cross-contract cost penalty on the
   hot authorization path.

2. **No-PHI invariant.** On-chain data is limited to: SHA-256 integrity hashes,
   DIDs, IPFS CIDs, status enumerations, **coded** scope and purpose-of-use
   identifiers (never free text), timestamps and expiries. Enforced by code
   review and audit.

3. **Identity / DID Registry.** DID Documents (or pointers to them),
   key-rotation history, and guardian / recovery signer sets. See
   [ADR-0004](adr-0004-did-klever-method.md) for the `did:klever` method.

4. **Credential Registry.** Issuance and revocation **status** of Verifiable
   Credentials — hashes only, never the credential body. Only credentialed
   issuer DIDs may write.

5. **Consent Registry.** Patient-signed consent records: patient DID, grantee
   DID, coded scope set, a resource-filter reference, purpose-of-use, expiry
   and status. Every grant and every revocation emits an event. This registry
   is the authorization source of truth.

6. **Anchor & Audit Registry.** Integrity hashes of encrypted FHIR bundles
   paired with their IPFS CIDs, plus an append-only log of data-access events.

7. **Event model.** Every state-changing call emits a structured event for the
   off-chain indexer and audit pipeline. Concrete rules:
   - Events are declared with the `#[event("name")]` macro provided by
     `klever-sc`. Each `#[indexed]` argument becomes one topic; **at most one**
     non-indexed argument becomes the data payload, ABI top-encoded into a
     single `ManagedBuffer`. The macro rejects multiple non-indexed args.
   - Topic 0 is the event name (literal bytes). Topics 1..N are the
     `#[indexed]` arguments in declaration order. **Addresses in topics are
     raw 32-byte buffers, not bech32.** The off-chain indexer must hex-encode
     addresses (strip the `klv1` HRP) before matching event topics. The
     emitting contract address — separately surfaced as `Logs.Address` on the
     proxy — remains bech32.
   - To keep events predictable under as-yet-undocumented VM limits (see
     *Open questions* — Klever-team confirmations), deEHR caps each event at
     **≤ 4 topics** and uses a single struct payload for any composite data.
   - Consumption: the proxy / node endpoint
     `GET /v1.0/transaction/{hash}` returns the per-tx `logs.events` array.
     A per-address websocket subscription is available on a local indexer
     node. There is **no public event-level filter API today**; the
     indexer fans out from address subscriptions and pulls
     `transaction/{hash}`. The audit pipeline therefore treats events as
     best-effort and uses contract state plus periodic full reconciliation
     as the source of truth — not the event stream alone.

8. **Access control.** Writes are authorized by the calling actor's DID and
   role. Patient-scoped writes are submitted by the Signing & Fee Service
   acting for the patient (see [ADR-0001](adr-0001-identity-and-key-management.md));
   institutional writes use credentialed institution DIDs.

9. **Upgradeability.** Built on Klever KVM's native upgrade primitive
   (verified Q1):
   - Each contract is deployed with `CodeMetadata::UPGRADEABLE` **explicitly
     set**. The framework's `#[upgrade]` attribute defines a separate
     entrypoint, distinct from `#[init]`, dispatched on upgrade transactions.
   - Storage persists across upgrade (same account address, same trie). The
     framework provides **no built-in migration tooling**: any storage-layout
     change must be implemented as explicit logic inside the `#[upgrade]`
     function, gated by a `storage_version` mapper, and accompanied by an
     explicit data-migration plan reviewed during the upgrade PR.
   - Authorization at the VM level is **owner-only**. Multisig-controlled
     upgrade is implemented by setting the contract's owner to a governance
     multisig contract via the `ChangeOwnerAddress` builtin; the VM does not
     itself know about multisig.
   - **Consent Registry stabilization & lock-in.** Once the Consent Registry
     reaches its stabilized schema, we will submit a final upgrade that
     clears the `UPGRADEABLE` bit (`CodeMetadata::DEFAULT`), permanently
     locking the contract. This trades upgradeability for stronger
     guarantees over the authorization source of truth.

10. **Fees & treasury posture.** Per-record costs are empirically tractable
    (verified Q4):
    - **KAppFee** is a flat 2 KLV per `SmartContractInvoke`; **BandwidthFee**
      scales with transaction envelope size and execution (~2 KLV base +
      ~0.008 KLV/byte per the docs; empirically 6–31 KLV total for typical
      SC calls).
    - Storage follows **pay-once-on-write with a refund on delete**. All
      deEHR contracts MUST clear (delete) revoked/expired records so the
      storage refund is claimed — material at 10K-patient scale.
    - All patient-side transactions are paid by the Signing & Fee Service
      treasury per ADR-0001. Optionally, a KDA fee pool can absorb the
      entire fee in a deEHR-issued token, giving an operational lever
      against KLV price volatility.
    - **Treasury budgeting**: forecast at **50 KLV per operation** as a
      safety multiplier over the observed ~12–27 KLV range. At 10,000
      patients × ~17 ops/year (5 consent ops + 10 audit-event emits +
      2 DID updates), the forecast is ~8.5M KLV/year — comfortably
      absorbable.
    - The fee schedule is **governance-mutable by KFI holders**; the
      operational forecast must be recalibrated annually against the
      then-current schedule.

## Consequences

### Positive

- Clear separation of concerns; least-privilege write permissions per registry.
- Independent audit and upgrade per contract; smaller per-contract attack
  surface; the Consent Registry can be permanently locked once stable.
- The no-PHI invariant is structurally simple to review.
- A consistent EVM-style event model makes the off-chain indexer and audit
  trail straightforward and grounded in a verified framework primitive.
- Authorization-server hot path is free: off-chain VM queries cost no gas and
  no on-chain cross-contract calls are required.
- Per-record costs are empirically affordable and absorbable by the platform
  treasury; KDA fee pools offer a future migration to a deEHR-issued token.

### Negative / risks

- **Indexer trust & maturity.** Off-chain authorization decisions depend on
  chain state being read correctly and promptly. No public event-level
  filter API exists today, so the indexer must fan out from per-address
  websocket subscriptions and pull per-tx receipts. Indexer reliability is
  part of the security model.
- **Fee-schedule governance risk.** Klever's per-byte and per-op fees are
  mutable by KFI-holder governance. A hostile change could 5×-10× the
  treasury burn overnight; the operational plan must include an annual
  recalibration and a contingency lever (KDA fee pool, fallback custodial
  posture).
- **Cross-contract failure semantics.** Synchronous on-chain cross-contract
  calls (if ever introduced) cascade failures and have no try/catch. Our
  architecture avoids them in the hot path; if a future feature needs them,
  the design must treat any panicking callee as a full caller revert.
- **Manual upgrade migration.** The framework provides no migration tooling;
  every schema change is bespoke code in `#[upgrade]`. Mistakes are
  unrecoverable (storage is opaque key/value at the VM level).
- **Treasury dependency.** Same as the original posture — the platform must
  keep the fee treasury / KDA pool funded; rate limits and quotas remain
  required.
- Four contracts mean more deployment and upgrade coordination than a
  monolith, mitigated by independent upgradeability and clearer audit
  scopes.

## Alternatives considered

- **A single monolithic registry contract.** Rejected — a larger blast radius,
  broader write permissions, and harder to audit and upgrade independently.
- **Off-chain consent database with periodic hash anchoring.** Rejected —
  consent must itself be the verifiable on-chain source of truth, not a hash
  of an off-chain record that could diverge.
- **Grouping Identity + Credential into one contract.** Considered as a
  fall-back if cross-contract calls turned out to be expensive. Rejected at
  this revision because off-chain reads are free (Q2), so the four-contract
  layout has no cost penalty.

## Resolved questions

The following items were tracked as `Proposed`-blocking when the ADR was first
drafted and have been resolved (2026-05-27) via source-cited research:

- **Q1 — Contract upgradeability.** First-class `#[upgrade]` attribute
  distinct from `#[init]`; owner-gated at the VM level;
  `CodeMetadata::UPGRADEABLE` gates eligibility; storage persists across
  upgrade; migration is manual code inside `#[upgrade]`. Immutability is
  achievable by clearing the bit (deploy-time or via a final upgrade).
  See §9.
- **Q2 — Cross-contract calls.** Sync only in the public `klever-sc 0.45.1`
  SDK; failures cascade; no try/catch; read-only calls still cost gas.
  Critically, **off-chain VM queries are free and gasless** via the
  proxy / node REST API, so the SMART authorization path does not need
  any on-chain cross-contract call. See §1.
- **Q3 — Event / log model.** EVM-style topics + single data buffer via the
  `#[event(...)]` macro; topic 0 is the event name; `#[indexed]` args are
  raw-byte topics; one data payload. Addresses in topics are raw 32-byte.
  See §7.
- **Q4 — Storage / fee cost model.** Pay-once-on-write storage with a
  refund on delete. KAppFee 2 KLV flat per SC invoke; BandwidthFee
  ~base + ~0.008 KLV/byte (empirically 6–31 KLV total per typical call).
  Treasury comfortably absorbable. See §10.
- **Q5 — Four contracts vs grouping.** Keep four. The original concern
  (cross-contract cost) is moot — reads happen off-chain for free
  (Q2). The independent upgrade and audit benefits stand. See §1.

## Open questions

These remain to resolve before any subsequent ADR-0002 revision (or as inputs
to other ADRs). ADR-0002 will be **amended via an Addenda section** if any
answer materially contradicts the assumptions above, consistent with the
repository's append-only ADR policy.

- **Q6 — Coded value sets** for scope and purpose-of-use. Depends on FHIR
  profile selection ([#6](https://github.com/brunocampos-ssa/deEHR/issues/6)).
  Once #6 resolves, ADR-0002 will be amended to bind the chosen SMART scope
  codes and RNDS coded sets.

- **Klever-team confirmations needed.** The 2026-05-27 research left several
  behavioural details that the SDK source and public docs do not
  authoritatively answer. These should be confirmed by the Klever developer
  team before mainnet deployment:
  1. **Upgrade tx encoding** — is upgrade a distinct `EnumContractType` in
     the transaction protobuf, or a regular SC call with function name
     `upgradeContract`?
  2. **Storage refund mechanics** — exact ratio, what triggers a refund
     (set-to-zero? explicit clear?), and credit timing (same tx vs settled
     later).
  3. **Cross-contract gas pricing** — per-opcode dispatch cost, max call
     depth, whether `execute_on_dest_context_readonly_raw` is VM-enforced
     (storage-write attempts trap) or only by convention.
  4. **Re-entrancy** — does the VM enforce a built-in lock, or is it purely
     the contract author's responsibility?
  5. **Event hard limits** — max topics per event, max bytes per topic, max
     data bytes, max events per tx; confirm failure-rollback semantics for
     logs (assumed: discarded on tx revert).
  6. **Public event indexer API** — block-range-by-contract event endpoint?
     Event-level websocket subscription? Roadmap?
  7. **Gas schedule transparency** — published per-opcode / storage gas
     YAML/JSON analogous to Arwen's `gasScheduleV1.yaml`?
  8. **KDA fee pool** — per-tx cap, behaviour when pool balance is
     insufficient, swap-settlement latency.
  9. **Async / promises roadmap** — is sync-only the long-term design, or
     are async opcodes / `#[callback]` planned for a future SDK release?
  10. **Recommended migration pattern** — official endorsement of a
      `storage_version` mapper or migration-helper macro?

## References

- [README.md](../../README.md) — *What Lives on Klever* section.
- [ADR-0001](adr-0001-identity-and-key-management.md) — Identity & Key
  Management (the Signing & Fee Service and treasury).
- [ADR-0004](adr-0004-did-klever-method.md) — `did:klever` DID Method.
- Klever KVM capability verification (2026-05-22) — verified crypto host
  functions, account-permission semantics, transaction protobuf.
- Klever KVM behaviour research (2026-05-27) — `klever-sc 0.45.1`
  source (`~/.cargo/registry/...`), `klever-go` VM-host execution flow,
  empirical mainnet observation via `api.mainnet.klever.org/v1.0/transaction/list?type=63`.
- Klever official documentation — <https://docs.klever.org/>
  (`smart-contracts/reference/annotations`, `smart-contracts/reference/calls`,
  `smart-contracts/reference/payments`, `klever-vm`, `about-our-technology`,
  `api-and-sdk`).
- `klever-io/klever-vm-sdk-rs` — <https://github.com/klever-io/klever-vm-sdk-rs>.
- `klever-io/klever-go` — <https://github.com/klever-io/klever-go>.
