# ADR-0007: Patient Identity Resolution & Master Patient Index — Match-First Persistence and the Golden Record

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](adr-0007-patient-identity-resolution.pt-BR.md)

- **Status:** Proposed
- **Date:** 2026-06-12
- **Deciders:** deEHR maintainers

## Context

[ADR-0006](adr-0006-multi-consumer-profile-strategy.md) lets any consumer
read and write deEHR data shaped to its own FHIR profile. It solves *profile
heterogeneity* — many legitimate shapes of the same logical resource — but it
**assumes the patient's identity is already resolved**. A
[follow-up review](../requirements/patient-identity-resolution.md) with the
same Brazilian-insurance-market CTO who seeded ADR-0006 surfaced the problem
that sits *upstream* of profile projection: deciding that two incoming records
**are the same person**.

Forces shaping the decision:

- **Requester identity ≠ subject identity.** The SMART on FHIR layer (ADR-0005)
  authenticates the *requester* (a hospital, a lab, an insurer, the patient)
  and carries the *scopes* it is entitled to. It does **not** name the
  *subject* of the data. A hospital with a valid consent grant persists
  *another person's* prontuário. The patient's canonical key is **not in the
  SMART token**.
- **No shared primary key across sources.** Sources write asynchronously, each
  under its own profile (ADR-0006), and none carries deEHR's patient key.
  In the Brazilian context **CPF** and **CNS** (Cartão Nacional de Saúde) are
  strong-but-imperfect identifiers — often absent, sometimes dirty, not
  guaranteed unique in source data. They are evidence, not a guaranteed key.
- **Atomic FHIR writes default to create.** Every FHIR REST call is atomic; a
  naïve write path `POST`s a new `Patient` each time, producing duplicate
  records for one real person — a split clinical history and a patient-safety
  hazard.
- **Resolution is probabilistic.** Because sources disagree on mandatory
  fields and value-set codings, "same person?" must be scored, not exact-keyed.
  This is the classic **Master Patient Index (MPI)** / record-linkage problem.
- **No PHI on chain (ADR-0002 §2).** The on-chain layer stores only DIDs,
  hashes, CIDs, and coded status. Any component that compares demographics
  must live entirely off-chain.
- **Consent is keyed by patient DID (ADR-0002 §5).** The Consent Registry — the
  authorization source of truth — is keyed by the patient's `did:klever` DID
  (ADR-0001). For a provider write to be governed by the right consent grant,
  incoming demographics must resolve to the right patient DID. The MPI is the
  only thing that can bridge *demographics → canonical patient → DID*.
- **HL7-canonical first.** FHIR already standardizes the resolution interface
  via the `Patient/$match` operation and the `match-grade` extension. deEHR
  adopts the standard rather than inventing a parallel mechanism — consistent
  with ADR-0005 / ADR-0006.
- **Data-engineering load.** Like the projection engine (ADR-0006), the MPI is
  a load-bearing data-engineering sub-system. Phase 1 needs a sub-arc for it.

## Decision

### 1. Master Patient Index as a first-class off-chain component

deEHR operates an **MPI** as a dedicated off-chain component in front of the
canonical FHIR store. It maintains, per real person, a **Golden Record** keyed
by a deEHR-issued **master patient identifier**, and it preserves each
contributing source's **local identifier** as a cross-reference. The MPI
links source records to one master identity; it does not discard source
identifiers. The MPI holds PHI (demographics) and therefore lives wholly
off-chain (ADR-0002 §2); it never writes demographic data to the chain.

### 2. Hybrid matching: deterministic pass, then probabilistic

Resolution runs a two-stage pipeline per the standard MPI shape
(normalize → block → score → link/merge):

1. **Normalization.** Incoming demographics are standardized (name casing,
   accents, date formats, identifier formatting) before comparison.
2. **Blocking.** Candidate Golden Records are pre-selected by weak criteria
   (e.g., soundex of surname + birth year) to bound comparison cost.
3. **Deterministic pass.** Strong-identifier rules (e.g., CNS match, or
   CPF + date-of-birth) yield high-confidence matches directly.
4. **Probabilistic pass.** For everything else, a **Fellegi–Sunter**
   weighted-evidence classifier scores all available fields — high-uniqueness
   fields (CNS, CPF, date of birth) carry more weight — against
   **configurable thresholds** producing one of three outcomes:
   - **auto-match** (above the upper threshold),
   - **possible-match** (between thresholds → human review, §6),
   - **no-match** (below the lower threshold).

Thresholds, field weights, and the algorithm version are configuration, not
code, and every decision records the version used (§7). The pipeline is tuned
to make a **false merge** (linking two different people) the costlier error,
because a false merge cross-contaminates clinical histories.

The choice of *building* Fellegi–Sunter in-house versus *integrating* an
existing EMPI is an open question (§ Open questions Q1); this ADR fixes the
*interface and semantics*, not the matching implementation — mirroring how
FHIR `$match` itself "deliberately avoids prescribing specific algorithms."

### 3. `Patient/$match` is the standard resolution interface

