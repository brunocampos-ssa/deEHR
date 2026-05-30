# ADR-0006: Multi-Consumer FHIR Profile Strategy — Registry, Dynamic Projection, and Bundle Atomicity

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](adr-0006-multi-consumer-profile-strategy.pt-BR.md)

- **Status:** Proposed
- **Date:** 2026-05-30
- **Deciders:** deEHR maintainers

## Context

[ADR-0005](adr-0005-fhir-profile-selection.md) locks deEHR's internal data
model to FHIR R4 + deEHR-canonical profiles and translates to BR-Core /
RNDS-Principal at the boundary through purpose-built connectors. That ADR's
"two-tier profile" pattern was designed for a small, known set of national
backbones.

A design-thinking review with a CTO in the Brazilian insurance market (see
[requirements doc](../requirements/consumer-profile-heterogeneity.md))
surfaced three concerns that ADR-0005 does not address:

1. **Profile heterogeneity beyond regulators.** Every commercial consumer —
   each insurer, each hospital network — defines its own profile. The
   "design-time hardcoded connector" model in ADR-0005 does not scale to a
   marketplace of consumers.
2. **Bundles as the atomic unit of clinical reality.** A consultation is a
   FHIR `Bundle` of multiple resources, not a sequence of independent resource
   writes. ADR-0005 mentions Bundles only in passing; ADR-0002's on-chain
   anchoring semantics were not designed around Bundle granularity.
3. **Dynamic profile projection for patient-centric storage.** When the
   patient is the root of the data, each consumer reading that data needs to
   receive it shaped to *their* profile, declared by the consumer at request
   time — not picked from a small fixed catalog.

Forces shaping the decision:

- **Compatibility with ADR-0005.** The two-tier pattern is correct in spirit;
  this ADR generalizes it without contradicting any of ADR-0005's per-resource
  decisions, scope manifest, or purpose-of-use accept-list.
- **Compatibility with ADR-0002.** Bundle atomicity must align with the
  on-chain anchor commit model: a Bundle is one transaction, one anchor.
- **PHI containment.** A profile transformation MUST NOT become a side channel
  that leaks data across the SMART scope boundary granted to a token.
- **Data-engineering load.** The validation + projection pipeline is the
  largest non-cryptographic workload in the platform. Phase 1 needs a sub-arc
  for it, distinct from the on-chain contract MVP and the Signing & Fee
  Service.
- **HL7-canonical first.** FHIR R4 §3.2.0.4 already defines the profile
  negotiation contract (`Accept-Profile` / `Content-Profile` HTTP headers).
  deEHR adopts the standard rather than inventing a parallel mechanism.

## Decision

### 1. Profile Registry

deEHR maintains an internal **Profile Registry** — the authoritative catalog
of every FHIR profile the platform recognizes. Each entry records:

- **canonical URL** — the profile's `StructureDefinition.url` value (e.g.,
  `https://hl7.org.br/fhir/core/StructureDefinition/br-core-patient`).
- **resource type** — the FHIR resource the profile constrains.
- **status** — `active`, `deprecated`, or `superseded` (one-way transitions).
- **read-support** — whether the profile may be requested via `Accept-Profile`.
- **write-support** — whether the profile may be supplied on a write.
- **validator binding** — reference to the StructureDefinition + value-set
  bindings used at validation time.
- **transformation binding** — reference to the projection/reverse-projection
  rules used at runtime (see §3).
- **jurisdiction** — informational tag (e.g., `BR`, `US`, `UV`/universal).
- **provenance** — link to the governance artefact (ADR addendum, profile-add
  PR) that admitted the profile.

The registry is seeded at Phase 1 launch with: deEHR-canonical (every
resource in ADR-0005 §3), BR-Core (every resource ADR-0005 maps to it), and
the RNDS-Principal workflow profiles active under ADR-0005. IPS adoption
scope is tracked separately — see open question #7.

The existing BR-Core / RNDS-Principal connector mappings from ADR-0005 are
re-expressed as Profile Registry entries with no semantic change. ADR-0005's
per-resource decisions and conformance posture (§3, §4, §8) remain in force.

### 2. Consumer Profile Negotiation

deEHR adopts the FHIR R4 standard profile-negotiation contract:

- **Read.** A client sends `Accept-Profile: <canonical-url>` on a `GET`. The
  server returns the resource projected to that profile and sets
  `Content-Profile: <canonical-url>` on the response. If the requested
  profile is not in the Registry, the server returns `406 Not Acceptable`
  with an `OperationOutcome` listing supported profiles for the resource
  type.
