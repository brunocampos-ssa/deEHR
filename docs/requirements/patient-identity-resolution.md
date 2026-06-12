# Patient Identity Resolution, Record Linkage, and the Master Patient Index

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](patient-identity-resolution.pt-BR.md)

- **Status:** Draft (Phase 0 → Phase 1 transition input)
- **Date:** 2026-06-12
- **Seeds:** [ADR-0007](../architecture/adr-0007-patient-identity-resolution.md) (Proposed)

## Provenance

These requirements were assembled from a **follow-up** design-thinking
conversation with the same CTO in the Brazilian insurance market who seeded
[ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.md)
(see the [consumer-profile-heterogeneity requirements](consumer-profile-heterogeneity.md)).
In June 2026 the CTO reviewed the macro architecture diagram and the off-chain
persistence components — **FHIR Gateway**, FHIR Server, IPFS (for exported
documents), and the RNDS connector — and surfaced a concern that sits
*upstream* of ADR-0006: before deEHR can project one person's data into many
profile shapes, it must first be able to decide that two incoming records
**are the same person at all**. ADR-0006 solves profile heterogeneity;
it assumes patient identity is already resolved. It is not.

Identity withheld; the substantive points are captured here in deEHR's own
framing.

## Problem statement

The actor authenticated at the SMART layer is **not** the subject of the data.
Several intersecting realities follow:

1. **Requester identity ≠ subject identity.** SMART on FHIR (ADR-0005's
   adopted auth pattern) authenticates *who is asking* — a patient, a
   hospital, an insurer — and carries the *scopes* that requester is entitled
   to. It does **not** identify *whose record* is being written. A hospital
   holding a valid consent grant can call the API to persist
   *Francisco José's* prontuário; the requester is the hospital, the subject
   is the patient.
2. **No shared patient primary key across sources.** Multiple hospitals,
   laboratories, and clinics persist data asynchronously, each under **its own
   FHIR profile** (per ADR-0006), and **none of them carries deEHR's patient
   key**. The patient's primary key is **not present in the SMART token**
   either — the token authorizes the requester, it does not name the subject's
   canonical identity in deEHR.
3. **Atomic FHIR writes default to "create."** Because every FHIR REST call is
   atomic and each source's demographics differ, a naïve write path issues a
   `POST` (create) every time. Two sources writing about the same real person
   produce **two `Patient` resources** — a split record. Fragmented clinical
   histories are a patient-safety risk, not merely a data-quality nuisance.
4. **Resolution is probabilistic, not exact.** Sources disagree on which
   fields are mandatory and how values are coded (the very heterogeneity
   ADR-0006 addresses). Matching "is this the same person?" cannot rely on an
   exact key match; it requires a **Master Patient Index (MPI)** that scores
   demographic similarity and returns a graded decision.
5. **Persistence must be match-first.** The write path must resolve identity
   *before* it persists: a confirmed match must **aggregate** onto the
   existing record (`PUT` / update against the resolved key) rather than
   **create** a duplicate (`POST`). The resolved canonical record is the
   patient's **Golden Record**.

deEHR's existing artefacts do not model any of this. ADR-0001 covers
*requester* identity (progressive-custody patient accounts, `did:klever`
DIDs). ADR-0005 / ADR-0006 cover *profile* heterogeneity. **No artefact covers
*subject*-identity resolution across heterogeneous, keyless, asynchronous
source writes.**

## Sources in scope

The following source classes drive the requirements. Note these are *writers
of records about a patient*, distinct from the *consumers* table in the
[consumer-profile-heterogeneity](consumer-profile-heterogeneity.md) doc —
though many actors are both.

| Source class | Identifies the patient by | Key reliability |
| --- | --- | --- |
| Hospital networks | EHR-internal MRN + demographics; sometimes CNS | Local MRN unique only within the source domain |
| Laboratories / clinics | Order demographics; often partial | Frequently no national identifier |
| National backbone (RNDS) | CNS (Cartão Nacional de Saúde) | High when present; not always present |
| Private insurers | Plan-membership ID + demographics | Carrier-scoped, not a clinical identity |
| Patient-direct apps | The patient's own deEHR account / DID | Authoritative for that patient |

Brazilian context: **CPF** and **CNS** are candidate strong identifiers but
are neither universally present nor guaranteed unique/clean in source data;
they are *evidence*, not a guaranteed primary key.

## Key scenarios

### UC-1: Source persists a record for a patient identified only by demographics

