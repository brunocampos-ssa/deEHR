# Heterogeneidade de Perfis de Consumidores, Composição de Bundles e Projeção Dinâmica

🌐 **Languages / Idiomas:** [English](consumer-profile-heterogeneity.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([consumer-profile-heterogeneity.md](consumer-profile-heterogeneity.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Rascunho (insumo para a transição Phase 0 → Phase 1)
- **Data:** 2026-05-30
- **Origens:** [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.pt-BR.md) (Proposto)

## Proveniência

Estes requisitos foram organizados a partir de uma conversa de
design-thinking com um CTO do mercado segurador brasileiro no final de maio
de 2026. O CTO revisou a arquitetura do deEHR no estado Phase 0 (ADRs
0001–0005, modelo de ameaças, design dos registros on-chain) e levantou três
preocupações que os artefatos existentes não cobrem plenamente. Identidade
preservada em anonimato; os pontos substantivos estão registrados aqui no
enquadramento próprio do deEHR.

## Declaração do problema

FHIR R4 é um *padrão de contratos*, não um único contrato. Três realidades
interligadas decorrem disso:

1. **Heterogeneidade de perfis.** Todo consumidor relevante de dados FHIR —
   um backbone nacional, uma operadora privada de saúde, uma rede
   hospitalar, uma instituição de pesquisa, uma jurisdição estrangeira —
   define o seu próprio perfil. O perfil especifica quais campos são
   obrigatórios, quais value sets estão vinculados e quais extensões
   aparecem. O mesmo recurso lógico `Patient` tem dezenas de formas
   canônicas legítimas.
2. **Bundles são a unidade atômica da realidade clínica.** Uma consulta não
   é uma escrita de `Patient`, depois uma escrita de `Practitioner`, depois
   uma escrita de `Encounter`. É um *Bundle* de Patient + Practitioner +
   Encounter + Observation + Condition + MedicationRequest, todos escritos e
   lidos juntos. Persistir como N escritas independentes de recurso perde a
   atomicidade que o evento clínico exige.
3. **Armazenamento centrado no paciente precisa servir consumidores
   heterogêneos.** Quando o paciente é a raiz dos dados, cada consumidor que
   lê esses dados precisa recebê-los moldados ao *seu* perfil. Código
   estático de conector definido em design-time por consumidor não escala
   para um marketplace de consumidores.

A [ADR-0005](../architecture/adr-0005-fhir-profile-selection.pt-BR.md)
antecipa o *espírito* de (1) com o padrão de dois níveis — deEHR-canônico
internamente + tradução por conector na borda — mas só modela backbones
regulatórios conhecidos (RNDS hoje, backbones nacionais irmãos amanhã). Não
modela consumidores comerciais arbitrários e não trata de (2) nem de (3).

## Consumidores no escopo

As classes de consumidor a seguir orientam os requisitos:

| Classe de consumidor | Exemplos | Forma típica de perfil |
| --- | --- | --- |
| Backbones nacionais | RNDS (Brasil), redes nacionais irmãs | Acoplado a workflow, mandatado por regulador |
| Operadoras privadas de saúde | Planos de saúde, coberturas suplementares | Específico do operador, orientado a faturamento |
| Redes hospitalares | Organizações prestadoras multi-site | Interno ao EHR, frequentemente baseado em US-Core / BR-Core |
| Autoridades de saúde pública | Vigilância, notificações | Value sets codificados, dado mínimo mandatário |
| Instituições de pesquisa | Consórcios de pesquisa vinculados a CEP/IRB | Desidentificado, específico do estudo |
| Aplicações do paciente | Cópia própria do paciente, exportações IPS, portabilidade | International Patient Summary ou similar |
| Paciente como consumidor dos próprios dados | App do próprio paciente, exportações para segunda opinião | deEHR-canônico ou IPS |

## Cenários-chave

### UC-1: Operadora lê recurso Patient no seu próprio perfil

Uma operadora detentora de uma concessão de consentimento verificável faz
`GET /fhir/Patient/{id}` com `Accept-Profile:
<url-canônica-perfil-operadora>`. O sistema DEVE retornar um recurso
`Patient` projetado para a forma do perfil da operadora (por exemplo, o
`PatientXProfile` da operadora-X que agrupa uma extensão `plan-membership` e
fixa cardinalidade 1..1 em `address`), ou retornar `406 Not Acceptable` com
uma lista dos perfis suportados se o solicitado for desconhecido.

### UC-2: Hospital grava um evento clínico como Bundle

Um hospital grava uma consulta completa como um `Bundle` FHIR do tipo
`transaction`, incluindo Patient + Practitioner + Encounter + Observation +
Condition + MedicationRequest. O sistema DEVE persistir todos os recursos
atomicamente: ou todos os recursos são gravados e uma única âncora on-chain
é confirmada, ou nenhum recurso é gravado e nenhuma âncora é confirmada.
Persistência parcial do Bundle NÃO DEVE ser possível.

### UC-3: Backbone nacional assina um workflow

O RNDS assina submissões de resultados laboratoriais. O sistema empurra
recursos moldados a `BRDiagnosticoLaboratorioClinico-3.2.1` conforme o
mapeamento existente na ADR-0005. Este caso já é coberto pela ADR-0005; está
listado aqui para verificar que a ADR-0006 não causa regressão.

### UC-4: Paciente exporta os próprios dados em perfil IPS

Um paciente solicita um resumo portátil no perfil HL7 International Patient
Summary via o seu app de paciente. O sistema DEVE projetar recursos
deEHR-canônicos para a forma IPS na leitura e retornar um Bundle do tipo
`document` adequado para continuidade de cuidado transfronteiriça.

### UC-5: Validação cruzada por perfil no momento da escrita

Quando uma escrita chega moldada a um perfil registrado (não deEHR-canônico),
o sistema DEVE validar a escrita contra o perfil declarado *e* contra o
perfil deEHR-canônico, rejeitar se qualquer um falhar, e persistir a projeção
deEHR-canônica. Mensagens de falha de validação DEVEM atribuir a falha a uma
restrição específica do perfil.

### UC-6: Descoberta de perfis declarados pelo consumidor

Um consumidor pergunta ao sistema "quais perfis você suporta para
`Patient`?". O sistema DEVE anunciar, em `/fhir/metadata`
(CapabilityStatement), a lista completa de perfis suportados por tipo de
recurso, com as suas URLs canônicas e uma indicação de se cada um suporta
leitura, escrita, ou ambos.

### UC-7: Relatórios de conformidade

Um operador precisa saber quais perfis de consumidor estão falhando
validação com mais frequência e quais restrições são as piores ofensoras.
O sistema DEVE emitir métricas de validação por perfil adequadas para uma
pilha de observabilidade.

## Requisitos não-funcionais

- **Atomicidade da escrita de Bundle.** Semântica tudo-ou-nada. Inclui a
  confirmação da âncora on-chain — a âncora NÃO DEVE ser confirmada a menos
  que todos os recursos tenham sido persistidos.
- **Orçamento de latência de leitura.** A projeção de perfil em tempo de
  leitura DEVE ser cacheável por tupla `(id do recurso, url do perfil, versão
  do recurso)`. Orçamento de latência da primeira leitura projetada: a
  definir em benchmarks de Phase 1; alvo de latência de leitura cacheada:
  < 50 ms p95 para um GET de recurso único.
- **Governança do registry de perfis.** Adicionar um perfil ao registry
  DEVE exigir um passo explícito de governança (adenda à ADR ou artefato
  revisável equivalente). Sem adições silenciosas de perfil.
- **Observabilidade de validação.** Contagens de aprovação/falha por perfil
  e drill-down de restrição DEVEM estar disponíveis sem mudança de código.
- **Compatibilidade retroativa com a ADR-0005.** Os mapeamentos existentes
  de conector BR-Core / RNDS-Principal DEVEM ser reexpressíveis como
  entradas do registry de perfis sem mudança semântica.
- **Contenção de PHI.** A transformação de perfis NÃO DEVE vazar dados
  através de fronteiras de criptografia. Um consumidor autorizado para
  `patient/Patient.rs` NÃO DEVE receber dados de `Observation` por efeito
  colateral de transformação de perfil.

## Fora de escopo (para este conjunto de requisitos)

- **Mapeamento de perfis entre jurisdições** quando os perfis de origem e
  destino divergem semanticamente (por exemplo, uma codificação brasileira
  de raça/cor não tem mapeamento um-para-um com uma codificação mexicana de
  autoidentificação racial). Isso é um problema de backbone irmão e está
  rastreado separadamente. A ADR-0006 assume que os perfis de origem e
  destino compartilham semântica de recurso FHIR canônica e divergem apenas
  em cardinalidade, vinculação de value set, ou extensões.
- **Criação de code system customizado.** A ADR-0005 §7 já restringe a
  accept-list de purpose-of-use; este conjunto de requisitos não propõe
  flexibilizá-la.
- **Evolução / versionamento de perfis.** Perfis publicados com URLs
  canônicas versionadas são tratados como entradas distintas no registry;
  migração de versão de perfil não está no escopo.

## Questões em aberto

Devem ser resolvidas durante a revisão da ADR-0006 e a prototipagem de
Phase 1:

1. **Mecanismo de declaração de perfil.** O cabeçalho HTTP `Accept-Profile`
   é o padrão FHIR R4 §3.2.0.4; deveríamos também aceitar uma extensão de
   escopo SMART (por exemplo, `patient/Patient.rs?_profile=<url>`) para
   clientes que não conseguem definir cabeçalhos customizados?
2. **Estratégia de cache de projeção.** Lazy-on-read com TTL,
   eager-on-write com pré-materialização, ou híbrida? Trade-off entre
   memória/armazenamento e latência de leitura.
3. **Estratégia de âncora do Bundle.** Âncora única sobre a serialização
   canônica do Bundle inteiro, ou raiz Merkle sobre hashes por recurso?
   Âncora única é mais barata; raiz Merkle suporta prova de inclusão por
   recurso sem revelar o resto do Bundle.
4. **Controle de mudança no registry.** É necessária uma adenda à ADR para
   cada novo perfil de consumidor, ou um PR leve de "adição de perfil" com
   revisão do mantenedor é suficiente quando o formato do registry estiver
   estável?
5. **Orçamento de performance do pipeline de validação.** Alvos concretos
   de latência / throughput para o pipeline de validação + projeção sob
   tamanhos realistas de Bundle (por exemplo, Bundle de consulta com 10
   recursos, Bundle de sumário de alta com 200 recursos).
6. **Bootstrap dos perfis de consumidor.** Conteúdo inicial do registry no
   lançamento de Phase 1 — o mínimo é deEHR-canônico + BR-Core + os perfis
   ativos de workflow RNDS-Principal. Devemos também já incluir o IPS
   (international patient summary) para casos de exportação ao paciente
   desde o primeiro dia?

## Implicações para Phase 1

Este conjunto de requisitos implica um sub-arco de Phase 1 focado em
**registry de perfis, validação e engine de transformação** — distinto do
MVP do contrato on-chain e do Signing & Fee Service. A observação paralela
do CTO de que isso é "trabalho para um bom engenheiro de dados" se encaixa:
o pipeline de validação + projeção é a peça de engenharia-de-dados que
sustenta a plataforma. A ADR-0006 captura a direção arquitetural; o
conjunto de issues de Phase 1 deve incluir esse sub-arco explicitamente.

## Referências

- [ADR-0006](../architecture/adr-0006-multi-consumer-profile-strategy.pt-BR.md)
  — a decisão arquitetural proposta a partir deste conjunto de requisitos.
- [ADR-0005](../architecture/adr-0005-fhir-profile-selection.pt-BR.md) — o
  padrão de perfis de dois níveis que este conjunto generaliza.
- [ADR-0002](../architecture/adr-0002-on-chain-registry-design.pt-BR.md) —
  semântica de ancoragem on-chain que intersecta a atomicidade de Bundle
  (UC-2).
- HL7 **FHIR R4 §3.2.0.4 (Profile negotiation)** —
  <https://hl7.org/fhir/R4/profiling.html#profile-negotiation>.
- HL7 **FHIR R4 Bundle** — <https://hl7.org/fhir/R4/bundle.html>.
- **Simplifier — projeto RNDS (semente da biblioteca de perfis)** —
  <https://simplifier.net/redenacionaldedadosemsaude/~resources?category=Profile>.
- HL7 **International Patient Summary (IPS)** —
  <https://hl7.org/fhir/uv/ips/>.
- [README — Visão geral da arquitetura](../../README.pt-BR.md).