- **Write.** A client sends `Content-Profile: <canonical-url>` on a `POST` /
  `PUT`. The server validates against that profile (§5) and persists the
  deEHR-canonical projection.
- **Default.** Absent `Accept-Profile`, reads return the deEHR-canonical
  shape. Absent `Content-Profile`, writes are assumed to be deEHR-canonical
  and validated as such.
- **Advertisement.** The `/fhir/metadata` `CapabilityStatement` declares, for
  every resource type, the full list of `supportedProfile` URLs plus a deEHR
  extension on each `rest.resource.profile` entry indicating
  `read-supported` and `write-supported`.

SMART scope syntax is not extended to carry profile information. ADR-0005's
scope manifest stands unchanged. Profile selection is an orthogonal HTTP-level
concern; a single SMART grant can read or write multiple profile shapes of
the same resource type if entitled.

### 3. Dynamic Projection Engine

A **Projection Engine** sits between the FHIR REST API and the canonical
store. It is bidirectional:

- **Read path.** deEHR-canonical resource (from storage) → consumer profile
  (per `Accept-Profile`). The engine applies the profile's read-projection
  rules: omit fields the consumer profile does not include, slice extensions
  the profile constrains, re-bind value sets per the profile's terminology
  bindings, and assert cardinality (e.g., fail-closed if the profile mandates
  a field deEHR-canonical has not populated).
- **Write path.** Incoming resource shaped to a consumer profile (per
  `Content-Profile`) → deEHR-canonical. The engine validates the input
  against the consumer profile, then applies the profile's
  reverse-projection rules to derive the canonical resource.

Projections are declarative. Each Profile Registry entry binds to a
transformation specification — at Phase 1 implementation, the simplest
viable form is a FHIRPath-based mapping document; FHIR Mapping Language
(FHIRPath / StructureMap) is the longer-term target.

**Caching.** Projected reads are cached by the tuple `(resource id,
profile canonical url, resource version)`. Cache invalidation is
write-driven: on any resource update, all projection cache entries keyed by
that resource id are invalidated. Cache backend is implementation-level and
not fixed by this ADR.

### 4. Bundle Write Atomicity

deEHR processes FHIR transaction Bundles (`Bundle.type = transaction`)
atomically. The full pipeline:

1. **Bundle-level validation.** Each `Bundle.entry.resource` is validated
   against its declared profile (per `meta.profile` on each entry, or
   defaulted to deEHR-canonical). Any single validation failure rejects the
   entire Bundle with `400 Bad Request` + an `OperationOutcome` enumerating
   every failure, including the entry index.
2. **Canonical projection.** Each validated entry is projected to the
   deEHR-canonical resource shape (§3 write path).
3. **Reference rewriting.** Internal `urn:uuid:` references between Bundle
   entries are resolved to deEHR-issued resource IDs in a single pass before
   persistence, per FHIR R4 Bundle §3.3.1.
4. **Atomic persistence.** Resources are persisted to off-chain storage in
   a single durable batch — either every resource lands or none does.
5. **On-chain anchor commit.** A single anchor transaction is submitted to
   the Klever chain. The anchor payload is a **merkle root** over the
   per-resource canonical hashes plus the Bundle metadata hash. Anchor
   commit is conditional on §4-step-4 success; off-chain persistence is
   final only after the anchor transaction is confirmed.

The merkle-root structure (rather than a single hash over the serialized
Bundle) is chosen so that a consumer can later prove inclusion of a single
resource in a Bundle without disclosing the rest. This refines
[ADR-0002](adr-0002-on-chain-registry-design.md) §6, which specifies the
Anchor & Audit Registry as storing "integrity hashes of encrypted FHIR
bundles paired with their IPFS CIDs" without fixing the structure of that
hash. ADR-0002 will require an addendum to declare the merkle-root
structure (over per-resource canonical hashes + Bundle metadata hash) as
the canonical anchor form.

If the on-chain anchor commit fails after off-chain persistence, the
platform retries with bounded backoff (operational policy, not ADR-level);
final-failure mode triggers a compensating off-chain delete + administrative
alert. The Bundle is observably either fully committed (off-chain +
on-chain) or fully rolled back; no intermediate state is visible to
consumers.

### 5. Bundle Read Composition

