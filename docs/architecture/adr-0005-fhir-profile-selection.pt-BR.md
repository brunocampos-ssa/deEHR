# ADR-0005: Seleção de Perfis FHIR — Baseline R4, Compatível com RNDS, SMART v2

🌐 **Languages / Idiomas:** [English](adr-0005-fhir-profile-selection.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0005-fhir-profile-selection.md](adr-0005-fhir-profile-selection.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Aceito
- **Data:** 2026-05-28
- **Decisores:** mantenedores da deEHR

## Contexto

O modelo de dados clínico da deEHR é HL7 FHIR R4 e sua autorização é SMART
App Launch 2.x — ambos comprometidos no README do projeto. O que resta é a
decisão **em nível de perfil**: a quais perfis canônicos os recursos da deEHR
se conformam internamente, como esses se traduzem para a rede brasileira RNDS
na borda, o vocabulário exato de escopos SMART e os conjuntos de valores
codificados que o Consent Registry on-chain referenciará. Este ADR fecha
essas escolhas para a Fase 0 e desbloqueia a Q6 no
[ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md).

Forças que moldam esta decisão:

- **Brasil-primeiro, não Brasil-apenas.** A RNDS é o primeiro backbone
  nacional, mas o padrão de conector do README explicitamente antecipa
  outros backbones como módulos irmãos. O modelo de dados interno não pode
  ficar atrelado à semântica de perfis de um único regulador.
- **Dois IGs brasileiros em jogo.** Dois implementation guides brasileiros
  HL7-canônicos cobrem o espaço: **BR-Core**
  (<https://hl7.org.br/fhir/core/>) — o padrão clínico cross-resource,
  reutilizado pela RAC, pelo Sumário de Alta e pelos perfis de documento da
  ANS CMD — e **RNDS-Principal** (<https://rnds-fhir.saude.gov.br/>) —
  perfis com escopo de workflow para os fluxos específicos de submissão da
  RNDS atualmente em produção (resultados de laboratório, vacinação,
  dispensação de medicamentos). A conformidade é limitada por workflow: a
  RNDS rejeita submissões fora da forma de perfil publicada apenas para
  fluxos de submissão ativados.
- **Extensões brasileiras obrigatórias.** O BR-Core torna vários elementos
  específicos do Brasil obrigatórios em `Patient` (CPF, raça/etnia,
  identidade de gênero, sexo ao nascer). Carregá-los nativamente no Patient
  deEHR-canônico evita um problema de upgrade com perda na borda do
  conector RNDS.
- **Vínculo com SMART v2.** SMART App Launch 2.x traz escopos granulares
  (`<context>/<resource>.<cruds>[?query]`). O Consent Registry armazena
  apenas identificadores codificados de escopo — nunca texto livre — então
  a codificação on-chain deve ter comprimento limitado e ser reversível de
  forma inequívoca para a string canônica do escopo.
- **Propósito de uso on-chain.** Mesma restrição: um código, não uma
  frase. Deve ser HL7-canônico e interoperável.

## Decisão

### 1. Estratégia de perfis em duas camadas

A deEHR se conforma a **perfis deEHR-canônicos** internamente (perfis base
FHIR R4 mais as extensões mínimas documentadas em §4). O **conector RNDS**
traduz recursos deEHR de/para perfis **BR-Core** ou **RNDS-Principal** na
borda, por workflow. A plataforma central nunca importa um perfil
RNDS-Principal; apenas o conector o faz. Isso preserva o padrão de
"conector irmão": um futuro backbone do México / Portugal / Argentina é um
módulo par, não um refactor.

### 2. Versão FHIR e postura de conformidade

- **Formato de fio:** HL7 FHIR R4 (4.0.1), JSON, RESTful.
- **Conformidade interna:** todo recurso interno DEVE se conformar ao seu
  perfil deEHR-canônico (definido em §3 e §4).
- **Conformidade externa com a RNDS:** o conector RNDS DEVE traduzir cada
  recurso de saída para o perfil exigido pela RNDS para o workflow ativo
  (resultado de laboratório, vacinação, dispensação, documento de
  encontro clínico). Fora desses workflows, o conector PODE emitir
  recursos no formato BR-Core; a RNDS não aceita POSTs de recursos
  genéricos hoje.
- **Fixtures de mapeamento do conector** para cada par
  (deEHR → RNDS-Principal, deEHR → BR-Core) fazem parte do contrato do
  conector e são testadas unitariamente.

### 3. Catálogo de recursos do MVP e decisões de perfil por recurso

O MVP da Fase 0 cobre os sete recursos abaixo. O restante do catálogo
(AllergyIntolerance, Immunization, Procedure, Coverage, Claim,
ExplanationOfBenefit) está **previsto mas adiado** para um ADR de
acompanhamento quando os workflows correspondentes se tornarem ativos.

Decisões de perfil por recurso (deEHR-canônico → alvos do conector):

- **Patient → `DEEHRPatient`**
  - Base interna: FHIR R4 `Patient` + extensões do §4.
  - Alvo BR-Core:
    [`br-core-patient`](https://hl7.org.br/fhir/core/StructureDefinition-br-core-patient.html).
  - Alvo RNDS-Principal:
    [`BRIndividuo-1.0`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRIndividuo-1.0.html)
    para submissões cadastrais.
- **Encounter → `DEEHREncounter`**
  - Base interna: FHIR R4 `Encounter`.
  - Alvo BR-Core:
    [`br-core-encounter`](https://hl7.org.br/fhir/core/).
  - Alvo RNDS-Principal: n/d (sem Encounter RNDS-Principal — usado dentro
    de Bundles de documento como RAC, Sumário de Alta).
- **Observation → `DEEHRObservation`**
  - Base interna: FHIR R4 `Observation`.
  - Alvo BR-Core:
    [`br-core-observation`](https://hl7.org.br/fhir/core/).
  - Alvo RNDS-Principal:
    [`BRDiagnosticoLaboratorioClinico-3.2.1`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRDiagnosticoLaboratorioClinico-3.2.1.html)
    para submissões de resultados de laboratório.
- **Condition → `DEEHRCondition`**
  - Base interna: FHIR R4 `Condition`.
  - Alvo BR-Core:
    [`br-core-condition`](https://hl7.org.br/fhir/core/).
  - Alvo RNDS-Principal:
    [`BRCondicaoSaude`](https://rnds-fhir.saude.gov.br/StructureDefinition-BRCondicaoSaude.html).
- **MedicationRequest → `DEEHRMedicationRequest`**
  - Base interna: FHIR R4 `MedicationRequest`.
  - Alvo BR-Core:
    [`br-core-medicationrequest`](https://hl7.org.br/fhir/core/).
  - Alvo RNDS-Principal: *Prescrição Eletrônica* da RNDS (Draft no
    Simplifier — rastrear e reavaliar no GA).
- **Consent → `DEEHRConsent`**
  - Base interna: FHIR R4 `Consent`.
  - Alvo BR-Core:
    [`br-core-consent`](https://hl7.org.br/fhir/core/).
  - Alvo RNDS-Principal: n/d (sem perfil RNDS Consent publicado;
    consentimento tratado out-of-band via Conecte SUS).
- **DocumentReference → `DEEHRDocumentReference`**
  - Base interna: FHIR R4 `DocumentReference` + hash de blob off-chain +
    extensões de ancoragem de tx Klever.
  - Alvos BR-Core / RNDS-Principal: n/d (sem perfil hoje).

As URIs canônicas dos perfis seguem o padrão
`https://deehr.org/fhir/StructureDefinition/DEEHR<ResourceName>`.

### 4. Extensões deEHR-canônicas obrigatórias (cientes do Brasil por padrão)

Como o BR-Core torna os seguintes elementos obrigatórios e o conector não
pode fazer upgrade com perda no momento da submissão, o deEHR-canônico os
carrega nativamente. São metadados não-PHI (ou PHI sob a postura normal de
criptografia de PHI da deEHR):

- **Identificador CPF** (`identifier.system = https://saude.gov.br/fhir/sid/cpf`)
  em `DEEHRPatient`, cardinalidade 1..1. Validação: 11 dígitos + checksum
  de CPF.
- **Identificador CNS** (`identifier.system = https://saude.gov.br/fhir/sid/cns`)
  em `DEEHRPatient`, cardinalidade 0..1.
- **Raça / etnia** (`raca-br-ips`,
  <https://ips.saude.gov.br/fhir/StructureDefinition/raca-br-ips>) — 1..1,
  valor vinculado a BRRacaCor.
- **Identidade de gênero** (`identidade-genero-br-ips`) — 0..1.
- **Sexo ao nascer** (`sexo-nascimento-br-ips`) — 0..1.
- **CNES** (`Organization.identifier` com system
  `https://saude.gov.br/fhir/sid/cnes` — padrão; slug exato sujeito a
  confirmação do time RNDS) em `DEEHROrganization` (prestador /
  estabelecimento), cardinalidade 1..1 para organizações brasileiras.

Para pacientes ou organizações não brasileiros onboardados através de
backbones futuros, as cardinalidades obrigatórias relaxam via um padrão de
slicing — `cpf`, `cns`, `raca-br-ips` e `cnes` são 0..1 no slice
internacional. Esta é uma decisão de produto da deEHR, não um relaxamento
exigido pelo FHIR; conectores irmãos declararão seus próprios slices
jurisdicionais.

### 5. Vocabulário de escopos SMART v2

A deEHR adota a sintaxe de escopo v2 do **SMART App Launch 2.0 (STU2.2)**:
`<context>/<resourceType>.<cruds>[?<query>]`, com permissões como um
subconjunto em ordem de `cruds` (`c`=create, `r`=read, `u`=update/patch,
`d`=delete, `s`=search/history-type). Spec:
<https://hl7.org/fhir/smart-app-launch/STU2/scopes-and-launch-context.html>.

O conjunto de escopos MVP `v1` da deEHR:

| # | Escopo | Pode ser solicitado por | Concede |
| --- | --- | --- | --- |
| 1 | `openid` | Todos os clientes | ID token OIDC |
| 2 | `fhirUser` | Todos os clientes | Referência de recurso FHIR para o usuário autenticado |
| 3 | `launch/patient` | Apps de paciente e prestador | Contexto de paciente em standalone launch |
| 4 | `offline_access` | Apps de paciente e prestador | Refresh token de longa duração |
| 5 | `online_access` | Apps de prestador | Refresh token vinculado à sessão |
| 6 | `patient/Patient.rs` | App de paciente | Ler os próprios dados demográficos |
| 7 | `patient/Encounter.rs` | App de paciente | Ler os próprios encounters |
| 8 | `patient/Observation.rs` | App de paciente | Ler as próprias observations |
| 9 | `patient/Condition.rs` | App de paciente | Ler as próprias conditions |
| 10 | `patient/MedicationRequest.rs` | App de paciente | Ler os próprios medicamentos |
| 11 | `patient/DocumentReference.rs` | App de paciente | Ler os próprios documentos clínicos |
| 12 | `patient/Consent.crus` | App de paciente | Gerenciar o próprio consentimento (espelha o registro on-chain) |
| 13 | `user/Patient.rs` | Prestador | Ler pacientes em relação de cuidado |
| 14 | `user/Encounter.crus` | Prestador | Gerenciar encounters |
| 15 | `user/Observation.crus` | Prestador | Gerenciar observations |
| 16 | `user/Condition.crus` | Prestador | Gerenciar conditions |
| 17 | `user/MedicationRequest.crus` | Prestador | Prescrever |
| 18 | `user/DocumentReference.crus` | Prestador | Gerenciar notas clínicas |
| 19 | `system/*.rs` | Conector RNDS / bulk | Leitura ampla para intercâmbio interinstitucional |
| 20 | `system/Observation.rs?category=http://terminology.hl7.org/CodeSystem/observation-category\|laboratory` | Pipeline de laboratório | Ingestão / leitura apenas de laboratório |

Notas:

- `.crus` deliberadamente omite `d`. Deletes destrutivos passam por um
  fluxo de tombstone / revogação-de-consentimento, não por OAuth.
- O sufixo `?query` é restrito às granularidades em nível de categoria
  exigidas pelo US-Core para o MVP. Chaining, modifiers e `_filter`
  (marcados como experimentais na spec) estão fora do escopo de `v1`.
- O servidor de autorização da deEHR DEVE anunciar tanto a capability
  `permission-v1` quanto `permission-v2` em
  `/.well-known/smart-configuration` e aplicar o mapeamento normativo
  v1→v2 da spec (`.read → .rs`, `.write → .cud`, `.* → .cruds`) para
  clientes legados.

### 6. Codificação on-chain de escopos codificados

O Consent Registry armazena cada escopo concedido como uma tupla de forma
fixa:

```text
(deehr_scopes_version: u16, template_code: u16, param_hash: bytes16)
```

- `deehr_scopes_version` — a versão do manifesto de escopos da deEHR
  (`v1` = a tabela deste ADR).
- `template_code` — um índice `u16` estável no manifesto; ex.:
  `0x0008` = `patient/Observation.rs`.
- `param_hash` — `blake2b-128(canonical_query_string)`, ou tudo zero se o
  escopo não tem restrição `?query`. A string de query canônica é o
  componente de query do escopo com os parâmetros ordenados
  lexicograficamente e os valores URL-decoded.

O manifesto em si é um documento JSON assinado publicado em
`https://deehr.org/fhir/scopes/<version>.json`, imutável uma vez publicado;
novos escopos recebem novos template codes — códigos nunca são reusados.
Isso tem comprimento limitado on-chain (20 bytes), é auditável (o
manifesto é a única fonte da verdade) e é reversível — dada a tupla mais o
manifesto, qualquer verificador pode reconstruir a string canônica do
escopo e re-hashear para confirmar a restrição `?query`.

### 7. Code system de propósito de uso

O Consent Registry da deEHR codifica propósito de uso como uma tupla
`{system, code}` onde:

- `system = http://terminology.hl7.org/CodeSystem/v3-ActReason`
- `code` é extraído da accept-list da deEHR — um subconjunto do
  [value set `v3-PurposeOfUse`](https://hl7.org/fhir/R4/valueset-v3-PurposeOfUse.html)

Accept-list `v1` da deEHR:

| Apelido deEHR | Código HL7 | Display | Quando usar |
| --- | --- | --- | --- |
| TREATMENT | `TREAT` | treatment | Acesso clínico rotineiro por prestador responsável pelo tratamento |
| EMERGENCY | `ETREAT` | Emergency Treatment | Break-glass autorizado por concessão prévia do paciente |
| EMERGENCY_ROOM | `ERTREAT` | emergency room treatment | Acesso específico de pronto-socorro (subtipo de ETREAT) |
| RESEARCH | `HRESCH` | healthcare research | Compartilhamento de dados para pesquisa, vinculado a CEP/IRB |
| CLINICAL_TRIAL | `CLINTRCH` | clinical trial research | Inscrição em ensaio / contribuição de dados |
| PAYMENT | `HPAYMT` | healthcare payment | Operações de faturamento de operadora/pagador |
| COVERAGE | `COVERAGE` | coverage under policy or program | Determinação de elegibilidade / cobertura de seguro |
| OPERATIONS | `HOPERAT` | healthcare operations | Qualidade, auditoria, operações internas |
| PUBLIC_HEALTH | `PUBHLTH` | public health | Notificação compulsória / vigilância |
| PATIENT_REQUEST | `PATRQT` | patient requested | Exportação / portabilidade / compartilhamento iniciado pelo paciente |
| CARE_COORDINATION | `COC` | coordination of care | Continuidade entre prestadores |
| LEGAL | `HLEGAL` | legal | Intimação / divulgação legal |

`Consent.scope` (o eixo ortogonal "qual tipo de declaração de
consentimento") é populado a partir de
`http://terminology.hl7.org/CodeSystem/consentscope` para conformidade
FHIR, mas **não** é usado na comparação para emissão de token — a
verificação de propósito é uma comparação exata de código contra a tabela
acima, exceto pela política explícita de escalada break-glass
`ETREAT`/`BTG`.

Se uma necessidade futura de produto da deEHR não puder ser expressa no
value set v3-PurposeOfUse (ex.: um "compartilhamento RNDS" específico do
Brasil), um `CodeSystem` da deEHR será introduzido via um ADR de
acompanhamento, com entradas `concept-map` explícitas de volta para
v3-PurposeOfUse para interoperabilidade.

### 8. Níveis de conformidade (resumo)

- **DEVE** conformar-se aos perfis deEHR-canônicos para qualquer recurso
  manipulado pela plataforma central.
- **DEVERIA** conformar-se ao BR-Core ao interoperar com entidades
  brasileiras de saúde fora dos workflows de submissão da RNDS.
- **DEVE** conformar-se ao perfil RNDS-Principal específico do workflow ao
  submeter à RNDS (resultado de laboratório, vacinação, dispensação, etc.).
- **DEVE** anunciar tanto as capabilities `permission-v1` quanto
  `permission-v2` do SMART.
- **DEVE** rejeitar concessões de consentimento cujo código de propósito
  de uso não esteja na accept-list do §7.

## Consequências

### Aspectos positivos

- Construído inteiramente sobre artefatos HL7-canônicos: base FHIR R4,
  BR-Core, RNDS-Principal, SMART v2, v3 PurposeOfUse. Zero code systems
  específicos da deEHR em `v1` — interoperabilidade é o padrão.
- O padrão de conector é genuinamente portável. Adicionar um backbone do
  México / Portugal / Argentina é um novo módulo de conector, não um
  reescrita do modelo interno.
- O modelo de dados do Consent Registry on-chain (Q6 do ADR-0002) está
  agora concretamente definido: escopo = `(version, template_code,
  param_hash)`; propósito de uso = código v3 ActReason da accept-list do
  §7.
- Extensões obrigatórias específicas do Brasil (CPF, raça/etnia, CNES)
  vivem nativamente no deEHR-canônico, removendo um risco de upgrade com
  perda na borda da RNDS.

### Aspectos negativos / riscos

- **Extensões obrigatórias do BR-Core adicionam peso a onboards não
  brasileiros.** `raca-br-ips` e CPF são 1..1 no slice brasileiro; o slice
  internacional os relaxa, mas o padrão de slicing carrega complexidade
  operacional na UI de produto e na validação.
- **Risco do perfil Draft RNDS-Principal.** O perfil RNDS Prescrição
  Eletrônica está em Draft no Simplifier; o mapeamento do conector para
  `MedicationRequest` PODE mudar antes do seu GA. Rastrear nas questões em
  aberto.
- **Sem perfil de Consent RNDS publicado.** A semântica de consentimento
  para o pipeline da RNDS é tratada operacionalmente fora do FHIR (via
  Conecte SUS). Se a RNDS posteriormente publicar um perfil de Consent, o
  conector precisará de um novo mapeamento; a semântica de consentimento
  da deEHR dentro da plataforma não mudará.
- **O manifesto de escopos torna-se infraestrutura operacional.** O
  manifesto `https://deehr.org/fhir/scopes/v1.json` é um artefato
  assinado, versionado e hospedado publicamente. Sua hospedagem,
  assinatura e postura de monitoramento de integridade é uma nova
  responsabilidade operacional.
- **Sem perfil brasileiro de DocumentReference hoje.** R4 base mais as
  extensões da deEHR são suficientes internamente; isso precisará de
  revisão se o Brasil publicar um perfil DocumentReference Prontuário do
  Cidadão.

## Alternativas consideradas

- **Usar perfis RNDS-Principal como modelo de dados interno.** Rejeitada
  — acopla toda a plataforma à semântica de um único regulador,
  contrariando o padrão de conector irmão do README. Mais barato no curto
  prazo, mas bloqueia evolução multi-backbone.
- **Usar HL7 IPS (International Patient Summary) como baseline interno.**
  Considerada por amigabilidade transfronteiriça. Rejeitada — IPS é seu
  próprio conjunto de perfis com seu próprio ônus de conformidade, e não
  casa com as especificidades do Brasil (CPF, CNS, raca-br-ips) sem
  trabalho de overlay. Ajuste pior para Brasil-primeiro, sem ajuste melhor
  para flexibilidade de conector irmão do que a abordagem escolhida.
- **Definir um code system customizado de propósito de uso da deEHR em
  `v1`.** Rejeitada — sem necessidade concreta de produto que o v3
  PurposeOfUse não consiga expressar hoje; um sistema customizado
  sacrificaria interoperabilidade por flexibilidade hipotética.
- **Fazer hash da string canônica completa do escopo on-chain (sem
  registro de templates).** Rejeitada — auditável mas não reversível sem
  um índice lateral, e não oferece superfície de governança para impedir
  que clientes inventem escopos que nunca foram revisados.
- **Codificar escopos on-chain como uma bitmask de permissões
  pré-definidas.** Rejeitada — quebra quando restrições `?query` entram
  em cena.

## Questões em aberto

Estes itens permanecem a serem resolvidos antes de revisões subsequentes do
ADR-0005 (registradas via Adenda conforme a política append-only de ADRs do
repositório):

1. **Autoridade formal do BR-Core.** O BR-Core parece ser o padrão de fato
   brasileiro cross-resource, mas não é formalmente referenciado a partir
   de `rnds-guia.saude.gov.br`. Confirmar com DATASUS / Ministério da
   Saúde qual IG governa novas submissões de documento.
2. **URL exato do system do identificador CNES.** Verificar a URL canônica
   do system contra BREstabelecimentoSaude-1.0.
3. **Estabilidade do perfil RNDS Prescrição Eletrônica** — rastrear o
   status no Simplifier; revisitar quando o perfil sair do Draft.
4. **Perfil brasileiro de DocumentReference** — rastrear se a RNDS /
   Brasil publica um (roadmap Prontuário do Cidadão).
5. **Perfil RNDS de Consent** — confirmar se a RNDS não tem um perfil de
   Consent não-público / draft em um release mais novo antes de travar o
   mapeamento de Consent da deEHR.
6. **Alinhamento com IHE Privacy Consent on FHIR (PCF)** — o Brasil às
   vezes se apoia em IHE PCF; avaliar em um ADR de acompanhamento se a
   deEHR deve vincular a PCF além do v3 PurposeOfUse.
7. **Plano operacional de hospedagem e assinatura do manifesto de escopos
   SMART** — escolha concreta de custódia da chave de assinatura,
   transparency log e postura de integrity-monitor para o artefato
   `deehr-scopes-v1.json`.
8. **Decisão de produto sobre `Patient-Consent.crus`** — confirmar com
   produto se pacientes devem poder atualizar o próprio Consent via SMART
   (o registro on-chain já é de propriedade do paciente).

## Referências

- [README.md](../../README.md) — seções *Standards & Building Blocks* e
  *RNDS & Government Integration*.
- [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md) — Identidade e
  Gestão de Chaves (Custódia Progressiva).
- [ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md) — Design dos
  Registros On-chain (vínculo de conjunto de valores codificados do
  Consent Registry).
- [ADR-0004](adr-0004-did-klever-method.pt-BR.md) — método DID
  `did:klever`.
- HL7 **FHIR R4 (4.0.1)** — <https://hl7.org/fhir/R4/>.
- HL7 **SMART App Launch 2.0 (STU2.2)** —
  <https://hl7.org/fhir/smart-app-launch/STU2/>.
- HL7 **Terminology v3 ActReason** —
  <https://terminology.hl7.org/CodeSystem-v3-ActReason.html>.
- HL7 **FHIR R4 v3 PurposeOfUse value set** —
  <https://hl7.org/fhir/R4/valueset-v3-PurposeOfUse.html>.
- **BR-Core IG** — <https://hl7.org.br/fhir/core/>.
- **Guia de Implementação da RNDS** — <https://rnds-guia.saude.gov.br/>.
- **RNDS-Principal FHIR IG** — <https://rnds-fhir.saude.gov.br/>.
- **Simplifier — projeto RNDS** — <https://simplifier.net/redenacionaldedadosemsaude>.
- **Extensões IPS-Brasil (raca-br-ips, identidade-genero-br-ips,
  sexo-nascimento-br-ips)** — <https://ips.saude.gov.br/fhir/>.
- US Core IG v9 — SMART on FHIR Obligations and Capabilities —
  <https://build.fhir.org/ig/HL7/US-Core/scopes.html>.
- IHE **Privacy Consent on FHIR (PCF) v1.1.0** —
  <https://profiles.ihe.net/ITI/PCF/>.
