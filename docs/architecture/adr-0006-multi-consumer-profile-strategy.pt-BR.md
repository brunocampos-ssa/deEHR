# ADR-0006: Estratégia FHIR Multi-Consumidor — Registry, Projeção Dinâmica e Atomicidade de Bundle

🌐 **Languages / Idiomas:** [English](adr-0006-multi-consumer-profile-strategy.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0006-multi-consumer-profile-strategy.md](adr-0006-multi-consumer-profile-strategy.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Proposto
- **Data:** 2026-05-30
- **Decisores:** mantenedores do deEHR

## Contexto

A [ADR-0005](adr-0005-fhir-profile-selection.pt-BR.md) fixa o modelo de dados
interno do deEHR em FHIR R4 + perfis deEHR-canônicos e traduz para BR-Core /
RNDS-Principal na borda por meio de conectores específicos. O padrão de
"perfis em dois níveis" daquela ADR foi desenhado para um conjunto pequeno e
conhecido de backbones nacionais.

Uma sessão de design-thinking com um CTO do mercado segurador brasileiro
(ver [documento de requisitos](../requirements/consumer-profile-heterogeneity.pt-BR.md))
levantou três preocupações que a ADR-0005 não cobre:

1. **Heterogeneidade de perfis para além de reguladores.** Todo consumidor
   comercial — cada operadora, cada rede hospitalar — define o próprio
   perfil. O modelo de "conector hardcoded em design-time" da ADR-0005 não
   escala para um marketplace de consumidores.
2. **Bundles como unidade atômica da realidade clínica.** Uma consulta é um
   `Bundle` FHIR de múltiplos recursos, não uma sequência de escritas
   independentes. A ADR-0005 menciona Bundles apenas en passant; a semântica
   de ancoragem on-chain da ADR-0002 não foi desenhada em torno da
   granularidade de Bundle.
3. **Projeção dinâmica de perfil para armazenamento centrado no paciente.**
   Quando o paciente é a raiz dos dados, cada consumidor que lê precisa
   recebê-los moldados ao *seu* perfil, declarado pelo consumidor em tempo
   de requisição — não escolhido de um catálogo fixo pequeno.

Forças que moldam a decisão:

- **Compatibilidade com a ADR-0005.** O padrão de dois níveis está correto em
  espírito; esta ADR o generaliza sem contradizer nenhuma das decisões
  por-recurso, o manifesto de escopos, ou a accept-list de purpose-of-use da
  ADR-0005.
- **Compatibilidade com a ADR-0002.** A atomicidade de Bundle precisa se
  alinhar ao modelo de commit de âncora on-chain: um Bundle, uma transação,
  uma âncora.
- **Contenção de PHI.** Uma transformação de perfil NÃO DEVE virar um canal
  paralelo que vaze dados através da fronteira do escopo SMART concedido a um
  token.
- **Carga de engenharia de dados.** O pipeline de validação + projeção é a
  maior carga não-criptográfica da plataforma. A Phase 1 precisa de um sub-arco
  para isso, distinto do MVP do contrato on-chain e do Signing & Fee Service.
- **HL7-canônico primeiro.** A FHIR R4 §3.2.0.4 já define o contrato de
  negociação de perfis (cabeçalhos HTTP `Accept-Profile` / `Content-Profile`).
  O deEHR adota o padrão em vez de inventar um mecanismo paralelo.

## Decisão

### 1. Profile Registry

O deEHR mantém um **Profile Registry** interno — o catálogo autoritativo de
todo perfil FHIR que a plataforma reconhece. Cada entrada registra:

- **URL canônica** — o valor de `StructureDefinition.url` do perfil (por
  exemplo,
  `https://hl7.org.br/fhir/core/StructureDefinition/br-core-patient`).
- **tipo de recurso** — o recurso FHIR que o perfil restringe.
- **status** — `active`, `deprecated`, ou `superseded` (transições
  unidirecionais).
- **suporte a leitura** — se o perfil pode ser solicitado via
  `Accept-Profile`.
- **suporte a escrita** — se o perfil pode ser fornecido em uma escrita.
- **vínculo de validador** — referência à StructureDefinition + vinculações
  de value set usadas em tempo de validação.
- **vínculo de transformação** — referência às regras de projeção /
  projeção-reversa usadas em runtime (ver §3).
- **jurisdição** — tag informacional (por exemplo, `BR`, `US`,
  `UV`/universal).
- **proveniência** — link para o artefato de governança (adenda à ADR, PR de
  adição de perfil) que admitiu o perfil.

O registry é semeado no lançamento de Phase 1 com: deEHR-canônico (todo
recurso na §3 da ADR-0005), BR-Core (todo recurso que a ADR-0005 mapeia
para ele) e os perfis de workflow RNDS-Principal ativos sob a ADR-0005. O
escopo de adoção do IPS é tratado separadamente — ver a questão em aberto
nº 7.

Os mapeamentos existentes de conector BR-Core / RNDS-Principal da ADR-0005
são reexpressos como entradas do Profile Registry sem mudança semântica. As
decisões por-recurso e a postura de conformidade da ADR-0005 (§3, §4, §8)
permanecem em vigor.

### 2. Negociação de Perfil de Consumidor

O deEHR adota o contrato padrão FHIR R4 de negociação de perfis:

- **Leitura.** Um cliente envia `Accept-Profile: <url-canônica>` em um `GET`.
  O servidor retorna o recurso projetado para aquele perfil e seta
  `Content-Profile: <url-canônica>` na resposta. Se o perfil solicitado não
  estiver no Registry, o servidor retorna `406 Not Acceptable` com um
  `OperationOutcome` listando perfis suportados para o tipo de recurso.
- **Escrita.** Um cliente envia `Content-Profile: <url-canônica>` em um
  `POST` / `PUT`. O servidor valida contra aquele perfil (§5) e persiste a
  projeção deEHR-canônica.
- **Default.** Sem `Accept-Profile`, leituras retornam a forma
  deEHR-canônica. Sem `Content-Profile`, escritas são assumidas como
  deEHR-canônicas e validadas como tal.
- **Anúncio.** O `CapabilityStatement` em `/fhir/metadata` declara, para cada
  tipo de recurso, a lista completa de URLs em `supportedProfile` mais uma
  extensão deEHR em cada entrada `rest.resource.profile` indicando
  `read-supported` e `write-supported`.

A sintaxe de escopos SMART não é estendida para carregar informação de
perfil. O manifesto de escopos da ADR-0005 permanece inalterado. A seleção
de perfil é uma preocupação ortogonal no nível HTTP; uma única concessão
SMART pode ler ou escrever múltiplas formas de perfil do mesmo tipo de
recurso se autorizada.

### 3. Engine de Projeção Dinâmica

Uma **Projection Engine** fica entre a API REST FHIR e o store canônico. É
bidirecional:

- **Caminho de leitura.** Recurso deEHR-canônico (do storage) → perfil de
  consumidor (por `Accept-Profile`). A engine aplica as regras de projeção
  de leitura do perfil: omite campos que o perfil do consumidor não inclui,
  slicia extensões que o perfil restringe, revincula value sets conforme as
  vinculações terminológicas do perfil, e impõe cardinalidade (por exemplo,
  falha-fechada se o perfil mandata um campo que o deEHR-canônico não
  populou).
- **Caminho de escrita.** Recurso entrando moldado a um perfil de consumidor
  (por `Content-Profile`) → deEHR-canônico. A engine valida o input contra o
  perfil do consumidor e então aplica as regras de projeção-reversa do
  perfil para derivar o recurso canônico.

Projeções são declarativas. Cada entrada do Profile Registry vincula a uma
especificação de transformação — na implementação de Phase 1, a forma viável
mais simples é um documento de mapeamento baseado em FHIRPath; FHIR Mapping
Language (FHIRPath / StructureMap) é o alvo de prazo mais longo.

**Cache.** Leituras projetadas são cacheadas pela tupla `(id do recurso, url
canônica do perfil, versão do recurso)`. A invalidação do cache é dirigida
pela escrita: em qualquer update de recurso, todas as entradas de cache de
projeção com chave naquele id de recurso são invalidadas. O backend do cache
é detalhe de implementação e não é fixado por esta ADR.

### 4. Atomicidade de Escrita de Bundle

O deEHR processa Bundles FHIR do tipo transaction (`Bundle.type =
transaction`) atomicamente. O pipeline completo:

1. **Validação no nível do Bundle.** Cada `Bundle.entry.resource` é validado
   contra o perfil declarado (via `meta.profile` em cada entry, ou
   defaultado para deEHR-canônico). Qualquer falha de validação rejeita o
   Bundle inteiro com `400 Bad Request` + um `OperationOutcome` enumerando
   cada falha, incluindo o índice da entry.
2. **Projeção canônica.** Cada entry validada é projetada para a forma de
   recurso deEHR-canônica (§3 caminho de escrita).
3. **Reescrita de referências.** Referências internas `urn:uuid:` entre
   entries do Bundle são resolvidas para IDs de recurso emitidos pelo deEHR
   em um único pass antes da persistência, conforme FHIR R4 Bundle §3.3.1.
4. **Persistência atômica.** Recursos são persistidos no storage off-chain
   em um único batch durável — ou todo recurso entra ou nenhum entra.
5. **Commit da âncora on-chain.** Uma única transação de âncora é submetida
   à chain Klever. O payload da âncora é uma **raiz Merkle** sobre os hashes
   canônicos por-recurso mais o hash de metadados do Bundle. O commit da
   âncora é condicional ao sucesso do §4-passo-4; a persistência off-chain só
   é final após a confirmação da transação de âncora.

A estrutura de raiz Merkle (em vez de um hash único sobre o Bundle
serializado) é escolhida para que um consumidor possa depois provar
inclusão de um único recurso em um Bundle sem revelar o resto. Isso
refina a §6 da
[ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md), que especifica o
Anchor & Audit Registry como armazenamento de "hashes de integridade de
Bundles FHIR criptografados pareados com os seus CIDs de IPFS" sem fixar
a estrutura desse hash. A ADR-0002 vai exigir uma adenda para declarar a
estrutura de raiz Merkle (sobre hashes canônicos por-recurso + hash de
metadados do Bundle) como a forma canônica de âncora.

Se o commit da âncora on-chain falhar após a persistência off-chain, a
plataforma faz retry com backoff limitado (política operacional, não de
nível de ADR); o modo de falha final dispara um delete off-chain
compensatório e um alerta administrativo. O Bundle é observavelmente ou
totalmente comitado (off-chain + on-chain) ou totalmente revertido;
nenhum estado intermediário é visível para os consumidores.

### 5. Composição de Leitura de Bundle

Um `GET` em um id de Bundle retorna os recursos do Bundle, cada um
projetado para o perfil solicitado via `Accept-Profile` (uniforme em todas
as entries do Bundle). Se um consumidor solicitar um perfil que seja
incompatível com uma ou mais entries (por exemplo, o perfil do consumidor
mandata um campo que um recurso Patient do Bundle não tem), o servidor
responde `406 Not Acceptable` com um `OperationOutcome` listando os índices
das entries ofensoras.

Document Bundles (`Bundle.type = document`) como Sumário de Alta e RAC são
saídas de primeira classe deste pipeline.

### 6. Validação Cruzada por Perfil no Tempo de Escrita

Em qualquer escrita — Bundle ou recurso único — o recurso é validado contra
**ambos** o perfil declarado pelo consumidor (`Content-Profile`) **e** o
perfil deEHR-canônico para o tipo de recurso. Qualquer falha rejeita a
escrita. Mensagens de falha de validação atribuem cada falha a um perfil +
id de restrição específicos.

### 7. Relatórios de Conformidade

O deEHR expõe `POST /fhir/<Resource>/$validate?profile=<url>` conforme a
operação padrão FHIR R4 de validação. A implementação roda o mesmo
validador usado internamente e retorna um `OperationOutcome`. Contagens de
aprovação/falha por perfil e distribuições de falha por restrição são
emitidas como métricas para a pilha de observabilidade.

### 8. Contenção de PHI

A transformação de perfis roda dentro da fronteira de autorização, após
verificação de token e avaliação de escopo. Uma transformação NÃO DEVE
ampliar o conjunto de campos a que um token tem direito de leitura. Se um
perfil declarado pelo consumidor exigir campos fora do escopo concedido do
token, o servidor retorna `403 Forbidden` em vez de projetar; a requisição
de perfil não atua como um canal paralelo de elevação de escopo.

### 9. Governança do Registry

Adicionar um perfil ao Registry exige um artefato revisável: ou uma adenda à
ADR ou um PR de adição de perfil seguindo um template que captura
proveniência, vínculo de validador e vínculo de transformação. Revisão de
mantenedor é necessária. A *remoção* de perfil é em dois estágios:
`active` → `deprecated` (nenhuma nova escrita aceita; leituras existentes
continuam funcionando) → `superseded` (sem leituras; HTTP 410 retornado,
apontando para o perfil sucessor). Sem adições silenciosas, sem remoções
silenciosas.

## Consequências

### Positivas

- **Marketplace de consumidores torna-se possível.** Qualquer consumidor
  pode declarar o seu perfil, registrá-lo, e ler/escrever recursos deEHR
  naquela forma sem código de conector dedicado.
- **Bundles preservam a atomicidade clínica.** UC-2 (hospital escreve uma
  consulta como Bundle) é o fluxo de persistência de primeira classe, não
  uma sequência de escritas amarradas.
- **Generaliza o padrão de dois níveis da ADR-0005.** Conectores BR-Core /
  RNDS-Principal encaixam limpamente como entradas do Profile Registry; o
  padrão permanece internamente consistente.
- **Flexibilidade schema-on-read.** O mesmo store canônico serve um número
  arbitrário de formas declaradas pelo consumidor.
- **Superfície HL7-canônica.** `Accept-Profile` / `Content-Profile` e
  `$validate?profile=` são padrões FHIR R4; o deEHR adota em vez de
  inventar.

### Negativas / riscos

- **Performance de projeção.** Transformação schema-on-read pode ser cara
  sob carga. A invalidação de cache é direta, mas leituras de cold-cache e
  escritas que tocam muitas projeções cacheadas serão o contribuinte
  dominante de latência. A Phase 1 precisa incluir benchmarks de latência
  contra tamanhos realistas de Bundle.
- **Carga de governança do registry.** Cada novo perfil de consumidor é
  trabalho real: vínculo de validador, regras de transformação, testes de
  regressão. O passo de governança é feature, não bug, mas é overhead
  operacional.
- **Âncora de Bundle no caminho crítico.** O modelo de âncora de raiz
  Merkle acopla o throughput de Bundle ao throughput de chain. A Phase 1
  precisa medir a latência de commit de âncora sob taxas esperadas de
  escrita de Bundle e definir uma política de retry com backoff limitado.
- **Investimento em engenharia de dados.** Profile Registry +
  Projection Engine + Validator + tooling são um sub-sistema substancial. A
  observação do CTO de "trabalho para um bom engenheiro de dados" é
  precisa; a Phase 1 precisa escopar isso explicitamente.
- **Adenda à ADR-0002 necessária.** A estrutura de âncora de raiz Merkle
  precisa ser declarada na ADR-0002. A §6 da ADR-0002 hoje especifica
  "hashes de integridade de Bundles FHIR criptografados" sem fixar a
  estrutura do hash; a adenda refina isso para uma raiz Merkle sobre
  hashes canônicos por-recurso mais metadados do Bundle. Não-quebrante mas
  não-trivial.
- **Mapeamento de perfis entre jurisdições adiado.** Perfis que divergem em
  conteúdo semântico (por exemplo, codificações divergentes de raça/cor)
  estão fora do escopo desta ADR — sinalizados no doc de requisitos e
  rastreados separadamente.

## Alternativas consideradas

- **Conectores hardcoded por consumidor (estender a ADR-0005
  indefinidamente).** Rejeitado — não escala além do caso de reguladores
  para o qual a ADR-0005 foi desenhada. Cada novo consumidor vira mudança
  de código + release.
- **"Um perfil para reger todos"** — forçar cada consumidor a adotar
  deEHR-canônico. Rejeitado — comercialmente inviável; consumidores não
  vão remodelar os seus modelos internos de dados em torno do canônico de um
  fornecedor.
- **Projeções pré-materializadas por `(recurso × perfil)`.** Rejeitado
  nesta fase — a amplificação de escrita escala com o número de perfis
  registrados; o custo de storage cresce não-linearmente com a adoção de
  consumidores. Reconsiderável como otimização opt-in por-perfil para
  caminhos quentes de leitura.
- **Projeção do lado do cliente (consumidores projetam a partir do
  canônico).** Rejeitado — empurra o ônus de validação e projeção para os
  consumidores, o que um marketplace de consumidores (especialmente os
  menores) não consegue suportar; enfraquece as garantias de contenção de
  PHI porque o servidor não consegue mais impor a fronteira do perfil.
- **Âncora única sobre o Bundle serializado (sem raiz Merkle).** Mais
  barata, mas bloqueia a prova de inclusão por-recurso. Rejeitada por não
  ganhar nada significativo em custo enquanto perde uma primitiva futura de
  auditoria / portabilidade.
- **Bundles FHIR `batch` no lugar de Bundles `transaction`.** Rejeitado —
  semântica de `batch` é não-atômica por especificação FHIR; o requisito de
  atomicidade clínica (UC-2) é satisfeito apenas por `transaction`.

## Questões em aberto

Devem ser resolvidas antes desta ADR mover de `Proposto` para `Aceito`:

1. **Linguagem de transformação na Phase 1.** Documentos de mapeamento
   baseados em FHIRPath para v1, ou comprometer com FHIR Mapping Language
   (StructureMap) desde o dia um? StructureMap é o padrão mas a maturidade
   de tooling em Rust/Go é a variável em aberto.
2. **Backend de cache.** Detalhe de nível de implementação, mas no mínimo:
   LRU in-process para v1, com caminho de upgrade explícito para um cache
   compartilhado (Redis ou equivalente) quando escala horizontal chegar.
   Decidir na Phase 1.
3. **Adenda de âncora da ADR-0002.** Forma concreta do payload de raiz
   Merkle (raiz + hash de metadados de bundle + byte de versão de
   protocolo) e assinatura do método de contrato `anchor` on-chain. De
   responsabilidade de uma adenda à ADR-0002, não desta ADR; esta ADR
   depende disso.
4. **Detalhe do anúncio de capability.** Forma JSON exata da extensão deEHR
   nas entradas `CapabilityStatement.rest.resource.profile` — flags
   `read-supported` / `write-supported`, URL do validador, notas de
   deprecação. Fechar durante a implementação de Phase 1.
5. **Teto de governança para adição de perfil.** É necessária uma adenda à
   ADR para cada perfil, ou um template leve de PR de adição de perfil pode
   ficar por si só quando a forma do registry estiver estável? Decidir
   depois que o conjunto v1 de Phase 1 entrar.
6. **Janela de compatibilidade retroativa para perfis `deprecated`.**
   Quanto tempo entre `active → deprecated` e `deprecated → superseded`?
   Default mínimo de 6 meses a menos que razão de segurança force antes;
   fechar durante a Phase 1.
7. **Escopo de IPS em v1.** Adotar o catálogo completo de recursos HL7 IPS
   como perfil de consumidor registrado desde o dia um, ou apenas Patient +
   Condition + MedicationStatement + Observation como fatia inicial menor?
   Tendência ao menor, mas confirmar com o caso de exportação ao paciente
   (UC-4).
8. **Política de retry da âncora de Bundle.** Cronograma de backoff
   limitado para o commit da âncora on-chain + o protocolo compensatório de
   delete off-chain em falha terminal. Política operacional, não de nível
   de ADR; documentar no runbook do Signing & Fee Service durante a Phase 1.

## Referências

- [Requisitos: Heterogeneidade de Perfis de Consumidores](../requirements/consumer-profile-heterogeneity.pt-BR.md)
  — o conjunto de requisitos que esta ADR endereça.
- [ADR-0005](adr-0005-fhir-profile-selection.pt-BR.md) — seleção de perfis
  FHIR; esta ADR generaliza o seu padrão de dois níveis.
- [ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md) — design dos
  registros on-chain; a forma de âncora de raiz Merkle (§4) exige uma adenda
  à ADR-0002.
- [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md) — identidade e
  gestão de chaves; a postura de contenção de PHI (§8) herda da fronteira
  de autorização da ADR-0001.
- HL7 **FHIR R4 §3.2.0.4 Profile negotiation** —
  <https://hl7.org/fhir/R4/profiling.html#profile-negotiation>.
- HL7 **FHIR R4 Bundle** — <https://hl7.org/fhir/R4/bundle.html>.
- HL7 **FHIR R4 operação `$validate`** —
  <https://hl7.org/fhir/R4/resource-operation-validate.html>.
- HL7 **FHIR R4 StructureMap (Mapping Language)** —
  <https://hl7.org/fhir/R4/structuremap.html>.
- **Simplifier — biblioteca de perfis RNDS** —
  <https://simplifier.net/redenacionaldedadosemsaude/~resources?category=Profile>.
- HL7 **International Patient Summary (IPS)** —
  <https://hl7.org/fhir/uv/ips/>.
- HL7 **BR-Core IG** — <https://hl7.org.br/fhir/core/>.