A `GET` on a Bundle id returns the Bundle's resources, each projected to the
profile requested via `Accept-Profile` (uniform across all entries of the
Bundle). If a consumer requests a profile that is incompatible with one or
more entries (e.g., the consumer's profile mandates a field a Patient
resource in the Bundle lacks), the server responds `406 Not Acceptable` with
an `OperationOutcome` listing the offending entry indexes.

Document Bundles (`Bundle.type = document`) such as Sumário de Alta and RAC
are first-class outputs of this pipeline.

### 6. Cross-profile Validation at Write Time

On any write — Bundle or single-resource — the resource is validated against
**both** the consumer-declared profile (`Content-Profile`) **and** the
deEHR-canonical profile for the resource type. Either failure rejects the
write. Validation failure messages attribute each failure to a specific
profile + constraint id.

### 7. Conformance Reporting

deEHR exposes `POST /fhir/<Resource>/$validate?profile=<url>` per the FHIR
R4 standard validation operation. The implementation runs the same validator
used internally and returns an `OperationOutcome`. Per-profile pass/fail
counts and per-constraint failure distributions are emitted as metrics for
the observability stack.

### 8. PHI Containment

Profile transformation runs inside the authorization boundary, after token
verification and scope evaluation. A transformation MUST NOT widen the
field set a token is entitled to read. If a consumer-declared profile would
require fields outside the token's granted scope, the server returns `403
Forbidden` rather than projecting; the profile request does not act as a
scope-elevation side channel.

### 9. Registry Governance

Adding a profile to the Registry requires a reviewable artefact: either an
ADR addendum or a profile-add PR following a template that captures
provenance, validator binding, and transformation binding. Maintainer
review is required. Profile *removal* is two-stage: `active` →
`deprecated` (no new writes accepted; existing reads keep working) →
`superseded` (no reads; HTTP 410 returned, pointing to the successor
profile). No silent additions, no silent removals.

## Consequences

### Positive

- **Marketplace of consumers becomes possible.** Any consumer can declare
  its profile, register it, and read/write deEHR resources in that shape
  without bespoke connector code.
- **Bundles preserve clinical atomicity.** UC-2 (hospital writes a
  consultation as a Bundle) is the first-class persistence flow, not a
  patched-together sequence of writes.
- **Generalizes ADR-0005's two-tier pattern.** BR-Core / RNDS-Principal
  connectors fit cleanly as Profile Registry entries; the pattern remains
  internally consistent.
- **Schema-on-read flexibility.** The same canonical store serves an
  arbitrary number of consumer-declared shapes.
- **HL7-canonical surface.** `Accept-Profile` / `Content-Profile` and
  `$validate?profile=` are FHIR R4 standards; deEHR adopts rather than
  invents.

### Negative / risks

- **Projection performance.** Schema-on-read transformation can be costly
  under load. Cache invalidation is straightforward, but cold-cache reads
  and writes that touch many cached projections will be the dominant
  latency contributor. Phase 1 must include latency benchmarks against
  realistic Bundle sizes.
- **Registry governance load.** Each new consumer profile is a real piece
  of work: validator binding, transformation rules, regression tests. The
  governance step is a feature, not a bug, but it is operational overhead.
- **Bundle anchor on the critical path.** The merkle-root anchor model
  couples Bundle throughput to chain throughput. Phase 1 must measure
  anchor-commit latency under expected Bundle write rates and define a
  bounded-backoff retry policy.
- **Data-engineering investment.** The Projection Engine + Validator +
  Registry tooling is a substantial sub-system. The CTO's "work for a good
  data engineer" remark is accurate; Phase 1 must scope this work
  explicitly.
- **ADR-0002 addendum required.** The merkle-root anchor structure must
  be declared in ADR-0002. ADR-0002 §6 currently specifies "integrity
  hashes of encrypted FHIR bundles" without fixing the hash structure; the
  addendum refines this to a merkle root over per-resource canonical
  hashes plus Bundle metadata. Non-breaking but non-trivial.
- **Cross-jurisdiction profile mapping deferred.** Profiles that diverge on
  semantic content (e.g., divergent race/ethnicity codings) are out of
  scope for this ADR — flagged in the requirements doc and tracked
  separately.

## Alternatives considered

- **Hardcoded per-consumer connectors (extend ADR-0005 indefinitely).**
  Rejected — does not scale beyond the regulator case ADR-0005 was designed
  for. Each new consumer becomes a code change + release.
- **"One profile to rule them all"** — force every consumer to adopt
  deEHR-canonical. Rejected — commercially non-viable; consumers will not
  reshape their internal data models around a vendor's canonical.
- **Pre-materialized projections per `(resource × profile)`.** Rejected at
  this stage — write amplification scales with the number of registered
  profiles; storage cost grows non-linearly with consumer adoption.
  Reconsiderable as a per-profile opt-in optimization for hot read paths.
- **Client-side projection (consumers project from canonical).** Rejected —
  pushes the validation and projection burden onto consumers, which a
  marketplace of consumers (especially smaller ones) cannot bear; weakens
  PHI-containment guarantees because the server can no longer enforce the
  profile boundary.
- **Single anchor over serialized Bundle (no merkle root).** Cheaper, but
  blocks per-resource proof of inclusion. Rejected for not gaining anything
  meaningful in cost while losing a future audit / portability primitive.
- **FHIR `batch` Bundles in place of `transaction` Bundles.** Rejected —
  `batch` semantics are non-atomic by FHIR specification; the clinical
  atomicity requirement (UC-2) is satisfied only by `transaction`.

## Open questions

Must be resolved before this ADR moves from `Proposed` to `Accepted`:

1. **Transformation language at Phase 1.** FHIRPath-based mapping documents
   for v1, or commit to FHIR Mapping Language (StructureMap) from day one?
   StructureMap is the standard but the tooling maturity in Rust/Go is the
   open variable.
2. **Cache backend.** Implementation-level detail, but at minimum: in-process
   LRU for v1, with an explicit upgrade path to a shared cache (Redis or
   equivalent) when horizontal scale arrives. Settle in Phase 1.
3. **ADR-0002 anchor addendum.** Concrete shape of the merkle-root payload
   (root hash + bundle metadata hash + protocol version byte) and the
   on-chain `anchor` contract method signature. Owned by an ADR-0002
   addendum, not this ADR; this ADR depends on it.
4. **Capability advertisement detail.** Exact JSON shape of the deEHR
   extension on `CapabilityStatement.rest.resource.profile` entries —
   `read-supported` / `write-supported` flags, validator URL, deprecation
   notes. Lock during Phase 1 implementation.
5. **Profile-add governance ceiling.** Is an ADR addendum required for
   every profile, or can a lightweight profile-add PR template stand alone
   once the registry shape is stable? Decide after the Phase 1 v1 set
   lands.
6. **Backwards compatibility window for `deprecated` profiles.** How long
   between `active → deprecated` and `deprecated → superseded`? Default
   minimum 6 months unless a security reason forces sooner; lock during
   Phase 1.
7. **IPS scope at v1.** Adopt the full HL7 IPS resource catalog as a
   registered consumer profile from day one, or only Patient + Condition +
   MedicationStatement + Observation as the smaller initial slice? Lean
   smaller, but confirm with the patient-export use case (UC-4).
8. **Bundle anchor retry policy.** Bounded-backoff schedule for the
   on-chain anchor commit + the compensating off-chain delete protocol on
   terminal failure. Operational policy, not ADR-level; document in the
   Signing & Fee Service runbook during Phase 1.

## References

- [Requirements: Consumer Profile Heterogeneity](../requirements/consumer-profile-heterogeneity.md)
  — the requirements set this ADR addresses.
- [ADR-0005](adr-0005-fhir-profile-selection.md) — FHIR profile selection;
  this ADR generalizes its two-tier pattern.
- [ADR-0002](adr-0002-on-chain-registry-design.md) — on-chain registry
  design; merkle-root anchor form (§4) requires an addendum to ADR-0002.
- [ADR-0001](adr-0001-identity-and-key-management.md) — identity & key
  management; PHI-containment posture (§8) inherits from ADR-0001's
  authorization boundary.
- HL7 **FHIR R4 §3.2.0.4 Profile negotiation** —
  <https://hl7.org/fhir/R4/profiling.html#profile-negotiation>.
- HL7 **FHIR R4 Bundle** — <https://hl7.org/fhir/R4/bundle.html>.
- HL7 **FHIR R4 `$validate` operation** —
  <https://hl7.org/fhir/R4/resource-operation-validate.html>.
- HL7 **FHIR R4 StructureMap (Mapping Language)** —
  <https://hl7.org/fhir/R4/structuremap.html>.
- **Simplifier — RNDS profile library** —
  <https://simplifier.net/redenacionaldedadosemsaude/~resources?category=Profile>.
- HL7 **International Patient Summary (IPS)** —
  <https://hl7.org/fhir/uv/ips/>.
- HL7 **BR-Core IG** — <https://hl7.org.br/fhir/core/>.
