# ADR-0007: Resolução de Identidade do Paciente & Master Patient Index — Persistência Match-First e o Golden Record

🌐 **Languages / Idiomas:** [English](adr-0007-patient-identity-resolution.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0007-patient-identity-resolution.md](adr-0007-patient-identity-resolution.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Proposto
- **Data:** 2026-06-12
- **Decisores:** mantenedores do deEHR

## Contexto

A [ADR-0006](adr-0006-multi-consumer-profile-strategy.pt-BR.md) permite que
qualquer consumidor leia e grave dados do deEHR moldados ao seu próprio profile
FHIR. Ela resolve a *heterogeneidade de perfis* — muitos formatos legítimos do
mesmo recurso lógico — mas **pressupõe que a identidade do paciente já está
resolvida**. Uma
[revisão de acompanhamento](../requirements/patient-identity-resolution.pt-BR.md)
com o mesmo CTO do mercado de seguros brasileiro que originou a ADR-0006
trouxe à tona o problema que está *a montante* da projeção de perfil: decidir
que dois registros recebidos **são a mesma pessoa**.

Forças que moldam a decisão:

- **Identidade do requisitante ≠ identidade do sujeito.** A camada SMART on
  FHIR (ADR-0005) autentica o *requisitante* (um hospital, um laboratório, uma
  seguradora, o paciente) e carrega os *escopos* a que ele tem direito. Ela
  **não** nomeia o *sujeito* do dado. Um hospital com uma concessão de
  consentimento válida persiste o prontuário de *outra pessoa*. A chave canônica
  do paciente **não está no token SMART**.
- **Sem chave primária compartilhada entre fontes.** As fontes gravam de forma
  assíncrona, cada uma sob seu próprio profile (ADR-0006), e nenhuma carrega a
  chave de paciente do deEHR. No contexto brasileiro, **CPF** e **CNS** (Cartão
  Nacional de Saúde) são identificadores fortes-porém-imperfeitos —
  frequentemente ausentes, às vezes sujos, não garantidamente únicos nos dados
  de origem. São evidência, não uma chave garantida.
- **Gravações FHIR atômicas assumem criar por padrão.** Toda chamada REST do
  FHIR é atômica; um caminho de gravação ingênuo faz `POST` de um novo
  `Patient` a cada vez, produzindo registros duplicados de uma pessoa real — um
  histórico clínico fragmentado e um risco à segurança do paciente.
- **A resolução é probabilística.** Como as fontes discordam sobre campos
  obrigatórios e codificações de value-set, "mesma pessoa?" precisa ser
  pontuado, não casado por chave exata. Este é o clássico problema de **Master
  Patient Index (MPI)** / record linkage.
- **Sem PHI on-chain (ADR-0002 §2).** A camada on-chain armazena apenas DIDs,
  hashes, CIDs e status codificado. Qualquer componente que compare dados
  demográficos precisa viver inteiramente off-chain.
- **O consentimento é chaveado pelo DID do paciente (ADR-0002 §5).** O Registro
  de Consentimento — a fonte da verdade de autorização — é chaveado pelo DID
  `did:klever` do paciente (ADR-0001). Para que uma gravação de provedor seja
  governada pela concessão de consentimento certa, os dados demográficos
  recebidos precisam resolver para o DID de paciente certo. O MPI é a única
  coisa capaz de fazer a ponte *dados demográficos → paciente canônico → DID*.
- **HL7-canônico primeiro.** O FHIR já padroniza a interface de resolução via a
  operação `Patient/$match` e a extensão `match-grade`. O deEHR adota o padrão
  em vez de inventar um mecanismo paralelo — consistente com ADR-0005 /
  ADR-0006.
- **Carga de engenharia de dados.** Como o engine de projeção (ADR-0006), o MPI
  é um subsistema load-bearing de engenharia de dados. A Phase 1 precisa de um
  sub-arco para ele.

## Decisão

### 1. Master Patient Index como componente off-chain de primeira classe

O deEHR opera um **MPI** como componente off-chain dedicado à frente do store
FHIR canônico. Ele mantém, por pessoa real, um **Golden Record** chaveado por um
**identificador mestre de paciente** emitido pelo deEHR, e preserva o
**identificador local** de cada fonte contribuinte como referência cruzada. O
MPI vincula registros de origem a uma identidade mestre; ele não descarta
identificadores de origem. O MPI guarda PHI (dados demográficos) e, portanto,
vive totalmente off-chain (ADR-0002 §2); ele nunca grava dado demográfico na
chain.

### 2. Correspondência híbrida: passada determinística, depois probabilística

A resolução roda um pipeline de duas etapas conforme o formato padrão de MPI
(normalizar → blocking → pontuar → link/merge):

1. **Normalização.** Os dados demográficos recebidos são padronizados (caixa de
   nome, acentos, formatos de data, formatação de identificadores) antes da
   comparação.
2. **Blocking.** Golden Records candidatos são pré-selecionados por critérios
   fracos (p.ex., soundex do sobrenome + ano de nascimento) para limitar o
   custo de comparação.
3. **Passada determinística.** Regras de identificador forte (p.ex., match de
   CNS, ou CPF + data de nascimento) produzem matches de alta confiança
   diretamente.
4. **Passada probabilística.** Para todo o resto, um classificador de evidência
   ponderada **Fellegi-Sunter** pontua todos os campos disponíveis — campos de
   alta unicidade (CNS, CPF, data de nascimento) carregam mais peso — contra
   **limiares configuráveis**, produzindo um de três resultados:
   - **auto-match** (acima do limiar superior),
   - **possível-match** (entre os limiares → revisão humana, §6),
   - **não-match** (abaixo do limiar inferior).

Limiares, pesos de campo e a versão do algoritmo são configuração, não código,
e toda decisão registra a versão utilizada (§7). O pipeline é calibrado para
tornar um **falso-merge** (vincular duas pessoas diferentes) o erro mais
custoso, porque um falso-merge contamina cruzadamente históricos clínicos.

A escolha entre *construir* o Fellegi-Sunter internamente versus *integrar* um
EMPI existente é uma questão em aberto (§ Questões em aberto Q1); esta ADR fixa
a *interface e a semântica*, não a implementação de correspondência — espelhando
como o próprio FHIR `$match` "deliberadamente evita prescrever algoritmos
específicos".

### 3. `Patient/$match` é a interface padrão de resolução

O deEHR expõe `POST /fhir/Patient/$match` conforme a operação FHIR R4:

- **Entrada:** `resource` (um `Patient` possivelmente parcial),
  `onlySingleMatch`, `onlyCertainMatches`, `count`.
- **Saída:** um `Bundle` searchset de recursos `Patient` candidatos ordenados
  do mais ao menos provável, cada entrada carregando um **search score (0–1)** e
  a extensão **`match-grade`** (`certain` / `probable` / `possible` /
  `certainly-not`).

O `$match` é usado tanto **internamente** pelo caminho de gravação (§4) quanto,
sujeito à questão em aberto Q7, **externamente** por consumidores reconciliando
seus próprios beneficiários com pacientes do deEHR.

### 4. Persistência match-first: criar vs atualizar condicional

Toda gravação de `Patient` resolve a identidade **antes** de persistir:

- **auto-match** → a gravação é vinculada ao Golden Record casado e
  **agregada** a ele (semântica de update condicional / `PUT`). Recursos
  clínicos na mesma gravação se acumulam no mesmo prontuário.
- **não-match** → um **novo** Golden Record é criado (semântica `POST`) com um
  identificador mestre recém-emitido.
- **possível-match** → encaminhado à fila de data steward (§6). A gravação não
  duplica silenciosamente e não faz merge silencioso.

Uma **gravação direta do paciente** (um paciente gravando pelo seu próprio app
deEHR, já vinculado a um DID — ADR-0001) **curto-circuita** a correspondência
probabilística: a identidade do sujeito é conhecida de forma autoritativa.

### 5. Link por padrão; merge e unmerge são explícitos e reversíveis

O MPI **vincula** (link) registros de origem a uma identidade mestre por
padrão, preservando cada registro de origem e seu identificador original
(reversível). **Merge** (consolidar dois Golden Records) e **unmerge** (separar
um match errado) são **operações explícitas e reversíveis** com auditoria
completa (§7). O deEHR nunca destrói registros de origem no merge; o merge
consolida a *visão mestre* enquanto os registros de origem subjacentes
permanecem individualmente endereçáveis.

### 6. Fluxo de data steward para possíveis-matches

Possíveis-matches são encaminhados a uma **fila de data steward** com uma
superfície de revisão que apresenta a evidência de comparação — quais campos
casaram, quais conflitaram, a pontuação — e os Golden Records candidatos. A
decisão do steward (aprovar match / rejeitar / merge / unmerge) é registrada na
trilha de auditoria (§7). Este fluxo é um entregável de primeira classe, não um
adendo operacional.

### 7. Auditabilidade de toda decisão de identidade

Toda decisão de resolução — auto-match, não-match, aprovada por steward,
rejeitada por steward, merge, unmerge — é registrada off-chain com: a referência
dos dados demográficos de entrada, o conjunto de candidatos, a pontuação, a
versão do algoritmo + config, o resultado e o ator decisor (o engine ou o
steward). As decisões são reproduzíveis a partir dos insumos registrados +
versão. O log de auditoria de acesso a dados off-chain compõe com o Registro de
Anchor & Auditoria da ADR-0002 §6: o *fato* de um acesso/anchor permanece
on-chain; a *evidência* demográfica de uma decisão de match permanece off-chain.

### 8. Vínculo Golden Record ↔ DID do paciente

O identificador mestre do MPI mapeia para o DID `did:klever` do paciente
(ADR-0001) quando a pessoa é um paciente deEHR onboarded. A resolução, portanto,
faz a ponte **dados demográficos recebidos → Golden Record (id mestre) → DID do
paciente**, que é o que permite que uma gravação de provedor seja governada pela
concessão de consentimento correta chaveada por DID (ADR-0002 §5). Um Golden
Record originado por provedor **pode existir sem DID** e ser vinculado a um DID
mais tarde no onboarding do paciente **sem perder o histórico acumulado**. O
vínculo cruza a fronteira on/off-chain apenas como uma referência de DID —
nunca como PHI.

### 9. Emenda à ADR-0006 §4 (pipeline de gravação de Bundle)

O pipeline de gravação de Bundle da ADR-0006 §4 é emendado para inserir um
**passo de resolução de identidade** antes da projeção canônica:

1. Validação em nível de Bundle (ADR-0006 §4.1) — inalterado.
2. **Resolução de identidade (novo).** A entrada `Patient` é resolvida via o MPI
   (§2–§4). O identificador mestre resolvido alimenta o reference rewriting.
3. Projeção canônica (ADR-0006 §4.2) — inalterado.
4. Reference rewriting (ADR-0006 §4.3) — agora resolve a referência `Patient`
   interna do Bundle para o **identificador mestre resolvido**, para que os
   recursos clínicos se vinculem ao paciente existente correto.
5. Persistência atômica + anchor on-chain (ADR-0006 §4.4–4.5) — inalterado.

Se a resolução de identidade retornar um possível-match, o Bundle segue a
política de possível-match (questão em aberto Q4): bloquear aguardando revisão
do steward, ou persistir contra um registro provisório. De qualquer forma, a
atomicidade (ADR-0006 §4) é preservada — o Bundle é totalmente comitado ou
totalmente revertido.

## Consequências

### Positivas

- **Uma pessoa, um registro.** O Golden Record evita o risco de segurança do
  paciente de histórico fragmentado que gravações atômicas ingênuas criam.
- **Gravações de provedor são governáveis.** Resolver dados demográficos → DID
  permite que o consentimento chaveado por DID (ADR-0002) controle gravações
  sobre um paciente feitas por um requisitante terceiro — fechando a lacuna
  requisitante-≠-sujeito.
- **Superfície HL7-canônica.** `$match` + `match-grade` são padrões FHIR R4; o
  deEHR adota em vez de inventar, consistente com ADR-0005 / ADR-0006.
- **Compõe com a ADR-0006.** A resolução de identidade encaixa limpa à frente da
  projeção; os dois sub-arcos de engenharia de dados compartilham um caminho de
  gravação coerente.
- **Sem PHI on-chain preservado.** O MPI é totalmente off-chain; o único
  cruzamento de fronteira é uma referência de DID.

### Negativas / riscos

- **Falso-merge é um risco à segurança clínica.** Vincular duas pessoas
  diferentes contamina cruzadamente históricos. Os limiares precisam favorecer
  evitá-lo, e os merges precisam ser reversíveis — mas o risco é intrínseco à
  correspondência probabilística.
- **Resolução no caminho quente de gravação.** O `$match` roda em toda gravação
  clínica, adicionando latência antes da projeção e do anchor. A Phase 1 precisa
  fazer benchmark dele dentro do orçamento compartilhado do caminho de gravação.
- **Operações de steward são custo contínuo.** A fila de possíveis-matches
  precisa de operadores humanos e um SLA; é uma função operacional permanente.
- **Colisão de merge de DIDs é genuinamente difícil.** Fazer merge de dois
  Golden Records que estão *cada um* já vinculados a um DID diferente colide
  duas identidades on-chain com seus próprios consentimentos e anchors (questão
  em aberto Q6). Este é o risco cross-layer mais agudo introduzido.
- **Segundo investimento de engenharia de dados.** O MPI é um subsistema
  substancial ao lado do engine de projeção da ADR-0006. A Phase 1 **constrói**
  o MPI (a resolução de identidade precisa rodar antes de qualquer escrita
  persistir) e **arquiteta** o engine de projeção, cuja implementação chega na
  Phase 2 (ADR-0006 §3); os dois sub-arcos compartilham um único caminho de
  escrita, mas apenas o MPI é construído na Phase 1.

## Alternativas consideradas

- **Correspondência apenas determinística (CPF/CNS como chave).** Rejeitada — os
  identificadores nacionais estão ausentes ou sujos com frequência demais nos
  dados de origem; depender apenas deles produz tanto duplicatas (chave ausente)
  quanto falsos-merges (chave compartilhada/com erro de digitação).
- **Confiar no identificador local de cada fonte como chave do paciente.**
  Rejeitada — MRNs locais são únicos apenas dentro de uma fonte; a mesma pessoa
  tem MRNs diferentes em cada hospital.
- **Deduplicação no lado do cliente (consumidores resolvem a identidade).**
  Rejeitada — empurra a comparação de PHI e a decisão crítica de segurança para
  os consumidores, e um único consumidor não enxerga a população cross-source
  que o MPI enxerga.
- **Apenas merge, sem link (destruir registros de origem na consolidação).**
  Rejeitada — irreversível; um merge errado se torna irrecuperável e a
  proveniência da origem se perde.
- **Colocar a resolução após a persistência (dedup como job em lote).**
  Rejeitada — deixa duplicatas vivas no store clínico entre a gravação e o
  dedup, e ancora a duplicata on-chain (ADR-0002) antes de ela ser resolvida.
- **Inventar uma API de match proprietária do deEHR.** Rejeitada — o FHIR
  `$match` já padroniza o contrato; inventar uma quebra a postura HL7-canônica.

## Questões em aberto

A serem resolvidas antes de esta ADR passar de `Proposto` para `Aceito`:

1. **Construir vs integrar o engine de correspondência.** Fellegi-Sunter
   in-house em Go/Rust (controle total, nenhum PHI sai da fronteira) vs um EMPI
   open-source existente (mais rápido, testado em produção). Decidir na
   prototipagem da Phase 1.
2. **Política de confiança em identificador nacional.** Peso de CPF/CNS na
   passada determinística, e comportamento quando um identificador forte casa
   mas os dados demográficos discordam fortemente (e vice-versa).
3. **Limiares padrão e SLA do steward.** Limiares iniciais de auto-match /
   não-match, a largura da faixa de revisão e o SLA operacional da fila de
   steward.
4. **Política de persistência em possível-match.** Bloquear a gravação
   aguardando revisão do steward, ou persistir em um registro provisório
   reconciliado depois? Trade-off de segurança clínica vs disponibilidade —
   interage com a atomicidade de Bundle da ADR-0006 §4.
5. **Momento da vinculação do DID.** Um Golden Record originado por provedor
   pode existir sem DID e ser reivindicado no onboarding, ou um DID é exigido de
   antemão? Liga-se à custódia progressiva da ADR-0001.
6. **Merge entre DIDs já vinculados.** Consequência on-chain de fazer merge de
   dois Golden Records cada um vinculado a um DID de paciente diferente (cada um
   com seus próprios consentimentos/anchors). Permitido automaticamente, ou
   apenas um procedimento manual de alta garantia? Pode exigir uma emenda à
   ADR-0001 / ADR-0002.
7. **Exposição externa de `$match` na v1.** Expor `$match` a consumidores
   externos desde o dia um, ou mantê-lo interno ao pipeline de gravação na v1?
8. **Conjunto de avaliação de correspondência.** Fonte de um dataset rotulado
   representativo do BR para calibrar e regredir precisão/recall sem usar PHI de
   produção.

## Referências

- [Requisitos: Resolução de Identidade do Paciente](../requirements/patient-identity-resolution.pt-BR.md)
  — os requisitos que esta ADR endereça.
- [ADR-0006](adr-0006-multi-consumer-profile-strategy.pt-BR.md) — estratégia
  FHIR multi-consumidor; esta ADR está a montante dela e emenda o pipeline de
  gravação de Bundle da §4.
- [ADR-0005](adr-0005-fhir-profile-selection.pt-BR.md) — seleção de profile
  FHIR; padrão de autenticação SMART on FHIR.
- [ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md) — design do registro
  on-chain; o invariante de sem-PHI (§2) e o Registro de Consentimento chaveado
  por DID (§5) limitam a colocação off-chain do MPI e o vínculo
  Golden-Record→DID. O Registro de Anchor & Auditoria (§6) compõe com a
  auditoria off-chain de decisões de match (§7).
- [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md) — identidade e
  gestão de chaves; DID `did:klever` do paciente e custódia progressiva.
- HL7 **Operação FHIR R4 `Patient/$match`** —
  <https://hl7.org/fhir/R4/patient-operation-match.html>.
- HL7 **Extensão FHIR `match-grade`** —
  <https://hl7.org/fhir/R4/valueset-match-grade.html>.
- **Health Samurai — Master Patient Index and Record Linkage** —
  <https://www.health-samurai.io/articles/master-patient-index-and-record-linkage>.
- **fastrivertech/fhir-mpi — interface EMPI baseada em FHIR** —
  <https://github.com/fastrivertech/fhir-mpi>.
- Fellegi, I. P., & Sunter, A. B. (1969). *A Theory for Record Linkage.*
  Journal of the American Statistical Association, 64(328), 1183–1210.
