# fhir-gateway

FHIR R4 facade in front of the FHIR server. Enforces consent on every
request, anchors integrity hashes of written resources on the **Anchor &
Audit Registry**, and isolates the FHIR server from the rest of the
platform.

See [ADR-0002](../../docs/architecture/adr-0002-on-chain-registry-design.md).

> **Status:** Phase 1 — not yet implemented.
