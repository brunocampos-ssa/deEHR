# ADR-0004: did:klever DID Method — Hybrid Classical / Post-Quantum

- **Status:** Proposed
- **Date:** 2026-05-26
- **Deciders:** deEHR maintainers

## Context

deEHR identifies patients and other actors (institutions, providers, the
Signing & Fee Service) with W3C Decentralized Identifiers. No standard DID
method exists for the Klever network, so deEHR must define one.

Forces shaping this decision:

- **W3C DID Core conformance.** The method must conform to the W3C
  Decentralized Identifiers 1.0 syntax, resolution model, and DID Document
  semantics.
- **Verified Klever KVM crypto reality.** KVM host functions verify only
  `ed25519`, `secp256k1`, and BLS. There is no on-chain verifier for any NIST
  post-quantum scheme (ML-DSA, ML-KEM, SLH-DSA, Falcon). PQ signatures cannot
  authorise Klever transactions today.
- **Long-lived sensitive data.** Health records signed today must remain
  verifiable for the lifetime of the patient — decades. **Harvest-now,
  decrypt-later** is an explicit, documented threat for healthcare data. NIST
  finalised FIPS 203 / 204 / 205 in 2024; the NSA's CNSA 2.0 directive
  expects national-security-system traffic to be exclusively PQ by 2033, and
  regulated sectors including healthcare are widely expected to follow.
  Designing PQ resistance in now is materially cheaper than retrofitting it.
- **Progressive custody (ADR-0001).** The same DID must survive across all
  custody states — assisted, hybrid, self-custodial. Key rotation must not
  change the identifier.
- **Identity Registry contract (ADR-0002).** A dedicated on-chain registry
  already exists in the architecture to hold DID-Document data, recovery sets,
  and service endpoints. `did:klever` resolution builds on it.
- **Privacy on a public chain.** All on-chain state is observable forever.
  Anything stored against a DID is correlatable forever.

## Decision

We adopt **`did:klever`** with a **hybrid classical / post-quantum**
verification-method profile.

### 1. DID syntax

```text
did-klever       = "did:klever:" klever-network ":" klever-id
klever-network   = "mainnet" / "testnet" / "devnet"
klever-id        = klever-bech32-address       ; e.g. klv1q5y…
```

Examples:

- `did:klever:mainnet:klv1q5yndp8gw3l4...`
- `did:klever:testnet:klv1q5yndp8gw3l4...`

The method-specific identifier **is the Klever bech32 account address**. The
account is its own controller. Key rotation is performed via Klever's native
`UpdateAccountPermission` transaction and never changes the DID. Alternative
identifier schemes (opaque random, hash-of-key) were considered and rejected
(see *Alternatives*).

This satisfies the W3C DID Core ABNF: the method name is `klever`, and the
method-specific identifier `<network>:<bech32>` uses only characters in the
`idchar` set.

### 1.5. Signer key-type policy

The Klever KVM verifies `ed25519`, `secp256k1`, and BLS signatures natively.
For **deEHR-managed accounts**, `did:klever` uses **Ed25519 signers
exclusively** — for alignment with the Klever wallet and SDK defaults, a
uniform `Multikey` shape per classical verification method, a single
KMS / HSM key type in custody, and a single rotation primitive across the
progressive-custody spectrum. This constraint is a deEHR policy, not a
restriction of the `did:klever` method itself; other implementers may use
`secp256k1` or BLS signers within the same method.

### 2. Resolution path

A resolver — implemented as a deEHR universal-resolver driver — performs:

1. Parse the DID into `(network, address)`.
2. Connect to the configured Klever node URL for the network.
3. Read the account's permission set (native, fee-free read). Each registered
   signer — its weight, threshold, and operation bitmask — produces one
   classical verification method. Under the §1.5 policy, deEHR-managed
   accounts produce `Multikey` Ed25519 entries.
4. Call `Identity.resolveDid(address)` on the Identity Registry contract.
   This returns the PQ verification-method commitments, key-agreement keys,
   service endpoints, and the deactivation flag.
5. Construct and return the DID Document by merging (3) and (4).

The DID Document is **not** stored as an opaque blob on-chain; it is
**derived** at resolution time from the account permission set and the
Identity Registry record. This keeps on-chain data minimal, structurally
auditable, and tamper-evident.

### 3. DID Document shape

A resolved DID Document looks like this (illustrative; field values shortened
for readability):

```json
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/multikey/v1"
  ],
  "id": "did:klever:mainnet:klv1q5y...",
  "controller": "did:klever:mainnet:klv1q5y...",

  "verificationMethod": [
    {
      "id": "did:klever:mainnet:klv1q5y...#klv-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z6Mk..."
    },
    {
      "id": "did:klever:mainnet:klv1q5y...#pq-sig-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z<pq-sig-multicodec><pubkey>",
      "expires": "2031-05-26T00:00:00Z"
    },
    {
      "id": "did:klever:mainnet:klv1q5y...#pq-kem-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z<pq-kem-multicodec><pubkey>"
    }
  ],

  "authentication":       ["#klv-1"],
  "capabilityInvocation": ["#klv-1"],
  "assertionMethod":      ["#pq-sig-1"],
  "keyAgreement":         ["#pq-kem-1"],

  "service": [
    {
      "id": "did:klever:mainnet:klv1q5y...#deehr-fhir",
      "type": "DEEHRFhirEndpoint",
      "serviceEndpoint": "https://fhir.deehr.example/p/<opaque-token>"
    }
  ]
}
```

