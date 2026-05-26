# ADR-0001: Identity & Key Management — Progressive Custody

- **Status:** Accepted
- **Date:** 2026-05-22
- **Deciders:** deEHR maintainers

## Context

deEHR is a patient-owned Electronic Health Record platform. Patients must
control their own identity and consent, and that control must be verifiable and
anchored on the Klever blockchain.

Several forces constrain how identity and keys can work:

- **Target users.** Elderly and low-digital-literacy patients are first-class
  users. Seed phrases and conventional self-custody wallets are a hard
  usability barrier *and* a data-loss risk — a lost phrase can mean a lost
  health record.
- **The blockchain must be invisible by default.** Logging in should feel like
  any modern app.
- **Verified Klever KVM constraints** (capability verification, 2026-05-22).
  Klever KVM **does not** provide:
  - ERC-4337-style account abstraction (no programmable on-chain account
    validation hook);
  - on-chain `secp256r1` / P-256 verification — the curve WebAuthn/passkeys
    use — so a passkey **cannot** sign a Klever transaction;
  - native account guardians (MultiversX's Guardians were not inherited);
  - native gasless / meta-transactions (no relayer/sponsor field in the
    transaction format).

  Klever KVM **does** provide:
  - smart contracts as first-class accounts;
  - a native **weighted-multisig account-permission system** — per-signer
    weight, an approval threshold, and per-operation scoping — that can evolve
    without changing the account address;
  - **KDA fee pools**, letting users transact paying fees in an app-issued
    token rather than KLV.

The "invisible blockchain / no seed phrase / no gas" promise must therefore be
delivered **without** relying on native account abstraction or native gas
sponsorship.

## Decision

We adopt an identity and key-management model called **Progressive Custody**.

1. **Authentication.** Email or social login (OIDC) plus a **passkey**
   (WebAuthn/FIDO2) with biometric unlock. The passkey is an **off-chain
   authentication factor**: it authenticates the patient to the deEHR
   platform. It does not — and on Klever cannot — sign blockchain
   transactions.

2. **On-chain account.** Each patient has a standard Klever account. By default
   its signing key is **custodied by the deEHR platform**, HSM-backed, and is
   never exposed to the patient.

3. **Signing & Fee Service.** A platform-operated service submits patients'
   transactions and pays the network fees from a platform treasury (optionally
   via a KDA fee pool). Because Klever has no native gasless mechanism, **this
   service is the gasless mechanism.**

4. **Recovery.** Social recovery is implemented on Klever's native
   weighted-multisig account permissions: guardians (for example a family
   member, the primary-care provider, and the platform) are registered as
   signers on a recovery permission with an M-of-N threshold.

5. **Identity.** Each patient has a `did:klever` DID with an on-chain DID
   Document. The `did:klever` method itself is specified separately in
   **ADR-0004** (planned).

6. **Progressive custody spectrum.** The default is **assisted custody**
   (platform-held key + guardian recovery). A patient may progressively take
   over: add their own device key as a signer, reduce the platform signer's
   weight, and ultimately export to full self-custody. This is progressive and
   never forced.

7. **Data-encryption keys.** The keys protecting PHI are guardian-backed
   through the same recovery mechanism, so a lost device never means a lost
   health record.

## Consequences

### Positive

- Familiar, passwordless UX; no seed phrases; accessible to the target users.
- Patients never need to hold KLV or understand gas.
- A genuine sovereignty path is preserved (opt-in self-custody).
- Built entirely on **verified** Klever primitives — no dependency on
  unavailable features.

### Negative / risks

- **The Signing & Fee Service and key custody become security-critical** — a
  central point of trust and a high-value attack target. It must be HSM-backed,
  strictly access-controlled, monitored, and independently audited. It is a
  first-class component, not glue code.
- **Treasury dependency.** The platform must keep the fee treasury / KDA pool
  funded — an operational and financial commitment. Spam or abuse could drain
  it, so per-account rate limits and quotas are required.
- **Custodial-by-default has regulatory weight.** deEHR holds patient keys by
  default; the LGPD and liability implications must be addressed in the threat
  model and in legal review.
- "Patient-owned" is partly aspirational until a patient takes over custody —
  product messaging must stay honest about this.
- The model depends on Klever's account-permission semantics; if those change,
  recovery and progressive custody must be revisited.

## Alternatives considered

- **Pure self-custody (patient-held seed phrase / wallet).** Rejected — a hard
  barrier for the target users and an unacceptable data-loss risk.
- **Pure custodial with no path to self-custody.** Rejected — contradicts the
  patient-ownership and sovereignty principles.
- **Depending on native Klever account abstraction / gas sponsorship.**
  Rejected — not available, with no public roadmap; it would block the project
  indefinitely.
- **Off-chain identity only (no DID, no on-chain account).** Rejected — this
  loses verifiable, portable, chain-anchored consent, which is the core value
  proposition of deEHR.

## Open questions

- **ADR-0004** must specify the `did:klever` DID method.
- HSM / KMS selection for the custody service.
- Treasury funding model and concrete anti-abuse quotas / rate limits.
- Whether to build on Klever-native custody infrastructure (e.g. KleverSafe)
  or a self-operated HSM service.
- Re-verify Klever's crypto host functions and account-permission semantics
  before implementation; ask Klever directly whether on-chain `secp256r1` or
  native fee sponsorship is on the roadmap.

## References

- [README.md](../../README.md) — *Identity & Key Management — "Progressive
  Custody"* section.
- Klever KVM capability verification (2026-05-22): no ERC-4337 account
  abstraction, no on-chain `secp256r1`, no native guardians, no native gasless
  transactions; native weighted-multisig account permissions and KDA fee pools
  are available.
- [ADR-0002](adr-0002-on-chain-registry-design.md) — On-chain Registry Design.
- W3C Decentralized Identifiers (DID) Core; W3C WebAuthn / FIDO2.

## Addenda

### 2026-05-26 — ADR-0004 published

[ADR-0004](adr-0004-did-klever-method.md) — `did:klever` DID Method — has
been published as **Proposed**, fulfilling the planned reference in §5 of
the *Decision* above and resolving the corresponding entry in *Open
questions*. ADR-0001's decision is unchanged; this addendum is recorded per
the repository's append-only ADR policy
(see [docs/architecture/README.md](README.md) — *What is an ADR?*).
