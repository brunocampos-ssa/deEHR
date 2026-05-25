# auth-server

SMART App Launch / OAuth2 / OIDC authorization server.

Reads patient consent from the on-chain **Consent Registry** before minting
an OAuth2 token, so the scopes issued are *provably* backed by patient
consent. See [ADR-0002](../../docs/architecture/adr-0002-on-chain-registry-design.md).

> **Status:** Phase 1 — not yet implemented.
