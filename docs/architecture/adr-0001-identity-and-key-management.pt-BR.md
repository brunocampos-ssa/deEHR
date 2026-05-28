# ADR-0001: Identidade e Gestão de Chaves — Custódia Progressiva

🌐 **Languages / Idiomas:** [English](adr-0001-identity-and-key-management.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([adr-0001-identity-and-key-management.md](adr-0001-identity-and-key-management.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

- **Status:** Aceito
- **Data:** 2026-05-22
- **Decisores:** mantenedores do deEHR

## Contexto

deEHR é uma plataforma de Registro Eletrônico de Saúde na qual o paciente é
dono dos seus dados. Pacientes precisam controlar a própria identidade e
consentimento, e esse controle precisa ser verificável e ancorado na
blockchain Klever.

Várias forças restringem como identidade e chaves podem funcionar:

- **Público-alvo.** Pacientes idosos e com baixa literacia digital são
  usuários de primeira classe. Seed phrases e carteiras convencionais de
  autocustódia são uma barreira severa de usabilidade *e* um risco de perda
  de dados — uma frase perdida pode significar um registro de saúde perdido.
- **A blockchain deve ser invisível por padrão.** Fazer login deve parecer
  como em qualquer aplicativo moderno.
- **Restrições verificadas da Klever KVM** (verificação de capacidades,
  2026-05-22). A Klever KVM **não** oferece:
  - account abstraction no estilo ERC-4337 (sem hook programável de
    validação de conta on-chain);
  - verificação on-chain de `secp256r1` / P-256 — a curva usada por
    WebAuthn/passkeys — então uma passkey **não pode** assinar uma
    transação Klever;
  - guardiões de conta nativos (os Guardians do MultiversX não foram
    herdados);
  - transações gasless / meta-transações nativas (sem campo de
    relayer/patrocinador no formato de transação).

  A Klever KVM **oferece**:
  - smart contracts como contas de primeira classe;
  - um **sistema nativo de permissões de conta multisig ponderado** —
    peso por signatário, limiar de aprovação e escopo por operação — que
    pode evoluir sem mudar o endereço da conta;
  - **pools de taxas em KDA**, permitindo que usuários transacionem pagando
    as taxas em um token emitido pelo app em vez de em KLV.

A promessa de "blockchain invisível / sem seed phrase / sem gas" precisa,
portanto, ser entregue **sem** depender de account abstraction nativa ou de
patrocínio nativo de gas.

## Decisão

Adotamos um modelo de identidade e gestão de chaves chamado **Custódia
Progressiva**.

1. **Autenticação.** Login por e-mail ou social (OIDC) somado a uma
   **passkey** (WebAuthn/FIDO2) com desbloqueio biométrico. A passkey é um
   **fator de autenticação off-chain**: ela autentica o paciente na
   plataforma deEHR. Ela não — e, na Klever, não pode — assinar transações
   da blockchain.

2. **Conta on-chain.** Cada paciente tem uma conta Klever padrão. Por padrão,
   a chave de assinatura dessa conta é **custodiada pela plataforma deEHR**,
   respaldada por HSM, e nunca é exposta ao paciente.

3. **Signing & Fee Service.** Um serviço operado pela plataforma submete as
   transações dos pacientes e paga as taxas de rede a partir de uma
   tesouraria da plataforma (opcionalmente via um pool de taxas em KDA). Como
   a Klever não tem mecanismo gasless nativo, **este serviço é o mecanismo
   gasless.**

4. **Recuperação.** A recuperação social é implementada sobre as permissões
   nativas de conta multisig ponderado da Klever: guardiões (por exemplo, um
   familiar, o médico de atenção primária e a plataforma) são registrados
   como signatários em uma permissão de recuperação com um limiar M-de-N.

5. **Identidade.** Cada paciente tem um DID `did:klever` com um DID Document
   on-chain. O método `did:klever` em si é especificado separadamente na
   **ADR-0004** (planejado).

6. **Espectro de custódia progressiva.** O padrão é **custódia assistida**
   (chave sob posse da plataforma + recuperação por guardiões). O paciente
   pode assumir progressivamente: adicionar a chave do próprio dispositivo
   como signatária, reduzir o peso do signatário da plataforma e, em última
   instância, exportar para autocustódia completa. Isso é progressivo e
   nunca forçado.

7. **Chaves de criptografia de dados.** As chaves que protegem a PHI são
   respaldadas por guardiões através do mesmo mecanismo de recuperação,
   de modo que um dispositivo perdido nunca significa um registro de saúde
   perdido.

## Consequências

### Aspectos positivos

- UX familiar, sem senha; sem seed phrases; acessível ao público-alvo.
- Pacientes nunca precisam guardar KLV nem entender de gas.
- Um caminho real de soberania é preservado (autocustódia opt-in).
- Construído inteiramente sobre primitivas **verificadas** da Klever — sem
  dependência de recursos indisponíveis.

### Aspectos negativos / riscos

- **O Signing & Fee Service e a custódia de chaves tornam-se críticos para
  segurança** — um ponto central de confiança e um alvo de ataque de alto
  valor. Precisa ser respaldado por HSM, estritamente controlado por
  acesso, monitorado e auditado de forma independente. É um componente de
  primeira classe, não código de cola.
- **Dependência de tesouraria.** A plataforma precisa manter a tesouraria
  de taxas / pool de KDA com saldo — um compromisso operacional e
  financeiro. Spam ou abuso podem drená-la, então limites de taxa e quotas
  por conta são necessários.
- **Custodial-por-padrão tem peso regulatório.** O deEHR guarda chaves dos
  pacientes por padrão; as implicações de LGPD e responsabilidade precisam
  ser tratadas no modelo de ameaças e em revisão jurídica.
- "Patient-owned" é em parte aspiracional até que o paciente assuma a
  custódia — a comunicação do produto precisa permanecer honesta sobre isso.
- O modelo depende da semântica de permissões de conta da Klever; se ela
  mudar, recuperação e custódia progressiva precisam ser revisitadas.

## Alternativas consideradas

- **Autocustódia pura (seed phrase / carteira sob posse do paciente).**
  Rejeitada — uma barreira severa para o público-alvo e um risco
  inaceitável de perda de dados.
- **Custódia pura sem caminho para autocustódia.** Rejeitada — contradiz os
  princípios de propriedade pelo paciente e de soberania.
- **Depender de account abstraction nativa / patrocínio de gas da Klever.**
  Rejeitada — não está disponível, sem roadmap público; bloquearia o
  projeto indefinidamente.
- **Identidade apenas off-chain (sem DID, sem conta on-chain).** Rejeitada —
  perde o consentimento verificável, portável e ancorado em chain, que é a
  proposta central de valor do deEHR.

## Questões em aberto

- A **ADR-0004** precisa especificar o método DID `did:klever`.
- Seleção de HSM / KMS para o serviço de custódia.
- Modelo de financiamento da tesouraria e quotas / limites de taxa concretos
  contra abuso.
- Se construir sobre infraestrutura nativa de custódia da Klever (ex.:
  KleverSafe) ou um serviço de HSM operado pelo próprio projeto.
- Reverificar as funções de host de criptografia da Klever e a semântica
  das permissões de conta antes da implementação; perguntar à Klever
  diretamente se `secp256r1` on-chain ou patrocínio nativo de taxas estão
  no roadmap.

## Referências

- [README.md](../../README.md) — seção *Identity & Key Management —
  "Progressive Custody"*.
- Verificação de capacidades da Klever KVM (2026-05-22): sem account
  abstraction ERC-4337, sem `secp256r1` on-chain, sem guardiões nativos,
  sem transações gasless nativas; permissões nativas de conta multisig
  ponderado e pools de taxas em KDA estão disponíveis.
- [ADR-0002](adr-0002-on-chain-registry-design.pt-BR.md) — Design dos
  Registros On-chain.
- W3C Decentralized Identifiers (DID) Core; W3C WebAuthn / FIDO2.

## Adenda

### 2026-05-26 — ADR-0004 publicada

A [ADR-0004](adr-0004-did-klever-method.pt-BR.md) — Método DID
`did:klever` — foi publicada como **Proposta**, cumprindo a referência
planejada no §5 da *Decisão* acima e resolvendo o item correspondente em
*Questões em aberto*. A decisão da ADR-0001 permanece inalterada; este
adendo é registrado conforme a política append-only de ADRs do repositório
(ver [docs/architecture/README.md](README.pt-BR.md) — *O que é um ADR?*).