Verification-method roles are scoped deliberately:

- `authentication` → `#klv-1`. Logging in to the platform asserts control of
  the Klever account.
- `capabilityInvocation` → `#klv-1`. Authorising on-chain calls — the only
  verifier the KVM has today.
- `assertionMethod` → `#pq-sig-1`. Signing long-lived off-chain artefacts:
  consent VCs, FHIR-bundle integrity attestations, audit-log entries.
- `keyAgreement` → `#pq-kem-1`. Envelope encryption of off-chain PHI for the
  holder.

This is the **hybrid** model: classical where the chain must verify;
post-quantum where the data outlives the chain.

### 4. On-chain anchoring of PQ keys

For each PQ verification method, the Identity Registry stores only:

- the **multicodec** identifier of the PQ scheme (so verifiers know how to
  interpret the key),
- the **SHA-256 hash** of the public key (32 bytes — **not** the full PQ key),
- an optional `expires` timestamp,
- the `verificationRelationship` (`assertionMethod`, `keyAgreement`, …).

The full PQ public key lives off-chain in the holder's keystore and the
deEHR custody service. A verifier obtains the full key off-chain (from a
deEHR service, a VC, or the holder), hashes it, and compares against the
on-chain commitment.

This keeps on-chain footprint at 32 B per PQ key regardless of scheme — a
material consideration when ML-DSA-65 public keys are ~1.9 KB and SLH-DSA-128f
signatures are ~17 KB. The on-chain record functions as a tamper-evident
binding ("at block N, this DID committed to this PQ pubkey"), not as a key
distribution channel.

### 5. Operations

- **Create.** Implicit on Klever account creation: a `did:klever` exists as
  soon as its account exists, with a minimal DID Document derived from the
  account permission set. An explicit `Identity.registerDid` call adds PQ-key
  commitments, services, and any extended metadata.
- **Read / Resolve.** Resolver flow described in §2. Resolvable by any party
  with network access; no special authorisation required.
- **Update — classical key rotation.** Native Klever
  `UpdateAccountPermission` transaction. The account permission set is the
  source of truth for `#klv-*` verification methods; the next resolution
  reflects the change.
- **Update — PQ key rotation.**
  `Identity.rotatePqKey(method_id, new_multicodec, new_pubkey_hash, prev_pubkey_signature)`.
  The previous PQ key co-signs the rotation as defence-in-depth against an
  attacker who has compromised only the classical key. Recovery uses the
  multisig recovery permission per ADR-0001.
- **Update — service endpoints.** `Identity.setServices(services)`.
  Holder-controlled.
- **Deactivate.** `Identity.deactivate()` sets a deactivation flag on the
  Identity Registry record. Subsequent resolutions return a minimal
  tombstone DID Document together with DID Document metadata in which
  `deactivated` is set to `true`. Per W3C DID Core, the deactivation signal
  belongs in `didDocumentMetadata.deactivated`, not as a top-level property
  of the DID Document itself.

Every state-changing operation emits a structured event under the ADR-0002
event model so the off-chain indexer can reconstruct history.

### 6. Privacy considerations

A public chain is not anonymous: anything anchored against a DID is
correlatable forever. The method's privacy properties are pseudonymity, not
anonymity. Mitigations:

- **No PHI on-chain.** No patient identifiers, no human-readable data — an
  architectural invariant from ADR-0002.
- **Opaque service endpoints.** `serviceEndpoint` URLs embed an opaque
  per-DID token; the off-chain service authorises before returning anything.
- **Pairwise DIDs supported.** A holder may hold multiple `did:klever` DIDs
  (one per relationship — e.g. one for primary care, one for a clinical
  trial). Each is its own Klever account. UX cost is real and must be
  weighed product-side.
- **PQ key hashes, not keys.** What is anchored is a 32-byte hash; absent the
  off-chain full key the on-chain record is a commitment, not a reusable
  identifier.
- **Selective disclosure via VCs.** Authorisation grants reveal only what the
  VC scope allows; they do not expose underlying PHI.

Residual exposure: any DID that anchors a PQ key, registers services, or
participates in consent/anchor events is observable. The method does not
attempt to hide that and product surfaces must not promise otherwise.

### 7. Crypto-agility

The method commits to *verification-method slots*, not to specific
algorithms. The classical slot is verified on-chain today; one or more PQ
slots are verified off-chain. The PQ algorithm in use is encoded in the
verification method's multicodec prefix, so new schemes can be added (and
deprecated ones rotated out) without revising this DID method.

