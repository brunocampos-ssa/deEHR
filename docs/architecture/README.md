# Architecture Decision Records

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](README.pt-BR.md)

This directory holds the Architecture Decision Records (ADRs) for deEHR.

## What is an ADR?

An ADR captures one significant architectural decision — the context that
forced it, the decision itself, and the consequences that follow. ADRs make the
*why* behind the architecture explicit and reviewable.

ADRs are **append-only**. Once an ADR is `Accepted` it is not rewritten; if the
decision later changes, a new ADR is written that supersedes it, and the old
one is marked `Superseded by ADR-XXXX`.

## Status values

- **Proposed** — under discussion; open questions remain.
- **Accepted** — decided and in effect.
- **Deprecated** — no longer relevant, not yet replaced.
- **Superseded by ADR-XXXX** — replaced by a later decision.

## Index

| ADR | Title | Status |
| --- | --- | --- |
| [0001](adr-0001-identity-and-key-management.md) | Identity & Key Management — Progressive Custody | Accepted |
| [0002](adr-0002-on-chain-registry-design.md) | On-chain Registry Design | Accepted |
| [0003](adr-0003-branching-and-release-model.md) | Branching & Release Model — Trunk-based Development | Accepted |
| [0004](adr-0004-did-klever-method.md) | `did:klever` DID Method — Hybrid Classical / Post-Quantum | Proposed |
| [0005](adr-0005-fhir-profile-selection.md) | FHIR Profile Selection — R4 baseline, RNDS-compatible, SMART v2 | Accepted |
| [0006](adr-0006-multi-consumer-profile-strategy.md) | Multi-Consumer FHIR Profile Strategy — Registry, Dynamic Projection, Bundle Atomicity | Proposed |
| [0007](adr-0007-patient-identity-resolution.md) | Patient Identity Resolution & Master Patient Index — Match-First Persistence, Golden Record | Proposed |

## Process

1. Copy [`adr-template.md`](adr-template.md) to
   `adr-NNNN-short-title.md` (next free number).
2. Open it as `Proposed` and discuss it in a pull request.
3. Resolve the open questions; merge as `Accepted` once decided.
4. Add it to the index above.

## Documentation policy

Canonical ADRs are written in English. Brazilian Portuguese translations
follow the repository's `.pt-BR` convention and are cross-linked when added.
