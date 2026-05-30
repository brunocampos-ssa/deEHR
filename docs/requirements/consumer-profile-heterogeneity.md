# Consumer Profile Heterogeneity, Bundle Composition, and Dynamic Projection

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](consumer-profile-heterogeneity.pt-BR.md)

- **Status:** Draft (Phase 0 → Phase 1 transition input)
- **Date:** 2026-05-30
- **Seeds:** [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.md) (Proposed)

## Provenance

These requirements were assembled from a design-thinking conversation with a
CTO in the Brazilian insurance market in late May 2026. The CTO reviewed the
deEHR architecture in its Phase 0 state (ADRs 0001–0005, threat model,
on-chain registry design) and surfaced three concerns that the existing
artefacts do not fully address. Identity withheld; the substantive points are
captured here in deEHR's own framing.

## Problem statement

FHIR R4 is a *contract pattern*, not a single contract. Three intersecting
realities follow:

1. **Profile heterogeneity.** Every meaningful consumer of FHIR data — a
   national backbone, a private insurer, a hospital network, a research
   institution, a foreign jurisdiction — defines its own profile. The profile
   specifies which fields are mandatory, which value sets are bound, and
   which extensions appear. The same logical `Patient` resource has dozens of
   legitimate canonical shapes.
2. **Bundles are the atomic unit of clinical reality.** A consultation is
   not a `Patient` write, then a `Practitioner` write, then an `Encounter`
   write. It is a *Bundle* of Patient + Practitioner + Encounter +
   Observation + Condition + MedicationRequest, all written and read
   together. Persisting it as N independent resource writes loses the
   atomicity the clinical event needs.
3. **Patient-centric storage must serve heterogeneous consumers.** When the
   patient is the root of the data, each consumer reading that data needs to
   receive it shaped to *their* profile. Static, design-time connector code
   per consumer does not scale to a marketplace of consumers.

[ADR-0005](../architecture/adr-0005-fhir-profile-selection.md) anticipates
the *spirit* of (1) with its two-tier pattern — deEHR-canonical internal +
connector translation at the boundary — but only models known regulator
backbones (RNDS today, sibling national backbones tomorrow). It does not
model arbitrary commercial consumers and does not address (2) or (3).

## Consumers in scope

The following consumer classes drive the requirements:

| Consumer class | Examples | Typical profile shape |
| --- | --- | --- |
| National backbones | RNDS (Brazil), sibling national networks | Workflow-bound, regulator-mandated |
| Private insurers | Health plans, supplemental coverage | Carrier-specific, billing-oriented |
| Hospital networks | Multi-site provider organizations | EHR-internal, often based on US-Core / BR-Core |
| Public health authorities | Surveillance, reporting | Coded value sets, mandatory minimum data |
| Research institutions | IRB-bound research consortia | De-identified, study-specific |
| Patient-direct apps | Patient's own copy, IPS exports, portability | International Patient Summary or similar |
| Patient as consumer of own data | The patient's own app, second-opinion exports | deEHR-canonical or IPS |

## Key scenarios

### UC-1: Insurer reads a Patient resource in its own profile

An insurer holding a verifiable patient consent grant requests
`GET /fhir/Patient/{id}` with `Accept-Profile:
<insurer-profile-canonical-url>`. The system MUST return a `Patient` resource
projected to the insurer's profile shape (e.g., insurer-X's `PatientXProfile`
that bundles a `plan-membership` extension and binds `address` cardinality
1..1), or return `406 Not Acceptable` with a list of supported profiles if
the requested one is unknown.

### UC-2: Hospital writes a clinical encounter as a Bundle

A hospital writes a complete consultation as a FHIR `Bundle` of type
`transaction`, including Patient + Practitioner + Encounter + Observation +
Condition + MedicationRequest. The system MUST persist all resources
atomically: either every resource lands and a single on-chain anchor commits,
or no resource lands and no anchor commits. Partial Bundle persistence MUST
NOT be possible.

### UC-3: National backbone subscribes to a workflow

RNDS subscribes to lab-result submissions. The system pushes resources
shaped to `BRDiagnosticoLaboratorioClinico-3.2.1` per the existing ADR-0005
mapping. This case is already covered by ADR-0005; it is listed here to
verify ADR-0006 does not regress it.

### UC-4: Patient exports own data in IPS profile

A patient requests a portable summary in the HL7 International Patient
Summary profile via their patient app. The system MUST project deEHR-canonical
resources to IPS shape on read and return a Bundle of type `document`
suitable for cross-border continuity-of-care.

### UC-5: Cross-profile validation at write time

When a write arrives shaped to a registered profile (not deEHR-canonical),
the system MUST validate the write against the declared profile *and* the
deEHR-canonical profile, reject if either fails, and persist the
deEHR-canonical projection. Validation failure messages MUST attribute the
failure to a specific profile constraint.

### UC-6: Consumer-declared profile discovery

