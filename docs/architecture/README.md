# Architecture Decision Records

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
| [0002](adr-0002-on-chain-registry-design.md) | On-chain Registry Design | Proposed |

### Planned

- **ADR-0003** — `did:klever` DID method specification (no standard method
  exists; deEHR will define one).

## Process

1. Copy [`adr-template.md`](adr-template.md) to
   `adr-NNNN-short-title.md` (next free number).
2. Open it as `Proposed` and discuss it in a pull request.
3. Resolve the open questions; merge as `Accepted` once decided.
4. Add it to the index above.

## Documentation policy

Canonical ADRs are written in English. Brazilian Portuguese translations
follow the repository's `.pt-BR` convention and are cross-linked when added.
