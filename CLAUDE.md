# CLAUDE.md — deEHR

Guidance for Claude Code when working in this repository.

## What this is

**deEHR** — a decentralized Electronic Health Record platform where **patients own
their health data**. FHIR/SMART for interoperability; the **Klever blockchain** anchors
data integrity, consent, identity and credentials. Brazil-first (RNDS integration, LGPD).

**Status:** early — Phase 0 (Foundations) done, Phase 1 (Patient Core) in progress. Most
of the repo is docs/ADRs/scaffolding; the only real code so far is the Rust/WASM
`identity-registry` contract.

## The one hard invariant

> **No PHI on-chain — ever.** Protected Health Information never touches the blockchain.
> Encrypted FHIR resources live **off-chain**; the chain stores only *proofs* — integrity
> hashes, consent receipts, access events, DIDs, credential status. This is enforced in
> code review and audits. Never write a contract, event, or storage field that could carry
> PHI. (ADR-0002 §2.)

## Architecture at a glance

- **Off-chain (Go services):** FHIR Gateway, SMART/OIDC Authorization Server, RNDS
  Connector, Consent Relayer (platform signing & fee service). FHIR R4 server (e.g. HAPI).
  IPFS for encrypted patient-held exports.
- **On-chain (Rust→WASM, Klever KVM):** four registries — `identity-registry` (DIDs),
  `credential-registry` (VC status), `consent-registry` (consent = source of truth for
  authz), `anchor-registry` (FHIR bundle hashes + IPFS CIDs + audit log).
- **Novel piece:** the SMART auth server consults the on-chain Consent Registry before
  minting an OAuth2 token, so issued scopes are provably backed by patient consent.
- **Progressive Custody:** custodial-by-default keys (HSM), passkey login, social recovery
  via Klever weighted-multisig account permissions. Patients never see seed phrases or gas.

## Repository layout

```
contracts/      Rust/WASM Klever smart contracts (workspace) — the only real code today
  identity-registry/   IMPLEMENTED (Ed25519 did:klever MVP). credential/consent/anchor = skeletons
services/       Go backend services (auth-server, fhir-gateway, rnds-connector, consent-relayer) — scaffolds
apps/           patient-web, patient-mobile, provider-portal (later phases) — scaffolds
packages/ tools/ deploy/   shared libs, tooling, Docker/K8s/Terraform — scaffolds
docs/architecture/   ADRs (source of truth for design decisions)
docs/requirements/ docs/security/   requirements + threat model
```

## Build & test (contracts)

Toolchain is installed locally: `ksc` + `koperator` at `~/klever-sdk/`, Rust with
`wasm32-unknown-unknown` target. (Docker/kubectl/Terraform are NOT installed.)

```bash
cd contracts
ksc all build --path identity-registry     # -> output/*.wasm + *.kleversc.json + *.abi.json
cargo test -p deehr-identity-registry       # whitebox tests (build the WASM first — tests load the artifact)
cargo check --workspace                      # validate workspace without producing WASM
```

Contract source of interest: `contracts/identity-registry/src/lib.rs`. It uses `klever-sc`,
`#![no_std]`, edition 2024. State-changing endpoints (`registerDid`/`updateDid`/
`deactivateDid`) are Ed25519 signature-gated with domain separation + per-record nonce
replay protection + SC-address instance binding. Read its
[README](contracts/identity-registry/README.md) before touching it.

## Conventions

- **Commits:** Conventional Commits, scoped — e.g. `feat(contracts): …`, `docs(adr): …`,
  `ci(deps): …`, `chore(security): …`.
- **Branching (ADR-0003):** trunk-based. `main` is the trunk; every change lands via PR.
  Squash/rebase only — **no merge commits**, linear history enforced. Short-lived topic
  branches: `feat/…`, `fix/…`, `docs/…`, `chore/…`. Direct pushes to `main` are blocked.
- **Work tracking:** GitHub Issues (atomic unit) grouped by Milestones = roadmap phases
  (Phase 0–5). The live public **[Projects v2 "deEHR Roadmap" board](https://github.com/users/brunocampos-ssa/projects/1)**
  is the single source of truth, with fields **Status** (Todo/In Progress/Done),
  **Phase** (0–5), **Area** (contracts, services, apps, identity, consent, fhir, rnds,
  security, ci, docs), **Priority** (high/medium/low). Phase 1 (Patient Core) is the
  active milestone.
- **i18n:** canonical docs are **English**; a Brazilian Portuguese mirror uses the
  `.pt-BR` suffix (`README.pt-BR.md`, `docs/pt-BR/…`) and is cross-linked. **If you edit a
  doc that has a `.pt-BR` twin, update both** (or flag the divergence).
- **ADRs are append-only.** Don't rewrite an Accepted ADR — supersede it with a new one.
- **Security is a gate, not a phase.** No PHI on-chain; smart-contract audits (reentrancy,
  overflow, access control, WASM concerns); mandatory security audit before releases. The
  `security-guidance` Claude plugin is enabled for this project.

## CI

Two GitHub Actions workflows on PR/push to `main`: `lint.yml` (markdownlint, actionlint)
and `security.yml` (gitleaks, Trivy fs scan HIGH/CRITICAL, OpenSSF Scorecard). Actions are
SHA-pinned; Dependabot keeps them current.

## Where the design lives (read these before big changes)

- `docs/architecture/README.md` — ADR index (0001–0007).
- ADR-0001 Identity & Key Management (Progressive Custody), 0002 On-chain Registry Design,
  0003 Branching/Release, 0004 `did:klever` method, 0005 FHIR Profile Selection,
  0006 Multi-Consumer FHIR Profiles (Proposed), 0007 Patient Identity Resolution/MPI (Proposed).
- `docs/security/threat-model.md` — living threat model.
- `README.md` — the project's anchor document (full vision, actors, roadmap, tech stack).