When Klever KVM eventually adds an on-chain PQ verifier (ML-DSA is the most
likely candidate), the same anchored PQ verification method becomes eligible
for `capabilityInvocation` as well — no DID migration required.

## Consequences

### Positive

- Long-lived patient data is quantum-resistant from day one. The realistic
  *harvest-now, decrypt-later* threat is addressed where it actually applies.
- The method conforms to W3C DID Core 1.0 and uses **only verified Klever
  primitives** — no dependency on unavailable KVM features.
- Crypto-agility: PQ algorithm choice is parameterised via multicodec, not
  baked into the method.
- Forward-compatible with future KVM PQ verifiers — same DID, no migration.
- On-chain footprint stays at 32-byte commitments regardless of PQ-scheme
  size.

### Negative / risks

- **Larger trusted custody surface.** PQ private keys live in the same
  custody scope as the classical keys; the Signing & Fee Service custody
  story now spans two key types. The threat model and HSM / KMS requirements
  must reflect this.
- **PQ library maturity.** Rust / WebAssembly implementations of NIST PQ
  schemes (e.g. `pqcrypto`, `liboqs-rust`, scheme-specific crates) are less
  battle-tested than `ed25519`. An audit-tier review of the chosen library
  is required before any production use.
- **Multicodec assignments still stabilising.** Some PQ multicodec entries
  are registered, others are in draft. The method profile may need a minor
  revision when these finalise.
- **Operational complexity.** Holders and guardians effectively manage two
  key types. UX, backup, and recovery flows must cover both.
- **Bigger DID Documents.** PQ Multikey entries are larger than Ed25519;
  resolver caching matters more.

## Alternatives considered

- **Pure `did:key` (no chain).** Rejected — `did:key` by spec has no
  rotation, no services, and no deactivation. deEHR needs all three. Useful
  as a primitive for ephemeral interactions, not for a patient's primary DID.
- **`did:web` over a deEHR-controlled domain.** Rejected — anchors trust in
  a TLS-secured server, losing the on-chain integrity that justifies a
  blockchain at all.
- **Bare classical `did:klever` (Ed25519 only).** Rejected — leaves all
  long-lived signed data on a 10- to 15-year quantum timer.
- **PQ-only `did:klever` (PQ as the on-chain signer).** Rejected — Klever
  KVM has no PQ verifier today; PQ keys cannot authorise transactions.
- **Opaque identifier with on-chain mapping
  (`did:klever:<network>:<random-32B>`).** Rejected — adds a storage write
  and a mapping lookup per DID for no real privacy gain on a public chain.
- **Sidetree-style anchored long-form (`did:ion`-style).** Considered —
  would allow self-certifying DIDs with batched anchoring. Rejected for
  Phase 0 as more complex than warranted; revisit if cross-chain portability
  becomes a goal.

## Open questions

These must be resolved (or explicitly deferred) before this ADR can move to
`Accepted`:

- **Which PQ signature scheme to standardise on first** — ML-DSA-65 vs
  Falcon-512 vs an Ed25519 / ML-DSA hybrid composite. Defer to a follow-up
  ADR after evaluating the Rust / WASM library landscape and current CNSA
  2.0 / RNDS guidance.
- **Which PQ KEM** — ML-KEM-768 is the default candidate, subject to the
  same library review.
- **Specific multicodec values** — pin codes for the chosen scheme(s) once
  finalised in the multiformats / W3C DID-extensions registries.
- **Universal-resolver driver hosting** — build internally for Phase 1, or
  contribute upstream to the universal-resolver project for discoverability
  and more eyes on the code.
- **Pairwise-DID UX** — surface multiple-DID-per-holder in Phase 1, or
  defer.
- **Recovery semantics for PQ keys** — must the recovery multisig also hold
  PQ guardian keys, or is a single classical recovery sufficient to rotate
  the PQ key? Trade-off between attack surface and recovery operability.
- **DID Document caching policy** — TTLs and invalidation triggers; relates
  to the off-chain indexer story in ADR-0002.

## References

- W3C **Decentralized Identifiers (DIDs) v1.0** —
  <https://www.w3.org/TR/did-core/>
- W3C **DID Specification Registries** —
  <https://www.w3.org/TR/did-spec-registries/>
- NIST **FIPS 203** (ML-KEM), **FIPS 204** (ML-DSA), **FIPS 205** (SLH-DSA),
  2024.
- NSA **Commercial National Security Algorithm Suite 2.0** (CNSA 2.0),
  2022.
- `did:key` Method Specification —
  <https://w3c-ccg.github.io/did-method-key/>
- Sidetree Protocol / `did:ion` —
  <https://identity.foundation/sidetree/spec/>
- Klever KVM capability verification (2026-05-22) — verified crypto host
  functions (`ed25519`, `secp256k1`, BLS only; no PQ), native weighted
  multisig via `UpdateAccountPermission`.
- [ADR-0001](adr-0001-identity-and-key-management.md) — Identity & Key
  Management — Progressive Custody.
- [ADR-0002](adr-0002-on-chain-registry-design.md) — On-chain Registry
  Design.
