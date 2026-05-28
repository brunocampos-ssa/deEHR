# Política de Segurança

🌐 **Languages / Idiomas:** [English](SECURITY.md) · **Português (Brasil)**

> ℹ️ A documentação canônica é mantida em **inglês** ([SECURITY.md](SECURITY.md)). Esta é a versão em **Português do Brasil**, mantida via convenção i18n. Em caso de divergência, prevalece a versão em inglês.

---

A deEHR é uma plataforma open-source de Registro Eletrônico de Saúde, na qual
o paciente é dono dos dados. Ela lida com **dados pessoais sensíveis de
saúde**, então a segurança é tratada como um gate de release a cada mudança —
não como uma fase.

Esta política explica como reportar uma vulnerabilidade e o que esperar.

## Reportando uma vulnerabilidade

**Não abra uma issue pública, pull request ou discussion para uma
vulnerabilidade de segurança.**

Reporte-a de forma privada por e-mail para **<brunocampos.ssa@gmail.com>**.

Por favor, inclua, na medida do possível:

- Uma descrição da vulnerabilidade e do seu impacto.
- Passos para reproduzir, ou uma prova de conceito.
- O(s) componente(s) afetado(s) e a versão, commit ou branch.
- Qualquer remediação sugerida.

Se você quiser criptografar seu reporte, envie uma mensagem inicial sem
detalhes sensíveis e combinaremos uma troca de chaves.

## O que esperar

A deEHR está em desenvolvimento inicial, então ainda não há um SLA formal.
A intenção é:

- **Confirmação** do seu reporte em até **5 dias úteis**.
- Uma **avaliação inicial** e classificação de severidade logo em seguida.
- **Comunicação coordenada e de boa-fé** até a correção.
- **Crédito** pela descoberta nas release notes, se você desejar.

## Versões suportadas

O projeto está em **pré-release**. Apenas o branch `main` é suportado, e as
correções de segurança são aplicadas ali. Branches de release versionados com
suas próprias janelas de suporte serão definidos na fase de endurecimento para
produção.

| Versão | Suportada |
| --- | --- |
| `main` (pré-release) | Sim |
| Releases com tag | Nenhuma publicada ainda |

## Escopo

Áreas relevantes para segurança da deEHR incluem, mas não se limitam a:

- **Tratamento de PHI** — o FHIR gateway, o armazenamento FHIR, criptografia
  em trânsito e em repouso, e as exportações criptografadas do registro sob
  posse do paciente.
- **Smart contracts** — os contratos Rust/WASM Klever (identidade, credencial,
  consentimento e os registries de ancoragem e auditoria): controle de
  acesso, overflow de inteiros, reentrância e questões específicas de WASM.
- **Autorização** — o servidor de autorização SMART App Launch / OAuth2 / OIDC
  e a ponte de consentimento on-chain.
- **Custódia de chaves e o serviço de assinatura e taxas** — o componente
  operado pela plataforma que custodia as chaves de conta e submete
  transações (veja a ADR-0001).
- **Identidade** — tratamento de DIDs, credenciais verificáveis e os fluxos
  de recuperação social / guardião.
- **O conector RNDS** — tratamento de certificados ICP-Brasil e a integração
  nacional.

## Modelo de segurança — invariantes-chave

- **Nenhuma PHI on-chain — jamais.** A blockchain armazena apenas hashes de
  integridade, recibos de consentimento, eventos de auditoria, DIDs e status
  de credenciais. Esse é um invariante arquitetural rígido, reforçado em
  revisão de código e auditorias.
- **Nenhuma PHI real no repositório.** Testes e fixtures usam apenas dados
  sintéticos.
- **Auditoria de segurança obrigatória** antes de cada release e antes de
  cada pull request, usando o ferramental de segurança designado pelo
  projeto. Mudanças em smart contracts exigem, adicionalmente, uma auditoria
  no nível do contrato.
- **Criptografia em todos os lugares** — TLS em trânsito, criptografia em
  envelope em repouso, autenticação vinculada ao dispositivo.
- **Proteção de dados desde a concepção** — alinhada à LGPD brasileira (dado
  de saúde é dado pessoal sensível) e informada pela HIPAA.

## Política de divulgação

Seguimos **divulgação coordenada**. Por favor, nos dê uma oportunidade
razoável de publicar uma correção antes de qualquer divulgação pública.
Combinaremos o timing com você, e não iremos perseguir nem apoiar ações
contra pesquisa de segurança de boa-fé conduzida em linha com esta política.

## Testes responsáveis

Ao pesquisar, não acesse, modifique ou exfiltre dados que não são seus, não
degrade o serviço para outras pessoas, e utilize apenas dados de teste
sintéticos. Não há implantação em produção neste momento.
