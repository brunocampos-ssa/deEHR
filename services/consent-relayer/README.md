# consent-relayer

The **platform Signing & Fee Service** — submits patient transactions to
Klever and pays the network fees from a platform treasury. Custodies
patient signing keys in an HSM, gated by passkey-based authentication on
the patient device.

Because Klever has no native gasless / meta-transaction primitive, this
service *is* the "no-gas, no-seed-phrase" mechanism for patients. It is
**security-critical** — full design and threat model in
[ADR-0001](../../docs/architecture/adr-0001-identity-and-key-management.md)
and [docs/security/threat-model.md](../../docs/security/threat-model.md).

> **Status:** Phase 1 — not yet implemented.
