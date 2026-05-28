# ADR-0005: FHIR Profile Selection — R4 baseline, RNDS-compatible, SMART v2

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](adr-0005-fhir-profile-selection.pt-BR.md)

- **Status:** Accepted
- **Date:** 2026-05-28
- **Deciders:** deEHR maintainers

## Context

deEHR's clinical data model is HL7 FHIR R4 and its authorization is SMART App
Launch 2.x — both committed to in the project README. What remains is the
**profile-level** decision: which canonical profiles deEHR resources conform
to internally, how those translate to the Brazilian RNDS network at the
boundary, the exact SMART scope vocabulary, and the coded value sets the
on-chain Consent Registry will reference. This ADR locks those choices for
Phase 0 and unblocks Q6 in [ADR-0002](adr-0002-on-chain-registry-design.md).

Forces shaping this decision:

- **Brazil-first, not Brazil-only.** RNDS is the first national backbone, but
  the README's connector pattern explicitly anticipates other backbones as
  sibling modules. The internal data model must not be tied to one
  regulator's profile semantics.
- **Two Brazilian IGs in play.** Two HL7-canonical Brazilian implementation
  guides cover the space: **BR-Core**
  (<https://hl7.org.br/fhir/core/>) — the cross-resource clinical standard,
  reused by RAC, Sumário de Alta, and ANS CMD document profiles — and
  **RNDS-Principal** (<https://rnds-fhir.saude.gov.br/>) — workflow-scoped
  profiles for the specific RNDS submission flows currently live (lab
  results, vaccination, medication dispensation). Conformance is
  workflow-bounded: RNDS rejects submissions outside the published profile
  shape only for activated submission flows.
- **Mandatory Brazilian extensions.** BR-Core makes several Brazil-specific
  elements mandatory on `Patient` (CPF, race/ethnicity, gender identity,
  birth sex). Carrying them natively in the deEHR-canonical Patient avoids
  a lossy-upgrade problem at the RNDS connector boundary.
- **SMART v2 binding.** SMART App Launch 2.x ships granular scopes
  (`<context>/<resource>.<cruds>[?query]`). The Consent Registry stores
  coded scope identifiers only — never free text — so the on-chain
  encoding must be bounded-length and unambiguously reversible to the
  canonical scope string.
- **Purpose-of-use on-chain.** Same constraint: a code, not a phrase. Must
  be HL7-canonical and interoperable.

## Decision

### 1. Two-tier profile strategy

deEHR conforms to **deEHR-canonical profiles** internally (FHIR R4 base
profiles plus the minimal extensions documented in §4). The **RNDS
connector** translates deEHR resources to/from **BR-Core** or
**RNDS-Principal** profiles at the boundary, on a per-workflow basis. The
core platform never imports an RNDS-Principal profile; only the connector
does. This preserves the "sibling-connector" pattern: a future Mexico /
Portugal / Argentina backbone is a peer module, not a refactor.

### 2. FHIR version & conformance posture

- **Wire format:** HL7 FHIR R4 (4.0.1), JSON, RESTful.
- **Internal conformance:** every internal resource MUST conform to its
  deEHR-canonical profile (defined in §3 and §4).
- **External conformance to RNDS:** the RNDS connector MUST translate each
  outbound resource into the RNDS-mandated profile for the active workflow
  (lab result, vaccination, dispensation, clinical-encounter document).
  Outside those workflows, the connector MAY emit BR-Core-shaped resources;
  RNDS doesn't accept generic resource POSTs today.
- **Connector mapping fixtures** for each (deEHR → RNDS-Principal,
  deEHR → BR-Core) pair are part of the connector contract and unit-tested.

### 3. MVP resource catalog & per-resource profile decisions

The Phase 0 MVP covers the seven resources below. The rest of the catalog
(AllergyIntolerance, Immunization, Procedure, Coverage, Claim,
ExplanationOfBenefit) is **targeted but deferred** to a follow-up ADR when
the corresponding workflows become active.

Per-resource profile decisions (deEHR-canonical → connector targets):

- **Patient → `DEEHRPatient`**
  - Internal base: FHIR R4 `Patient` + §4 extensions.
  - BR-Core target:
    [`br-core-patient`](https://hl7.org.br/fhir/core/StructureDefinition-br-core-patient.html).
  - RNDS-Principal target:
    [`BRIndividuo-1.0`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRIndividuo-1.0.html)
    for cadastral submissions.
- **Encounter → `DEEHREncounter`**
  - Internal base: FHIR R4 `Encounter`.
  - BR-Core target:
    [`br-core-encounter`](https://hl7.org.br/fhir/core/).
  - RNDS-Principal target: n/a (no RNDS-Principal Encounter — used within
    document Bundles like RAC, Sumário de Alta).
- **Observation → `DEEHRObservation`**
  - Internal base: FHIR R4 `Observation`.
  - BR-Core target:
    [`br-core-observation`](https://hl7.org.br/fhir/core/).
  - RNDS-Principal target:
    [`BRDiagnosticoLaboratorioClinico-3.2.1`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRDiagnosticoLaboratorioClinico-3.2.1.html)
    for lab-result submissions.
- **Condition → `DEEHRCondition`**
  - Internal base: FHIR R4 `Condition`.
  - BR-Core target:
    [`br-core-condition`](https://hl7.org.br/fhir/core/).
  - RNDS-Principal target:
    [`BRCondicaoSaude`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRCondicaoSaude.html).
- **MedicationRequest → `DEEHRMedicationRequest`**
  - Internal base: FHIR R4 `MedicationRequest`.
  - BR-Core target:
    [`br-core-medicationrequest`](https://hl7.org.br/fhir/core/).
  - RNDS-Principal target: RNDS *Prescrição Eletrônica* (Draft on
    Simplifier — track and re-evaluate at GA).
- **Consent → `DEEHRConsent`**
  - Internal base: FHIR R4 `Consent`.
  - BR-Core target:
    [`br-core-consent`](https://hl7.org.br/fhir/core/).
  - RNDS-Principal target: n/a (no RNDS Consent profile published; consent
    handled out-of-band via Conecte SUS).
- **DocumentReference → `DEEHRDocumentReference`**
  - Internal base: FHIR R4 `DocumentReference` + off-chain blob hash +
    Klever tx anchor extensions.
  - BR-Core / RNDS-Principal targets: n/a (no profile today).

Profile canonical URIs follow the pattern
`https://deehr.org/fhir/StructureDefinition/DEEHR<ResourceName>`.

### 4. Mandatory deEHR-canonical extensions (Brazil-aware by default)

Because BR-Core makes the following elements mandatory and the connector
cannot lossy-upgrade them at submission time, deEHR-canonical carries them
natively. They are non-PHI metadata (or PHI under deEHR's normal encryption
posture for PHI):

- **CPF identifier** (`identifier.system = https://saude.gov.br/fhir/sid/cpf`)
  on `DEEHRPatient`, cardinality 1..1. Validation: 11 digits + CPF
  checksum.
- **CNS identifier** (`identifier.system = https://saude.gov.br/fhir/sid/cns`)
  on `DEEHRPatient`, cardinality 0..1.
- **Race / ethnicity** (`raca-br-ips`,
  <https://ips.saude.gov.br/fhir/StructureDefinition/raca-br-ips>) — 1..1,
  value bound to BRRacaCor.
- **Gender identity** (`identidade-genero-br-ips`) — 0..1.
- **Birth sex** (`sexo-nascimento-br-ips`) — 0..1.
- **CNES** (`Organization.identifier` with system
  `https://saude.gov.br/fhir/sid/cnes` — pattern; exact slug subject to
  RNDS-team confirmation) on `DEEHROrganization` (provider/establishment),
  cardinality 1..1 for Brazilian organizations.

For non-Brazilian patients or organizations onboarded through future
backbones, the mandatory cardinalities relax via a slicing pattern — `cpf`,
`cns`, `raca-br-ips`, and `cnes` are 0..1 in the international slice. This
is a deEHR-product decision, not an FHIR-mandated relaxation; sibling
connectors will declare their own jurisdictional slices.

### 5. SMART v2 scope vocabulary

deEHR adopts **SMART App Launch 2.0 (STU2.2)** v2 scope syntax:
`<context>/<resourceType>.<cruds>[?<query>]`, with permissions as an
in-order subset of `cruds` (`c`=create, `r`=read, `u`=update/patch,
`d`=delete, `s`=search/history-type). Spec:
<https://hl7.org/fhir/smart-app-launch/STU2/scopes-and-launch-context.html>.

The deEHR `v1` MVP scope set:

| # | Scope | Requestable by | Grants |
| --- | --- | --- | --- |
| 1 | `openid` | All clients | OIDC ID token |
| 2 | `fhirUser` | All clients | FHIR resource reference for the authenticated user |
| 3 | `launch/patient` | Patient & provider apps | Patient context at standalone launch |
| 4 | `offline_access` | Patient & provider apps | Long-lived refresh token |
| 5 | `online_access` | Provider apps | Session-bound refresh token |
| 6 | `patient/Patient.rs` | Patient app | Read own demographics |
| 7 | `patient/Encounter.rs` | Patient app | Read own encounters |
| 8 | `patient/Observation.rs` | Patient app | Read own observations |
| 9 | `patient/Condition.rs` | Patient app | Read own conditions |
| 10 | `patient/MedicationRequest.rs` | Patient app | Read own meds |
| 11 | `patient/DocumentReference.rs` | Patient app | Read own clinical documents |
| 12 | `patient/Consent.crus` | Patient app | Manage own consent (mirrors on-chain registry) |
| 13 | `user/Patient.rs` | Provider | Read patients in care relationship |
| 14 | `user/Encounter.crus` | Provider | Manage encounters |
| 15 | `user/Observation.crus` | Provider | Manage observations |
| 16 | `user/Condition.crus` | Provider | Manage conditions |
| 17 | `user/MedicationRequest.crus` | Provider | Prescribe |
| 18 | `user/DocumentReference.crus` | Provider | Manage clinical notes |
| 19 | `system/*.rs` | RNDS / bulk connector | Read-all for inter-institutional exchange |
| 20 | `system/Observation.rs?category=http://terminology.hl7.org/CodeSystem/observation-category\|laboratory` | Lab pipeline | Lab-only ingest/read |

Notes:

- `.crus` deliberately omits `d`. Destructive deletes route through a
  tombstone/consent-revocation flow, not OAuth.
- The `?query` suffix is restricted to US-Core-mandated category-level
  granularities for the MVP. Chaining, modifiers, and `_filter` (marked
  experimental in the spec) are out of scope for `v1`.
- deEHR's authorization server MUST advertise both `permission-v1` and
  `permission-v2` capabilities at `/.well-known/smart-configuration` and
  apply the spec's normative v1→v2 mapping (`.read → .rs`, `.write → .cud`,
  `.* → .cruds`) for legacy clients.

### 6. On-chain coded scope encoding

The Consent Registry stores each granted scope as a fixed-shape tuple:

```text
(deehr_scopes_version: u16, template_code: u16, param_hash: bytes16)
```

- `deehr_scopes_version` — the version of the deEHR scope manifest (`v1` =
  this ADR's table).
- `template_code` — a stable `u16` index into the manifest; e.g.,
  `0x0008` = `patient/Observation.rs`.
- `param_hash` — `blake2b-128(canonical_query_string)`, or all-zero if the
  scope has no `?query` constraint. The canonical query string is the
  scope's query component with parameters sorted lexicographically and
  values URL-decoded.

The manifest itself is a signed JSON document published at
`https://deehr.org/fhir/scopes/<version>.json`, immutable once published;
new scopes get new template codes — codes are never reused. This is
bounded-length on-chain (20 bytes), auditable (the manifest is the single
source of truth), and reversible — given the tuple plus the manifest, any
verifier can reconstruct the canonical scope string and re-hash to confirm
the `?query` constraint.

### 7. Purpose-of-use code system

deEHR's Consent Registry encodes purpose-of-use as a `{system, code}`
tuple where:

- `system = http://terminology.hl7.org/CodeSystem/v3-ActReason`
- `code` is drawn from the deEHR accept-list — a subset of the
  [`v3-PurposeOfUse` value set](https://hl7.org/fhir/R4/valueset-v3-PurposeOfUse.html)

deEHR `v1` accept-list:

| deEHR alias | HL7 code | Display | When to use |
| --- | --- | --- | --- |
| TREATMENT | `TREAT` | treatment | Routine clinical care access by a treating provider |
| EMERGENCY | `ETREAT` | Emergency Treatment | Break-glass authorized by patient pre-grant |
| EMERGENCY_ROOM | `ERTREAT` | emergency room treatment | ER-specific access (subtype of ETREAT) |
| RESEARCH | `HRESCH` | healthcare research | Research data sharing, IRB-bound |
| CLINICAL_TRIAL | `CLINTRCH` | clinical trial research | Trial enrollment / data contribution |
| PAYMENT | `HPAYMT` | healthcare payment | Insurer/payer billing operations |
| COVERAGE | `COVERAGE` | coverage under policy or program | Insurance eligibility / coverage determination |
| OPERATIONS | `HOPERAT` | healthcare operations | Quality, audit, internal ops |
| PUBLIC_HEALTH | `PUBHLTH` | public health | Mandatory reporting / surveillance |
| PATIENT_REQUEST | `PATRQT` | patient requested | Patient-initiated export / portability / sharing |
| CARE_COORDINATION | `COC` | coordination of care | Cross-provider continuity |
| LEGAL | `HLEGAL` | legal | Subpoena / legal disclosure |

`Consent.scope` (the orthogonal "what kind of consent statement" axis) is
populated from `http://terminology.hl7.org/CodeSystem/consentscope` for
FHIR conformance but is **not** used in token-issuance comparison —
purpose-matching is exact code match against the table above, except for
the explicit `ETREAT`/`BTG` break-glass escalation policy.

If a future deEHR product need cannot be expressed in the v3-PurposeOfUse
value set (e.g., a Brazil-specific "compartilhamento RNDS"), a deEHR
`CodeSystem` will be introduced via a follow-up ADR, with explicit
`concept-map` entries back to v3-PurposeOfUse for interop.

### 8. Conformance levels (summary)

- **MUST** conform to deEHR-canonical profiles for any resource handled by
  the core platform.
- **SHOULD** conform to BR-Core when interoperating with Brazilian
  healthcare entities outside the RNDS submission workflows.
- **MUST** conform to the workflow-specific RNDS-Principal profile when
  submitting to RNDS (lab result, vaccination, dispensation, etc.).
- **MUST** advertise both SMART `permission-v1` and `permission-v2`
  capabilities.
- **MUST** reject consent grants whose purpose-of-use code is not in §7's
  accept-list.

## Consequences

### Positive

- Built entirely on HL7-canonical artifacts: FHIR R4 base, BR-Core,
  RNDS-Principal, SMART v2, v3 PurposeOfUse. Zero deEHR-specific code
  systems at `v1` — interop is the default.
- The connector pattern is genuinely portable. Adding a Mexico / Portugal /
  Argentina backbone is a new connector module, not an internal-model
  rewrite.
- The on-chain Consent Registry data model (Q6 of ADR-0002) is now
  concretely defined: scope = `(version, template_code, param_hash)`;
  purpose-of-use = v3 ActReason code from the §7 accept-list.
- Brazil-specific mandatory extensions (CPF, race/ethnicity, CNES) live in
  deEHR-canonical natively, removing a lossy-upgrade hazard at the RNDS
  boundary.

### Negative / risks

- **BR-Core mandatory extensions add weight to non-Brazilian onboards.**
  `raca-br-ips` and CPF are 1..1 in the Brazilian slice; the international
  slice relaxes them, but the slicing-pattern carries operational
  complexity in product UI and validation.
- **RNDS-Principal Draft profile risk.** The RNDS Prescrição Eletrônica
  profile is Draft on Simplifier; the connector mapping for
  `MedicationRequest` MAY shift before its GA. Track in the open questions.
- **No published RNDS Consent profile.** Consent semantics for the RNDS
  pipeline are operationally handled outside FHIR (via Conecte SUS). If
  RNDS later publishes a Consent profile, the connector will need a new
  mapping; deEHR's consent semantics inside the platform won't change.
- **Scope manifest becomes operational infrastructure.** The
  `https://deehr.org/fhir/scopes/v1.json` manifest is a signed, versioned,
  publicly-hosted artifact. Its hosting, signing, and integrity-monitoring
  posture is a new operational responsibility.
- **No DocumentReference Brazilian profile today.** Base R4 plus deEHR
  extensions are sufficient internally; this will need revisit if Brazil
  publishes a Prontuário do Cidadão DocumentReference profile.

## Alternatives considered

- **Use RNDS-Principal profiles as the internal data model.** Rejected —
  couples the entire platform to one regulator's semantics, contradicting
  the README's sibling-connector pattern. Short-term cheaper but blocks
  multi-backbone evolution.
- **Use HL7 IPS (International Patient Summary) as the internal baseline.**
  Considered for cross-border friendliness. Rejected — IPS is its own
  profile set with its own conformance burden, and doesn't match Brazil's
  specifics (CPF, CNS, raca-br-ips) without overlay work. Worse fit for
  Brazil-first, no better fit for sibling-connector flexibility than the
  chosen approach.
- **Define a deEHR custom purpose-of-use code system at `v1`.** Rejected —
  no concrete product need that v3 PurposeOfUse cannot express today; a
  custom system would sacrifice interop for hypothetical flexibility.
- **Hash the full canonical scope string on-chain (no template registry).**
  Rejected — auditable but unreversible without a side index, and gives no
  governance surface to prevent clients from inventing scopes that have
  never been reviewed.
- **Encode on-chain scopes as a bitmask of pre-defined permissions.**
  Rejected — breaks down once `?query` constraints enter the picture.

## Open questions

These remain to resolve before subsequent ADR-0005 revisions (recorded via
Addenda per the repository's append-only ADR policy):

1. **BR-Core formal authority.** BR-Core looks like the de facto Brazilian
   cross-resource standard but is not formally referenced from
   `rnds-guia.saude.gov.br`. Confirm with DATASUS / Ministério da Saúde
   which IG governs new document submissions.
2. **CNES identifier system URL exact slug.** Verify the canonical system
   URL against BREstabelecimentoSaude-1.0.
3. **RNDS Prescrição Eletrônica profile stability** — track Simplifier
   status; revisit when the profile leaves Draft.
4. **DocumentReference Brazilian profile** — track whether RNDS / Brazil
   publishes one (Prontuário do Cidadão roadmap).
5. **RNDS Consent profile** — confirm RNDS has no non-public / draft
   Consent profile in a newer release before locking deEHR's Consent
   mapping.
6. **IHE Privacy Consent on FHIR (PCF)** alignment — Brazil sometimes
   relies on IHE PCF; evaluate in a follow-up ADR whether deEHR should
   bind to PCF as well as v3 PurposeOfUse.
7. **SMART scope manifest hosting & signing operational plan** — concrete
   choice of signing key custody, transparency log, and integrity-monitor
   posture for the `deehr-scopes-v1.json` artifact.
8. **Patient-Consent.crus product call** — confirm with product whether
   patients should be able to update their own Consent via SMART (the
   on-chain registry is already patient-owned).

## References

- [README.md](../../README.md) — *Standards & Building Blocks* and *RNDS &
  Government Integration* sections.
- [ADR-0001](adr-0001-identity-and-key-management.md) — Identity & Key
  Management (Progressive Custody).
- [ADR-0002](adr-0002-on-chain-registry-design.md) — On-chain Registry
  Design (Consent Registry coded-value-set binding).
- [ADR-0004](adr-0004-did-klever-method.md) — `did:klever` DID Method.
- HL7 **FHIR R4 (4.0.1)** — <https://hl7.org/fhir/R4/>.
- HL7 **SMART App Launch 2.0 (STU2.2)** —
  <https://hl7.org/fhir/smart-app-launch/STU2/>.
- HL7 **Terminology v3 ActReason** —
  <https://terminology.hl7.org/CodeSystem-v3-ActReason.html>.
- HL7 **FHIR R4 v3 PurposeOfUse value set** —
  <https://hl7.org/fhir/R4/valueset-v3-PurposeOfUse.html>.
- **BR-Core IG** — <https://hl7.org.br/fhir/core/>.
- **RNDS Implementation Guide** — <https://rnds-guia.saude.gov.br/>.
- **RNDS-Principal FHIR IG** — <https://rnds-fhir.saude.gov.br/>.
- **Simplifier — RNDS project** — <https://simplifier.net/redenacionaldedadosemsaude>.
- **IPS-Brazil extensions (raca-br-ips, identidade-genero-br-ips,
  sexo-nascimento-br-ips)** — <https://ips.saude.gov.br/fhir/>.
- US Core IG v9 — SMART on FHIR Obligations and Capabilities —
  <https://build.fhir.org/ig/HL7/US-Core/scopes.html>.
- IHE **Privacy Consent on FHIR (PCF) v1.1.0** —
  <https://profiles.ihe.net/ITI/PCF/>.