deEHR exposes `POST /fhir/Patient/$match` per the FHIR R4 operation:

- **Input:** `resource` (a possibly-partial `Patient`), `onlySingleMatch`,
  `onlyCertainMatches`, `count`.
- **Output:** a searchset `Bundle` of candidate `Patient` resources ranked
  most-to-least likely, each entry carrying a **search score (0–1)** and the
  **`match-grade`** extension (`certain` / `probable` / `possible` /
  `certainly-not`).

`$match` is used both **internally** by the write path (§4) and, subject to
open question Q7, **externally** by consumers reconciling their own members to
deEHR patients.

### 4. Match-first persistence: conditional create vs update

Every `Patient` write resolves identity **before** persisting:

- **auto-match** → the write is bound to the matched Golden Record and
  **aggregated** onto it (conditional update / `PUT` semantics). Clinical
  resources in the same write accrue to the same prontuário.
- **no-match** → a **new** Golden Record is created (`POST` semantics) with a
  freshly issued master identifier.
- **possible-match** → routed to the data-steward queue (§6). The write does
  not silently duplicate and does not silently merge.

A **patient-direct write** (a patient writing through their own deEHR app,
already bound to a DID — ADR-0001) **short-circuits** probabilistic matching:
the subject identity is known authoritatively.

### 5. Link by default; merge and unmerge are explicit and reversible

The MPI **links** source records to a master identity by default, preserving
every source record and its original identifier (reversible). **Merge**
(consolidating two Golden Records) and **unmerge** (splitting a wrong match)
are **explicit, reversible operations** with full audit (§7). deEHR never
destroys source records on merge; the merge consolidates the *master view*
while the underlying source records remain individually addressable.

### 6. Data-steward workflow for possible-matches

Possible-matches are routed to a **data-steward queue** with a review surface
that presents the comparison evidence — which fields matched, which conflicted,
the score — and the candidate Golden Records. The steward's decision (approve
match / reject / merge / unmerge) is recorded in the audit trail (§7). This
workflow is a first-class deliverable, not an operational afterthought.

### 7. Auditability of every identity decision

Every resolution decision — auto-match, no-match, steward-approved,
steward-rejected, merge, unmerge — is logged off-chain with: the input
demographics reference, the candidate set, the score, the algorithm + config
version, the outcome, and the deciding actor (the engine or the steward).
Decisions are reproducible from the recorded inputs + version. The off-chain
data-access audit log composes with ADR-0002 §6's Anchor & Audit Registry:
the *fact* of an access/anchor remains on-chain; the demographic *evidence* of
a match decision stays off-chain.

### 8. Golden Record ↔ patient DID binding

The MPI master identifier maps to the patient's `did:klever` DID (ADR-0001)
when the person is an onboarded deEHR patient. Resolution therefore bridges
**incoming demographics → Golden Record (master id) → patient DID**, which is
what lets a provider write be governed by the correct DID-keyed consent grant
(ADR-0002 §5). A provider-originated Golden Record **may exist DID-less** and
be bound to a DID later at patient onboarding **without losing accreted
history**. The binding crosses the on/off-chain boundary as a DID reference
only — never as PHI.

### 9. Amendment to ADR-0006 §4 (Bundle write pipeline)

ADR-0006 §4's Bundle write pipeline is amended to insert an
**identity-resolution step** before canonical projection:

1. Bundle-level validation (ADR-0006 §4.1) — unchanged.
2. **Identity resolution (new).** The `Patient` entry is resolved via the MPI
   (§2–§4). The resolved master identifier feeds reference rewriting.
3. Canonical projection (ADR-0006 §4.2) — unchanged.
4. Reference rewriting (ADR-0006 §4.3) — now resolves the Bundle's internal
   `Patient` reference to the **resolved master identifier**, so clinical
   resources attach to the correct existing patient.
5. Atomic persistence + on-chain anchor (ADR-0006 §4.4–4.5) — unchanged.

If identity resolution returns a possible-match, the Bundle follows the
possible-match policy (open question Q4): block pending steward review, or
persist against a provisional record. Either way, atomicity (ADR-0006 §4) is
preserved — the Bundle is fully committed or fully rolled back.

## Consequences

### Positive

- **One person, one record.** The Golden Record prevents the split-history
  patient-safety hazard that naïve atomic writes create.
- **Provider writes are governable.** Resolving demographics → DID lets
  DID-keyed consent (ADR-0002) gate writes about a patient by a third-party
  requester — closing the requester-≠-subject gap.
- **HL7-canonical surface.** `$match` + `match-grade` are FHIR R4 standards;
  deEHR adopts rather than invents, consistent with ADR-0005 / ADR-0006.
- **Composes with ADR-0006.** Identity resolution slots cleanly ahead of
  projection; the two data-engineering sub-arcs share a coherent write path.
- **No PHI on chain preserved.** The MPI is wholly off-chain; the only
  boundary crossing is a DID reference.

### Negative / risks

- **False-merge is a clinical-safety risk.** Linking two different people
  cross-contaminates histories. Thresholds must favour avoiding it, and merges
  must be reversible — but the risk is intrinsic to probabilistic matching.
