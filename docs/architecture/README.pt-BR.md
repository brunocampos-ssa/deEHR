# Architecture Decision Records

🌐 **Languages / Idiomas:** [English](README.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([README.md](README.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

Este diretório contém os Architecture Decision Records (ADRs) do deEHR.

## O que é um ADR?

Um ADR captura uma decisão arquitetural significativa — o contexto que a
forçou, a decisão em si e as consequências que dela decorrem. Os ADRs tornam o
*porquê* por trás da arquitetura explícito e revisável.

ADRs são **append-only**. Uma vez que um ADR está `Aceito`, ele não é
reescrito; se a decisão mudar depois, um novo ADR é redigido para substituí-lo,
e o antigo é marcado como `Substituído por ADR-XXXX`.

## Valores de status

- **Proposto** — em discussão; ainda há questões em aberto.
- **Aceito** — decidido e em vigor.
- **Descontinuado** — não mais relevante, ainda não substituído.
- **Substituído por ADR-XXXX** — substituído por uma decisão posterior.

## Índice

| ADR | Título | Status |
| --- | --- | --- |
| [0001](adr-0001-identity-and-key-management.pt-BR.md) | Identidade e Gestão de Chaves — Custódia Progressiva | Aceito |
| [0002](adr-0002-on-chain-registry-design.pt-BR.md) | Design dos Registros On-chain | Aceito |
| [0003](adr-0003-branching-and-release-model.pt-BR.md) | Modelo de Branching e Release — Trunk-based Development | Aceito |
| [0004](adr-0004-did-klever-method.pt-BR.md) | Método DID `did:klever` — Híbrido Clássico / Pós-Quântico | Aceito |
| [0005](adr-0005-fhir-profile-selection.pt-BR.md) | Seleção de Perfis FHIR — Baseline R4, compatível com RNDS, SMART v2 | Aceito |
| [0006](adr-0006-multi-consumer-profile-strategy.pt-BR.md) | Estratégia FHIR Multi-Consumidor — Registry, Projeção Dinâmica, Atomicidade de Bundle | Proposto |
| [0007](adr-0007-patient-identity-resolution.pt-BR.md) | Resolução de Identidade do Paciente & Master Patient Index — Persistência Match-First, Golden Record | Proposto |

## Processo

1. Copie [`adr-template.md`](adr-template.pt-BR.md) para
   `adr-NNNN-titulo-curto.md` (próximo número livre).
2. Abra como `Proposto` e discuta em um pull request.
3. Resolva as questões em aberto; faça merge como `Aceito` quando decidido.
4. Adicione-o ao índice acima.

## Política de documentação

ADRs canônicos são escritos em inglês. Traduções em Português do Brasil
seguem a convenção `.pt-BR` do repositório e são referenciadas
reciprocamente quando adicionadas.
