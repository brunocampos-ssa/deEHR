# ADR-0003: Branching and Release Model — Trunk-based Development

🌐 **Languages / Idiomas:** **English** · [Português (Brasil)](adr-0003-branching-and-release-model.pt-BR.md)

- **Status:** Accepted
- **Date:** 2026-05-23
- **Deciders:** deEHR maintainers

## Context

deEHR is an open-source project with a public roadmap and, initially, a single
maintainer. The development workflow needs to:

- Keep the public history easy to read, audit and bisect.
- Be cheap to operate with one to a few maintainers.
- Make releases an unambiguous artifact — *what is in production?*.
- Decouple deployment environments (sandbox, staging, production) from
  branches so a specific commit or version can be promoted independently.

The candidate models considered:

- **GitFlow / merge-commit releases.** Long-lived `develop` and `release/*`
  branches; merge commits on `main` mark releases.
- **Linear with a long-lived `develop`.** `develop` is the trunk; `main` is a
  trailing pointer to "what is in production," advanced by fast-forward or
  rebase-merge from `develop`.
- **Trunk-based.** A single long-lived branch (`main`); environments are keyed
  to tags or commit SHAs rather than branches.

## Decision

We adopt **trunk-based development**:

1. **`main` is the trunk.** Every change lands on `main` via a pull request.
2. **Merge methods.** Squash or rebase only — merge commits are not allowed.
   Linear history is enforced by the repository ruleset.
3. **No long-lived `develop` or `release/*` branches.** Topic branches
   (`feat/...`, `fix/...`, `docs/...`, `chore/...`) are short-lived and
   deleted on merge.
4. **Environments are decoupled from branches.**
   - **Sandbox / staging** auto-deploys from the latest `main`.
   - **Production** deploys from a **tagged release** following Semantic
     Versioning (`v0.1.0`, `v1.0.0`, …), published as a GitHub Release.
5. **Reverts are commits**, not branch operations, on `main`.
6. **Direct pushes to `main` are blocked** by the ruleset. Admins may bypass
   only in limited, transparent circumstances (solo maintainer; meta-setup
   commits); the bypass is dropped as soon as there is a second maintainer.

## Consequences

### Positive

- Minimal branch mechanics — one trunk to reason about.
- Clean linear history — `git log`, `git bisect`, `git revert` are easy.
- Releases are an explicit, immutable artifact (a tag + a GitHub Release),
  not a branch position.
- Promoting a specific commit between environments is independent of branching.
- Matches modern OSS norms and contributor expectations.

### Negative / risks

- Trunk-based development requires that every change merged to `main` is in
  principle releasable. Partially finished work needs a **feature-flag**
  mechanism, or it must not merge yet. The feature-flag mechanism is an open
  follow-up.
- Release discipline matters: do not tag while `main` is unstable. CI must
  enforce *green on tag*.
- Deployment automation must understand tags vs. branches; this is part of
  the Phase 0 CI/CD work.

## Alternatives considered

- **GitFlow with merge-commit releases.** Rejected — too much branch overhead
  for the maintainer set, and merge commits as release markers are less robust
  than tags plus GitHub Releases.
- **Linear with a long-lived `develop`.** Rejected — `develop` adds value only
  when releases are infrequent and predictable; decoupling environments from
  branches is cleaner and removes the develop → main promotion mechanic
  entirely.

## Open questions

- The **feature-flag** mechanism for in-progress work — target: defined before
  Phase 1 starts producing user-visible code.
- The **release cadence** — date-based, change-based, or on-demand.
- The **CI/CD wiring** for environment-keyed deploys (carried in P0.5 —
  Security & quality gate / CI).

## References

- [README.md](../../README.md) — *Roadmap*.
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — *Branching and release model*.
- Repository ruleset on `main`: PR required, signed commits, linear history,
  no force-push, no deletion, CodeQL required.
