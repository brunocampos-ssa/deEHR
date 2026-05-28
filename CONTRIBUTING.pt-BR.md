# Contribuindo para a deEHR

🌐 **Languages / Idiomas:** [English](CONTRIBUTING.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([CONTRIBUTING.md](CONTRIBUTING.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

Obrigado pelo seu interesse na deEHR — uma plataforma open-source de Registro
Eletrônico de Saúde, na qual o paciente é dono dos seus dados, construída sobre
os padrões FHIR / SMART com ancoragem na blockchain Klever. Contribuições de
todos os tipos são bem-vindas.

> **Status do projeto:** planejamento inicial / Fase 0 (Fundações). A
> arquitetura ainda está sendo estabelecida e as coisas vão se mover. O
> [README](README.md) é o documento âncora do projeto.

## Código de Conduta

Este projeto e todos os que dele participam são regidos pelo
[Código de Conduta](CODE_OF_CONDUCT.md). Ao participar, espera-se que você o
respeite. Por favor, reporte comportamento inaceitável para
<brunocampos.ssa@gmail.com>.

## Formas de contribuir

Você não precisa escrever código para ajudar:

- **Código** — serviços em Go, smart contracts Rust/WASM, apps de frontend.
- **Expertise em perfis FHIR** — recursos R4, perfis da RNDS, terminologia.
- **Revisão de segurança** — modelagem de ameaças, revisão segura de código,
  auditoria de contratos.
- **Traduções** — Português do Brasil (e outras); veja a política de
  documentação abaixo.
- **Testes de acessibilidade** — especialmente para pessoas idosas e usuários
  com baixa literacia digital, que são usuários de primeira classe deste
  projeto.
- **Conhecimento de domínio** — expertise em saúde, seguros e regulação
  (LGPD, HIPAA, ANS, CFM, RNDS).
- **Documentação** — guias, diagramas, revisão de ADRs.

## Antes de começar

Para qualquer coisa além de um pequeno fix, **abra uma issue primeiro** para
discutir a abordagem. Isso evita esforço duplicado e permite que o design seja
revisado antes que o código seja escrito. Assuntos arquiteturais maiores podem
exigir um Registro de Decisão Arquitetural (ADR) sob `docs/architecture/`.

## Setup de desenvolvimento

O layout do monorepo está sendo estabelecido durante a Fase 0 — veja a seção
*Project Structure* do [README](README.md). Em alto nível:

- **Serviços de backend** — Go (versão do toolchain fixada em `go.mod` assim
  que for criado).
- **Smart contracts** — Rust, compilados para WebAssembly para a KVM da Klever
  (alvo `wasm32-unknown-unknown`).
- **Frontend** (fases posteriores) — TypeScript com React / Next.js.

Instruções detalhadas de setup, por componente, serão adicionadas ao README de
cada módulo conforme forem surgindo.

## Modelo de branching e release

A deEHR usa **trunk-based development**:

- `main` é o **trunk** — o único branch de vida longa. Toda mudança chega ali
  via pull request, com merge por squash ou rebase. Histórico linear é
  obrigatório.
- **Não existe branch `develop` ou `release/*` de vida longa.** Os ambientes
  são desacoplados dos branches.
- **Sandbox / staging** faz deploy automático a partir do `main` mais recente.
- **Produção** faz deploy a partir de **releases com tag** (`v0.1.0`,
  `v1.0.0`, …), publicados como GitHub Releases seguindo Semantic Versioning.
- Reversões são commits, não operações de branch.

A justificativa e os trade-offs estão registrados na
[ADR-0003](docs/architecture/adr-0003-branching-and-release-model.md).

## Branching e commits

- Faça branch a partir de `main`. Nomeie os branches de forma descritiva:
  `feat/consent-registry`, `fix/auth-token-expiry`, `docs/adr-0004`,
  `chore/ci-lint`.
- Mensagens de commit seguem [Conventional Commits](https://www.conventionalcommits.org/):
  `type(scope): summary`. Tipos comuns: `feat`, `fix`, `docs`, `refactor`,
  `test`, `chore`, `ci`.
- Commits precisam ser **assinados** (`commit.gpgsign = true`) — o ruleset
  obriga isso no `main`.
- Mantenha os commits focados e o histórico legível.

## Processo de pull request

1. Mantenha os PRs pequenos e com um único propósito — eles são mais fáceis
   de revisar e auditar.
2. Garanta que o build, os testes, os linters e os formatadores passam
   localmente.
3. Adicione ou atualize testes para qualquer mudança de comportamento.
4. Atualize a documentação afetada pela mudança, incluindo ADRs quando
   relevante.
5. **Uma auditoria de segurança é obrigatória antes de cada pull request** —
   veja [SECURITY.md](SECURITY.md). Mudanças que tocam em contratos Rust/WASM
   exigem, adicionalmente, uma auditoria de smart contract.
6. Preencha o template de PR, vincule a issue relacionada e peça revisão.

Um(a) mantenedor(a) revisará quanto a correção, segurança, conformidade com
padrões e acessibilidade.

## Revisão de código

Todos os PRs recebem uma revisão automatizada do **GitHub Copilot** a cada
push. Um PR não pode ter merge até que:

- o Copilot tenha revisado o último commit, **e**
- qualquer comentário válido da revisão do Copilot tenha sido resolvido, **e**
- um(a) mantenedor(a) tenha aprovado o PR.

Merges para `main` usam apenas **squash** ou **rebase** — merge commits são
bloqueados pelo ruleset para manter o histórico linear (veja a
[ADR-0003](docs/architecture/adr-0003-branching-and-release-model.md)).

## Os invariantes rígidos

Estes são não-negociáveis e são reforçados em revisão:

- **Nenhuma PHI on-chain — jamais.** A blockchain armazena apenas hashes de
  integridade, recibos de consentimento, eventos de auditoria, DIDs e status
  de credenciais. Nunca coloque Informação de Saúde Protegida — ou qualquer
  coisa que possa identificar um paciente — em um smart contract, em um
  payload de transação ou em um evento on-chain.
- **Nenhuma PHI real em lugar algum do repositório.** Testes, fixtures, seed
  data e exemplos devem usar **somente dados sintéticos**. Nunca faça commit
  de dados reais de pacientes, credenciais, secrets ou tokens.
- **Segurança é um gate, não uma fase.** Veja [SECURITY.md](SECURITY.md).

## Padrões de código

- **Go** — formatado com `gofmt` / `goimports`; Go idiomático; verificado com
  `go vet` e o linter do projeto.
- **Rust** — formatado com `rustfmt`; sem lints sob `clippy`; smart contracts
  escritos com segurança em primeiro lugar (controle de acesso, overflow de
  inteiros, reentrância, questões específicas de WASM).
- **Padrões acima de invenção** — FHIR R4 e SMART App Launch são o contrato.
  Prefira o padrão a um mecanismo customizado.
- Escreva testes. Documente interfaces públicas.

## Política de documentação

A documentação canônica é escrita em **inglês**. Uma versão em **Português do
Brasil** é mantida ao lado dela usando a convenção de sufixo `.pt-BR`
(ex.: `CONTRIBUTING.pt-BR.md`, `docs/pt-BR/…`) e está sempre referenciada
cruzadamente com o original em inglês. Se você alterar um documento em inglês,
por favor sinalize o arquivo `.pt-BR` correspondente para atualização — ou
atualize você mesmo(a) se puder.

## Segurança

Para reportar uma vulnerabilidade, **não abra uma issue pública** — siga o
processo descrito em [SECURITY.md](SECURITY.md).

## Licença

A deEHR é distribuída sob a [Licença MIT](LICENSE). Ao contribuir, você
concorda que suas contribuições serão licenciadas sob os mesmos termos.
