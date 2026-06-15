# ADR-0004: Método DID did:klever — Híbrido Clássico / Pós-Quântico

🌐 **Languages / Idiomas:** [English](adr-0004-did-klever-method.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0004-did-klever-method.md](adr-0004-did-klever-method.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Aceito
- **Data:** 2026-05-26
- **Decisores:** mantenedores da deEHR

## Contexto

A deEHR identifica pacientes e demais atores (instituições, prestadores, o
Signing & Fee Service) por meio de Decentralized Identifiers do W3C. Não
existe um método DID padrão para a rede Klever, então a deEHR precisa
definir um.

Forças que moldam esta decisão:

- **Conformidade com o W3C DID Core.** O método deve estar em conformidade
  com a sintaxe, o modelo de resolução e a semântica de DID Document do
  W3C Decentralized Identifiers 1.0.
- **A realidade verificada da criptografia da KVM Klever.** As host
  functions da KVM verificam apenas `ed25519`, `secp256k1` e BLS. Não há
  verificador on-chain para nenhum esquema pós-quântico do NIST (ML-DSA,
  ML-KEM, SLH-DSA, Falcon). Assinaturas PQ não podem autorizar transações
  Klever hoje.
- **Dados sensíveis de longa duração.** Registros de saúde assinados hoje
  precisam permanecer verificáveis por toda a vida do paciente — décadas.
  **Harvest-now, decrypt-later** ("coletar agora, descriptografar depois")
  é uma ameaça explícita e documentada para dados de saúde. O NIST
  finalizou FIPS 203 / 204 / 205 em 2024; a diretiva CNSA 2.0 da NSA
  espera que o tráfego de national-security-systems seja exclusivamente
  PQ até 2033, e setores regulados — incluindo o de saúde — são
  amplamente esperados para seguir o mesmo caminho. Projetar resistência
  PQ agora é materialmente mais barato do que readaptar depois.
- **Custódia progressiva (ADR-0001).** O mesmo DID deve sobreviver a
  todos os estados de custódia — assistida, híbrida, auto-custodial. A
  rotação de chaves não pode alterar o identificador.
- **Contrato do Identity Registry (ADR-0002).** Um registry on-chain
  dedicado já existe na arquitetura para guardar dados de DID Document,
  conjuntos de recuperação e service endpoints. A resolução de
  `did:klever` se apoia nele.
- **Privacidade em uma chain pública.** Todo estado on-chain é observável
  para sempre. Qualquer coisa armazenada vinculada a um DID é
  correlacionável para sempre.

## Decisão

Adotamos **`did:klever`** com um perfil de verification method
**híbrido clássico / pós-quântico**.

### 1. Sintaxe do DID

```text
did-klever       = "did:klever:" klever-network ":" klever-id
klever-network   = "mainnet" / "testnet" / "devnet"
klever-id        = klever-bech32-address       ; e.g. klv1q5y…
```

Exemplos:

- `did:klever:mainnet:klv1q5yndp8gw3l4...`
- `did:klever:testnet:klv1q5yndp8gw3l4...`

O identificador específico do método **é o endereço bech32 da conta
Klever**. A conta é seu próprio controller. A rotação de chaves é
realizada pela transação nativa `UpdateAccountPermission` da Klever e
nunca altera o DID. Esquemas alternativos de identificador (aleatório
opaco, hash-of-key) foram considerados e rejeitados (veja
*Alternativas*).

Isso satisfaz o ABNF do W3C DID Core: o nome do método é `klever`, e o
identificador específico do método `<network>:<bech32>` usa apenas
caracteres do conjunto `idchar`.

### 1.5. Política de tipo de chave do signatário

A KVM Klever verifica assinaturas `ed25519`, `secp256k1` e BLS
nativamente. Para **contas gerenciadas pela deEHR**, `did:klever` usa
**exclusivamente signatários Ed25519** — para alinhamento com os
padrões da wallet e do SDK Klever, uma forma `Multikey` uniforme por
verification method clássico, um único tipo de chave KMS / HSM em
custódia e uma única primitiva de rotação ao longo de todo o espectro
de custódia progressiva. Essa restrição é uma política da deEHR, não
uma restrição do próprio método `did:klever`; outros implementadores
podem usar signatários `secp256k1` ou BLS dentro do mesmo método.

### 2. Caminho de resolução

Um resolver — implementado como um driver do universal-resolver da
deEHR — executa:

1. Faz o parse do DID em `(network, address)`.
2. Conecta-se à URL do nó Klever configurada para a rede.
3. Lê o conjunto de permissões da conta (leitura nativa, sem taxas).
   Cada signatário registrado — seu peso, threshold e bitmask de
   operações — produz um verification method clássico. Sob a política
   da §1.5, as contas gerenciadas pela deEHR produzem entradas
   `Multikey` Ed25519.
4. Chama `Identity.resolveDid(address)` no contrato do Identity
   Registry. Isso retorna os commitments dos verification methods PQ,
   as chaves de key agreement, os service endpoints e a flag de
   deactivation.
5. Constrói e retorna o DID Document mesclando (3) e (4).

O DID Document **não** é armazenado on-chain como um blob opaco; ele é
**derivado** no momento da resolução a partir do conjunto de permissões
da conta e do registro do Identity Registry. Isso mantém os dados
on-chain mínimos, estruturalmente auditáveis e tamper-evident
(resistentes a adulteração).

### 3. Forma do DID Document

Um DID Document resolvido tem a forma a seguir (ilustrativo; valores de
campos abreviados para legibilidade):

```json
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/multikey/v1"
  ],
  "id": "did:klever:mainnet:klv1q5y...",
  "controller": "did:klever:mainnet:klv1q5y...",

  "verificationMethod": [
    {
      "id": "did:klever:mainnet:klv1q5y...#klv-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z6Mk..."
    },
    {
      "id": "did:klever:mainnet:klv1q5y...#pq-sig-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z<pq-sig-multicodec><pubkey>",
      "expires": "2031-05-26T00:00:00Z"
    },
    {
      "id": "did:klever:mainnet:klv1q5y...#pq-kem-1",
      "type": "Multikey",
      "controller": "did:klever:mainnet:klv1q5y...",
      "publicKeyMultibase": "z<pq-kem-multicodec><pubkey>"
    }
  ],

  "authentication":       ["#klv-1"],
  "capabilityInvocation": ["#klv-1"],
  "assertionMethod":      ["#pq-sig-1"],
  "keyAgreement":         ["#pq-kem-1"],

  "service": [
    {
      "id": "did:klever:mainnet:klv1q5y...#deehr-fhir",
      "type": "DEEHRFhirEndpoint",
      "serviceEndpoint": "https://fhir.deehr.example/p/<opaque-token>"
    }
  ]
}
```

Os papéis dos verification methods são escopados deliberadamente:

- `authentication` → `#klv-1`. Fazer login na plataforma comprova
  controle da conta Klever.
- `capabilityInvocation` → `#klv-1`. Autoriza chamadas on-chain — o
  único verificador que a KVM tem hoje.
- `assertionMethod` → `#pq-sig-1`. Assinatura de artefatos off-chain de
  longa duração: VCs de consentimento, atestações de integridade de
  bundles FHIR, entradas de log de auditoria.
- `keyAgreement` → `#pq-kem-1`. Criptografia em envelope da PHI
  off-chain para o holder.

Este é o modelo **híbrido**: clássico onde a chain precisa verificar;
pós-quântico onde os dados sobrevivem à chain.

### 4. Ancoragem on-chain de chaves PQ

Para cada verification method PQ, o Identity Registry armazena apenas:

- o identificador **multicodec** do esquema PQ (para que verificadores
  saibam como interpretar a chave),
- o **hash SHA-256** da chave pública (32 bytes — **não** a chave PQ
  completa),
- um timestamp `expires` opcional,
- o `verificationRelationship` (`assertionMethod`, `keyAgreement`, …).

A chave pública PQ completa vive off-chain, no keystore do holder e no
serviço de custódia da deEHR. Um verificador obtém a chave completa
off-chain (de um serviço da deEHR, de uma VC ou do próprio holder),
calcula seu hash e o compara com o commitment on-chain.

Isso mantém a footprint on-chain em 32 B por chave PQ, independentemente
do esquema — uma consideração material quando chaves públicas
ML-DSA-65 têm ~1,9 KB e assinaturas SLH-DSA-128f têm ~17 KB. O registro
on-chain funciona como um vínculo tamper-evident ("no bloco N, este
DID se comprometeu com esta pubkey PQ"), não como canal de distribuição
de chaves.

### 5. Operações

- **Create.** Implícito na criação da conta Klever: um `did:klever`
  existe assim que sua conta existe, com um DID Document mínimo derivado
  do conjunto de permissões da conta. Uma chamada explícita
  `Identity.registerDid` adiciona commitments de chave PQ, services e
  qualquer metadata estendido.
- **Read / Resolve.** Fluxo do resolver descrito na §2. Resolvível por
  qualquer parte com acesso à rede; nenhuma autorização especial é
  exigida.
- **Update — rotação de chave clássica.** Transação nativa Klever
  `UpdateAccountPermission`. O conjunto de permissões da conta é a fonte
  da verdade para os verification methods `#klv-*`; a próxima resolução
  reflete a mudança.
- **Update — rotação de chave PQ.**
  `Identity.rotatePqKey(method_id, new_multicodec, new_pubkey_hash, prev_pubkey_signature)`.
  A chave PQ anterior coassina a rotação como defesa em profundidade
  contra um atacante que tenha comprometido apenas a chave clássica. A
  recuperação usa a permissão de recuperação multisig conforme a
  ADR-0001.
- **Update — service endpoints.** `Identity.setServices(services)`.
  Controlado pelo holder.
- **Deactivate.** `Identity.deactivate()` define uma flag de deactivation
  no registro do Identity Registry. Resoluções subsequentes retornam um
  DID Document tombstone mínimo junto a um DID Document metadata em que
  `deactivated` é `true`. Pelo W3C DID Core, o sinal de deactivation
  pertence a `didDocumentMetadata.deactivated`, não a uma propriedade de
  topo do próprio DID Document.

Toda operação que altera estado emite um evento estruturado segundo o
modelo de eventos da ADR-0002, para que o indexador off-chain possa
reconstruir o histórico.

### 6. Considerações de privacidade

Uma chain pública não é anônima: qualquer coisa ancorada vinculada a um
DID é correlacionável para sempre. As propriedades de privacidade do
método são de pseudonimato, não de anonimato. Mitigações:

- **Nenhuma PHI on-chain.** Sem identificadores de paciente, sem dados
  legíveis por humanos — um invariante arquitetural da ADR-0002.
- **Service endpoints opacos.** As URLs em `serviceEndpoint` embutem um
  token opaco por DID; o serviço off-chain autoriza antes de retornar
  qualquer coisa.
- **DIDs pairwise suportados.** Um holder pode possuir múltiplos DIDs
  `did:klever` (um por relação — por exemplo, um para atenção primária,
  outro para um ensaio clínico). Cada um é sua própria conta Klever. O
  custo de UX é real e precisa ser ponderado do lado do produto.
- **Hashes de chaves PQ, não as chaves.** O que é ancorado é um hash de
  32 bytes; sem a chave completa off-chain, o registro on-chain é um
  commitment, não um identificador reutilizável.
- **Divulgação seletiva via VCs.** Concessões de autorização revelam
  apenas o que o escopo da VC permite; não expõem a PHI subjacente.

Exposição residual: qualquer DID que ancore uma chave PQ, registre
services ou participe de eventos de consent/anchor é observável. O
método não tenta ocultar isso, e as superfícies de produto não devem
prometer o contrário.

### 7. Cripto-agilidade

O método se compromete com *"slots" de verification method*, não com
algoritmos específicos. O slot clássico é verificado on-chain hoje; um
ou mais slots PQ são verificados off-chain. O algoritmo PQ em uso é
codificado no prefixo multicodec do verification method, de modo que
novos esquemas podem ser adicionados (e os obsoletos, rotacionados para
fora) sem revisar este método DID.

Quando a KVM Klever eventualmente adicionar um verificador PQ on-chain
(ML-DSA é o candidato mais provável), o mesmo verification method PQ
ancorado também se torna elegível para `capabilityInvocation` — sem
necessidade de migração de DID.

## Consequências

### Aspectos positivos

- Os dados de paciente de longa duração são resistentes a ataques
  quânticos desde o primeiro dia. A ameaça realista de *harvest-now,
  decrypt-later* é endereçada exatamente onde se aplica.
- O método está em conformidade com o W3C DID Core 1.0 e usa
  **apenas primitivas verificadas da Klever** — sem dependência de
  funcionalidades indisponíveis da KVM.
- Cripto-agilidade: a escolha do algoritmo PQ é parametrizada via
  multicodec, e não embutida no método.
- Compatibilidade futura com verificadores PQ on-chain da KVM — mesmo
  DID, sem migração.
- A footprint on-chain permanece em commitments de 32 bytes,
  independentemente do tamanho do esquema PQ.

### Aspectos negativos / riscos

- **Superfície de custódia confiável maior.** As chaves privadas PQ
  vivem no mesmo escopo de custódia das clássicas; o cenário de
  custódia do Signing & Fee Service agora cobre dois tipos de chave. O
  modelo de ameaças e os requisitos de HSM / KMS precisam refletir isso.
- **Maturidade das bibliotecas PQ.** Implementações em Rust /
  WebAssembly de esquemas PQ do NIST (por exemplo, `pqcrypto`,
  `liboqs-rust`, crates específicas de esquema) são menos
  battle-tested que `ed25519`. Uma revisão em nível de auditoria da
  biblioteca escolhida é obrigatória antes de qualquer uso em produção.
- **Atribuições de multicodec ainda em estabilização.** Algumas
  entradas de multicodec para PQ estão registradas, outras estão em
  draft. O perfil do método pode precisar de uma revisão menor quando
  essas entradas forem finalizadas.
- **Complexidade operacional.** Holders e guardiões efetivamente
  gerenciam dois tipos de chave. Fluxos de UX, backup e recuperação
  precisam cobrir os dois.
- **DID Documents maiores.** Entradas `Multikey` PQ são maiores que
  Ed25519; o cache do resolver passa a importar mais.

## Alternativas consideradas

- **`did:key` puro (sem chain).** Rejeitada — por especificação,
  `did:key` não tem rotação, services nem deactivation. A deEHR precisa
  das três. Útil como primitiva para interações efêmeras, não para o
  DID primário de um paciente.
- **`did:web` sobre um domínio controlado pela deEHR.** Rejeitada —
  ancora a confiança em um servidor protegido por TLS, perdendo a
  integridade on-chain que justifica uma blockchain de saída.
- **`did:klever` puramente clássico (somente Ed25519).** Rejeitada —
  deixa todos os dados assinados de longa duração em um cronômetro
  quântico de 10 a 15 anos.
- **`did:klever` somente PQ (PQ como signatário on-chain).** Rejeitada
  — a KVM Klever não tem verificador PQ hoje; chaves PQ não podem
  autorizar transações.
- **Identificador opaco com mapeamento on-chain
  (`did:klever:<network>:<random-32B>`).** Rejeitada — adiciona uma
  escrita de storage e um lookup de mapeamento por DID sem ganho real
  de privacidade em uma chain pública.
- **Long-form ancorado estilo Sidetree (estilo `did:ion`).**
  Considerada — permitiria DIDs auto-certificáveis com ancoragem em
  lote. Rejeitada para a Fase 0 por ser mais complexa do que justifica;
  revisitar se a portabilidade cross-chain virar objetivo.

## Questões resolvidas

As questões em aberto na proposta estão agora resolvidas — cada uma é
decidida aqui ou explicitamente postergada para um ADR de follow-up com
justificativa — promovendo este ADR a `Aceito`. Nenhuma delas bloqueia o
contrato de identidade da Fase 1 (apenas Ed25519,
[#27](https://github.com/brunocampos-ssa/deEHR/issues/27)), que não implementa
nenhum método de verificação PQ.

- **Hospedagem do driver do universal-resolver.** Decidido: **construir
  internamente para a Fase 1.** A deEHR entrega seu próprio driver de
  resolução `did:klever` para manter o MVP autocontido e sob seu controle;
  contribuir upstream ao projeto universal-resolver (descoberta e mais olhos)
  é reavaliado após a Fase 1, quando o driver estiver estável.
- **UX de DIDs pairwise.** Decidido: **postergar a exposição para a Fase 2.**
  O método já permite múltiplos DIDs `did:klever` por holder — cada um é sua
  própria conta Klever — então nenhuma mudança no método é necessária para
  adicionar DIDs pairwise depois. A Fase 1 expõe um único DID por paciente
  para manter onboarding e custódia simples.
- **Política de cache de DID Document.** Decidido: **postergar para o trabalho
  de resolver / indexador off-chain.** TTLs e gatilhos de invalidação são uma
  propriedade do resolver e do indexador da ADR-0002, não do método DID; são
  definidos quando esse componente for construído na Fase 1.

### Postergado para um ADR de follow-up (perfil de método de verificação pós-quântico)

As especificidades pós-quânticas são postergadas para um ADR de follow-up
dedicado, a ser escrito após avaliar o cenário de bibliotecas PQ Rust / WASM
e a orientação atual CNSA 2.0 / RNDS. O design de cripto-agilidade (§7)
garante que adicioná-las depois **não exige revisão do método DID em si** —
apenas o ADR de follow-up mais o suporte do Identity Registry aos campos de
commitment PQ já especificados no §4.

- **Esquema de assinatura PQ** — ML-DSA-65 vs Falcon-512 vs um composto
  híbrido Ed25519 / ML-DSA.
- **KEM PQ** — ML-KEM-768 é o candidato padrão, sujeito à mesma revisão de
  biblioteca.
- **Valores específicos de multicodec** — fixar os códigos para o(s)
  esquema(s) escolhido(s) assim que finalizados nos registries multiformats /
  W3C DID-extensions.
- **Semântica de recuperação para chaves PQ** — se o multisig de recuperação
  também precisa portar chaves PQ de guardião, ou se uma recuperação clássica
  única basta para rotacionar a chave PQ (trade-off superfície-de-ataque vs
  operabilidade de recuperação).

## Referências

- W3C **Decentralized Identifiers (DIDs) v1.0** —
  <https://www.w3.org/TR/did-core/>
- W3C **DID Specification Registries** —
  <https://www.w3.org/TR/did-spec-registries/>
- NIST **FIPS 203** (ML-KEM), **FIPS 204** (ML-DSA), **FIPS 205**
  (SLH-DSA), 2024.
- NSA **Commercial National Security Algorithm Suite 2.0** (CNSA 2.0),
  2022.
- `did:key` Method Specification —
  <https://w3c-ccg.github.io/did-method-key/>
- Sidetree Protocol / `did:ion` —
  <https://identity.foundation/sidetree/spec/>
- Verificação de capacidades da KVM Klever (2026-05-22) — host
  functions de criptografia verificadas (`ed25519`, `secp256k1`, BLS
  apenas; sem PQ), multisig ponderado nativo via
  `UpdateAccountPermission`.
- [ADR-0001](adr-0001-identity-and-key-management.pt-BR.md) — Identidade
  e Gestão de Chaves — Custódia Progressiva.
- [ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md) — Design dos
  Registries On-chain.

## Adendos

### 2026-06-15 — Questões em aberto resolvidas; promovido a Aceito

A ADR-0004 originalmente entrou como **Proposto** com sete questões em aberto.
Elas estão agora resolvidas (ver *Questões resolvidas*): o driver do
universal-resolver é **construído internamente na Fase 1**, a UX de DIDs
pairwise é **postergada para a Fase 2**, e o cache de DID Document é
**postergado para o trabalho de resolver / indexador off-chain**. As quatro
questões pós-quânticas (esquema de assinatura PQ, KEM PQ, valores de
multicodec, semântica de recuperação de chave PQ) são **postergadas para um
ADR de follow-up de perfil de método de verificação pós-quântico**; nenhuma
bloqueia o contrato de identidade da Fase 1 (apenas Ed25519,
[#27](https://github.com/brunocampos-ssa/deEHR/issues/27)), que não implementa
nenhum método de verificação PQ. Apenas a seção de questões em aberto foi
reestruturada (em *Questões resolvidas*); as seções de Decisão, Consequências
e Alternativas permanecem inalteradas. Com as
questões substantivas encerradas, a ADR é promovida de **Proposto** para
**Aceito** conforme este registro; emendas futuras são registradas como
entradas adicionais aqui, conforme a política append-only de ADRs do
repositório.
