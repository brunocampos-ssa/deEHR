# Security Policy

deEHR is an open-source, patient-owned Electronic Health Record platform. It
handles **sensitive personal health data**, so security is treated as a release
gate on every change — not a phase.

This policy explains how to report a vulnerability and what to expect.

## Reporting a vulnerability

**Do not open a public issue, pull request or discussion for a security
vulnerability.**

Report it privately by email to **<brunocampos.ssa@gmail.com>**.

Please include, as far as you can:

- A description of the vulnerability and its impact.
- Steps to reproduce, or a proof of concept.
- The affected component(s), and the version, commit or branch.
- Any suggested remediation.

If you would like to encrypt your report, send an initial message without
sensitive detail and we will arrange a key exchange.

## What to expect

deEHR is in early development, so there is not yet a formal SLA. The intent is:

- **Acknowledgement** of your report within **5 business days**.
- An **initial assessment** and severity rating shortly afterwards.
- **Coordinated, good-faith communication** through to a fix.
- **Credit** for the discovery in the release notes, if you would like it.

## Supported versions

The project is **pre-release**. Only the `main` branch is supported, and
security fixes are applied there. Versioned release branches with their own
support windows will be defined at the production-hardening phase.

| Version | Supported |
| --- | --- |
| `main` (pre-release) | Yes |
| Tagged releases | None published yet |

## Scope

Security-relevant areas of deEHR include, but are not limited to:

- **PHI handling** — the FHIR gateway, FHIR storage, encryption in transit and
  at rest, and the encrypted patient-held record exports.
- **Smart contracts** — the Rust/WASM Klever contracts (identity, credential,
  consent, and anchor & audit registries): access control, integer overflow,
  reentrancy and WASM-specific concerns.
- **Authorization** — the SMART App Launch / OAuth2 / OIDC authorization server
  and the on-chain consent bridge.
- **Key custody and the signing & fee service** — the platform-operated
  component that custodies account keys and submits transactions (see
  ADR-0001).
- **Identity** — DID handling, verifiable credentials, and the social-recovery
  / guardian flows.
- **The RNDS connector** — ICP-Brasil certificate handling and the national
  integration.

## Security model — key invariants

- **No PHI on-chain — ever.** The blockchain stores only integrity hashes,
  consent receipts, audit events, DIDs and credential status. This is a hard
  architectural invariant, enforced in code review and audits.
- **No real PHI in the repository.** Tests and fixtures use synthetic data
  only.
- **Mandatory security audit** before every release and before every pull
  request, using the project's designated security tooling. Smart-contract
  changes additionally require a contract-level audit.
- **Encryption everywhere** — TLS in transit, envelope encryption at rest,
  device-bound authentication.
- **Data protection by design** — aligned with Brazil's LGPD (health data is
  sensitive personal data) and informed by HIPAA.

## Disclosure policy

We follow **coordinated disclosure**. Please give us a reasonable opportunity
to release a fix before any public disclosure. We will agree timing with you,
and we will not pursue or support action against good-faith security research
conducted in line with this policy.

## Responsible testing

When researching, do not access, modify or exfiltrate data that is not yours,
do not degrade the service for others, and only ever use synthetic test data.
There is no production deployment at this stage.