- **Resolution on the write hot path.** `$match` runs on every clinical write,
  adding latency before projection and anchor. Phase 1 must benchmark it within
  the shared write-path budget.
- **Steward operations are ongoing cost.** The possible-match queue needs human
  operators and an SLA; it is a permanent operational function.
- **DID-merge collision is genuinely hard.** Merging two Golden Records that
  are *each* already bound to a different DID collides two on-chain identities
  with their own consents and anchors (open question Q6). This is the sharpest
  cross-layer risk introduced.
- **Second data-engineering investment.** The MPI is a substantial sub-system
  on top of the ADR-0006 projection engine. Phase 1 scope must own both.

## Alternatives considered

- **Deterministic-only matching (CPF/CNS as the key).** Rejected — national
  identifiers are too often absent or dirty in source data; relying on them
  alone produces both duplicates (missing key) and false merges (shared/typo'd
  key).
- **Trust each source's local identifier as the patient key.** Rejected — local
  MRNs are unique only within a source domain; the same person has different
  MRNs at every hospital.
- **Client-side deduplication (consumers resolve identity).** Rejected — pushes
  PHI-comparison and the safety-critical decision onto consumers, and a single
  consumer cannot see the cross-source population the MPI sees.
- **Merge-only, no link (destroy source records on consolidation).** Rejected —
  irreversible; a wrong merge becomes unrecoverable and source provenance is
  lost.
- **Put resolution after persistence (dedup as a batch job).** Rejected —
  leaves duplicates live in the clinical store between write and dedup, and
  anchors the duplicate on-chain (ADR-0002) before it is resolved.
- **Invent a deEHR-proprietary match API.** Rejected — FHIR `$match` already
  standardizes the contract; inventing one breaks the HL7-canonical posture.

## Open questions

Must be resolved before this ADR moves from `Proposed` to `Accepted`:

1. **Build vs integrate the matching engine.** In-house Fellegi–Sunter in
   Go/Rust (full control, no PHI leaves the boundary) vs an existing
   open-source EMPI (faster, battle-tested). Decide in Phase 1 prototyping.
2. **National-ID trust policy.** Weight of CPF/CNS in the deterministic pass,
   and behaviour when a strong identifier matches but demographics strongly
   disagree (and vice versa).
3. **Threshold defaults & steward SLA.** Initial auto-match / no-match
   thresholds, the review-band width, and the steward-queue operational SLA.
4. **Possible-match persistence policy.** Block the write pending steward
   review, or persist to a provisional record reconciled later? Clinical-safety
   vs availability trade-off — interacts with ADR-0006 §4 Bundle atomicity.
5. **DID-binding timing.** Can a provider-originated Golden Record exist
   DID-less and be claimed at onboarding, or is a DID required up front? Ties to
   ADR-0001 progressive custody.
6. **Merge across already-bound DIDs.** On-chain consequence of merging two
   Golden Records each bound to a different patient DID (each with its own
   consents/anchors). Permitted automatically, or a manual high-assurance
   procedure only? May require an ADR-0001 / ADR-0002 addendum.
7. **`$match` external exposure at v1.** Expose `$match` to external consumers
   from day one, or keep it internal to the write pipeline for v1?
8. **Matching evaluation set.** Source of a labelled BR-representative dataset
   to tune and regression-test precision/recall without using production PHI.

## References

- [Requirements: Patient Identity Resolution](../requirements/patient-identity-resolution.md)
  — the requirements this ADR addresses.
- [ADR-0006](adr-0006-multi-consumer-profile-strategy.md) — multi-consumer
  profile strategy; this ADR sits upstream of it and amends its §4 Bundle write
  pipeline.
- [ADR-0005](adr-0005-fhir-profile-selection.md) — FHIR profile selection;
  SMART on FHIR auth pattern.
- [ADR-0002](adr-0002-on-chain-registry-design.md) — on-chain registry design;
  no-PHI invariant (§2) and DID-keyed Consent Registry (§5) bound the MPI's
  off-chain placement and the Golden-Record-to-DID binding. The Anchor & Audit
  Registry (§6) composes with the off-chain match-decision audit (§7).
- [ADR-0001](adr-0001-identity-and-key-management.md) — identity & key
  management; patient `did:klever` DID and progressive custody.
- HL7 **FHIR R4 `Patient/$match` operation** —
  <https://hl7.org/fhir/R4/patient-operation-match.html>.
- HL7 **FHIR `match-grade` extension** —
  <https://hl7.org/fhir/R4/valueset-match-grade.html>.
- **Health Samurai — Master Patient Index and Record Linkage** —
  <https://www.health-samurai.io/articles/master-patient-index-and-record-linkage>.
- **fastrivertech/fhir-mpi — FHIR-based EMPI interface** —
  <https://github.com/fastrivertech/fhir-mpi>.
- Fellegi, I. P., & Sunter, A. B. (1969). *A Theory for Record Linkage.*
  Journal of the American Statistical Association, 64(328), 1183–1210.