A hospital holding a valid consent grant `POST`s a `Patient` (and an attached
clinical Bundle — see ADR-0006 UC-2) carrying name, date of birth, mother's
name, and a local MRN, but no deEHR patient key. The system MUST resolve the
demographics against the MPI **before** persistence and bind the write to the
resolved Golden Record rather than minting a new patient identity by default.

### UC-2: Probabilistic match returns a "possible match" → steward review

An incoming `Patient` scores in the ambiguous band (above the no-match
threshold, below the auto-match threshold). The system MUST NOT silently merge
or silently duplicate. It MUST route the candidate to a **data-steward queue**
for human adjudication, with the comparison evidence (which fields matched,
which conflicted, the score) presented for review, and MUST record the
steward's decision in an audit trail.

### UC-3: Two existing Golden Records are found to be the same person

Two distinct Golden Records (each already accreted from several sources) are
later determined to be one person. The system MUST support a **merge** that
consolidates them under one master identifier while **preserving the source
records and their original identifiers** (link semantics underneath), and the
merge MUST be **reversible** (unmerge) to correct a wrong decision.

### UC-4: Incoming record matches an existing Golden Record → aggregate, not duplicate

An incoming `Patient` matches an existing Golden Record with high confidence.
The persistence operation MUST aggregate onto the existing canonical key
(conditional update / `PUT` semantics) rather than create a second `Patient`
(`POST`). The clinical resources in the same write accrue to the **same
prontuário**.

### UC-5: Consumer invokes `$match` directly

A consumer (e.g., an insurer reconciling its own member to deEHR's patient,
or a hospital checking before a write) calls
`POST /fhir/Patient/$match` with a partial `Patient`. The system MUST return a
searchset `Bundle` of candidate `Patient` resources ranked most-to-least
likely, each entry carrying a **search score (0–1)** and the FHIR
**`match-grade`** extension (`certain` / `probable` / `possible` /
`certainly-not`), honouring `onlyCertainMatches`, `onlySingleMatch`, and
`count`.

### UC-6: Golden Record ↔ patient on-chain identity binding

deEHR patients have a `did:klever` DID (ADR-0001), and consent (ADR-0002) is
keyed by patient DID. When a source writes about a patient who *is* an
onboarded deEHR patient, the MPI MUST be able to resolve incoming demographics
→ Golden Record → **patient DID**, so that the write is governed by the
correct consent grant. When a source writes about a person who is *not* yet an
onboarded patient, the system MUST be able to hold a Golden Record that has no
DID yet and bind a DID later at onboarding without losing accreted history.

### UC-7: Correction of a wrong match (un-link / un-merge)

A previously auto-matched or steward-approved match is later found to be
wrong (two different people were linked). The system MUST support splitting
them back apart, re-deriving each Golden Record, and MUST record the
correction — including which clinical resources move to which record — in the
audit trail.

### UC-8: Identity resolution inside the Bundle write pipeline

A `transaction` Bundle (ADR-0006 UC-2 / §4) contains a `Patient` plus clinical
resources. Identity resolution MUST run on the `Patient` entry **before**
canonical projection and on-chain anchoring, and the resolved master key MUST
feed the Bundle's internal reference rewriting (ADR-0006 §4 step 3) so the
clinical resources attach to the resolved patient. This **amends ADR-0006 §4**.

## Non-functional requirements

- **Match quality.** Configurable auto-match and no-match thresholds with a
  human-review band between them. Concrete precision/recall targets: TBD in
  Phase 1 against a labelled evaluation set; the pipeline MUST make
  false-merge (linking two different people) the costlier error to favour,
  since a false merge cross-contaminates clinical histories.
- **Determinism & auditability of decisions.** Every match decision (auto,
  steward-approved, steward-rejected, merge, unmerge) MUST be reproducible
  from the recorded inputs + algorithm version, and MUST be logged with the
  evidence and the deciding actor.
- **Data-steward workflow.** A queue + review UI for possible-matches, with
  comparison evidence and an audit trail, is a first-class requirement, not an
  afterthought.
- **No PHI on chain.** The MPI operates **entirely off-chain**. Demographic
  comparison data, candidate sets, and steward decisions never touch the
  Klever chain; the on-chain layer continues to see only DIDs, hashes, CIDs,
  and coded status (ADR-0002 §2). The Golden-Record-to-DID binding is the only
  point of contact, and it crosses the boundary as a DID reference, not as PHI.
- **Reversibility.** Link and merge MUST be reversible; the system MUST NOT
  destroy source records on merge.
- **Performance budget for `$match`.** Concrete latency/throughput targets for
  resolution on the write hot path: TBD in Phase 1 benchmarks; resolution sits
  on the critical path of every clinical write, so it shares the latency
  budget with the projection engine (ADR-0006) and the anchor commit
  (ADR-0002 / ADR-0006 §4).
