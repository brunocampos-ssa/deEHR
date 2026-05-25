# Backend Services

Go services that make up the deEHR backend.

> **Status:** Directory structure only (Phase 0). Go modules and source code
> land in **Phase 1** when each service is built. See the architecture in
> [ADR-0001](../docs/architecture/adr-0001-identity-and-key-management.md)
> and [ADR-0002](../docs/architecture/adr-0002-on-chain-registry-design.md).

## Services

| Service | Role |
| --- | --- |
| [`auth-server/`](auth-server/) | SMART App Launch / OAuth2 / OIDC authorization server; consults the Consent Registry before minting tokens |
| [`fhir-gateway/`](fhir-gateway/) | FHIR R4 facade in front of the FHIR server; enforces consent; anchors integrity hashes on-chain |
| [`rnds-connector/`](rnds-connector/) | Isolated module for Brazil's RNDS integration (ICP-Brasil certs, RNDS FHIR profiles) |
| [`consent-relayer/`](consent-relayer/) | Platform Signing & Fee Service — submits patient transactions and pays network fees from the platform treasury (see [ADR-0001](../docs/architecture/adr-0001-identity-and-key-management.md)) |

## Layout (planned for Phase 1)

A Go workspace (`go.work` at the repo root, per-service `go.mod`) will be
added when the first service is implemented. The Go toolchain version will
be pinned in each service's `go.mod`.