A consumer asks the system "which profiles do you support for `Patient`?".
The system MUST advertise, at `/fhir/metadata` (CapabilityStatement), the
complete list of supported profiles per resource type, with their canonical
URLs and an indication of whether each is read-supported, write-supported,
or both.

### UC-7: Conformance reporting

An operator needs to know which consumer profiles are most frequently
failing validation and which constraints are the worst offenders. The system
MUST emit per-profile validation metrics suitable for an observability stack.

## Non-functional requirements

- **Bundle write atomicity.** All-or-nothing semantics. Includes the on-chain
  anchor commit — anchor MUST NOT commit unless all resources persisted.
- **Read latency budget.** Profile projection at read time MUST be cacheable
  per `(resource id, profile url, resource version)` tuple. First-read
  projection latency budget: TBD in Phase 1 benchmarks; cached-read latency
  target: < 50 ms p95 for single-resource GET.
- **Profile registry governance.** Adding a profile to the registry MUST
  require an explicit governance step (ADR addendum or equivalent reviewable
  artefact). No silent profile additions.
- **Validation observability.** Per-profile pass/fail counts and constraint
  drill-down MUST be available without code change.
- **Backward compatibility with ADR-0005.** Existing BR-Core / RNDS-Principal
  connector mappings MUST be re-expressible as profile-registry entries with
  no semantic change.
- **PHI containment.** Profile transformation MUST NOT leak data across
  encryption boundaries. A consumer authorized for `patient/Patient.rs` MUST
  NOT receive `Observation` data via a profile transformation side-effect.

## Out of scope (for this requirements set)

- **Cross-jurisdiction profile mapping** when source and target profiles
  diverge semantically (e.g., a Brazilian race/ethnicity coding has no
  one-to-one mapping to a Mexican racial-self-identification coding). This
  is a sibling-backbone problem and is tracked separately. ADR-0006 assumes
  source and target profiles share canonical FHIR resource semantics and
  diverge only on cardinality, value-set binding, or extensions.
- **Custom code-system creation.** ADR-0005 §7 already constrains the
  purpose-of-use accept-list; this requirements set does not propose
  loosening it.
- **Profile evolution / versioning.** Profiles published with versioned
  canonical URLs are treated as distinct registry entries; profile-version
  migration is not in scope.

## Open questions

These should be resolved during ADR-0006 review and Phase 1 prototyping:

1. **Profile declaration mechanism.** `Accept-Profile` HTTP header is the
   FHIR R4 §3.2.0.4 standard; should we also accept a SMART scope extension
   (e.g., `patient/Patient.rs?_profile=<url>`) for clients that cannot set
   custom headers?
2. **Projection caching strategy.** Lazy-on-read with TTL, eager-on-write
   pre-materialization, or hybrid? Memory/storage trade-off vs. read
   latency.
3. **Bundle anchor strategy.** Single anchor over the canonical
   serialization of the whole Bundle, or merkle root over per-resource
   hashes? Single anchor is cheaper; merkle root supports per-resource proof
   of inclusion without revealing the rest of the Bundle.
4. **Registry change-control.** Is an ADR addendum required for each new
   consumer profile, or can a lightweight "profile-add" PR with maintainer
   review suffice once the registry shape is stable?
5. **Validation-pipeline performance budget.** Concrete latency / throughput
   targets for the validation + projection pipeline under realistic Bundle
   sizes (e.g., 10-resource consultation Bundle, 200-resource discharge
   summary).
6. **Consumer profile bootstrap.** Seed registry contents at Phase 1 launch
   — minimum is deEHR-canonical + BR-Core + the active RNDS-Principal
   workflow profiles. Should we also seed IPS (international patient
   summary) for patient-export use cases on day one?

## Phase 1 implications

This requirements set implies a Phase 1 sub-arc focused on **profile
registry, validation, and transformation engine** — distinct from the
on-chain contract MVP and the Signing & Fee Service. The CTO's parallel
observation that this is "work for a good data engineer" lines up: the
validation + projection pipeline is the data-engineering load-bearing piece
of the platform. ADR-0006 captures the architectural direction; the Phase 1
issue set must include this sub-arc explicitly.

## References

- [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.md) —
  the proposed architectural decision driven by this requirements set.
- [ADR-0005](../architecture/adr-0005-fhir-profile-selection.md) — the
  two-tier profile pattern this set generalizes.
- [ADR-0002](../architecture/adr-0002-on-chain-registry-design.md) — on-chain
  anchoring semantics that intersect Bundle atomicity (UC-2).
- HL7 **FHIR R4 §3.2.0.4 (Profile negotiation)** —
  <https://hl7.org/fhir/R4/profiling.html#profile-negotiation>.
- HL7 **FHIR R4 Bundle** — <https://hl7.org/fhir/R4/bundle.html>.
- **Simplifier — RNDS project (profile library seed)** —
  <https://simplifier.net/redenacionaldedadosemsaude/~resources?category=Profile>.
- HL7 **International Patient Summary (IPS)** —
  <https://hl7.org/fhir/uv/ips/>.
- [README — Architecture overview](../../README.md).
