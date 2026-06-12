# Resolução de Identidade do Paciente, Record Linkage e o Master Patient Index

🌐 **Languages / Idiomas:** [English](patient-identity-resolution.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([patient-identity-resolution.md](patient-identity-resolution.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Rascunho (insumo para a transição Phase 0 → Phase 1)
- **Data:** 2026-06-12
- **Origens:** [ADR-0007](../architecture/adr-0007-patient-identity-resolution.pt-BR.md) (Proposto)

## Procedência

Estes requisitos foram montados a partir de uma conversa de design thinking de
**acompanhamento** com o mesmo CTO do mercado de seguros brasileiro que
originou a [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.pt-BR.md)
(ver os [requisitos de heterogeneidade de perfis de consumidores](consumer-profile-heterogeneity.pt-BR.md)).
Em junho de 2026 o CTO revisou o diagrama macro de arquitetura e os componentes
de persistência off-chain — **FHIR Gateway**, FHIR Server, IPFS (para os
documentos exportados) e o conector do RNDS — e levantou uma preocupação que
está *a montante* da ADR-0006: antes de o deEHR conseguir projetar os dados de
uma pessoa em vários formatos de perfil, ele precisa primeiro ser capaz de
decidir que dois registros recebidos **são a mesma pessoa**. A ADR-0006
resolve a heterogeneidade de perfis; ela pressupõe que a identidade do paciente
já está resolvida. E não está.

Identidade preservada; os pontos substantivos estão capturados aqui no
enquadramento do próprio deEHR.

## Definição do problema

O ator autenticado na camada SMART **não** é o sujeito do dado. Seguem-se
várias realidades que se cruzam:

1. **Identidade do requisitante ≠ identidade do sujeito.** O SMART on FHIR
   (padrão de autenticação adotado pela ADR-0005) autentica *quem está
   solicitando* — um paciente, um hospital, uma seguradora — e carrega os
   *escopos* a que esse requisitante tem direito. Ele **não** identifica *de
   quem é o registro* que está sendo gravado. Um hospital com uma concessão de
   consentimento válida pode chamar a API para persistir o prontuário de
   *Francisco José*; o requisitante é o hospital, o sujeito é o paciente.
2. **Não há chave primária de paciente compartilhada entre as fontes.**
   Múltiplos hospitais, laboratórios e clínicas persistem dados de forma
   assíncrona, cada um sob **o seu próprio profile FHIR** (conforme a
   ADR-0006), e **nenhum deles carrega a chave de paciente do deEHR**. A chave
   primária do paciente também **não está no token SMART** — o token autoriza o
   requisitante, não nomeia a identidade canônica do sujeito no deEHR.
3. **Gravações FHIR atômicas assumem "criar" por padrão.** Como toda chamada
   REST do FHIR é atômica e os dados demográficos de cada fonte diferem, um
   caminho de gravação ingênuo emite um `POST` (criar) a cada vez. Duas fontes
   gravando sobre a mesma pessoa real produzem **dois recursos `Patient`** — um
   registro fragmentado. Históricos clínicos fragmentados são um risco à
   segurança do paciente, não meramente um incômodo de qualidade de dado.
4. **A resolução é probabilística, não exata.** As fontes discordam sobre quais
   campos são obrigatórios e como os valores são codificados (exatamente a
   heterogeneidade que a ADR-0006 trata). A correspondência "é a mesma
   pessoa?" não pode depender de uma chave exata; exige um **Master Patient
   Index (MPI)** que pontue a similaridade demográfica e retorne uma decisão
   graduada.
5. **A persistência precisa ser match-first.** O caminho de gravação precisa
   resolver a identidade *antes* de persistir: um match confirmado deve
   **agregar** ao registro existente (`PUT` / update contra a chave resolvida)
   em vez de **criar** uma duplicata (`POST`). O registro canônico resolvido é
   o **Golden Record** (registro de ouro) do paciente.

Os artefatos existentes do deEHR não modelam nada disso. A ADR-0001 cobre a
identidade do *requisitante* (contas de paciente em custódia progressiva, DIDs
`did:klever`). A ADR-0005 / ADR-0006 cobrem a heterogeneidade de *perfis*.
**Nenhum artefato cobre a resolução de identidade do *sujeito* entre gravações
de fontes heterogêneas, sem chave e assíncronas.**

## Fontes no escopo

As seguintes classes de fonte orientam os requisitos. Note que estas são
*produtoras de registros sobre um paciente*, distintas da tabela de
*consumidores* do documento de
[heterogeneidade de perfis de consumidores](consumer-profile-heterogeneity.pt-BR.md)
— embora muitos atores sejam ambos.

| Classe de fonte | Identifica o paciente por | Confiabilidade da chave |
| --- | --- | --- |
| Redes hospitalares | MRN interno do EHR + dados demográficos; às vezes CNS | MRN local é único apenas dentro da fonte |
| Laboratórios / clínicas | Dados demográficos do pedido; frequentemente parciais | Frequentemente sem identificador nacional |
| Backbone nacional (RNDS) | CNS (Cartão Nacional de Saúde) | Alta quando presente; nem sempre presente |
| Seguradoras privadas | ID de plano/beneficiário + dados demográficos | Escopo da operadora, não é identidade clínica |
| Apps diretos do paciente | Conta deEHR / DID do próprio paciente | Autoritativa para aquele paciente |

Contexto brasileiro: **CPF** e **CNS** são candidatos a identificadores fortes,
mas não estão universalmente presentes nem garantidamente únicos/limpos nos
dados de origem; são *evidência*, não uma chave primária garantida.

## Cenários-chave

### UC-1: Fonte persiste um registro de um paciente identificado só por dados demográficos

Um hospital com uma concessão de consentimento válida faz `POST` de um
`Patient` (e um Bundle clínico anexo — ver ADR-0006 UC-2) carregando nome,
data de nascimento, nome da mãe e um MRN local, mas nenhuma chave de paciente
do deEHR. O sistema DEVE resolver os dados demográficos contra o MPI **antes**
da persistência e vincular a gravação ao Golden Record resolvido, em vez de
cunhar uma nova identidade de paciente por padrão.

### UC-2: A correspondência probabilística retorna "possível match" → revisão por steward

Um `Patient` recebido pontua na faixa ambígua (acima do limiar de não-match,
abaixo do limiar de auto-match). O sistema NÃO DEVE fazer merge silencioso nem
duplicar silenciosamente. DEVE encaminhar o candidato a uma **fila de data
steward** para adjudicação humana, com a evidência de comparação (quais campos
casaram, quais conflitaram, a pontuação) apresentada para revisão, e DEVE
registrar a decisão do steward em uma trilha de auditoria.

### UC-3: Dois Golden Records existentes são descobertos como a mesma pessoa

Dois Golden Records distintos (cada um já acrescido a partir de várias fontes)
são posteriormente determinados como sendo uma só pessoa. O sistema DEVE
suportar um **merge** que consolide ambos sob um identificador mestre
**preservando os registros de origem e seus identificadores originais**
(semântica de link por baixo), e o merge DEVE ser **reversível** (unmerge)
para corrigir uma decisão errada.

### UC-4: Registro recebido casa com um Golden Record existente → agregar, não duplicar

Um `Patient` recebido casa com um Golden Record existente com alta confiança.
A operação de persistência DEVE agregar à chave canônica existente (semântica
de update condicional / `PUT`) em vez de criar um segundo `Patient` (`POST`).
Os recursos clínicos na mesma gravação se acumulam no **mesmo prontuário**.

### UC-5: Consumidor invoca `$match` diretamente

Um consumidor (p.ex., uma seguradora reconciliando seu próprio beneficiário com
o paciente do deEHR, ou um hospital verificando antes de uma gravação) chama
`POST /fhir/Patient/$match` com um `Patient` parcial. O sistema DEVE retornar
um `Bundle` searchset de recursos `Patient` candidatos ordenados do mais ao
menos provável, cada entrada carregando um **search score (0–1)** e a extensão
FHIR **`match-grade`** (`certain` / `probable` / `possible` /
`certainly-not`), honrando `onlyCertainMatches`, `onlySingleMatch` e `count`.

### UC-6: Vínculo Golden Record ↔ identidade on-chain do paciente

Pacientes do deEHR têm um DID `did:klever` (ADR-0001), e o consentimento
(ADR-0002) é chaveado pelo DID do paciente. Quando uma fonte grava sobre um
paciente que *é* um paciente deEHR onboarded, o MPI DEVE ser capaz de resolver
dados demográficos recebidos → Golden Record → **DID do paciente**, de modo que
a gravação seja governada pela concessão de consentimento correta. Quando uma
fonte grava sobre uma pessoa que *ainda não* é um paciente onboarded, o sistema
DEVE ser capaz de manter um Golden Record que ainda não tem DID e vincular um
DID mais tarde no onboarding sem perder o histórico acumulado.

### UC-7: Correção de um match errado (desvincular / desfazer merge)

Um match anteriormente auto-aprovado ou aprovado por steward é descoberto como
errado (duas pessoas diferentes foram vinculadas). O sistema DEVE suportar
separá-las novamente, re-derivar cada Golden Record, e DEVE registrar a
correção — incluindo quais recursos clínicos vão para qual registro — na trilha
de auditoria.

### UC-8: Resolução de identidade dentro do pipeline de gravação de Bundle

Um Bundle `transaction` (ADR-0006 UC-2 / §4) contém um `Patient` mais recursos
clínicos. A resolução de identidade DEVE rodar sobre a entrada `Patient`
**antes** da projeção canônica e do anchoring on-chain, e a chave mestre
resolvida DEVE alimentar o reference rewriting do Bundle (ADR-0006 §4 passo 3)
para que os recursos clínicos se vinculem ao paciente resolvido. Isso
**emenda a ADR-0006 §4**.

## Requisitos não funcionais

- **Qualidade do match.** Limiares de auto-match e não-match configuráveis, com
  uma faixa de revisão humana entre eles. Metas concretas de precisão/recall:
  a definir na Phase 1 contra um conjunto de avaliação rotulado; o pipeline
  DEVE tornar o falso-merge (vincular duas pessoas diferentes) o erro mais
  custoso a se evitar, já que um falso-merge contamina cruzadamente históricos
  clínicos.
- **Determinismo e auditabilidade das decisões.** Toda decisão de match (auto,
  aprovada por steward, rejeitada por steward, merge, unmerge) DEVE ser
  reproduzível a partir dos insumos registrados + versão do algoritmo, e DEVE
  ser registrada com a evidência e o ator decisor.
- **Fluxo de data steward.** Uma fila + UI de revisão para possíveis matches,
  com evidência de comparação e trilha de auditoria, é um requisito de primeira
  classe, não um adendo.
- **Sem PHI on-chain.** O MPI opera **inteiramente off-chain**. Dados
  demográficos de comparação, conjuntos de candidatos e decisões de steward
  nunca tocam a chain Klever; a camada on-chain continua vendo apenas DIDs,
  hashes, CIDs e status codificado (ADR-0002 §2). O vínculo Golden-Record→DID é
  o único ponto de contato, e ele cruza a fronteira como uma referência de DID,
  não como PHI.
- **Reversibilidade.** Link e merge DEVEM ser reversíveis; o sistema NÃO DEVE
  destruir registros de origem no merge.
- **Orçamento de desempenho para `$match`.** Metas concretas de
  latência/throughput para a resolução no caminho quente de gravação: a definir
  em benchmarks da Phase 1; a resolução fica no caminho crítico de toda
  gravação clínica, então compartilha o orçamento de latência com o engine de
  projeção (ADR-0006) e o commit do anchor (ADR-0002 / ADR-0006 §4).
- **Compatibilidade retroativa.** Gravações diretas do paciente (um paciente
  gravando pelo seu próprio app deEHR, já vinculado a um DID) DEVEM curto-
  circuitar a correspondência probabilística — a identidade do sujeito é
  conhecida de forma autoritativa.

## Fora de escopo (para este conjunto de requisitos)

- **Correspondência biométrica / fuzzy de imagem.** Apenas record linkage
  baseado em dados demográficos e identificadores.
- **Reconciliação de identidade entre jurisdições.** Casar um paciente
  brasileiro com um sistema de identidade nacional estrangeiro é uma
  preocupação de backbone-irmão, rastreada junto com o mapeamento de perfis
  entre jurisdições adiado pela ADR-0006.
- **CPF/CNS como chave primária única garantida.** Tratados como evidência
  forte na passada determinística, não como chave única e limpa presumida. Uma
  política de confiança em identificador nacional é uma questão em aberto, não
  um invariante decidido.
- **Efeitos do merge de pacientes sobre extrações de seguradora/pesquisa
  já entregues.** Como um merge re-chaveia dados já entregues a um consumidor é
  adiado.

## Questões em aberto

A serem resolvidas durante a revisão da ADR-0007 e na prototipagem da Phase 1:

1. **Construir vs integrar o engine de correspondência.** Implementar record
   linkage probabilístico Fellegi-Sunter internamente (Go/Rust), ou integrar um
   EMPI open-source existente? Trade-off: controle + nenhum PHI saindo da
   fronteira vs time-to-value + correspondência testada em produção.
2. **Política de confiança em identificador nacional.** Quanto peso CPF e CNS
   carregam na passada determinística, e qual a política quando eles conflitam
   com uma forte discordância demográfica?
3. **Limiares padrão e SLA do steward.** Limiares iniciais de auto-match /
   não-match, o tamanho da faixa de revisão e o SLA operacional para a fila de
   steward.
4. **Política de persistência em possível-match.** Em um possível-match, a
   gravação bloqueia aguardando a revisão do steward, ou persiste em um registro
   **provisório** reconciliado depois? Trade-off de segurança clínica vs
   disponibilidade.
5. **Momento da vinculação do DID.** Quando um Golden Record adquire um DID
   `did:klever` — apenas no onboarding do paciente, ou um Golden Record
   originado por provedor pode existir sem DID e ser reivindicado depois? (Liga-
   se à custódia progressiva da ADR-0001.)
6. **Merge entre DIDs já vinculados.** Se dois Golden Records que estão *cada
   um* já vinculados a um DID de paciente diferente vierem a ser uma só pessoa,
   o merge colide duas identidades on-chain — cada uma potencialmente com suas
   próprias concessões de consentimento e anchors. Qual a consequência on-chain,
   e isso é sequer permitido, ou precisa ser um procedimento manual de alta
   garantia?
7. **Exposição de `$match` a consumidores externos na v1.** Expor `$match` a
   consumidores externos desde o dia um, ou mantê-lo interno ao pipeline de
   gravação na v1 e expô-lo depois?

## Implicações para a Phase 1

Este conjunto de requisitos implica um **segundo** sub-arco de engenharia de
dados load-bearing para a Phase 1, ao lado do engine de registry/projeção de
perfis da ADR-0006: o **pipeline de MPI / resolução de identidade** (normalizar
→ blocking → pontuar → link/merge → revisão por steward). Os dois sub-arcos são
acoplados — a resolução de identidade roda *antes* da projeção no caminho de
gravação — mas são corpos de trabalho distintos, distintos do MVP de contrato
on-chain e do Signing & Fee Service. O conjunto de issues da Phase 1 precisa
incluir o sub-arco de MPI explicitamente e sequenciá-lo à frente (ou ao lado)
do engine de projeção, já que a projeção pressupõe uma identidade de paciente
resolvida.

## Referências

- [ADR-0007](../architecture/adr-0007-patient-identity-resolution.pt-BR.md) — a
  decisão arquitetural proposta orientada por este conjunto de requisitos.
- [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.pt-BR.md)
  — estratégia FHIR multi-consumidor; este conjunto está a montante dela e
  emenda o pipeline de gravação de Bundle da §4.
- [ADR-0005](../architecture/adr-0005-fhir-profile-selection.pt-BR.md) — seleção
  de profile FHIR; o padrão de autenticação SMART on FHIR referenciado na
  definição do problema.
- [ADR-0002](../architecture/adr-0002-on-chain-registry-design.pt-BR.md) —
  design do registro on-chain; o invariante de sem-PHI e o Registro de
  Consentimento chaveado por DID limitam a colocação off-chain do MPI e o
  vínculo Golden-Record→DID.
- [ADR-0001](../architecture/adr-0001-identity-and-key-management.pt-BR.md) —
  identidade e gestão de chaves; DID `did:klever` do paciente e custódia
  progressiva.
- HL7 **Operação FHIR R4 `Patient/$match`** —
  <https://hl7.org/fhir/R4/patient-operation-match.html>.
- HL7 **Extensão FHIR `match-grade`** —
  <https://hl7.org/fhir/R4/valueset-match-grade.html>.
- **Health Samurai — Master Patient Index and Record Linkage** —
  <https://www.health-samurai.io/articles/master-patient-index-and-record-linkage>.
- **fastrivertech/fhir-mpi — interface EMPI baseada em FHIR** —
  <https://github.com/fastrivertech/fhir-mpi>.
- Fellegi, I. P., & Sunter, A. B. (1969). *A Theory for Record Linkage.*
  Journal of the American Statistical Association.
