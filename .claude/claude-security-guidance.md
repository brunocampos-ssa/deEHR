# Security guidance — deEHR

deEHR is a patient-owned EHR: a Go backend (FHIR gateway, SMART/OAuth
authorization server, Signing & Fee Service, RNDS connector, chain indexer),
Rust→WASM Klever smart contracts (identity / consent / anchor / credential
registries), and patient/institution apps. It handles **PHI** — LGPD sensitive
personal data. The rules below are deEHR-specific; apply them in addition to
the standard vulnerability checklist.

Crown-jewel assets: custodied patient signing keys, envelope data-encryption
keys, on-chain consent records (the authorization source of truth), and PHI at
rest. Highest-value internal target: the **Signing & Fee Service / HSM**. Full
model: `docs/security/threat-model.md`.

## Hard invariants — a violation is always a finding

- **No PHI on-chain, ever.** Anything written to a contract or any chain-bound
  payload may carry ONLY: SHA-256 hashes, DIDs, IPFS CIDs, coded
  scope/purpose-of-use/status enums, timestamps/expiries. NEVER free text,
  names, CPF/CNS, dates of birth, MRNs, or any identifying/clinical field.
  On-chain data is public and immutable — it cannot be erased (LGPD), so a leak
  is irreversible. Flag any new contract storage field, event topic/arg, or
  cross-contract argument that could carry such data.
- **No real PHI in the repo.** Tests, fixtures, and seeds use synthetic data
  only. Flag realistic CPF/CNS, patient names, DOBs, MRNs, or clinical free
  text in code, tests, or docs.
- **Salt/pepper low-entropy hashes.** Never hash a small enumerable field (a
  CPF, a status code) directly for on-chain commitment — it is brute-forceable.

## Consent & authorization — deny-by-default

- The authorization server MUST query the on-chain **Consent Registry** before
  minting any token. No token without an active, verified grant.
- Issued scope = **intersection** of requested scope ∧ active on-chain consent.
  Never the union, never "requested as-is." Flag any scope widening.
- Revocation must be reflected before token issuance; prefer short token
  lifetimes and direct-chain reads for critical authorization. Flag authz
  decisions that trust a possibly-lagging indexer for a stale read.
- Consent approval requests carry a nonce + expiry (replay protection).

## Signing & Fee Service / key custody — highest-value target

- Private/account keys live in an HSM/KMS and **never leave that boundary**:
  never export, log, serialize, base64, write to disk, or return a key in a
  response or error. Signing happens inside the HSM.
- Every signing request is gated by a fresh, verified patient authentication
  and protected against replay (nonce/expiry, service-to-service request
  signing).
- Treasury / KDA pool: enforce per-account rate limits and quotas. Flag any
  unbounded or unauthenticated path that can trigger fee-bearing transactions
  (spam → treasury drain → DoS).
- Separation of duties / dual control for sensitive ops; audit events are
  append-only and externally shipped. Flag code letting one actor both
  authorize and execute a sensitive signing operation.

## Klever smart contracts (Rust/WASM)

- Writes are public but **must be signature-gated**: verify the caller controls
  the subject (Ed25519 `#klv-1` for the MVP). Flag missing
  signature/authorization checks on any state change.
- **Replay protection** on signed operations (nonce or per-key sequence). Flag
  update / rotate / deactivate without anti-replay.
- Upgrade is **owner-only**; deploy with `UPGRADEABLE` set explicitly; any
  storage-layout change needs versioned `#[upgrade]` migration logic.
- Use checked/safe arithmetic — never silently wrap balances, counts, weights.
- No `panic!` / `unwrap` / `expect` on attacker-influenced input; a WASM trap is
  a DoS. Handle errors explicitly.
- Events follow ADR-0002 §7: ≤ 4 topics, a single struct data payload,
  addresses as raw 32-byte buffers. No PHI in topics or data.
- Clear (delete) revoked/expired records so the storage refund is claimed.

## Identity / did:klever

- Key rotation and DID-Document changes are authorized by the controlling
  account only; on-chain history must stay tamper-evident.
- PQ keys are anchored on-chain as a **32-byte hash commitment only** — never
  the full PQ public key. Flag any full-key on-chain write.
- Service endpoints use opaque per-DID tokens, never PHI or human-readable
  identifiers. Deactivation belongs in `didDocumentMetadata.deactivated`.

## PHI confidentiality in services (Go / apps)

- Envelope encryption at rest; per-record data keys; encryption keys stored
  separately from ciphertext. Flag any plaintext PHI persistence.
- **No PHI in logs, errors, traces, metrics, or analytics** at any level. Flag
  logging of patient identifiers, FHIR resource bodies, tokens, or keys.
- FHIR Gateway responses are constrained to granted scope with field-level
  filtering; verbose errors must not leak PHI or internal detail.
- Verify integrity hashes on read (Anchor Registry); flag read paths that skip
  verification.

## MPI / identity resolution (ADR-0007)

- **A false merge (linking two different people) is the costliest error** —
  bias matching toward avoiding it, and every merge must be reversible (never
  destroy source records).
- PHI comparison for matching stays server-side; only a DID reference crosses
  to the chain.

## RNDS connector

- ICP-Brasil certificates live in an HSM/secret manager, scoped to the
  connector, with access logged. Flag certificates in code, config, or env.
- Minimal-necessary disclosure to RNDS, via the RNDS FHIR profiles only.

## Secrets & supply chain

- No secrets, credentials, or tokens committed anywhere (gitleaks runs in CI —
  still flag in-session).
- GitHub Actions are SHA-pinned with `# vN` comments; scrutinize any
  workflow-permission or `pull_request_target` change.
