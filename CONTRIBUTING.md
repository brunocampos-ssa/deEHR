# Contributing to deEHR

Thank you for your interest in deEHR — an open-source, patient-owned
Electronic Health Record platform built on FHIR / SMART standards with
Klever blockchain anchoring. Contributions of all kinds are welcome.

> **Project status:** early planning / Phase 0 (Foundations). The architecture
> is still being established and things will move. The [README](README.md) is
> the project's anchor document.

## Code of Conduct

This project and everyone participating in it is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to
uphold it. Please report unacceptable behavior to <brunocampos.ssa@gmail.com>.

## Ways to contribute

You do not have to write code to help:

- **Code** — Go services, Rust/WASM smart contracts, frontend apps.
- **FHIR profile expertise** — R4 resources, RNDS profiles, terminology.
- **Security review** — threat modeling, secure code review, contract auditing.
- **Translations** — Brazilian Portuguese (and others); see the documentation
  policy below.
- **Accessibility testing** — especially for elderly and low-digital-literacy
  users, who are first-class users of this project.
- **Domain knowledge** — healthcare, insurance and regulatory expertise
  (LGPD, HIPAA, ANS, CFM, RNDS).
- **Documentation** — guides, diagrams, ADR review.

## Before you start

For anything beyond a small fix, **open an issue first** to discuss the
approach. This avoids duplicated effort and lets the design be reviewed before
code is written. Larger architectural matters may require an Architecture
Decision Record (ADR) under `docs/architecture/`.

## Development setup

The monorepo layout is being established during Phase 0 — see the
*Project Structure* section of the [README](README.md). At a high level:

- **Backend services** — Go (toolchain version pinned in `go.mod` once
  scaffolded).
- **Smart contracts** — Rust, compiled to WebAssembly for the Klever KVM
  (`wasm32-unknown-unknown` target).
- **Frontend** (later phases) — TypeScript with React / Next.js.

Detailed, per-component setup instructions will be added to each module's
README as it lands.

## Branching and release model

deEHR uses **trunk-based development**:

- `main` is the **trunk** — the single long-lived branch. Every change lands
  here via a pull request, squash- or rebase-merged. Linear history is
  enforced.
- There is **no long-lived `develop` or `release/*` branch.** Environments are
  decoupled from branches.
- **Sandbox / staging** auto-deploys from the latest `main`.
- **Production** deploys from **tagged releases** (`v0.1.0`, `v1.0.0`, …),
  published as GitHub Releases following Semantic Versioning.
- Reverts are commits, not branch operations.

Rationale and trade-offs are recorded in
[ADR-0003](docs/architecture/adr-0003-branching-and-release-model.md).

## Branching and commits

- Branch from `main`. Name branches descriptively: `feat/consent-registry`,
  `fix/auth-token-expiry`, `docs/adr-0004`, `chore/ci-lint`.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): summary`. Common types: `feat`, `fix`, `docs`, `refactor`,
  `test`, `chore`, `ci`.
- Commits must be **signed** (`commit.gpgsign = true`) — the ruleset enforces
  this on `main`.
- Keep commits focused and the history readable.

## Pull request process

1. Keep PRs small and single-purpose — they are easier to review and audit.
2. Make sure the build, tests, linters and formatters pass locally.
3. Add or update tests for any behavior change.
4. Update the documentation affected by the change, including ADRs where
   relevant.
5. **A security audit is mandatory before every pull request** — see
   [SECURITY.md](SECURITY.md). Changes that touch Rust/WASM contracts
   additionally require a smart-contract audit.
6. Fill in the PR template, link the related issue, and request review.

A maintainer will review for correctness, security, standards compliance and
accessibility.

## Code review

All PRs receive an automated review from **GitHub Copilot** on every push.
A PR may not be merged until:

- Copilot has reviewed the latest commit, **and**
- any valid Copilot review comments have been resolved, **and**
- a maintainer has approved the PR.

Merges to `main` use **squash** or **rebase** only — merge commits are blocked
by the ruleset to keep history linear (see
[ADR-0003](docs/architecture/adr-0003-branching-and-release-model.md)).

## The hard invariants

These are non-negotiable and are enforced in review:

- **No PHI on-chain — ever.** The blockchain stores only integrity hashes,
  consent receipts, audit events, DIDs and credential status. Never put
  Protected Health Information — or anything that could identify a patient —
  into a smart contract, a transaction payload or an on-chain event.
- **No real PHI anywhere in the repository.** Tests, fixtures, seed data and
  examples must use **synthetic data only**. Never commit real patient data,
  credentials, secrets or tokens.
- **Security is a gate, not a phase.** See [SECURITY.md](SECURITY.md).

## Coding standards

- **Go** — formatted with `gofmt` / `goimports`; idiomatic Go; checked with
  `go vet` and the project linter.
- **Rust** — formatted with `rustfmt`; lint-clean under `clippy`; smart
  contracts written security-first (access control, integer overflow,
  reentrancy, WASM-specific concerns).
- **Standards over invention** — FHIR R4 and SMART App Launch are the contract.
  Prefer the standard to a custom mechanism.
- Write tests. Document public interfaces.

## Documentation policy

Canonical documentation is written in **English**. A **Brazilian Portuguese**
version is maintained alongside it using the `.pt-BR` suffix convention
(e.g. `CONTRIBUTING.pt-BR.md`, `docs/pt-BR/…`) and is always cross-linked with
the English original. If you change an English document, please flag the
corresponding `.pt-BR` file for update — or update it yourself if you can.

## Security

To report a vulnerability, **do not open a public issue** — follow the process
in [SECURITY.md](SECURITY.md).

## License

deEHR is released under the [MIT License](LICENSE). By contributing, you agree
that your contributions will be licensed under the same terms.
