# OpenBoom - Decision Log (ADRs)

## ADR-001: Separação do Projeto em Repositório Próprio
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Estávamos executando comandos no diretório de `decompilator` (ferramentas de engenharia reversa REA).
* **Decisão:** Criar o projeto de forma limpa e isolada em `c:\Users\joaov\Documents\GitHub\open-boom`.
* **Consequência:** O código de desenvolvimento fica completamente isolado do ferramental de análise, mantendo o histórico de commits e controle de dependências limpos. O acesso aos MCPs do REA permanece intacto, pois o servidor MCP é registrado globalmente no Antigravity.

## ADR-002: Adoção do Harness de Memória BMAD
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Projetos de áudio e UI complexos sofrem com perda de contexto e desvio de escopo ao longo de múltiplas iterações com agentes de IA.
* **Decisão:** Estruturar a pasta `.bmad/` com Briefing, Arquitetura, Milestones, Task Backlog, Decision Log e Session State.
* **Consequência:** Toda sessão recupera imediatamente o estado atual, próximas ações e decisões anteriores sem ambiguidades.

## ADR-003: Estratégia de Áudio no Windows (Evitar Driver de Kernel Pago)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** O Boom 3D original utiliza um driver de kernel proprietário (`boomvad.sys`) assinado com certificado WHQL comercial.
* **Decisão:** Para o OpenBoom, a interceptação de áudio deve utilizar APIs padrão do Windows (WASAPI Loopback, dispositivos virtuais livres ou Windows Audio Processing Objects - APO), eliminando a barreira de instalação de drivers não assinados.