- **Backward compatibility.** Patient-direct writes (a patient writing through
  their own deEHR app, already bound to a DID) MUST short-circuit probabilistic
  matching — the subject identity is known authoritatively.

## Out of scope (for this requirements set)

- **Biometric / fuzzy-image matching.** Demographic and identifier-based
  record linkage only.
- **Cross-jurisdiction identity reconciliation.** Matching a Brazilian patient
  to a foreign national identity system is a sibling-backbone concern, tracked
  with the cross-jurisdiction profile mapping deferred by ADR-0006.
- **CPF/CNS as a sole guaranteed primary key.** Treated as strong evidence in
  the deterministic pass, not as a presumed unique clean key. A national-ID
  trust policy is an open question, not a decided invariant.
- **Patient-merge effects on downstream insurer/research extracts.** How a
  merge re-keys data already delivered to a consumer is deferred.

## Open questions

To be resolved during ADR-0007 review and Phase 1 prototyping:

1. **Build vs integrate the matching engine.** Implement Fellegi-Sunter
   probabilistic linkage in-house (Go/Rust), or integrate an existing
   open-source EMPI? Trade-off: control + no PHI leaving the boundary vs
   time-to-value + battle-tested matching.
2. **National-ID trust policy.** How much weight do CPF and CNS carry in the
   deterministic pass, and what is the policy when they conflict with strong
   demographic disagreement?
3. **Threshold defaults & steward SLA.** Initial auto-match / no-match
   thresholds, the size of the review band, and the operational SLA for the
   steward queue.
4. **Possible-match persistence policy.** On a possible-match, does the write
   block pending steward review, or persist to a **provisional** record that is
   reconciled later? Clinical-safety and availability trade-off.
5. **DID-binding timing.** When does a Golden Record acquire a `did:klever`
   DID — only at patient onboarding, or can a provider-originated Golden Record
   exist DID-less and be claimed later? (Ties to ADR-0001 progressive custody.)
6. **Merge across already-bound DIDs.** If two Golden Records that are *each*
   already bound to a different patient DID turn out to be one person, merging
   them collides two on-chain identities — each potentially with its own
   consent grants and anchors. What is the on-chain consequence, and is this
   even permitted, or must it be a manual high-assurance procedure?
7. **`$match` exposure to external consumers at v1.** Expose `$match` to
   external consumers from day one, or keep it internal to the write pipeline
   for v1 and expose it later?

## Phase 1 implications

This requirements set implies a **second** data-engineering load-bearing
sub-arc for Phase 1, alongside the profile-registry / projection engine from
ADR-0006: the **MPI / identity-resolution pipeline** (normalize → block →
score → link/merge → steward review). The two sub-arcs are coupled — identity
resolution runs *before* projection on the write path — but are distinct
bodies of work, distinct from the on-chain contract MVP and the Signing & Fee
Service. The Phase 1 issue set must include the MPI sub-arc explicitly and
sequence it ahead of (or alongside) the projection engine, since projection
assumes a resolved patient identity.

## References

- [ADR-0007](../architecture/adr-0007-patient-identity-resolution.md) — the
  proposed architectural decision driven by this requirements set.
- [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.md) —
  multi-consumer profile strategy; this set sits upstream of it and amends its
  §4 Bundle write pipeline.
- [ADR-0005](../architecture/adr-0005-fhir-profile-selection.md) — FHIR
  profile selection; SMART on FHIR auth pattern referenced in the problem
  statement.
- [ADR-0002](../architecture/adr-0002-on-chain-registry-design.md) — on-chain
  registry design; the no-PHI invariant and DID-keyed Consent Registry bound
  the MPI's off-chain placement and the Golden-Record-to-DID binding.
- [ADR-0001](../architecture/adr-0001-identity-and-key-management.md) —
  identity & key management; patient `did:klever` DID and progressive custody.
- HL7 **FHIR R4 `Patient/$match` operation** —
  <https://build.fhir.org/patient-operation-match.html>.
- HL7 **FHIR `match-grade` extension** —
  <https://hl7.org/fhir/R4/valueset-match-grade.html>.
- **Health Samurai — Master Patient Index and Record Linkage** —
  <https://www.health-samurai.io/articles/master-patient-index-and-record-linkage>.
- **fastrivertech/fhir-mpi — FHIR-based EMPI interface** —
  <https://github.com/fastrivertech/fhir-mpi>.
- Fellegi, I. P., & Sunter, A. B. (1969). *A Theory for Record Linkage.*
  Journal of the American Statistical Association.
