# ADR-0003: Modelo de Branching e Release — Trunk-based Development

🌐 **Languages / Idiomas:** [English](adr-0003-branching-and-release-model.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0003-branching-and-release-model.md](adr-0003-branching-and-release-model.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Aceito
- **Data:** 2026-05-23
- **Decisores:** mantenedores do deEHR

## Contexto

deEHR é um projeto open source com roadmap público e, inicialmente, um único
mantenedor. O fluxo de desenvolvimento precisa:

- Manter o histórico público fácil de ler, auditar e fazer bisect.
- Ser barato de operar com um a poucos mantenedores.
- Tornar releases um artefato inequívoco — *o que está em produção?*.
- Desacoplar ambientes de deploy (sandbox, staging, produção) dos branches,
  de modo que um commit ou versão específica possa ser promovida de forma
  independente.

Os modelos candidatos considerados:

- **GitFlow / releases via merge commits.** Branches de longa duração
  `develop` e `release/*`; merge commits em `main` marcam releases.
- **Linear com um `develop` de longa duração.** `develop` é o trunk;
  `main` é um ponteiro defasado para "o que está em produção", avançado
  por fast-forward ou rebase-merge a partir de `develop`.
- **Trunk-based.** Um único branch de longa duração (`main`); ambientes
  são identificados por tags ou SHAs de commit em vez de branches.

## Decisão

Adotamos **trunk-based development**:

1. **`main` é o trunk.** Toda mudança entra em `main` via pull request.
2. **Métodos de merge.** Apenas squash ou rebase — merge commits não são
   permitidos. O histórico linear é imposto pelo ruleset do repositório.
3. **Sem branches de longa duração `develop` ou `release/*`.** Branches
   de tópico (`feat/...`, `fix/...`, `docs/...`, `chore/...`) são de
   curta duração e deletados no merge.
4. **Ambientes são desacoplados de branches.**
   - **Sandbox / staging** faz auto-deploy a partir do último `main`.
   - **Produção** faz deploy a partir de um **release com tag** seguindo
     Semantic Versioning (`v0.1.0`, `v1.0.0`, …), publicado como um
     GitHub Release.
5. **Reverts são commits**, não operações de branch, em `main`.
6. **Pushes diretos para `main` são bloqueados** pelo ruleset. Admins
   podem fazer bypass apenas em circunstâncias limitadas e transparentes
   (mantenedor solo; commits de meta-setup); o bypass é removido assim
   que houver um segundo mantenedor.

## Consequências

### Aspectos positivos

- Mecânica de branches mínima — um único trunk para raciocinar sobre.
- Histórico linear limpo — `git log`, `git bisect`, `git revert` ficam
  fáceis.
- Releases são um artefato explícito e imutável (uma tag + um GitHub
  Release), não uma posição de branch.
- Promover um commit específico entre ambientes é independente do
  branching.
- Alinhado às normas modernas de OSS e às expectativas dos contribuidores.

### Aspectos negativos / riscos

- Trunk-based development exige que toda mudança mergeada em `main` seja
  em princípio passível de release. Trabalho parcialmente finalizado
  precisa de um mecanismo de **feature flag**, ou não deve ser mergeado
  ainda. O mecanismo de feature flag é um follow-up em aberto.
- Disciplina de release importa: não fazer tag enquanto `main` estiver
  instável. CI precisa garantir *verde no tag*.
- A automação de deploy precisa entender tags vs. branches; isso faz
  parte do trabalho de CI/CD da Fase 0.

## Alternativas consideradas

- **GitFlow com releases via merge commits.** Rejeitada — overhead de
  branches alto demais para o conjunto de mantenedores, e merge commits
  como marcadores de release são menos robustos que tags somadas a
  GitHub Releases.
- **Linear com um `develop` de longa duração.** Rejeitada — `develop`
  agrega valor apenas quando releases são pouco frequentes e
  previsíveis; desacoplar ambientes de branches é mais limpo e remove
  inteiramente a mecânica de promoção `develop` → `main`.

## Questões em aberto

- O mecanismo de **feature flag** para trabalho em andamento — alvo:
  definido antes da Fase 1 começar a produzir código visível ao usuário.
- A **cadência de release** — baseada em data, em mudanças ou sob
  demanda.
- O **wiring de CI/CD** para deploys identificados por ambiente
  (conduzido em P0.5 — gate de segurança e qualidade / CI).

## Referências

- [README.md](../../README.md) — *Roadmap*.
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — *Branching and release
  model*.
- Ruleset do repositório em `main`: PR obrigatório, commits assinados,
  histórico linear, sem force-push, sem deleção, CodeQL obrigatório.
