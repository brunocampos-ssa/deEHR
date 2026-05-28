# ADR-0002: Design dos Registros On-chain

🌐 **Languages / Idiomas:** [English](adr-0002-on-chain-registry-design.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0002-on-chain-registry-design.md](adr-0002-on-chain-registry-design.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Aceito
- **Data:** 2026-05-22
- **Decisores:** mantenedores da deEHR

## Contexto

A camada on-chain da deEHR armazena **apenas provas — jamais PHI**. Ela é composta por
quatro registros lógicos: Identity / DID, Credential, Consent e Anchor & Audit.
Os contratos são escritos em Rust e compilados para WebAssembly, executando na
KVM da Klever.

Este ADR decide como essa camada on-chain é estruturada: a decomposição dos
contratos, o modelo de dados, o modelo de eventos, o controle de acesso, a
capacidade de upgrade e a postura de taxas por registro. O **Consent Registry
é a fonte da verdade para autorização** — o servidor de autorização SMART o
consulta antes de emitir qualquer token — então seu design carrega
especialmente peso.

O ADR foi originalmente marcado como `Proposto` porque várias decisões
dependiam de comportamentos da KVM da Klever que precisavam ser verificados.
Pesquisa com fontes citadas em 2026-05-27 contra `klever-sc 0.45.1`,
`klever-go`, a documentação oficial e observação de transações na mainnet
resolveu as questões Q1-Q5; os itens em aberto remanescentes estão registrados
abaixo.

## Decisão

1. **Decomposição em contratos — quatro contratos separados, um por registro**,
   em vez de um único monolito. Cada registro é independentemente auditável e
   passível de upgrade, tem um conjunto de permissões de escrita mínimo e
   distinto, e um raio de impacto menor caso seja comprometido. Como o servidor
   de autorização SMART pode ler tanto Consent quanto Credential via
   **off-chain VM queries** gratuitas e sem gas (resolução de Q2), a separação
   não impõe nenhuma penalidade de custo entre contratos no caminho quente de
   autorização.

2. **Invariante de não-PHI.** Os dados on-chain ficam limitados a: hashes de
   integridade SHA-256, DIDs, CIDs de IPFS, enumerações de status,
   identificadores **codificados** de escopo e de propósito de uso (nunca texto
   livre), timestamps e validades. Garantido por code review e auditoria.

3. **Identity / DID Registry.** DID Documents (ou ponteiros para eles),
   histórico de rotação de chaves e conjuntos de signatários de
   guardião / recuperação. Veja
   [ADR-0004](adr-0004-did-klever-method.pt-BR.md) para o método `did:klever`.

4. **Credential Registry.** **Status** de emissão e revogação de Verifiable
   Credentials — apenas hashes, nunca o corpo da credencial. Apenas DIDs de
   emissores credenciados podem escrever.

5. **Consent Registry.** Registros de consentimento assinados pelo paciente:
   DID do paciente, DID do destinatário, conjunto codificado de escopos, uma
   referência de filtro de recursos, propósito de uso, validade e status. Toda
   concessão e toda revogação emite um evento. Esse registro é a fonte da
   verdade para autorização.

6. **Anchor & Audit Registry.** Hashes de integridade dos bundles FHIR
   criptografados pareados com seus CIDs de IPFS, além de um log append-only de
   eventos de acesso a dados.

7. **Modelo de eventos.** Toda chamada que altera estado emite um evento
   estruturado para o indexador off-chain e para o pipeline de auditoria.
   Regras concretas:
   - Os eventos são declarados com a macro `#[event("name")]` provida pelo
     `klever-sc`. Cada argumento `#[indexed]` torna-se um topic; **no máximo
     um** argumento não-indexado torna-se o payload de dados, top-encoded pela
     ABI em um único `ManagedBuffer`. A macro rejeita múltiplos argumentos
     não-indexados.
   - O Topic 0 é o nome do evento (bytes literais). Os Topics 1..N são os
     argumentos `#[indexed]` na ordem de declaração. **Endereços nos topics
     são buffers crus de 32 bytes, não bech32.** Para casar um endereço
     `klv1…` contra `topics[1..]`, o indexador off-chain deve fazer
     **bech32-decode** do endereço para sua carga crua de 32 bytes (validando
     o checksum e convertendo dos grupos bech32 de 5 bits de volta para bytes
     de 8 bits), e **então hex-encode** esses bytes crus para comparação.
     Remover o prefixo `klv1` como operação de string **não** produz os bytes
     crus. O endereço do contrato emissor — exposto separadamente como
     `Logs.Address` no proxy — permanece bech32.
   - Para manter os eventos previsíveis sob limites da VM ainda não
     documentados (veja *Questões em aberto* — confirmações do time da
     Klever), a deEHR limita cada evento a **≤ 4 topics** e usa um único
     payload de struct para qualquer dado composto.
   - Consumo: o endpoint do proxy / nó
     `GET /v1.0/transaction/{hash}` retorna o array `logs.events` por
     transação. Uma assinatura websocket por endereço está disponível em um
     nó indexador local. **Hoje não existe API pública de filtro em nível de
     evento**; o indexador faz fan-out a partir de assinaturas por endereço e
     puxa `transaction/{hash}`. O pipeline de auditoria, portanto, trata os
     eventos como best-effort e usa o estado do contrato mais uma
     reconciliação completa periódica como fonte da verdade — não apenas o
     stream de eventos.

8. **Controle de acesso.** As escritas são autorizadas pelo DID e pelo papel
   do ator que está chamando. Escritas no escopo do paciente são submetidas
   pelo Signing & Fee Service em nome do paciente (veja
   [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md)); escritas
   institucionais usam DIDs de instituições credenciadas.

9. **Capacidade de upgrade.** Construída sobre a primitiva nativa de upgrade
   da KVM Klever (verificada em Q1):
   - Cada contrato é deployado com `CodeMetadata::UPGRADEABLE` **definido
     explicitamente**. O atributo `#[upgrade]` do framework define um
     entrypoint separado, distinto de `#[init]`, despachado em transações de
     upgrade.
   - O storage persiste através do upgrade (mesmo endereço de conta, mesma
     trie). O framework **não fornece ferramental nativo de migração**:
     qualquer mudança de layout de storage deve ser implementada como lógica
     explícita dentro da função `#[upgrade]`, gateada por um mapper
     `storage_version` e acompanhada de um plano explícito de migração de
     dados revisado durante o PR de upgrade.
   - A autorização no nível da VM é **owner-only**. Upgrade controlado por
     multisig é implementado definindo o owner do contrato como um contrato
     de multisig de governança via o builtin `ChangeOwnerAddress`; a VM em
     si não conhece multisig.
   - **Estabilização e lock-in do Consent Registry.** Quando o Consent
     Registry alcançar seu schema estabilizado, submeteremos um upgrade
     final que limpa o bit `UPGRADEABLE` (`CodeMetadata::DEFAULT`),
     travando o contrato permanentemente. Isso troca a capacidade de
     upgrade por garantias mais fortes sobre a fonte da verdade de
     autorização.

10. **Postura de taxas e tesouraria.** Os custos por registro são
    empiricamente tratáveis (verificado em Q4):
    - **KAppFee** é fixo em 2 KLV por `SmartContractInvoke`; **BandwidthFee**
      escala com o tamanho do envelope da transação e a execução
      (~2 KLV base + ~0.008 KLV/byte conforme a documentação;
      empiricamente 6–31 KLV no total para chamadas típicas de SC).
    - Storage segue o modelo **pay-once-on-write com reembolso no delete**.
      Todos os contratos da deEHR DEVEM limpar (deletar) registros revogados
      ou expirados para que o reembolso de storage seja reivindicado —
      material em escala de 10K pacientes.
    - Todas as transações do lado do paciente são pagas pela tesouraria do
      Signing & Fee Service, conforme ADR-0001. Opcionalmente, um KDA fee
      pool pode absorver toda a taxa em um token emitido pela deEHR, dando
      uma alavanca operacional contra a volatilidade do preço do KLV.
    - **Orçamento de tesouraria**: previsão de **50 KLV por operação** como
      multiplicador de segurança sobre a faixa observada de ~12–27 KLV. Com
      10.000 pacientes × ~17 operações/ano (5 operações de consentimento +
      10 emissões de evento de auditoria + 2 atualizações de DID), a
      previsão é de ~8,5M KLV/ano — confortavelmente absorvível.
    - A tabela de taxas é **mutável por governança via detentores de KFI**;
      a previsão operacional deve ser recalibrada anualmente contra a
      tabela então vigente.

## Consequências

### Aspectos positivos

- Separação clara de responsabilidades; permissões de escrita por registro
  no princípio do menor privilégio.
- Auditoria e upgrade independentes por contrato; superfície de ataque por
  contrato menor; o Consent Registry pode ser travado permanentemente
  quando estiver estável.
- O invariante de não-PHI é estruturalmente simples de revisar.
- Um modelo de eventos consistente, estilo EVM, deixa o indexador off-chain
  e a trilha de auditoria diretos e ancorados em uma primitiva verificada do
  framework.
- O caminho quente do servidor de autorização é gratuito: off-chain VM
  queries não custam gas e não são necessárias chamadas entre contratos
  on-chain.
- Os custos por registro são empiricamente acessíveis e absorvíveis pela
  tesouraria da plataforma; KDA fee pools oferecem uma futura migração para
  um token emitido pela deEHR.

### Aspectos negativos / riscos

- **Confiança e maturidade do indexador.** Decisões de autorização
  off-chain dependem de o estado da chain ser lido corretamente e a tempo.
  Hoje não existe API pública de filtro em nível de evento, então o
  indexador deve fazer fan-out a partir de assinaturas websocket por
  endereço e puxar os recibos por transação. A confiabilidade do indexador
  faz parte do modelo de segurança.
- **Risco de governança da tabela de taxas.** As taxas por byte e por
  operação da Klever são mutáveis pela governança dos detentores de KFI.
  Uma mudança hostil poderia multiplicar a queima da tesouraria por 5×-10×
  da noite para o dia; o plano operacional deve incluir uma recalibração
  anual e uma alavanca de contingência (KDA fee pool, postura custodial
  alternativa).
- **Semântica de falha entre contratos.** Chamadas síncronas entre
  contratos on-chain (caso sejam introduzidas no futuro) cascateiam falhas
  e não têm try/catch. Nossa arquitetura as evita no caminho quente; se uma
  funcionalidade futura precisar delas, o design deve tratar qualquer
  callee que dê panic como um revert completo do caller.
- **Migração manual de upgrade.** O framework não fornece ferramental de
  migração; toda mudança de schema é código sob medida em `#[upgrade]`.
  Erros são irrecuperáveis (no nível da VM, o storage é key/value opaco).
- **Dependência de tesouraria.** Igual à postura original — a plataforma
  precisa manter a tesouraria de taxas / KDA pool financiados; rate limits
  e cotas permanecem necessários.
- Quatro contratos significam mais coordenação de deploy e de upgrade do
  que um monolito, mitigado pela capacidade de upgrade independente e por
  escopos de auditoria mais claros.

## Alternativas consideradas

- **Um único contrato de registro monolítico.** Rejeitada — raio de impacto
  maior, permissões de escrita mais amplas e mais difícil de auditar e fazer
  upgrade independentemente.
- **Base de dados de consentimento off-chain com ancoragem periódica de
  hash.** Rejeitada — o consentimento em si precisa ser a fonte da verdade
  verificável on-chain, não um hash de um registro off-chain que poderia
  divergir.
- **Agrupar Identity + Credential em um só contrato.** Considerada como
  fallback caso chamadas entre contratos se mostrassem custosas. Rejeitada
  nesta revisão porque as leituras off-chain são gratuitas (Q2), de modo
  que o layout de quatro contratos não tem penalidade de custo.

## Questões resolvidas

Os itens a seguir eram rastreados como bloqueadores de `Proposto` quando o ADR
foi inicialmente redigido e foram resolvidos (2026-05-27) via pesquisa com
fontes citadas:

- **Q1 — Capacidade de upgrade dos contratos.** Atributo `#[upgrade]` de
  primeira classe, distinto de `#[init]`; owner-gated no nível da VM;
  `CodeMetadata::UPGRADEABLE` controla elegibilidade; storage persiste
  através do upgrade; migração é código manual dentro de `#[upgrade]`.
  Imutabilidade é alcançável limpando o bit (em tempo de deploy ou via um
  upgrade final). Veja §9.
- **Q2 — Chamadas entre contratos.** Apenas síncronas no SDK público
  `klever-sc 0.45.1`; falhas cascateiam; sem try/catch; chamadas
  read-only ainda custam gas. Crucialmente, **off-chain VM queries são
  gratuitas e sem gas** via a REST API do proxy / nó, então o caminho de
  autorização SMART não precisa de nenhuma chamada entre contratos on-chain.
  Veja §1.
- **Q3 — Modelo de eventos / logs.** Topics estilo EVM + buffer único de
  dados via a macro `#[event(...)]`; o topic 0 é o nome do evento; os
  args `#[indexed]` são topics em bytes crus; um payload de dados.
  Endereços em topics são crus de 32 bytes. Veja §7.
- **Q4 — Modelo de custos de storage / taxas.** Storage pay-once-on-write
  com reembolso no delete. KAppFee fixo em 2 KLV por SC invoke;
  BandwidthFee ~base + ~0.008 KLV/byte (empiricamente 6–31 KLV no total
  por chamada típica). Tesouraria confortavelmente absorvível. Veja §10.
- **Q5 — Quatro contratos vs agrupamento.** Manter quatro. A preocupação
  original (custo entre contratos) é discutível — as leituras acontecem
  off-chain gratuitamente (Q2). Os benefícios de upgrade e auditoria
  independentes se mantêm. Veja §1.

## Questões em aberto

Estes itens permanecem a serem resolvidos antes de qualquer revisão
subsequente do ADR-0002 (ou como entradas para outros ADRs). O ADR-0002 será
**emendado via uma seção Adenda** caso alguma resposta contradiga
materialmente as suposições acima, consistente com a política append-only de
ADRs do repositório.

- ~~**Q6 — Conjuntos de valores codificados** para escopo e propósito de
  uso~~ — **Resolvido** em 2026-05-28 pela
  [ADR-0005](adr-0005-fhir-profile-selection.pt-BR.md). Veja a entrada
  *Adenda* abaixo para o resumo vinculante.

- **Confirmações necessárias do time Klever.** A pesquisa de 2026-05-27
  deixou vários detalhes comportamentais que o código-fonte do SDK e a
  documentação pública não respondem autoritativamente. Eles devem ser
  confirmados pelo time de desenvolvedores da Klever antes do deploy em
  mainnet:
  1. **Encoding da tx de upgrade** — upgrade é um `EnumContractType`
     distinto no protobuf da transação, ou uma chamada de SC normal com
     nome de função `upgradeContract`?
  2. **Mecânica do reembolso de storage** — razão exata, o que dispara o
     reembolso (set-to-zero? clear explícito?) e o timing do crédito (na
     mesma transação vs liquidado depois).
  3. **Precificação de gas entre contratos** — custo de dispatch por
     opcode, profundidade máxima de chamada, se
     `execute_on_dest_context_readonly_raw` é imposto pela VM (tentativas
     de write em storage dão trap) ou apenas por convenção.
  4. **Reentrância** — a VM impõe um lock embutido, ou é puramente
     responsabilidade do autor do contrato?
  5. **Limites rígidos de evento** — máximo de topics por evento, máximo de
     bytes por topic, máximo de bytes de dados, máximo de eventos por
     transação; confirmar a semântica de rollback em falha para logs
     (assumido: descartados no revert da transação).
  6. **API pública de indexador de eventos** — endpoint de evento por
     block-range-por-contrato? Assinatura websocket em nível de evento?
     Roadmap?
  7. **Transparência da gas schedule** — gas schedule de storage / por
     opcode publicada em YAML/JSON análoga ao `gasScheduleV1.yaml` do
     Arwen?
  8. **KDA fee pool** — cap por transação, comportamento quando o saldo do
     pool é insuficiente, latência de liquidação do swap.
  9. **Roadmap de assíncrono / promises** — o sync-only é o design de
     longo prazo, ou opcodes assíncronos / `#[callback]` estão planejados
     para um release futuro do SDK?
  10. **Padrão de migração recomendado** — endosso oficial de um mapper
      `storage_version` ou de uma macro auxiliar de migração?

## Referências

- [README.md](../../README.md) — seção *What Lives on Klever*.
- [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md) — Identidade e
  Gestão de Chaves (o Signing & Fee Service e a tesouraria).
- [ADR-0004](adr-0004-did-klever-method.pt-BR.md) — método DID `did:klever`.
- Verificação de capacidades da KVM Klever (2026-05-22) — funções
  criptográficas host verificadas, semântica de permissões de conta,
  protobuf de transação.
- Pesquisa de comportamento da KVM Klever (2026-05-27) — código-fonte do
  `klever-sc 0.45.1` (`~/.cargo/registry/...`), fluxo de execução do
  VM-host `klever-go`, observação empírica em mainnet via
  `api.mainnet.klever.org/v1.0/transaction/list?type=63`.
- Documentação oficial da Klever — <https://docs.klever.org/>
  (`smart-contracts/reference/annotations`, `smart-contracts/reference/calls`,
  `smart-contracts/reference/payments`, `klever-vm`, `about-our-technology`,
  `api-and-sdk`).
- `klever-io/klever-vm-sdk-rs` — <https://github.com/klever-io/klever-vm-sdk-rs>.
- `klever-io/klever-go` — <https://github.com/klever-io/klever-go>.

## Adenda

### 2026-05-27 — Questões em aberto Q1-Q5 resolvidas; promovido a Aceito

O ADR-0002 originalmente entrou como **Proposto** pendente de seis questões
em aberto sobre o comportamento da KVM Klever. Pesquisa com fontes citadas em
2026-05-27 contra `klever-sc 0.45.1`, `klever-go`, a documentação oficial e
observação de transações na mainnet resolveu Q1-Q5; as resoluções estão
refletidas em linha nos §§1, 7, 9, 10 da *Decisão* e resumidas em *Questões
resolvidas*. A **Q6** (conjuntos de valores codificados) permanece em aberto
e é rastreada em [#6](https://github.com/brunocampos-ssa/deEHR/issues/6). A
lista de 10 itens de confirmação do time de desenvolvedores da Klever está
registrada em *Questões em aberto*. Com as questões em aberto substantivas
fechadas, o ADR é promovido de **Proposto** para **Aceito** por esta entrada.
Emendas futuras serão registradas como entradas adicionais nesta seção,
conforme a política append-only de ADRs do repositório.

### 2026-05-28 — Q6 resolvida (conjuntos de valores codificados)

A [ADR-0005](adr-0005-fhir-profile-selection.pt-BR.md) — Seleção de Perfis
FHIR — foi publicada como **Aceita**, resolvendo a questão em aberto
remanescente Q6 sobre conjuntos de valores codificados para escopos SMART e
propósito de uso:

- **Escopos SMART** — a deEHR adota a sintaxe SMART App Launch 2.0 v2 com um
  vocabulário de escopos MVP `v1` (20 escopos) publicado como um manifesto
  assinado em `https://deehr.org/fhir/scopes/v1.json`. O Consent Registry
  codifica cada escopo concedido como uma tupla de forma fixa
  `(deehr_scopes_version: u16, template_code: u16, param_hash: bytes16)` —
  20 bytes on-chain, reversível via o manifesto. Veja ADR-0005 §§5-6.
- **Propósito de uso** — a deEHR vincula a
  `http://terminology.hl7.org/CodeSystem/v3-ActReason` (HL7 v3 ActReason),
  com o valor on-chain extraído da accept-list `v1` da deEHR (12 códigos do
  value set [`v3-PurposeOfUse`](https://hl7.org/fhir/R4/valueset-v3-PurposeOfUse.html):
  `TREAT`, `ETREAT`, `ERTREAT`, `HRESCH`, `CLINTRCH`, `HPAYMT`, `COVERAGE`,
  `HOPERAT`, `PUBHLTH`, `PATRQT`, `COC`, `HLEGAL`). Veja ADR-0005 §7.

Com a Q6 resolvida, o §2 da *Decisão* do ADR-0002 (invariante de não-PHI —
"identificadores codificados de escopo e propósito de uso, nunca texto livre")
agora tem URIs concretas de code system HL7-canônicas para referenciar. A
lista de 10 itens de confirmação do time de desenvolvedores da Klever em
*Questões em aberto* permanece como o único item pendente do ADR.
