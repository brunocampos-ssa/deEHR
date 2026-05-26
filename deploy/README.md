# Deployment

Infrastructure-as-code and container definitions for deEHR.

> **Status:** Directory structure only (Phase 0). Concrete manifests land
> progressively as services are built and as the production-hardening phase
> approaches.

| Subdirectory | Concerns |
| --- | --- |
| [`docker/`](docker/) | Dockerfiles and container compositions |
| [`k8s/`](k8s/) | Kubernetes manifests (Helm or plain YAML, TBD) |
| [`terraform/`](terraform/) | Cloud infrastructure (network, storage, secrets, HSM/KMS for the Signing & Fee Service) |

GitOps and environment topology — including how `main` (sandbox/staging
auto-deploy) and tagged releases (production) map to environments per
[ADR-0003](../docs/architecture/adr-0003-branching-and-release-model.md) —
will be defined when the first service is deployed.
