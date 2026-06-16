# Identity / DID Registry

The on-chain authority for **`did:klever`** DID Documents — the foundation
contract every other deEHR component addresses actors by. Rust → WebAssembly for
the Klever Virtual Machine (KVM).

- Implements [ADR-0004](../../docs/architecture/adr-0004-did-klever-method.md)
  (the `did:klever` method) and the Identity registry slot in
  [ADR-0002 §3](../../docs/architecture/adr-0002-on-chain-registry-design.md).
- **MVP scope (issue #27): Ed25519 `#klv-1` only.** Post-quantum verification
  methods are deferred per ADR-0004 (a follow-up PQ-profile ADR).

## No PHI on-chain

Per the ADR-0002 §2 invariant, this contract stores **only** commitments and key
state — never PHI: a 32-byte SHA-256 **hash** of the off-chain DID Document, the
Ed25519 verification-method **public key**, key-rotation history, timestamps, a
replay nonce, and a deactivation flag.

## Identity model

A `did:klever` DID's method-specific identifier **is the Klever account address**
(ADR-0004 §1); the account is its own controller. Every state-changing endpoint
therefore keys off `get_caller()` — an account can only touch its own DID record.
`update`/`deactivate` additionally require an Ed25519 signature by the **current**
primary key, giving proof of `#klv-1` key control independent of whatever account
submits the transaction (e.g. the custodial Signing & Fee Service, P1.4).

## Endpoints

| Endpoint | Args | Notes |
| --- | --- | --- |
| `registerDid` | `doc_hash: bytes32`, `primary_key: bytes32` | First write for the caller's DID; fails if already registered. Authorised by the account's own transaction signature. |
| `resolveDid` *(view)* | `did: Address` → `DidRecord` | Panics for an unknown DID. A deactivated DID returns its tombstone record (`deactivated == true`). |
| `updateDid` | `new_doc_hash: bytes32`, `new_primary_key: bytes32`, `signature: bytes64` | Rotates the `#klv-1` key and patches the doc hash under signature proof; pushes the old key to history. |
| `deactivateDid` | `signature: bytes64` | Tombstones the DID under signature proof. The record is **retained** (W3C DID Core: deactivation lives in DID-document metadata, the DID stays resolvable). |
| `keyHistory` *(view)* | `did: Address` → `[KeyRotation]` | Past primary keys with rotation timestamps. |

### Signed-message format

`update`/`deactivate` verify the signature against the current primary key over a
**domain-separated, nonce-bound** message (big-endian `u64` nonce):

```text
update:     0x01 || did(32) || new_doc_hash(32) || new_primary_key(32) || nonce(8)
deactivate: 0x02 || did(32) || nonce(8)
```

- **Domain separation** (`0x01` / `0x02`) stops an `update` signature being
  replayed as a `deactivate`.
- **Nonce binding** — the record's nonce is included and incremented on every
  state change, so a captured signature cannot be replayed.

## Storage layout

| Key | Type | Contents |
| --- | --- | --- |
| `did` + address | `DidRecord` | `doc_hash`, `primary_key`, `created_at`, `updated_at`, `nonce`, `deactivated` |
| `keyHistory` + address | `Vec<KeyRotation>` | `{ previous_key, rotated_at }` per rotation |

## Events (ADR-0002 §7)

`didRegistered`, `didUpdated`, `didDeactivated` — each ≤ 4 topics, addresses as
raw 32-byte buffers, no PHI in topics or data.

## Security properties

- **No PHI on-chain** (hashes / keys / status only).
- **Signature-gated** mutations with **replay protection** (per-record nonce) and
  **domain separation**.
- **Explicit nonce-overflow guards** — the generated WASM release profile sets
  `overflow-checks = false`, so the contract checks `nonce < u64::MAX` itself.
- **Owner-only upgrade** (`UPGRADEABLE` set at deploy, ADR-0002 §9).
- Deactivation is a **tombstone**, not a delete — the DID must stay resolvable.

## Build & test

```bash
# from contracts/
ksc all build --path identity-registry   # -> output/*.wasm + *.kleversc.json + *.abi.json
cargo test -p deehr-identity-registry     # whitebox tests (build first; tests load the artifact)
```

The tests (`tests/identity_registry_whitebox_test.rs`) cover the full lifecycle
plus negative cases — replay, wrong signature, wrong key, unknown DID,
double-register, update-after-deactivate — using synthetic data only.

## Deploy (testnet, manual)

```bash
KEY_FILE=./walletKey.pem ./interaction/deploy_testnet.sh
```

Manual operator step (not CI). Requires the Klever SDK (`ksc`, `koperator`) and a
funded testnet wallet. Record the deployed contract address below after a deploy.

| Network | Address |
| --- | --- |
| testnet | *not yet deployed* |

## Not in this contract (deferred)

- **PQ verification methods** — Phase 3+ (a follow-up PQ-profile ADR).
- **Account-permission-derived `#klv-*` methods** — ADR-0004 §2 derives classical
  verification methods from the account permission set at resolution time; the MVP
  stores the primary key directly on-chain for simplicity.
- **Wiring to the Signing & Fee Service** — P1.4 (#30).
- **`CodeMetadata::DEFAULT` lock-in** — later, per ADR-0002 §9.
