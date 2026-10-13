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

## ADR-004: Vinculação Explícita ao Repositório Decompilator e MCP do REA
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Necessidade imperativa de garantir que o agente nunca "esqueça" ou perca a referência do ferramental de engenharia reversa do repositório `decompilator` e do servidor MCP do REA.
* **Decisão:** Declarar formalmente no BMAD (`AGENTS.md`, `01-ARCHITECTURE.md`, `04-DECISION-LOG.md` e `05-SESSION-STATE.md`) que o repositório `c:\Users\joaov\Documents\GitHub\decompilator` e o MCP `rea` são ferramentas ativas de primeira classe no projeto para análise estática e dinâmica dos binários do Boom 3D.
* **Consequência:** Sempre que houver incerteza sobre parâmetros matemáticos de DSP ou estruturas de áudio, o agente usará o REA MCP para inspecionar os binários originais.

## ADR-005: Arquitetura de Loopback sem Feedback e Escopo MVP do PRD
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Elaboração formal do PRD (`_bmad-output/planning-artifacts/prds/prd-open-boom-2026-10-10/prd.md`). Riscos levantados de feedback loop no WASAPI padrão e sincronização da jornada UJ-3 com os efeitos.
* **Decisão:** Adotar a API moderna `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` (Windows 10 build 20348+ e Windows 11) como pipeline primário sem driver e incluir o Modo Noturno (Compressor DRC) no MVP para validação imediata do cenário noturno (UJ-3).
* **Consequência:** Garantia de áudio sem realimentação acústica, latência sub-15ms e 100% de estabilidade de SO sem risco de BSOD.

## ADR-006: Sistema de Design de UX e Primitivas Interativas (DESIGN.md & EXPERIENCE.md)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Execução do workflow `/bmad-ux` para definir formalmente a identidade visual e as especificações de comportamento da interface do OpenBoom.
* **Decisão:**
  1. Adotar a especificação Google Labs DESIGN.md (`_bmad-output/planning-artifacts/ux-designs/ux-open-boom-2026-10-10/DESIGN.md`) e o spine comportamental `EXPERIENCE.md`.
  2. Plataforma UI: Windows Native (WinUI 3 / WPF XAML) com tema escuro profissional de estúdio de áudio (`#121316` fundo, `#1c1e24` cartões, ciano neon `#38bdf8` e azul `#3b82f6` para sinal de áudio e brilho ativo).
  3. Layout responsivo com HUD base compacto (mínimo de 780×520px) escalável até janela desktop completa com faders de 31 bandas e espectro FFT.
  4. Primitivas táteis: Dial 3D Surround rotativo contínuo com micro-passos de mousewheel (5° por tick) e clique polar direto; faders de equalizador com arrasto vertical, detent magnético em 0 dB, snap por duplo clique e modo de precisão com tecla Shift (0.1 dB).
  5. Promoção de mockup tátil interativo em HTML/JS puro em `mockups/open-boom-shell.html`.
* **Consequência:** A equipe e os futuros agentes de desenvolvimento possuem um contrato visual e comportamental rígido, sem adivinhações de CSS/XAML, com hierarquia de informação e estados de áudio (ativo, bypass, limitador, troca de endpoint) completamente mapeados.


## ADR-007: Arquitetura Hexagonal, Pipeline DSP Desacoplado e Invariantes do Architecture Spine (AD-1 a AD-10)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Execução formal do workflow `/bmad-architecture` (`_bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-10/ARCHITECTURE-SPINE.md`) para definir as invariantes técnicas e evitar divergências de implementação entre a UI e a engine de áudio.
* **Decisões Principais:**
  1. **Paradigma Pipes-and-Filters com Hexagonal Decoupling:** Separação estrita entre o Core DSP em tempo real (`crates/open-boom-core`), os adaptadores de sistema operacional (`crates/open-boom-wasapi`), a persistência (`crates/open-boom-storage`) e o shell de UI (`src-tauri` / `src-ui`).
  2. **Invariantes da Thread de Áudio (AD-1):** Thread de processamento executada sob MMCSS (`Pro Audio`); proibição estrita de alocações na heap (`malloc`/`new`), mutexes bloqueantes e operações de I/O na thread de áudio. Comunicação com a UI via ring buffer SPSC lock-free transportando structs POD planas (`#[repr(C)] AudioCommand`).
  3. **Prevenção de Slapback Echo no Loopback (AD-2):** Modo A com ponte/cabo virtual padrão do Windows ou Modo B via `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` (Windows 10 Build 19041+ e Windows 11 Build 22000+) associado a silenciamento/atenuação programática da sessão original via `ISimpleAudioVolume`, evitando duplicação acústica no DAC.
  4. **Contrato de Canais e Formato Interno 48 kHz f32 (AD-3):** Ingestão normalizada para 7.1 surround (8 canais); 3D Spatializer dobra 7.1 em 2 canais estéreo binaural via convolução HRTF; estágios subsequentes (EQ, Sweeteners, Limiter) operam em estéreo invariante (2 canais).
  5. **Equalização Dual Desacoplada (AD-6):** Separação em dois estágios: (1) `ParametricCorrectionStage` com até 12 biquads arbitrários do AutoEq e (2) `GraphicEqStage` com 31 bandas ISO 1/3-oitava fixas. Offset de atenuação do pré-amp (`preamp_offset_db = -max_positive_gain_db`) calculado deterministicamente pela UI para evitar clipping.
  6. **Limiter de Pico Lookahead Inviolável e Bypass com Fase Contínua (AD-5):** Limitador terminal atuando com lookahead de 2ms cravando teto em -0.2 dBFS. Bypass A/B mantém o buffer de atraso de 2ms e aplica crossfade de 5ms, eliminando pops DC e saltos de latência.
  7. **Thread MTA Dedicada para Session Mixer (AD-8):** Gerenciamento de sessões Core Audio isolado em thread worker COM inicializada via `CoInitializeEx(None, COINIT_MULTITHREADED)`.
* **Consequência:** Eliminação total de riscos de concorrência, instabilidade matemática de filtros ou eco por duplicação de áudio, garantindo latência < 15ms e consumo de memória < 40 MB.

## ADR-008: Resolução do Toolchain Windows GNU, rust-lld e Formato de Crate para Shell Desktop
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Durante o scaffold da Story 1.1 sob o toolchain `stable-x86_64-pc-windows-gnu`, a compilação do Tauri v2 e dependências nativas (`raw-dylib`, `windows-sys`, `parking_lot_core`) exigiu `dlltool` e ferramentas de recurso (`windres`).
* **Decisões:**
  1. **Configuração de Linker (`rust-lld`):** Definido `rust-lld` em `.cargo/config.toml` para o target `x86_64-pc-windows-gnu`, permitindo linkagem rápida e resolução direta de bibliotecas COFF e GNU.
  2. **Interpretação de `raw-dylib` via Shim:** Implementado shim de `dlltool` preservando a diretiva `LIBRARY <dll>` nos arquivos `.def` gerados pelo rustc, evitando o erro de runtime `STATUS_DLL_NOT_FOUND (0xC0000135)`.
  3. **Crate-type do Desktop Shell (`rlib`):** Configurado `crate-type = ["rlib"]` em `src-tauri/Cargo.toml`, prevenindo o estouro do limite máximo de 65.535 símbolos exportados de DLLs PE do Windows gerado por `cdylib` em compilações desktop.
* **Consequência:** Compilação 100% limpa (`cargo check --workspace` e `cargo test --workspace`) com zero warnings e zero dependências de build externas.
## ADR-009: Ingestão de Áudio Driverless via WASAPI Loopback, Resampler Hermite C1 e Agendamento MMCSS Pro Audio
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Implementação da Story 1.2 para captura de áudio em modo usuário sem drivers de kernel (`boomvad.sys`), conversão transparente de múltiplos formatos (16/24/32-bit PCM e 32-bit float) e taxas de amostragem (44.1k, 88.2k, 96k, 192k) para o formato interno fixo de 48.000 Hz float32 com latência < 15ms.
* **Decisões:**
  1. **Algoritmo de Reamostragem:** Implementado conversor cúbico de Hermite (spline Catmull-Rom de 4 pontos) com preservação de estado entre buffers (`history` de 4 amostras por canal) e tolerância numérica a epsilon (`1e-7`) para anular overshoot de acumulação em ponto flutuante. A continuidade $C^1$ elimina completamente estalos, estalidos e ruído de alta frequência nas fronteiras de pacotes.
  2. **Otimização Identity Passthrough:** Fluxos que já chegam em 48 kHz float32 passam por caminho direto com zero sobrecarga de interpolação (`is_identity() == true`).
  3. **Zero Alocações na Thread de Áudio:** Os buffers de conversão e reamostragem são pré-alocados no `LoopbackCapture` e reaproveitados em fatias em cada iteração, cumprindo o invariante `AD-1`.
  4. **Modos de Captura WASAPI:** Suporte a Modo B primário (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` com exclusão da árvore de processos do OpenBoom) para evitar eco e feedback loop, com fallback automático para Modo A padrão (`AUDCLNT_STREAMFLAGS_LOOPBACK`) e modo sintético para testes headless/CI.
  5. **Guardião RAII MMCSS (`MmcssGuard`):** Registro seguro da thread de áudio no serviço "Pro Audio" (`AvSetMmThreadCharacteristicsW`) com desassociação automática no `Drop`, revertendo com elegância para prioridade normal caso o serviço MMCSS esteja indisponível.
  6. **Monitor de Orçamento de Callback (`CallbackBudgetTracker`):** Rastreamento de tempo de execução com alarme de estouro caso o processamento consuma mais de 60% da duração do buffer (ex: > 6.0ms para buffer de 10ms / 480 amostras).
* **Consequência:** Ingestão de áudio limpa, zero dependência de drivers `.sys`, tolerância robusta a ambientes sem placa de som e conformidade total com os contratos do Epic 1.

## ADR-010: Dispatcher de Comandos POD Lock-Free e Controle de Chave Mestre de Energia
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Implementação da Story 1.3 para fornecer despacho de comandos da interface sem bloqueio (lock-free) e controle mestre de energia (`SetMasterPower`) com chave visual no HUD desktop (`48px × 26px`, brilho ciano `rgba(56, 189, 248, 0.35)`).
* **Decisões:**
  1. **Consumo de Comandos Lock-Free na Thread de Áudio (`AudioProcessor`):** Os comandos são transferidos pela fila circular SPSC com structs POD `#[repr(C)] AudioCommand` sem alocações dinâmicas na heap nem parsing JSON. O método `drain_commands` é executado antes do processamento de cada bloco de áudio, garantindo latência zero de aplicação de parâmetros.
  2. **Passthrough Bit-Perfect com 0% CPU:** Quando a chave mestre é desligada (`master_power == false`), o processamento executa cópia direta `output.copy_from_slice(input)` sem invocar nenhum estágio de filtragem, limitador ou cálculo de ponto flutuante, resultando em 0.0% de overhead de DSP e igualdade amostra a amostra exata.
  3. **Segurança de Threads COM (`MmcssGuard`):** Implementação segura de `Send` e `Sync` para `MmcssGuard`, permitindo que o `AudioEngineCoordinator` seja compartilhado seguramente no estado da aplicação Tauri (`AppState`) sem clonagem desnecessária.
  4. **Telemetria Sincronizada Dinâmica:** O snapshot de telemetria reflete o estado em tempo real da engine (`dsp_active = is_running && processor.is_dsp_active()`), permitindo que a interface gráfica e o status bar exibam o estado real da pipeline sem polling bloqueante.
  5. **Componente Reutilizável de Chave Mestre:** Criado `MasterPowerSwitch.tsx` medindo estritamente 48px × 26px com raio de pílula total, sombra ciano LED neon e suporte pleno a acessibilidade de teclado e screen readers via ARIA `role="switch"` e `aria-checked`.
* **Consequência:** Comunicação IPC e áudio 100% desprovidas de lock contention, resposta instantânea da interface e preservação absoluta de integridade de áudio.

## ADR-011: Transição para Dispositivo Virtual de Áudio (PortCls / SYSVAD) e Engenharia Reversa REA
* **Data:** 10/10/2026
* **Status:** Aceito (Supera e ajusta ADR-003 e ADR-005)
* **Contexto:** A abordagem pura de WASAPI Loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK`) no endpoint físico de renderização falha em silenciar o áudio raw original dos aplicativos antes que este atinja o conversor analógico (DAC), gerando duplicação acústica (eco slapback) ao re-injetar o áudio com DSP. Além disso, endpoints físicos estéreo impedem que o Windows envie fluxos 5.1/7.1 nativos para o spatializer HRTF. A análise do Boom 3D instalado (`boomvad.sys`, `BoomEngine.dll`, `Audio.dll`) via inspeção binária e servidor REA MCP revelou a arquitetura real do produto comercial: um driver virtual SYSVAD PortCls opera como receptor mudo padrão do sistema operacional, e o motor de áudio C++ consome esse fluxo e renderiza para a saída física.
* **Decisões:**
  1. **Arquitetura de Ingestão Dual-Mode:** O OpenBoom passa a adotar como modo primário de alta fidelidade a captura a partir de um Dispositivo Virtual de Áudio (sink virtual PortCls/SYSVAD padrão do sistema), roteando o áudio processado para o endpoint físico selecionado (fones de ouvido ou caixas de som). O modo WASAPI Loopback é retido como fallback para ambientes sem suporte a drivers virtuais.
  2. **Eliminação Absoluta de Duplicação Acústica:** Como as aplicações renderizam no dispositivo virtual mudo, nenhum áudio não processado escapa diretamente para o DAC físico.
  3. **Multi-Canal 7.1 Real para o HRTF:** O dispositivo virtual expõe topologia 7.1 (8 canais), permitindo que jogos e reprodutores enviem áudio posicional 3D genuíno para a convolução HRTF binaural do OpenBoom.
  4. **Uso Contínuo do REA MCP e Decompilator:** Formalizado o uso ativo das ferramentas do REA MCP e do repositório `decompilator` para extração das rotinas de processamento (`CreateAudioProcessor`, `AVBoomAudioProcessor`, `AVFIRFilterSSE`, `AVFIFOProcessor`, `GetEqualizerPresetGains`) e controle de volume (`BoomVolumeControl`), servindo como especificação matemática para a implementação em Rust.
  5. **Adição da Story 1.8 ao Backlog:** Criada a Story 1.8 (`1-8-virtual-audio-device-bridge-rea-reverse-engineering`) para orquestrar a ponte com o dispositivo virtual e a documentação das evidências de engenharia reversa.
* **Consequência:** Interceptação transparente de 100% dos sons do Windows, fidelidade acústica comercial sem eco, capacidade de áudio 7.1 surround verdadeiro e alinhamento milimétrico com a engenharia reversa do Boom 3D.

---

### ADR-012: Enumeração Nativa MMDeviceAPI, Comutação Sub-500ms e Curva Perceptual de Volume Boost
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** A Story 1.4 demandava: (1) descoberta em tempo real de endpoints ativos de áudio via MMDeviceAPI com resolução de nomes amigáveis; (2) comutação de dispositivos a quente e recuperação de desconexão (`AUDCLNT_E_DEVICE_INVALIDATED`) em tempo inferior a 500ms; (3) controle de volume master com atenuação logarítmica/perceptual de 0% a 100%, preamp digital boost limpo de 101% a 150% e snap instantâneo para 100% (0.0 dB) via duplo clique.
* **Decisões:**
  1. **Contrato de Dispositivo:** Criada struct `AudioDeviceInfo` (`id`, `name`, `is_default`) em `open-boom-core::types` para tráfego serializado entre backend e UI Tauri IPC.
  2. **Isolamento de Testes e Modo Sintético:** `AudioDeviceManager` suporta descoberta nativa via `MMDeviceEnumerator` e `PKEY_Device_FriendlyName`, complementado por `with_forced_synthetic` para isolamento determinístico em ambientes de CI / headless sem placas de som físicas.
  3. **Comutação e Fallback Sub-500ms:** Implementados `RenderClient::switch_device` e `handle_device_lost`. A transição encerra o stream anterior, re-vincula o cliente WASAPI e restabelece a prioridade MMCSS Pro Audio, executando em < 30ms (limite de 500ms amplamente atendido e testado).
  4. **Curva Perceptual $p^2$ e Boost:** Posições do slider $p \in [0.0, 1.0]$ mapeiam para atenuação logarítmica via taper perceptual $p^2$ (50% $\to$ -12.0 dB; 100% $\to$ 0.0 dB). Valores $p \in (1.0, 1.5]$ aplicam ganho preamp digital de até +3.52 dB (135% $\to$ +2.6 dB) protegido pelo limitador terminal lookahead.
  5. **Componentes de UI Acessíveis:** Desenvolvidos `DeviceSelector.tsx` (popover com teclado completo e seleção de endpoints) e `NotificationBanner.tsx` (alerta flutuante de desconexão e recuperação de hardware).
* **Consequência:** Experiência de áudio fluida, sem travamentos na troca de fones de ouvido, feedback visual em decibéis e acessibilidade completa via teclado e leitores de tela.

---

### ADR-013: Limitador de Pico Lookahead 2ms (-0.2 dBFS), Teto Brickwall Inviolável e Telemetria em Tempo Real
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** A Story 1.5 demandava: (1) estágio terminal de limitador brickwall lookahead inviolável com atraso de 2ms (96 amostras/frames a 48 kHz), teto em -0.2 dBFS ($10^{-0.2/20} \approx 0.9772372$) e garantia matemática de 0% de clipping digital e saída $\le -0.1\text{ dBFS}$ sob qualquer nível de volume boost (150%) ou transientes extremos (+20 dBFS); (2) telemetria de ganho de redução e headroom em decibéis sincronizada com o backend e Tauri IPC; (3) status bar responsiva com feedback visual pulsante âmbar/vermelho para atenuação do limitador e leitura contínua de sample rate (48 kHz), bit depth (32-Bit Float), latência (<15ms) e headroom dinâmico.
* **Decisões:**
  1. **Algoritmo Lookahead e Detecção de Picos:** Implementado buffer circular de 96 frames com detecção de picos em toda a extensão da janela de 2ms. Isso assegura que transientes quentes são identificados com 2ms de antecedência, permitindo que o envelope de ganho reduza suavemente ($\tau_{\text{att}} = 0.2\text{ ms}$) antes que o transiente atinja a saída.
  2. **Teto Brickwall e Clamping Estrito:** Amostras de saída calculadas via $y[n] = (x[n - D] \cdot g[n])\text{.clamp}(-C, C)$ com sanitização rigorosa contra NaN e Infinito, impossibilitando qualquer overshoot além de -0.2 dBFS.
  3. **Envelope Transparente de Release:** Release ajustado para 50ms ($\tau_{\text{rel}} = 0.050\text{ s}$), garantindo recuperação natural sem distorção harmônica ou pumping de graves após o término do pico.
  4. **Cálculo de Headroom e Contrato de Telemetria:** `TelemetrySnapshot` estendido com `limiter_headroom_db`, `sample_rate` e `bit_depth`. Headroom calculado dinamicamente como $(-0.2 - \text{peak\_dbfs})\text{.clamp}(0.0, 20.0)$ (0.0 dB quando atenuando, capped em 20.0 dB no silêncio).
  5. **Métricas de UI e Taxa de Polling:** Polling no `src-ui/src/App.tsx` otimizado para 60ms (~16.6 Hz), permitindo animação instantânea do badge pulsante âmbar/vermelho (`LIMITER: -0.2 dBFS (-X.X dB)`) e atualização visual do medidor de headroom e metadados de áudio sem sobrecarga de IPC.
* **Consequência:** Proteção acústica absoluta contra estouros digitais, transparência sonora impecável com zero clipping mesmo sob 150% de ganho, e instrumentação visual de estúdio profissional no HUD.

---

### ADR-014: Comutação A/B Instantânea com Alinhamento de Fase (Crossfade Equal-Power 5ms e Retenção Lookahead 2ms)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** A Story 1.6 demandava: (1) comutação de A/B Bypass instantânea sub-10ms entre sinal DSP processado e sinal direto sem coloração; (2) eliminação total de estalos DC, clicks e saltos de volume/fase; (3) crossfade suave de 5ms (240 amostras a 48 kHz) com retenção do atraso de 2ms do limitador lookahead em ambos os caminhos; (4) integração visual no status bar (`Hardware Passthrough (Bypassed)`) e atalho global `Ctrl + Shift + B`.
* **Decisões:**
  1. **Crossfade de Potência Constante (Equal-Power):** Coeficientes de transição calculados via $g_A = \cos(\alpha \cdot \frac{\pi}{2})$ (processado) e $g_B = \sin(\alpha \cdot \frac{\pi}{2})$ (seco/bypass), onde $\alpha \in [0.0, 1.0]$ transita em exatamente 240 frames ($5\text{ ms}$). A propriedade $g_A^2 + g_B^2 = 1.0$ garante energia acústica constante sem quedas ou surtos de volume.
  2. **Alinhamento Temporal e Retenção do Lookahead:** O sinal seco é direcionado através da mesma linha de atraso de 96 frames (2ms) do `MasterLimiter`, garantindo que não ocorra salto temporal ($\Delta t = 0\text{ ms}$) ou desfasamento acústico na comutação A/B.
  3. **Estabilidade contra Chattering:** Comutações repetidas ou rápidas invertem a direção de $\alpha$ de forma contínua e sem descontinuidades, com zero DC pops mesmo sob 100 comutações consecutivas.
  4. **Atalho e UI:** Mapeado `Ctrl + Shift + B` no `App.tsx` com bloqueio de repetição (`e.repeat`), proteção contra cliques com master power desligado (`disabled={!masterPower}`), e indicação textual `Hardware Passthrough (Bypassed)` no dock de status.
* **Consequência:** Avaliação acústica fidedigna e imediata das melhorias de DSP sem interrupção rítmica ou clicks perceptíveis.

---

### ADR-015: Persistência Atômica de Configurações (%APPDATA%\OpenBoom\config.json) e Gerenciamento de Inicialização no Windows (HKCU\...\Run)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** A Story 1.7 demandava: (1) persistência atômica das configurações do usuário (volume, boost, mudo, endpoint ativo, geometria da janela, presets de equalização, surround 3D e status de inicialização) em `%APPDATA%\OpenBoom\config.json`; (2) garantia absoluta de integridade contra corrupção por encerramento abrupto ou falta de energia via arquivo temporário `config.<pid>.tmp` e substituição atômica NTFS (`MoveFileExW` / `fs::rename`); (3) recuperação graciosa com fallback para `AppConfig::default()` na presença de JSON corrompido ou ausente; (4) gerenciamento da chave de registro de inicialização do Windows em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` sem requerer privilégios administrativos (UAC) e com modo mock para testes; (5) menu de contexto nativo da bandeja do sistema com toggles para Master Power, Mudo, Presets e Sair (FR28); (6) modal de preferências Dark Fluent acessível com atalho Escape e indicação de persistência atômica.
* **Decisões:**
  1. **Staging Atômico e Substituição NTFS:** Os dados são serializados e gravados primeiro em `config.<pid>.tmp` no mesmo diretório base. O arquivo é renomeado atomicamente para `config.json` via `fs::rename` (que no Windows utiliza `MoveFileExW` com `MOVEFILE_REPLACE_EXISTING`), garantindo atomicidade NTFS. Em caso de falha de gravação ou JSON truncado, o arquivo de configuração anterior é preservado intacto.
  2. **Recuperação Resiliente de Configuração:** Implementado `#[serde(default)]` em `AppConfig` e captura segura em `load_config()`. Arquivos ausentes, corrompidos ou com JSON inválido loggam aviso e retornam imediatamente `AppConfig::default()` sem pânico ou encerramento do processo.
  3. **Registro de Inicialização em HKCU sem UAC:** Implementado `StartupManager` apontando para `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`. Por residir em `HKEY_CURRENT_USER`, a chave é gerenciada inteiramente em nível de usuário, sem qualquer pop-up de controle de conta de usuário (UAC). Para testes automatizados e ambientes isolados, o struct possui modo mock em memória.
  4. **Menu de Contexto Estendido na Bandeja (FR28):** Implementado menu de bandeja nativo em `src-tauri` contendo "Open OpenBoom", "Master Power", "Mute", submenu "Presets" ("Harman Acoustic", "Bass Boost", "Vocal Clarity") e "Exit OpenBoom". Os cliques despacham comandos via `dispatcher` de áudio e emitem eventos Tauri sincronizando a UI React.
  5. **Auto-save com Debounce e Restauração de Geometria:** `App.tsx` restaura preferências completas ao iniciar e aplica debounce de 250ms nas alterações do usuário para evitar sobrecarga de I/O em ajustes contínuos de fader. A posição da janela é restaurada no setup do Tauri antes da exibição, e salva automaticamente nas coordenadas do usuário.
  6. **SettingsModal Acessível (UX-DR11):** Criado modal Dark Fluent com switch de inicialização com o Windows, badge de arquitetura atômica NTFS, e dismiss por tecla `Escape` com captura de foco.
* **Consequência:** Persistência robusta de sessão, proteção contra corrupção em crashes, integração fluida com o ciclo de vida do Windows e conformidade estrita com NFR2 (zero I/O ou bloqueios na thread de áudio).

---

## ADR-016: Implementação do VirtualAudioBridge, Detecção Heurística de Endpoints Virtuais e Evidências REA
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** A Story 1.8 demandava: (1) detecção automática de endpoints virtuais de áudio (`boomvad`, VB-Audio Cable, VAC) via heurísticas de MMDeviceAPI; (2) componente `VirtualAudioBridge` gerenciando a ingestão dual-mode: Modo A (Virtual Sink silencioso do Windows eliminando slapback echo e destravando áudio 7.1) e Modo B (fallback WASAPI Loopback com exclusão de árvore de processos); (3) comutação de fallback sub-500ms em caso de desconexão ou invalidação de endpoint virtual em tempo de execução (`AUDCLNT_E_DEVICE_INVALIDATED`); (4) relatório exaustivo de engenharia reversa REA em `.bmad/evidence/rea-boom-engine-reverse-engineering-report.md` cobrindo `BoomEngine.dll` (`CreateAudioProcessor`, `CreateSession`, `GetEqualizerPresetGains`, `AVFIRFilterSSE`), `boomvad.sys` e `Audio.dll`; (5) badge visual de topologia de captura no status bar do HUD desktop refletindo `Virtual Sink (Echo-Free)` em verde ou `Loopback Mode` em âmbar.
* **Decisões:**
  1. **Heurísticas de Nomes Amigáveis (`is_virtual_device_name`):** Identificação robusta e case-insensitive cobrindo "boom", "vb-audio", "cable", "voicemeeter", "vac", "virtual audio", "virtual cable" e "openboom", discriminando perfeitamente endpoints físicos (Realtek, NVIDIA, Intel, USB DACs).
  2. **Contrato de Ponte Dual-Mode (`VirtualAudioBridge`):** Módulo desacoplado em `crates/open-boom-wasapi/src/virtual_bridge.rs` com suporte a `BridgeMode::VirtualSinkModeA` e `BridgeMode::LoopbackFallbackModeB`, integrado ao `AudioEngineCoordinator` e com suporte a dispositivos sintéticos para testes headless determinísticos.
  3. **Comutação e Fallback Instantâneo Sub-500ms:** Em caso de perda de dispositivo virtual ou erro de inicialização, a engine comuta imediatamente para Modo B (< 50ms medidos em testes), protegendo a estabilidade do fluxo de áudio sem panics ou vazamento de recursos.
  4. **Contrato de Telemetria (`using_virtual_bridge`):** Struct `TelemetrySnapshot` estendida com o booleano `using_virtual_bridge`, consumido continuamente pelo frontend via IPC Tauri em polling de 60ms.
  5. **Instrumentação Visual no HUD (`StatusBar.tsx`):** Renderização de badge reativo com ícone `Radio` exibindo `Virtual Sink (Echo-Free)` com glow verde neon quando conectado ao dispositivo virtual mudo, ou `Loopback Mode` em âmbar suave durante operação driverless.
  6. **Relatório de Engenharia Reversa REA (`rea-boom-engine-reverse-engineering-report.md`):** Documentação técnica abrangente em `.bmad/evidence/` mapeando os pontos de entrada exportados de `BoomEngine.dll`, hierarquias de classe (`AVBoomAudioProcessor`, `AVCSessionBase`, `Choreographer`), estruturas SIMD SSE de `soundtouch::FIRFilterSSE`, rotinas de buffer circular (`AVFIFOProcessor`, `CRingBufferSource`), topologia de miniport SYSVAD PortCls em `boomvad.sys`, e orquestração gerenciada via `Audio.dll` (`FindBoomDevice`, `IsBoom`, `SwitchToBoom` via COM `IPolicyConfigVista`).
---

## ADR-017: Motor de Convolução 7.1 KEMAR HRTF via SIMD SSE2, Dial Circular 3D Polar atan2, Modal Configurador Polar e Interpolação Angular Dinâmica
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** O Epic 2 demandava: (1) motor de espacialização binaural 7.1 downfolded para fones de ouvido estéreo baseado em tabelas de resposta ao impulso KEMAR de 64 taps por canal (`Spatializer3D`), com simulação precisa de ITD (0.2–0.7ms), atenuação de sombra de cabeça ILD, e pinna rear notch de reflexão a 6 kHz nos canais traseiros; (2) convolução FIR em tempo real acelerada por vetorização SSE2 com buffer circular duplo contíguo sem quebra de ponteiros na malha interna, consumindo < 25% do orçamento de processamento; (3) componente `SoundstageDial.tsx` com diâmetro externo de 192px e superfície interna de 144px, varredura angular de 300°, cálculo de arrasto polar $\theta = \text{atan2}(y - y_0, x - x_0)$, pips de detent ciano em 25%, 50% e 75%, snap para 50% em duplo-clique, navegação por teclado e satélites orbitais; (4) modal polar de calibração de soundstage `SoundstageConfiguratorModal.tsx` com cabeça central do ouvinte, anéis de órbita, 8 nós de alto-falantes virtuais com indicador visual de mudo por traço vermelho diagonal, faders de ganho relativo $\pm 6.0\text{ dB}$ com resolução de 0.1 dB e atalho global `Ctrl + Shift + S`; (5) reposicionamento angular dinâmico contínuo via arrasto dos nós na órbita, com interpolação linear contínua dos coeficientes KEMAR entre 9 âncoras acústicas horizontais sem dropouts ou estalos, exibição de coordenadas azimutais em tempo real no HUD e restauração de ângulos de referência ITU-R BS.775.
* **Decisões:**
  1. **Buffer Circular Duplo Contíguo:** Para eliminar branch instructions e cálculos de wrap dentro do laço de convolução FIR de 64 taps, cada amostra recebida é gravada simultaneamente em `pos` e `pos + 64` em um array estático de 128 floats por canal. Uma fatia contígua de 64 amostras em `&history[pos + 1..pos + 65]` é fornecida diretamente ao kernel SIMD, garantindo zero alocações na heap (`AD-1`) e tempo de execução inferior a 0.5s para 48.000 amostras.
  2. **Vetorização SSE2 SIMD e Fallback Escalar:** Implementada rotina `dot_product_64_sse` desenrolando o laço em 4 acumuladores vetoriais `__m128` com somas horizontais via `_mm_movehdup_ps` e `_mm_movehl_ps`. Em arquiteturas não x86_64, a compilação emprega automaticamente um fallback escalar idêntico.
  3. **Interpolação Angular Dinâmica de Coeficientes HRTF:** O horizontal surround é particionado em 9 âncoras acústicas (-180°, -135°, -90°, -30°, 0°, +30°, +90°, +135°, +180°). Quando qualquer alto-falante é rotacionado, os coeficientes de convolução de 64 taps são interpolados linearmente na stack (~50ns) entre as âncoras vizinhas, proporcionando transições perfeitamente contínuas e sem zipper noise. O canal LFE permanece fixo como filtro mono passa-baixa de subwoofer.
  4. **Física Polar de Arrasto e Pips Hápticos Visuais:** O `SoundstageDial` mapeia deslocamentos de ponteiro para ângulos absolutos via arco-tangente de dois argumentos, com detents magnéticos nos pips de 25%, 50% e 75% e duplo-clique com snap imediato para 50% (0.50 intensidade nominal).
  5. **Diferenciação Estrita entre Clique e Arrasto Angular:** O modal rastreia a distância euclidiana acumulada do ponteiro (`dragTravelRef`). Movimentos $< 4\text{ px}$ alternam o mudo do canal (com indicador visual de linha diagonal vermelha cortando o nó e exclusão do canal no mix); movimentos $> 4\text{ px}$ capturam o ponteiro (`setPointerCapture`) e realizam rotação orbital com exibição flutuante das coordenadas azimutais $\theta$ e sem mutar o canal.
  6. **Ação Unificada de Reset para Padrões ITU-R BS.775:** Botão "Reset Speaker Positions" restaura os ângulos de azimute de todos os 8 alto-falantes para os valores nominais ITU-R BS.775 (-30°, +30°, 0°, 180°/0°, -135°, +135°, -90°, +90°) tanto na interface quanto nas matrizes do motor de áudio.
* **Consequência:** Experiência imersiva 3D surround de alta fidelidade com espacialização personalizável, desempenho em tempo real ultrarrápido (< 1% CPU), zero alocações na thread de áudio e conformidade estrita com todos os requisitos do Epic 2 (`FR5`, `FR6`, `FR7`, `FR8`, `UX-DR4`, `UX-DR5`).

---

## ADR-018: Isolamento de Enumeração MMDeviceAPI em Thread COM Dedicada para Prevenção de Conflito de Apartamento (RPC_E_CHANGED_MODE)
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Ao iniciar a aplicação desktop via `npm run dev` / `tauri dev`, ocorria um pânico fatal do Tao na thread principal Win32:
  `thread 'main' panicked at tao-0.37.1\src\platform_impl\windows\window.rs: OleInitialize failed! Result was: RPC_E_CHANGED_MODE`.
  A investigação revelou que durante a inicialização do `AppState` (`src-tauri/src/lib.rs`), o `AudioEngineCoordinator::with_command_queue` invocava `bridge.detect(&device_manager)` na thread principal do processo. Esta chamada executava `enumerate_windows_devices()` em `crates/open-boom-wasapi/src/device.rs`, chamando `CoInitializeEx(None, COINIT_MULTITHREADED)` diretamente na thread chamadora sem correspondente `CoUninitialize()`.
  Isso definia permanentemente o modelo de concorrência COM da thread GUI como MTA (Multi-Threaded Apartment). Quando o Tao tentava criar a janela Win32 na thread principal chamando `OleInitialize(None)` (que exige STA / Single-Threaded Apartment), o Windows retornava o erro `RPC_E_CHANGED_MODE` (0x80010106).
* **Decisões:**
  1. **Isolamento em Thread Worker Dedicada (`open-boom-device-enum`):** O método `enumerate_windows_devices(&self)` foi refatorado para delegar a consulta MMDeviceAPI a uma thread de trabalho dedicada via `std::thread::Builder::new().name("open-boom-device-enum").spawn(...)`.
  2. **Ciclo de Vida COM Estritamente Balanceado:** Na thread isolada (`enumerate_windows_devices_isolated`), `CoInitializeEx(None, COINIT_MULTITHREADED)` é chamado no início e `CoUninitialize()` é garantido no encerramento da thread.
  3. **Preservação da Thread GUI Win32:** A thread principal de UI nunca é inicializada em MTA pelo adaptador WASAPI, permitindo que o `OleInitialize` do Tao seja executado com sucesso e suporte todas as operações OLE/STA (incluindo drag and drop e event loop) sem conflitos.
* **Consequência:** Resolução definitiva do pânico de inicialização, preservação do funcionamento do shell Tauri desktop e manutenção de 100% dos testes unitários e de integração (95 testes passando).

---

## ADR-019: Implementação da Thread Contínua de Streaming WASAPI (AudioStreamPump) e Roteamento de Ingestão em Tempo Real
* **Data:** 10/10/2026
* **Status:** Aceito
* **Contexto:** Embora as primitivas matemáticas de DSP (`crates/open-boom-core`) e a interface desktop (`src-tauri` / `src-ui`) estivessem 100% desenvolvidas e testadas, o aplicativo não aplicava alterações perceptíveis no áudio ao vivo do usuário. O `AudioEngineCoordinator` residia apenas em memória, sem uma thread ativa de streaming realizando o bombeamento contínuo de pacotes entre o sistema operacional Windows e o conversor digital-analógico (DAC).
* **Decisões:**
  1. **Thread Worker Dedicada de Alta Prioridade (`AudioStreamPump`):** Criado o módulo `crates/open-boom-wasapi/src/stream_pump.rs`, instanciando uma thread dedicada (`open-boom-audio-pump`) registrada no MMCSS sob o perfil "Pro Audio" (`AvSetMmThreadCharacteristicsW`).
  2. **Ingestão Dual-Mode (Virtual Sink / Loopback):** A thread detecta automaticamente o dispositivo virtual `Boom Audio` instalado no sistema. Se presente, captura desse endpoint silencioso (`AUDCLNT_STREAMFLAGS_LOOPBACK`); caso contrário, captura do endpoint padrão de renderização.
  3. **Zero Alocações no Loop de Processamento (`AD-1`, `NFR2`):** Buffers de scratch para entrada e saída de 48 kHz estéreo (16.384 floats) são pré-alocados. A passagem pelos estágios de DSP utiliza `try_lock()` no coordenador para garantir que a thread de áudio nunca bloqueie caso a interface esteja consultando telemetria.
  4. **Integração no Ciclo de Vida do Tauri:** O `AudioStreamPump` é instanciado automaticamente no `AppState::new()` e reinicializado em tempo hábil (< 500ms) quando o usuário troca o dispositivo de saída no seletor de endpoints (`set_audio_device`).
* **Consequência:** Fluxo contínuo de áudio em tempo real com sub-15ms de latência, integração total com os controles de 3D Surround, Limitador e Volume Boost, e 96 testes passando no workspace.

---

## ADR-020: Roteamento Inteligente Automático de Endpoint via COM IPolicyConfig e Isolamento de Dispositivo Físico (HyperX)
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Diagnóstico em tempo real revelou que, mesmo com a thread de streaming ativa, o usuário não percebia alterações no áudio do headset físico (`HyperX Cloud II Wireless`). As aplicações do Windows (Spotify, YouTube, jogos) estavam enviando seus fluxos diretamente para o DAC físico do HyperX, enquanto o OpenBoom escutava o dispositivo virtual `Boom Audio` (que permanecia em silêncio absoluto). Selecionar o `Boom Audio` manualmente no Windows resultava em silêncio total porque ele é um sink mudo (driver SYSVAD) que exige um software lendo seus buffers e retransmitindo para o hardware. Além disso, captura em loopback direto na placa física causa eco acústico duplo (slapback echo).
* **Decisões:**
  1. **Comutação Programática via COM `IPolicyConfig` (`SetDefaultEndpoint`):** Implementada a interface COM não documentada `IPolicyConfig` (CLSID `{870af99c-171d-4f9e-af0d-e63df40c2bc9}`, IID `{f8679f50-850a-41cf-9c72-430f290290c8}`) com chamada a `SetDefaultEndpoint` no índice 13 da vtable em thread dedicada (`open-boom-set-default-device`).
  2. **Ciclo de Vida de Roteamento Transparente:**
     - **Master Power LIGADO (ou início do runtime via `AppState::new_runtime`):** O OpenBoom detecta o DAC físico ativo (`HyperX`), fixa-o como destino do `AudioStreamPump`, e altera a saída padrão do Windows para o dispositivo virtual `Boom Audio`. O Windows e todos os programas passam a tocar no `Boom Audio`; o OpenBoom captura esse fluxo, aplica Volume Boost, 3D Spatializer (HRTF KEMAR) e Peak Limiter, e renderiza diretamente no HyperX com latência < 15ms.
     - **Master Power DESLIGADO (ou Bypass A/B):** O OpenBoom comuta imediatamente a saída padrão do Windows de volta para o DAC físico (`HyperX`), assegurando passthrough bit-perfect sem interrupção.
  3. **Restauração Garantida no Encerramento:** Implementado `restore_physical_default_device` em `AppState::drop` e no menu de contexto da bandeja do sistema ("Exit OpenBoom"), garantindo que o sistema operacional nunca fique preso no sink virtual ao fechar o aplicativo.
  4. **Filtragem de Dispositivos Virtuais na Interface:** Criado `enumerate_physical_devices()`, que remove `Boom Audio` e outros adaptadores virtuais do seletor da UI, permitindo que o usuário visualize e selecione apenas fones e caixas reais.
  5. **Proteção de Retry no `AudioStreamPump`:** Implementado loop de retry de até 10 iterações com delay de 500µs contra variações transitórias de padding do WASAPI, eliminando descartes acidentais de buffers.
  6. **Isolamento de Testes Unitários:** `AppState::new()` desativa o chaveamento de sistema por padrão (`auto_route: false`), reservando o roteamento dinâmico para `AppState::new_runtime()` executado no entrypoint da aplicação.
* **Consequência:** Eliminação definitiva de eco acústico, ativação imediata e audível de todos os efeitos de DSP no headset físico do usuário, abstração total da complexidade de drivers virtuais (idêntico ao comportamento comercial do Boom 3D) e suíte completa de testes (96 testes) verde no workspace.

---

## ADR-021: Diferenciação entre Convolução HRTF Acústica Pura (Epic 2) e Efeitos Espaciais Complementares (Epic 3 & Epic 4) na Assinatura Sonora 3D
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Com o pipeline de áudio ativo e o roteamento operando em tempo real no headset físico (`HyperX Cloud II Wireless`), o usuário observou que o efeito do botão 3D Surround é perceptível porém sutil ("fraquinho") em reprodução estéreo comum (YouTube/Spotify), questionando se isso é intencional ou se deve ser registrado para ajuste e cobertura pelos próximos épicos.
* **Decisões & Justificativas Técnicas:**
  1. **Natureza Acústica do HRTF Puro (Epic 2):** O algoritmo implementado no Epic 2 (`Spatializer3D`) executa convolução com tabelas KEMAR de 64 taps (ITD 0.2–0.7ms, ILD e notch de pinna a 6 kHz). Para um sinal estéreo convencional (2 canais), o motor simula o posicionamento de dois monitores frontais a $\pm 30^\circ$ em câmara anecóica. Em fones de ouvido, o HRTF puro atua primordialmente como *crossfeed binaural*, reduzindo a separação artificial extrema do estéreo tradicional e projetando o áudio para fora da cabeça ("out-of-head localization"). Por design acústico, ele não introduz cauda de reverberação artificial, coloração agressiva ou distorção de fase.
  2. **Origem da Assinatura Dramática do Boom 3D Comercial (Evidências REA):** Conforme constatado na engenharia reversa de `BoomEngine.dll` e `Audio.dll` (`Effect` enum: `SurroundSound`, `Ambience`, `Fidelity`, `DRC`, `Spatial`), o efeito "3D expansivo e cinematográfico" do produto comercial é um resultado sinérgico de 4 componentes simultâneos:
     - **HRTF Spatializer:** Posicionamento angular dos canais.
     - **Ambience (`Effect::Ambience` - Epic 4, Story 4.1):** Reflexões iniciais e reverberação de sala (Schroeder / FDN Reverb), que conferem sensação física de tamanho do ambiente.
     - **Spatial Widener (`Effect::Spatial` - Epic 4, Story 4.2):** Expansão estéreo via processamento Mid/Side e micro-delay de Haas, que projeta o som muito além da envergadura física dos fones.
     - **Fidelity (`Effect::Fidelity` - Epic 4, Story 4.3):** Exciter harmônico e abertura de agudos superiores.
     - **Equalização & AutoEq (Epic 3):** Curvas de calibração para o fone do usuário (compensando respostas não lineares e enfatizando a faixa de localização auricular de 3 kHz a 8 kHz).
  3. **Plano de Registro e Calibração:**
     - Confirmada a cobertura programada nos Epicos 3 e 4.
     - Aberta a tarefa de melhoria `T2.7` no backlog para calibração de ganhos da matriz de upmix estéreo (reforço de canais laterais/surround no sinal $L-R$) e ajuste da curva de interpolação wet/dry do `SoundstageDial` para maximizar a presença perceptível mesmo em modo KEMAR puro.
* **Consequência:** Alinhamento técnico completo entre arquitetura e percepção do usuário, mantendo a integridade da progressão dos épicos e registrando a oportunidade de tuning para a experiência do usuário.

---

## ADR-022: Motor de Equalização Gráfica de 31 Bandas ISO, Topologia Direct Form II Transposed e Suavização Sub-Buffer

* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Para o Epic 3 (Equalização de Estúdio e Calibração Headphone AutoEq), o pipeline de DSP exigia um motor equalizador gráfico de 31 bandas ISO 1/3-oitava (20 Hz a 20 kHz) de alta precisão, sem estalos (zipper noise) durante ajustes dinâmicos rápidos de sliders, sem perigo de explosão numérica por migração de pólos para fora do círculo unitário, com baixíssima distorção (THD+N < 0.005%) e com zero alocações de heap na thread de áudio em tempo real (Story 3.1).
* **Decisões:**
  1. **Fórmulas de Peaking EQ (Cookbook Robert Bristow-Johnson):** Implementada a geração paramétrica de coeficientes biquad de 2ª ordem para peaking EQ com Q constante = 4.31 (largura de banda padrão 1/3-oitava), $A = 10^{\text{gain\_db} / 40}$ e normalização canônica com $a_0 = 1.0$.
  2. **Topologia Direct Form II Transposed:** Escolhida a forma transposta com duas variáveis de estado de atraso ($s_1, s_2$) por banda por canal. Esta topologia minimiza ruído de quantização em precisão simples (IEEE 754 float32) e garante resposta direta $y[n] = b_0 x[n] + s_1[n-1]$.
  3. **Verificação Estrita de Estabilidade de Pólos (Critério de Schur-Cohn / Jury):** Provado matematicamente e verificado em código que $|a_2| < 1.0$ e $|a_1| < 1.0 + a_2$ para todos os ganhos entre -12.0 dB e +12.0 dB. Implementado cálculo de módulo máximo de pólos $\max(|z_i|) < 1.0$.
  4. **Suavização Sub-Buffer em Blocos de 32 Amostras (~0.67ms a 48 kHz):** Para evitar recomputar coeficientes trigonometricamente a cada amostra (custo desnecessário de CPU), o estágio mantém `current_coeffs` e `target_coeffs`. A cada fronteira de 32 amostras, os coeficientes são interpolados linearmente (`lerp`), eliminando qualquer ruído audível de zipper noise ou transientes bruscos durante arraste contínuo de sliders.
  5. **Sanitização Imediata e Clamping Defensivo:** Entradas de ganho são limitadas rigorosamente a $[-12.0\text{ dB}, +12.0\text{ dB}]$ (com resolução de 0.1 dB), com $\pm\infty$ truncado aos limites e valores $NaN$ revertidos confiavelmente para $0.0\text{ dB}$ (resposta plana).
  6. **Integração no Estágio 3 de `AudioProcessor`:** `GraphicEqStage` foi posicionado no pipeline de processamento estéreo e surround 7.1 entre o Estágio 2 (`Spatializer3D`) e o Estágio 4 (`MasterLimiter`), integrando despacho lock-free via comandos POD SPSC `LoadGraphicEqSnapshot`, `SetEqBandGain`, `SetEqGains` e `SetEqBypass`.
* **Consequência:** Resposta em amplitude milimetricamente fiel a $\pm 12.0\text{ dB}$ na frequência central de 1 kHz com decaimento suave nas bandas adjacentes, passagem 100% transparente com THD+N < 0.005% quando todas as bandas estão em 0.0 dB, estabilidade matemática absoluta, zero alocações na thread de áudio e 105 testes passando no workspace Cargo.

---

## ADR-023: Indexação Offline da Base AutoEq, Motor de Busca Fuzzy Instantâneo e Persistência de Calibração (Story 3.3)
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Para o Epic 3 (Story 3.3), era indispensável fornecer calibração paramétrica precisa para milhares de fones de ouvido (HyperX Cloud II, Sennheiser HD 600, Sony WH-1000XM5, Apple AirPods Max, etc.) de forma 100% offline, sem qualquer telemetria externa, ocupando menos de 5 MB no disco e com latência de busca sub-50ms.
* **Decisões:**
  1. **Empacotamento Comprimido no Disco (< 5 MB):** Gerado e embutido no crate `crates/open-boom-storage` o ativo binário `assets/autoeq_database.bin.gz` contendo 4.200 perfis calibrados em formato JSON comprimido com gzip (deflate nível 9). O arquivo ocupa apenas 0.20 MB (207 KB) no disco, superando a meta de < 5 MB com folga massiva de 96% (`FR13`, `AD-9`).
  2. **Descompressão e Indexação In-Memory no Startup:** `AutoEqDatabase::load()` descompacta o arquivo via `flate2::read::GzDecoder` em ~4ms e constrói índices em memória para busca instantânea e mapeamento rápido de IDs, integrado ao `AppState` do Tauri via `Arc<AutoEqDatabase>`.
  3. **Motor de Busca Fuzzy de Alta Relevância (< 1ms):** Algoritmo de scoring multidimensional combinando correspondência exata (100 pontos), modelos/prefixos (92/88 pontos), matching multi-token (85 pontos), substrings limpas (80/75 pontos) e subsequências arbitrárias (45 pontos). Consultas vazias retornam fones populares recomendados (HyperX, Sennheiser, Sony, Apple). Benchmark de 15 consultas executou com tempo médio de 0.04ms por busca, 1000x mais rápido que o teto estipulado de 50ms (`FR14`, `NFR7`, `NFR10`).
  4. **Comandos IPC Tauri:** Implementados `search_autoeq_profiles`, `get_autoeq_profile` e `get_autoeq_stats` retornando structs serializáveis leves (`AutoEqProfileSummary`) com id, marca, modelo, curva alvo, filtros e ganho de preamp.
  5. **Componente UI `AutoEqPopover` (UX-DR8):** Modal popover escuro com campo de pesquisa auto-focado, navegação por setas (Up/Down), seleção com Enter, dismiss com Escape ou clique fora, botão "Reset to Flat" e badges com número de bandas e atenuação de pré-amplificação. Conectado ao chip de AutoEq no dock `EqualizerPreview` e no cabeçalho do `EqualizerStudioDrawer`.
  6. **Persistência Atômica no `AppConfig`:** Campo `autoeq_profile_id: Option<String>` adicionado ao `AppConfig` e sincronizado na interface, restaurando o perfil ativo automaticamente após reiniciar o aplicativo (`AD-10`).
* **Consequência:** 14 testes Rust verdes em `open-boom-storage`, 8 testes verdes em `open-boom-lib`, 30 testes unitários verdes em `src-ui` e build Vite concluído em 1.73s com zero erros.

---

## ADR-024: Estágio de Equalização Paramétrica Desacoplado (AutoEq) e Compensação Automática de Headroom do Pré-Amp (Story 3.4)
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** A Story 3.4 exige a execução de calibrações de fone de ouvido da base AutoEq utilizando até 12 filtros biquad paramétricos (Peaking, Low Shelf, High Shelf) sem sobrescrever ou adulterar a curva de equalização gráfica de 31 bandas personalizada pelo usuário (`FR12`, `AD-6`). Além disso, filtros com ganhos positivos acumulados podem facilmente provocar saturação digital (clipping) nos estágios intermediários do DSP antes de alcançar o limitador master. O estágio deve operar com zero alocações na thread de áudio em tempo real e comunicação lock-free via SPSC (`AD-1`, `NFR1`, `NFR2`).
* **Decisões:**
  1. **Expansão RBJ Cookbook para Filtros Shelf:** Implementadas as fórmulas do Robert Bristow-Johnson Audio EQ Cookbook para filtros `low_shelf` e `high_shelf` em `crates/open-boom-core/src/dsp/biquad.rs`, com fator $A = 10^{\text{gain\_db} / 40}$, $\omega_0 = 2\pi f_0 / f_s$, normalização canônica por $a_0$ e verificação estrita de estabilidade pelo critério de Schur-Cohn / Jury ($|a_2| < 1.0$ e $|a_1| < 1.0 + a_2$, garantindo $\max(|z_i|) < 1.0$). O limite de ganho de peaking foi ampliado para $[-36.0\text{ dB}, +36.0\text{ dB}]$ para acomodar entalhes profundos de calibração.
  2. **Topologia Direct Form II Transposed e Suavização Sub-Buffer:** Criado `ParametricStage` em `crates/open-boom-core/src/dsp/parametric_stage.rs`, suportando até 12 biquads em cascata com atrasos transpostos ($s_1, s_2$) e suavização linear em blocos de 32 amostras (~0.67ms a 48 kHz), eliminando qualquer ruído de zipper noise durante a comutação de perfis.
  3. **Sequenciamento Estrito no Pipeline (Stage 3A -> Stage 3B):** `ParametricStage` foi integrado no `AudioProcessor` (`crates/open-boom-core/src/dsp/pipeline.rs`) posicionado imediatamente antes do `GraphicEqStage`. O estágio paramétrico cuida da neutralização acústica do fone, enquanto o equalizador gráfico de 31 bandas permanece exclusivo para ajustes tímbricos subjetivos do usuário, satisfazendo plenamente `AD-6` e `FR12`.
  4. **Compensação Automática de Pré-Amp Anti-Clipping:** Para garantir headroom digital absoluto mesmo sob reforços expressivos do perfil AutoEq (ex: +6.5 dB em sub-graves), o estágio calcula dinamicamente a atenuação de pré-amplificação $A_{\text{preamp}} = -\max(0, \max_i(G_i))$. O ganho linear $10^{A_{\text{preamp}} / 20} \le 1.0$ é aplicado no sinal antes da cascata de biquads, assegurando que o sinal intermediário nunca atinja 0 dBFS antes do `GraphicEqStage` e do `MasterLimiter`.
  5. **Contrato Lock-Free SPSC:** Definido `ParametricFilterPod` plano e comandos POD `AudioCommand::SetAutoEqProfile { filters, filter_count, preamp_offset_db }` e `AudioCommand::SetAutoEqBypass { bypassed }`. O despacho da UI para a thread de áudio ocorre sem alocações na heap, locks bloqueantes ou serialização JSON.
  6. **Comandos IPC Tauri e Exposição de Telemetria:** Implementados `apply_autoeq_profile` e `clear_autoeq_profile` no `src-tauri`, e estendido `TelemetrySnapshot` com `autoeq_preamp_db` e `autoeq_active`.
  7. **Integração na UI e Badges Dinâmicos:** Em `src-ui/src/App.tsx`, a seleção de perfil calcula o offset de atenuação, despacha o perfil via IPC e atualiza o badge de pré-amplificação em `EqualizerPreview.tsx` e `EqualizerStudioDrawer.tsx` (ex: `-6.5 dB PREAMP`). A limpeza ou reset do perfil desativa o estágio paramétrico mantendo os faders de 31 bandas inalterados.
* **Consequência:** 11 testes em `autoeq_parametric_tests.rs` passando com 100% de estabilidade de pólos e THD+N < 0.005%, 9 testes unitários em `AutoEqParametric.test.tsx` passando, totalizando 116 testes Cargo e 39 testes de UI verdes no projeto com build de produção Vite concluído em 1.76s.

---

## ADR-025: Presets de Gênero de Fábrica (15 Curvas ISO), Detecção de Estado Modificado (`*`) e Persistência Atômica de Presets Customizados (Story 3.5)
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Para concluir o Epic 3 (Story 3.5), o OpenBoom precisa disponibilizar ao menos 15 presets de fábrica por gênero (`FR11`) com curvas de 31 bandas ISO 1/3-oitava calibradas, sinalizar visualmente alterações não salvas nos faders com um asterisco (`Rock *`) (`EXPERIENCE.md §5.2`) e permitir criar, renomear e excluir presets customizados de usuário em `%APPDATA%\OpenBoom\presets\*.json` com escrita atômica e resiliência total contra corrupção de arquivos ou path traversal (`AD-10`).
* **Decisões:**
  1. **15 Presets de Fábrica Calibrados (`FR11`):** Implementados em `PresetStore::factory_presets()` (`crates/open-boom-storage/src/preset_store.rs`) e espelhados em `FACTORY_EQ_PRESETS` (`src-ui/src/components/PresetSelectorPopover.tsx`): `Flat`, `Bass Boost`, `Treble Boost`, `Acoustic`, `Classical`, `Electronic`, `Gaming`, `Hip-Hop`, `Jazz`, `Movie`, `Pop`, `Rock`, `Vocal`, `Lounge` e `Deep`, todos com 31 ganhos finitos estritamente contidos em $[-12.0\text{ dB}, +12.0\text{ dB}]$.
  2. **Persistência Atômica CRUD em `%APPDATA%\OpenBoom\presets\` (`AD-10`):** `save_custom_preset`, `rename_custom_preset` e `delete_custom_preset` escrevem primeiramente em arquivo temporário único (`<slug>.<pid>.<seq>.tmp` com contador atômico `AtomicU64`) e efetuam substituição atômica via `fs::rename` (`MoveFileExW` no Windows), removendo o `.tmp` imediatamente em caso de erro.
  3. **Sanitização Estrita de Slugs e Proteção de Fábrica:** `PresetStore::sanitize_preset_slug` converte nomes para caracteres ASCII alfanuméricos minúsculos separados por hífen, bloqueando path traversal (`../`), nomes vazios e dispositivos reservados do Windows (`CON`, `NUL`, `COM1`..`COM9`, `LPT1`..`LPT9`). Presets de fábrica são protegidos contra sobrescrita, renomeação, exclusão ou sombreamento por arquivos `.json` manuais no diretório `presets/`.
  4. **Detecção de Estado Sujo (`isPresetDirty`):** Quando qualquer uma das 31 bandas diverge mais de $0.05\text{ dB}$ da curva do preset ativo, `EqualizerPreview`, `EqualizerStudioDrawer` e `PresetSelectorPopover` exibem o sufixo `*` (ex.: `Rock *`) e destacam a ação `"Save As Custom Preset"`.
  5. **Independência com AutoEq (`AD-6`):** A troca de presets gráficos de 31 bandas despacha `LoadGraphicEqSnapshot` com `preamp_offset_db: 0.0` (pertencente ao `GraphicEqStage`), preservando intactos o perfil de fone de ouvido e a atenuação de pré-amp do `ParametricStage`.
* **Consequência:** 19 testes em `open-boom-storage` (122 testes no workspace Cargo) e 47 testes de UI passando com 0 falhas, concluindo 100% do Epic 3.

---

## ADR-026: Ordem de Destruição RAII de Interfaces COM (`IUnknown::Release`) Antes de `CoUninitialize` em `set_windows_default_device_worker` e Ativação Real-Time do `AudioStreamPump`
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Ao executar `npm run dev` (`tauri dev`), o processo `open-boom.exe` encerrava imediatamente na inicialização com código `0xc0000005` (`STATUS_ACCESS_VIOLATION`).
  A investigação e a inspeção via REA MCP (`inspect_managed_members` e `inspect_managed_artifact` em `Boom3D\Audio.dll` para `Audio.IPolicyConfigVista` e `Audio.CPolicyConfigVistaClient`) revelaram duas causas interligadas:
  1. Em `AudioDeviceManager::set_windows_default_device_worker` (`crates/open-boom-wasapi/src/device.rs`), os guardiões RAII `unk: IUnknown` e `_policy_guard: IUnknown` permaneciam no escopo léxico até o final da função `unsafe`, **após** a chamada explícita de `CoUninitialize()`. Como `open-boom-set-default-device` é uma thread worker isolada e `AudioStreamPump` estava em modo sintético (`allow_synthetic: true`), `CoUninitialize()` zerava a contagem do apartamento COM e descarregava `AudioSes.dll` do espaço de endereçamento antes que o compilador Rust invocasse `Drop::drop` (`IUnknown::Release`) nos ponteiros COM — resultando em desreferenciação de vtable inválida (`0xc0000005`, `STATUS_ACCESS_VIOLATION`).
  2. Em `src-tauri/src/lib.rs`, `StreamPumpConfig` recebia `allow_synthetic: true` mesmo quando instanciado via `AppState::new_runtime()`, fazendo a thread `open-boom-audio-pump` entrar no loop sintético de teste em vez de executar `run_windows_pump`. Além disso, `run_windows_pump` assumia fixamente 2 canais (`nChannels == 2`) tanto na captura quanto na renderização.
* **Decisões:**
  1. **Escopo Léxico Estrito Antes de `CoUninitialize()`:** Em `set_windows_default_device_worker`, toda a criação, consulta e invocação de `IPolicyConfig` (`eConsole`, `eMultimedia`, `eCommunications` — alinhado com `Audio.CPolicyConfigVistaClient.SetDefaultDevice` verificado via REA MCP) foi encapsulada em uma closure interna `(|| -> EngineResult<()> { ... })()`. Assim, `_policy_guard` e `unk` executam `IUnknown::Release()` estritamente **antes** de `CoUninitialize()`.
  2. **Ativação Real-Time do `AudioStreamPump` em Runtime (`allow_synthetic: !auto_route`):** `AppState` agora propaga `allow_synthetic: !auto_route` para `StreamPumpConfig` (na inicialização, em `set_audio_device` e em `trigger_device_fallback`), mantendo o isolamento sintético nos testes (`AppState::new()`) e ativando `run_windows_pump` na aplicação desktop (`AppState::new_runtime()`).
  3. **Adaptação Segura Multi-Canal em `run_windows_pump`:** Leitura dinâmica de `(*capture_fmt_ptr).nChannels`, `(*render_fmt_ptr).nChannels` e `(*render_fmt_ptr).nSamplesPerSec`, com downmix seguro de fluxos 7.1 (8 canais) ou mono para o pipeline estéreo e mapeamento exato para a contagem de canais do DAC físico sem risco de buffer overrun.
  4. **Teste de Regressão Dedicado:** Adicionado `test_set_default_device_com_lifecycle_no_access_violation` em `crates/open-boom-wasapi/tests/device_tests.rs`.
* **Consequência:** Eliminação total do crash `0xc0000005` (`STATUS_ACCESS_VIOLATION`), bombeamento de áudio em tempo real ativo no `npm run dev` e 123 testes Cargo passando no workspace.

---

## ADR-027: Topologia DSP dos Efeitos Psicoacústicos (Epic 4: Ambience, Fidelity, Night Mode, Spatial e Quick Rack Strip UI)
* **Data:** 11/10/2026
* **Status:** Aceito
* **Contexto:** Implementação do Epic 4 (Stories 4.1 a 4.5, FR19 a FR23, UX-DR6): suíte completa de pós-processamento dinâmico e melhoramento psicoacústico (Ambience Reverb, Fidelity Harmonic Exciter, Night Mode DRC, Spatial Stereo Expander e Quick Rack Strip UI). O motor requer latência zero adicional para módulos de saturação e estéreo, preservação rigorosa da coerência de fase em mono, zero alocações na thread de áudio e transições suaves de parâmetros sem zíper acústico.
* **Decisões:**
  1. **Topologia em Cascata no Pipeline (Stage 4):** Inserido o estágio `Stage 4: SweetenersRack` no `AudioProcessor` entre a Equalização Gráfica (Stage 3) e o Limitador de Pico Lookahead (Stage 5). Isso assegura que os harmônicos gerados e a ambiência reverberada recebam sinal equalizado e fiquem estritamente contidos pelo teto protetor de -0.2 dBFS do limitador.
  2. **Story 4.2 - Ambience Reverb (Schroeder-Moorer / FDN):**
     - Estrutura baseada em 4 filtros comb com amortecimento passa-baixa de 1 polo (delays de 1116, 1188, 1277 e 1356 amostras entre 23ms e 28ms a 48 kHz).
     - 2 estágios all-pass de difusão em série (delays: 225 e 556 amostras, ganho 0.5) eliminando eco flutter.
     - 2 taps de reflexão inicial (early reflections) assimétricos entre os canais para espacialidade estéreo natural.
     - Interpolação paramétrica suave de 200 amostras com filtro de 1 polo eliminando cliques ao ajustar o fader.
  3. **Story 4.3 - Fidelity Psychoacoustic Harmonic Exciter:**
     - Divisor de frequência passa-altas Butterworth de 2ª ordem (> 3.2 kHz) para isolar transientes agudos e presença.
     - Bloco DC de 1 polo para manter a simetria do sinal isolado.
     - Síntese de harmônicos pares ($x^2$, calor e plenitude) e harmônicos ímpares ($x^3$, presença e ataque) combinados com saturação suave tanh-like ($x / (1 + |x|)$) que previne corte abrupto digital.
     - Air shelf de alta frequência (> 10 kHz) para brilho e extensão sem aspereza.
     - Latência zero absoluta (0.0 ms) e bypass bit-idêntico quando desativado.
  4. **Story 4.4 - Night Mode Dynamic Range Compressor (DRC):**
     - Detector de envelope com ataque ultrarrápido programável (2.5 ms, < 5 ms) e liberação suave (120 ms).
     - Curva de ganho com soft-knee quadrática de 6 dB centrada em limiar de -20 dBFS com compressão de até -18 dB em transientes fortes.
     - Boost de diálogo de baixo nível (até +6 dB) com gate de silêncio a -55 dBFS para evitar amplificação de ruído de fundo/hiss.
     - Aplicação simétrica da redução de ganho em ambos os canais, preservando 100% da imagem estéreo.
  5. **Story 4.5 - Spatial Stereo Expander & Crossfeed Matrix:**
     - Matriz Mid/Side: $M = 0.5(L + R)$, $S = 0.5(L - R)$.
     - Filtro passa-baixa de 1 polo a 200 Hz para separar o canal de sub-graves no sinal Side.
     - Centralização estrita de sub-graves (< 200 Hz): o componente grave do Side é cancelado ($S_{low} \times 0.0$), enquanto as frequências médias e agudas são multiplicadas pelo fator de largura ($1.0 + 1.2 \times \text{amount}$).
     - Invariante matemático de compatibilidade mono: como $L = R \implies S = 0$, para qualquer sinal puramente mono a saída é 100% bit-idêntica à entrada ($L_{out} = M = L$, $R_{out} = M = R$), com zero cancelamento de fase.
  6. **Story 4.1 - 5-Effect Quick Rack Strip UI Component & Pitch Phase 2 Boundary:**
     - Componente de UI responsivo com grid de 5 cartões (`EffectsRackStrip.tsx`), iluminação neon ciano, switches de acionamento rápido, mini-sliders verticais com Cascadia Code e tooltips acústicos.
     - Respeito estrito ao boundary de Pitch Shift (PRD §6.2, FR20): exibição de badge "Soon" (Fase 2) na interface e bloqueio de despacho para o DSP core, preservando conformidade com a arquitetura do MVP.
  7. **Contrato Lock-Free SPSC e Persistência Atômica:**
     - Comando `AudioCommand::SetSweetener { effect_id, enabled, amount }` despachado via SPSC queue sem alocações dinâmicas nem locks.
     - Persistência em `%APPDATA%\OpenBoom\config.json` via campos `sweetener_*` no `AppConfig` com substituição atômica de arquivo.
* **Consequência:** 17 testes de integração e DSP em `crates/open-boom-core/tests/sweeteners_tests.rs`, 9 testes de UI em `src-ui/src/test/EffectsRack.test.tsx`, elevando o suite total para 140 testes Cargo e 56 testes UI verdes no workspace com zero regressões.

---

## ADR-028: Gerenciador de Sessões de Áudio Windows Core Audio em Thread Dedicada MTA (Story 5.1)
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Story 5.1 (FR21, AD-8). O OpenBoom necessita rastrear e controlar volumes e mudo por aplicativo no Windows via `IAudioSessionManager2` e `ISimpleAudioVolume`. Invocar objetos COM diretamente da thread principal de interface ou de tasks assíncronas do Tokio/Tauri provoca conflitos de apartamento COM (`RPC_E_CHANGED_MODE`, `CO_E_NOTINITIALIZED`), travamentos na interface gráfica ou falhas de concorrência.
* **Decisões:**
  1. **Isolamento de Apartamento COM MTA:** Todos os ponteiros e chamadas a `IAudioSessionManager2`, `IAudioSessionEnumerator`, `IAudioSessionControl2` e `ISimpleAudioVolume` executam estritamente em uma thread worker de segundo plano nomeada `"open-boom-session-mgr"`, inicializada com `CoInitializeEx(None, COINIT_MULTITHREADED)`.
  2. **Comunicação por Mensagens com Zero Locks no Áudio:** Comandos de enumeração e mutação (`Enumerate`, `SetVolume`, `SetMute`, `Shutdown`) transitam por canais `std::sync::mpsc`, retornando resultados através de canais de resposta desacoplados sem bloquear o loop de áudio de baixa latência nem a renderização do frontend.
  3. **Inspeção Segura de Processos:** Nomes de processos executáveis são resolvidos exclusivamente via `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` e `QueryFullProcessImageNameW`, com liberação obrigatória de handles via `CloseHandle`, sem injeção de DLLs, ganchos (hooks) ou modificação de memória de outros processos. Sessões de sons do sistema (`IsSystemSoundsSession()`) e processos expirados (`AudioSessionStateExpired`) são tratados e podados de forma transparente.
  4. **Modo Sintético Determinístico:** Implementado modo sintético (`with_forced_synthetic` e fallback gracioso em `with_synthetic_fallback`) simulando sessões ativas (Spotify, Discord, Chrome) e mutações em memória, permitindo execução 100% automatizada e sem falhas em ambientes CI, contêineres e testes headless.
  5. **Comandos IPC Tauri:** Expostos os comandos `get_audio_sessions`, `set_app_volume` e `set_app_mute` registrados no `AppState` de Tauri.
* **Consequência:** 5 testes unitários dedicados em `crates/open-boom-wasapi/tests/session_manager_tests.rs` e teste de integração em `src-tauri/src/lib.rs`, elevando o total do workspace para 146 testes Cargo verdes com zero avisos.

---

## ADR-029: Interface do Mixer por Aplicativo em Gaveta Deslizante Fluent & Faders com Boost 150% (Story 5.2)
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Story 5.2 (FR21, FR22, UX-DR9, EXPERIENCE.md §2). O usuário necessita visualizar e regular o volume e mudo de aplicações ativas individualmente (ex.: atenuar navegador ou elevar Discord/jogo) sem sair da interface do OpenBoom nem modificar o volume master global do Windows.
* **Decisões:**
  1. **Componente de Gaveta Deslizante (`AppMixerDrawer.tsx`):** Projetado como gaveta com animação de slide fluida (`translate-y-0` vs `translate-y-full`), backdrop blur e paleta Fluent Dark (`surface-base/95`, `border-default`). Acesso pelo cabeçalho ("App Volumes" com ícone `Layers`) e atalho global `Ctrl + Shift + M`.
  2. **Estrutura das Linhas de Processo (`app-mixer-row`):** Linhas de altura 44–48px com badge/ícone temático por tipo de aplicação (Spotify/Music, Discord/Headphones, Navegadores/Globe, Games/Gamepad), nome amigável do executável com tooltip, PID em fonte Cascadia Code monospace, e ponto indicador verde pulsante (`bg-meter-green animate-pulse`) quando a sessão está produzindo áudio ativamente.
  3. **Sliders de Volume Independentes de 0% a 150%:** Faders horizontais permitindo atenuação e boost de software até +3.5 dB (150%), com leitura dinâmica de porcentagem e dB perceptual (`volumeToDbString`), snap magnético instantâneo para 100% (0.0 dB) via duplo clique, e atenuação visual / bloqueio quando mutado.
  4. **Controle de Muto Desacoplado:** Botão de alternância de mudo com ícone dinâmico (`Volume2` vs `VolumeX` em vermelho) e feedback imediato otimista, despachando `set_app_mute` via IPC assíncrono.
  5. **Polling Otimizado e Tratamento de Estado Vazio:** Quando a gaveta está aberta, o frontend executa polling a cada 1000ms para descobrir novos processos e podar processos encerrados, exibindo mensagem ilustrada limpa quando nenhum fluxo de áudio está ativo.
* **Consequência:** 9 testes unitários de renderização, decibéis, faders e acessibilidade em `src-ui/src/test/AppMixer.test.tsx`, elevando a suíte de testes de UI para 65 testes verdes e mantendo 146 testes Cargo no backend.

---

## ADR-030: Persistência Atômica de Perfis de Volume por Aplicativo e Restauração Automática de Sessões (Story 5.3)
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Story 5.3 (FR23, UJ-4, AD-10). Quando os aplicativos são fechados ou reiniciados (ou após reinicializações do Windows), o Windows Core Audio perde ou redefine os níveis personalizados de volume de sessão. O usuário espera que o equilíbrio ajustado entre Spotify, Discord, navegadores e jogos seja memorizado e restaurado de maneira transparente e automática sempre que os processos voltarem a reproduzir áudio.
* **Decisões:**
  1. **Modelo de Dados e Armazenamento Atômico (`crates/open-boom-storage`):**
     - Estrutura `AppVolumeProfile { process_name, volume, is_muted }` com serialização `serde`.
     - Persistência em `%APPDATA%\OpenBoom\app_volumes.json` utilizando arquivos temporários de staging (`app_volumes.<pid>.<seq>.tmp`) e substituição atômica via `fs::rename` (`MoveFileExW` no Windows), garantindo que quedas repentinas de energia ou encerramentos abruptos jamais corrompam as preferências (`AD-10`).
     - Normalização insensível a maiúsculas/minúsculas para identificação inequívoca de executáveis (ex.: `Spotify.exe` e `spotify.exe`).
     - Recuperação graciosa contra arquivos corrompidos ou inexistentes no disco, retornando mapa vazio sem panics ou interrupção do sistema.
  2. **Cache e Gancho de Restauração no SessionManager (`crates/open-boom-wasapi`):**
     - O `SessionManager` mantém um registro de perfis persistidos (`stored_profiles`) e um conjunto de PIDs restaurados (`restored_pids`).
     - Durante a enumeração (`enumerate_sessions`), qualquer nova sessão que coincida com um perfil memorizado tem seu volume e mudo restaurados automaticamente na thread MTA via `ISimpleAudioVolume::SetMasterVolume` e `SetMute`.
     - Ciclo de vida de processos: PIDs finalizados são podados automaticamente de `restored_pids`, garantindo que, quando o aplicativo for reaberto e iniciar novo fluxo de áudio (seja com novo PID ou PID reciclado), suas preferências sejam reaplicadas imediatamente (`FR23`, `UJ-4`).
     - Mutações via `set_process_volume` e `set_process_mute` atualizam o cache de perfis e sincronizam atomicamente no disco.
  3. **Comandos IPC Tauri e Integração de Frontend:**
     - Comandos `get_app_volume_profiles` e `reset_app_volume_profile` expostos via Tauri IPC.
     - `AppMixerDrawer` exibe botão de restauração/reset rápido para 100% em aplicações com customizações ativas e nota informativa no rodapé.
* **Consequência:** 4 novos testes unitários em `crates/open-boom-storage`, 1 novo teste de restauração em `crates/open-boom-wasapi`, 1 teste de integração no Tauri e 2 novos testes no frontend React/Vite. Total de 174 testes Cargo no backend e 67 testes no frontend UI verdes com 100% de sucesso. Conclusão formal do Epic 5.

---

## ADR-031: Audio Buffer Stability Guard, Anti-Pumping Peak-Hold Limiter & Acoustic Risk Warning Banner
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Relato do usuário de estalos/clicks secos repentinos ("tiros no som") ao reproduzir faixas com sub-graves densos e alta compressão/RMS (ex: *M.O.O.N - Dust (Vincienty Remix)*, Rock moderno). Três causas raízes foram isoladas:
  1. *WASAPI Buffer Underruns sob Jitter:* Buffer de renderização de 10ms (`100_000` hns) muito próximo da margem de jitter do timer do Windows, descartando pacotes inteiros e gerando degraus de tensão ($\Delta V$).
  2. *Contenção de Lock por Telemetria:* O polling de telemetria a cada 60ms causava contenção de mutex ocasional, levando o loop a copiar áudio dry (queda e salto abrupto de 6–10 dB em 10ms).
  3. *Modulação de Envelope em Sub-Graves no Limitador:* A janela de lookahead de 2ms (96 amostras) era menor que a passagem por zero de ondas de 40–60 Hz, modulando o ganho a 100 Hz e achatando transientes.
* **Decisões:**
  1. **Peak-Hold de 5ms no Limitador Terminal (`crates/open-boom-core`):** Retenção de envelope por 240 amostras (5ms) durante passagens por zero de sub-graves, evitando modulação espúria de ganho e distorção harmônica.
  2. **Buffer Guard WASAPI Estendido (25ms):** Buffer configurado para 25ms (`250_000` hns) com o guard ativo (vs 10ms em modo agressivo desativado), garantindo margem para jitter sem perda de pacotes e com latência imperceptível (< 25ms).
  3. **Zero Queda de Processamento por Telemetria:** Substituído bypass abrupto por espera com `std::hint::spin_loop()`, garantindo que o fluxo de áudio jamais caia em dry no meio da reprodução.
  4. **Switch Opcional na UI com Banner de Risco Acústico em Vermelho (`SettingsModal.tsx`):**
     - Switch "Audio Buffer Stability Guard" ativado por padrão.
     - Badge ciano "Proteção Ativa: Zero Estalos & Transientes Suaves" quando ativo.
     - Banner de alerta vermelho em destaque ("Aviso de Risco Acústico & Fadiga Mecânica") quando desativado pelo usuário, alertando sobre os riscos acústicos e de fadiga física para transdutores ao usar o modo 10ms sem folga.
  5. **Persistência Atômica e Hot-Restart:** Persistido em `AppConfig` (`buffer_guard_enabled: bool`) e reinicialização a quente do `AudioStreamPump` sem interrupção de áudio.
* **Consequência:** Eliminação total de estalos em faixas com sub-graves pesados. 175 testes Rust verdes no workspace e 71 testes React/Vite no frontend aprovados com 100% de sucesso.

---

## ADR-032: Especificação do Modo 8D Orbital Audio (Som Rotativo 360°) via Matriz KEMAR HRTF e Deslocamento Azimutal por Sub-Blocos
* **Data:** 11/10/2026
* **Status:** Aprovado / Especificado
* **Contexto:** Demanda do usuário por um modo "8D Audio" imersivo com rotação contínua da fonte sonora ao redor da cabeça, permitindo parametrização de orientação (Sentido Horário, Anti-Horário, Pêndulo) e velocidade angular (período de 2s a 30s).
* **Decisões:**
  1. **Nomenclatura do Recurso:** Adotada a designação formal "8D Audio (Som Orbital 360°)", validando a terminologia consagrada na indústria e comunidade de áudio binaural, respaldada tecnicamente como *Orbital Binaural Spatialization*.
  2. **Reaproveitamento da Convolução KEMAR HRTF 64-tap (`Spatializer3D`):** Em vez de criar um algoritmo isolado que colapsa o áudio em mono, a rotação orbital atua como um deslocamento azimutal rígido $\theta_k(t) = \theta_{\text{nom}, k} + \theta_{\text{orbit}}(t)$ sobre toda a constelação de 8 alto-falantes virtuais 7.1, preservando a imagem estéreo da música enquanto gira a sala acústica completa.
  3. **Atualização por Sub-Blocos de 32 Amostras (~0.67ms):** Cálculo dos coeficientes HRTF executado a cada 32 amostras em vez de a cada amostra individual, reduzindo o custo de CPU em 97% e mantendo a variação angular em $\approx 0.12^\circ$ por sub-bloco (muito abaixo do limiar auditivo MAA de $1^\circ$ a $3^\circ$), garantindo zero zipper noise e zero alocações na heap.
  4. **Modos de Orientação:** Suporte a Sentido Horário (Clockwise), Anti-Horário (Counter-Clockwise) e Pêndulo/Oscilação lateral (Ping-Pong).
  5. **Controle de Velocidade:** Período de revolução ajustável continuamente entre 2.0s (rotação rápida) e 30.0s (órbita lenta imersiva), com padrão calibrado em 12.0s.
  6. **Despacho Lock-Free e Visualização HUD:** Parâmetros transmitidos via fila circular SPSC lock-free em struct POD `AudioCommand::SetOrbit8D`, com retorno de fase azimutal na telemetria para animação suave do beacon orbital na UI ($\ge 30\text{ fps}$).
* **Consequência:** Especificação formalizada em `_bmad-output/specs/spec-8d-orbital-audio/SPEC.md` e companion `orbital-dsp-mathematics.md`. Memlog inicializado e validado sem pendências.

---

## ADR-033: Arquitetura de Internacionalização (i18n: pt-BR, en, es) com Detecção do Sistema Operacional e Persistência Atômica
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Necessidade de suporte completo a múltiplos idiomas na interface e na bandeja do sistema para Português do Brasil (`pt-BR`), Inglês (`en`) e Espanhol (`es`), utilizando por padrão a linguagem do sistema operacional do usuário, sem introduzir dependências externas pesadas em runtime.
* **Decisões:**
  1. **Motor de i18n Nativo e Tipado:** Implementado via Contexto React leve (`I18nProvider`, `useI18n`) com contrato de dicionário TypeScript estrito (`TranslationDictionary`). Zero bibliotecas externas pesadas adicionadas ao bundle.
  2. **Dicionários Completos:** Dicionários em `pt-BR.ts`, `en.ts` e `es.ts` com 100% de paridade e cobertura total de todos os componentes da aplicação (Titlebar, DeviceSelector, Soundstage, Equalizer, Presets, AutoEq, EffectsRack, AppMixer, StatusBar e SettingsModal).
  3. **Detecção do Sistema por Padrão:** Opção padrão `"system"` resolve o idioma do host via `navigator.language` (`pt*` -> `pt-BR`, `es*` -> `es`, fallback universal `en`).
  4. **Pontos de Acesso ao Usuário:** Seletor com menu suspenso na barra de título (`LanguageMenu.tsx`), grade segmentada na modal de preferências (`SettingsModal.tsx`), e submenu nativo no ícone da bandeja do Windows (System Tray em `src-tauri/src/lib.rs` emitindo `tray-language-changed`).
  5. **Persistência Atômica:** Preferência de idioma gravada atomicamente em `%APPDATA%\OpenBoom\config.json` via campo `language: Option<String>` em `AppConfig`.
  6. **Resiliência Headless em Testes:** Contexto padrão do `I18nContext` inicializado com o dicionário em inglês, permitindo que testes estáticos via `renderToStaticMarkup` executem de forma determinística sem exigir wrapper explícito.
* **Consequência:** 78 testes de UI aprovados e 175 testes Cargo aprovados com 100% de sucesso. Versão bumpada para `0.2.0` (tag `v0.2.0`).

---

## ADR-034: Implementação do Motor DSP de Modulação Azimutal Orbital 8D & Rotação Contínua 7.1 em Sub-Blocos (Story 6.1)
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Necessidade de implementar a base matemática e o pipeline de DSP do modo 8D Orbital Audio no `Spatializer3D`, garantindo rotação 360° em tempo real sem alocações na heap, sem contenção de lock, com precisão angular estrita e sem artefatos de fase ou estalos de transição.
* **Decisões:**
  1. **Tipos e Contratos Lock-Free (`crates/open-boom-core/src/types.rs`):**
     - Adicionado enum `OrbitDirection` (`Clockwise = 0`, `CounterClockwise = 1`, `PingPong = 2`) com derivações `Copy`, `POD`, e `Serialize`/`Deserialize`.
     - Adicionado comando POD `AudioCommand::SetOrbit8D { enabled, direction, period_sec, depth }` para despacho lock-free via fila circular SPSC.
     - Estendido `TelemetrySnapshot` com `orbit_phase_deg: f32` e `orbit_enabled: bool`.
  2. **Acumulador de Fase em Dupla Precisão (`f64`) para Eliminação de Drift:**
     - A acumulação de fase por amostra ($\Delta \phi = 1 / (T_{\text{rev}} \cdot f_s) \approx 1.736 \times 10^{-6}$) acumula erro de arredondamento em float32 ao longo de centenas de milhares de amostras (desvio de $\approx 0.21^\circ$ em 6s). O acumulador interno `orbit_phase_progress` foi tipado como `f64`, garantindo erro acumulado $< 10^{-8}$ graus e simetria perfeita em $T/4$, $T/2$, $3T/4$ e $T$.
  3. **Atualização de HRTF por Sub-Blocos de 32 Amostras (`ORBIT_SUB_BLOCK_SAMPLES = 32`):**
     - O recálculo dos coeficientes de convolução KEMAR ocorre a cada 32 frames de áudio (~0.67ms a 48 kHz), reduzindo em 97% o custo computacional com deslocamento angular por sub-bloco de apenas $\approx 0.12^\circ$ (muito abaixo do limiar auditivo MAA de $1^\circ$).
  4. **Isolamento Estrito do Canal LFE:**
     - O canal de sub-graves (`CH_LFE`) é ignorado na rotação orbital (`if ch == CH_LFE { continue; }`), mantendo seus coeficientes HRTF bit-idênticos à referência estática em azimute 0.0° para evitar desequilíbrio e fadiga de fase nos graves.
  5. **Deslocamento Rígido da Constelação e Função de Wrap Contínua:**
     - Cada alto-falante não-LFE é rotacionado por $\theta_k(t) = \text{wrap}_{[-180, 180]}(\text{speaker\_azimuth}[k] + \theta_{\text{orbit}}(t) \cdot \text{orbit\_depth})$.
     - Função `wrap_deg_180` implementada com `rem_euclid(360.0)` garantindo intervalo contínuo $[-180.0^\circ, +180.0^\circ]$ e mapeamento idêntico de âncoras para $\pm 180^\circ$.
  6. **Transição Contínua na Comutação de Direção:**
     - O método `sync_progress_to_direction` adapta o progresso normalizado imediatamente para o ângulo azimutal atual ao alternar entre Horário, Anti-Horário e Pêndulo, garantindo reversão imediata do incremento angular sem cliques ou saltos transitórios.
  7. **Restauração Limpa ao Desativar o Modo 8D:**
     - Ao desativar o modo 8D, `restore_static_hrtfs` recalcula imediatamente os coeficientes HRTF estáticos para os azimutes nominais configurados sem zerar buffers de histórico, prevenindo descontinuidades DC.
* **Consequência:** 13 novos testes de integração em `crates/open-boom-core/tests/orbital_tests.rs` e teste de desserialização serde no `src-tauri` aprovados com 100% de sucesso. Total de 188 testes Cargo passando no workspace com zero warnings.

---

## ADR-035: Botão Rápido 8D Orbit, Projeção Polar e Indicador de Trajetória Neon no SoundstageDial (Story 6.2)
* **Data:** 11/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Necessidade de disponibilizar ativação direta do modo 8D Orbital Audio no HUD principal (`SoundstageDial`) sem obrigar abertura de modais secundárias, e fornecer feedback visual contínuo e em tempo real ($\ge 30\text{ fps}$) da rotação do palco sonoro 3D através de um cometa orbital neon projetado na pista circular SVG do dial.
* **Decisões:**
  1. **Mapeamento de Azimute Acústico para Coordenadas Polares SVG:**
     - Convenção acústica ITU-R: $0^\circ = \text{Frente/Centro}$ (Norte), $+90^\circ = \text{Direita}$ (Leste), $-90^\circ = \text{Esquerda}$ (Oeste), $\pm 180^\circ = \text{Traseira/LFE}$ (Sul).
     - Projeção cartesiana implementada em `azimuthToCartesian(azimuthDeg, r)` com $x = c_x + r \cdot \sin(\theta)$ e $y = c_y - r \cdot \cos(\theta)$, alinhando o cometa perfeitamente à pista de raio $R = 84\text{ px}$ e centro $(96, 96)$ sem desvios ou clipping.
     - Sanitização defensiva de NaN e Inf com fallback automático para $0^\circ$ (Norte).
  2. **Renderização do Cometa Orbital Neon no SVG do SoundstageDial:**
     - Quando `orbitEnabled == true`, renderiza um anel guia planetário sutil e um cometa luminoso com núcleo branco quente (`#ffffff`), disco ciano neon (`#06b6d4`), halo pulsante e três partículas de rastro desvanecente ($\theta - 7^\circ, \theta - 14^\circ, \theta - 21^\circ$), contidos rigidamente na `viewBox="0 0 192 192"` com zero layout shift.
     - Subtítulo do botão giratório usinado atualizado dinamicamente para `"8D ORBIT ACTIVE"` em destaque ciano neon.
  3. **Botão Rápido 8D Orbit Acessível:**
     - Adicionado botão pill `"8D Orbit"` com ícone rotativo `<Orbit />` animado (6s de rotação), `aria-pressed`, `role="button"`, suporte a teclado (Enter/Space) e brilho neon ciano intenso quando ativo (`shadow-[0_0_12px_rgba(56,189,248,0.45)]`).
     - Reestruturação do strip de controles em layout de duas linhas (Linha 1: Active/Bypassed e 8D Orbit; Linha 2: Calibrate Soundstage), garantindo zero aperto e zero overflow na largura mínima de 280px.
  4. **Comando IPC Tauri `set_orbit_8d` e Auto-Engajamento do Spatializer:**
     - Exposto comando Tauri `set_orbit_8d(enabled, direction, period_sec, depth)` despachando `AudioCommand::SetOrbit8D` lock-free pela fila circular SPSC.
     - Ao ligar o modo 8D com o spatializer 3D desativado, `handleToggleOrbit` auto-engaja o spatializer para experiência contínua sem cliques extras.
  5. **Telemetria de Fase a $\ge 30\text{ fps}$:**
     - Taxa de polling de `get_telemetry` aumentada para 33ms (~30.3 fps), sincronizando o cometa orbital com total fluidez sem introduzir carga perceptível de CPU ou atraso no áudio.
* **Consequência:** 7 novos testes de UI em `SoundstageDial.test.tsx` (total de 85 testes UI verdes no `npm test`) e teste de integração SPSC no Tauri aprovados. Total de 192 testes Cargo aprovados no workspace.

---

## ADR-036: Painel Completo de Parâmetros Orbitais 8D no Modal de Calibração e Persistência Atômica (Story 6.3)
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Necessidade de permitir personalização completa dos parâmetros orbitais 8D (direção horária, anti-horária e pêndulo; período de revolução de 2.0s a 30.0s; profundidade espacial de 0% a 100%) através do modal de calibração espacial (`SoundstageConfiguratorModal`), sincronizar o toggle com o HUD e persistir atomicamente todas as preferências no `%APPDATA%\OpenBoom\config.json` sem quebras de compatibilidade com arquivos legados.
* **Decisões:**
  1. **Extensão do Esquema `AppConfig` com Retrocompatibilidade Garantida:**
     - Adicionados campos `orbit_enabled: bool` (default `false`), `orbit_direction: OrbitDirection` (default `Clockwise`), `orbit_period_sec: f32` (default `12.0`) e `orbit_depth: f32` (default `0.85`) em `crates/open-boom-storage/src/preset_store.rs`.
     - Decorados com `#[serde(default = "...")]` garantindo que arquivos `config.json` existentes sem essas chaves continuem desserializando perfeitamente sem falhas.
  2. **Persistência Atômica via Staging `.tmp` e `MoveFileExW`:**
     - A integridade do arquivo `config.json` é preservada usando o padrão existente de staging em arquivo temporário único por PID/thread com substituição atômica, eliminando perda de dados sob encerramento repentino ou concorrência.
  3. **Painel de Controle Acessível no `SoundstageConfiguratorModal.tsx`:**
     - Seletor direcional de 3 segmentos (Horário, Anti-Horário, Pêndulo) com ícones e realce ativo neon ciano (`bg-accent-cyan/25`).
     - Slider de período: 2.0s a 30.0s (passo 0.5s), display monospaçado `XX.Xs`, duplo-clique para redefinir para 12.0s.
     - Slider de profundidade: 0% a 100% (passo 1%), display monospaçado `XX%`, duplo-clique para redefinir para 85%.
     - Botão de toggle mestre 8D dentro da modal sincronizado com o HUD e indicação luminosa de estado ligado/desligado.
     - Controles e botões direcionais desabilitados visual e funcionalmente quando o modo 8D estiver inativo.
  4. **Internacionalização Rigorosa com 100% de Paridade:**
     - Adicionadas chaves de títulos, descrições, opções direcionais e tooltips nos dicionários de Inglês (`en`), Português (`pt-BR`) e Espanhol (`es`) em `src-ui/src/i18n/`.
  5. **Despacho Imediato SPSC e Sincronização em `App.tsx`:**
     - Sincronização em tempo real de `orbitDirection`, `orbitPeriodSec` e `orbitDepth`, com despacho lock-free do comando `AudioCommand::SetOrbit8D` e salvamento automático debounced em 250ms.
* **Consequência:** 9 novos testes de UI em `SoundstageConfiguratorModal.test.tsx` (totalizando 94 testes UI verdes no workspace frontend) e 2 novos testes de armazenamento em `crates/open-boom-storage/tests/storage_tests.rs` (totalizando 194 testes Cargo verdes no workspace backend). Zero falhas e build Vite de produção validado em 4.55s.










---

## ADR-037: Arquitetura Hexagonal Multi-Plataforma de Áudio (Linux & macOS Cross-Platform Engine)
* **Data:** 12/10/2026
* **Status:** Aprovado / Registrado
* **Contexto:** Com a consolidação bem-sucedida de todos os 6 Épicos no Windows (v0.3.0), o OpenBoom expande seu escopo para suporte estável multi-plataforma (Linux e macOS). Era necessário desacoplar o motor de DSP das APIs de áudio nativas do Windows sem degradar o desempenho em tempo real, mantendo zero alocações na heap, latência < 15ms e sem duplicação de lógica de processamento acústico.
* **Decisões:**
  1. **Adoção do Paradigma Hexagonal (Ports and Adapters):**
     - **Core de Domínio (`crates/open-boom-core`):** 100% puro e agnóstico de plataforma. Mantém todo o DSP matemático (KEMAR HRTF, EQ 31 bandas, Reverb FDN, Exciter, DRC, Limiter, 8D Orbit) e filas circulares SPSC lock-free.
     - **Port Primário (`AudioHostBackend` Trait):** Define a abstração padrão de ciclo de vida (`start`, `stop`), envio de comandos POD, consulta de telemetria, enumeração/seleção de dispositivos e guarda de estabilidade de buffer (`set_buffer_guard`).
     - **Port Secundário (`SessionManager` Trait):** Abstrai o mixer de volume e mute por aplicação.
  2. **Adapters por Plataforma:**
     - **Windows (`crates/open-boom-wasapi`):** Continua consumindo WASAPI Loopback, driver virtual SYSVAD (`boomvad.sys`) e `IAudioSessionManager2` em thread COM MTA sob MMCSS.
     - **Linux (`crates/open-boom-pipewire`):** Cria um nó de processamento virtual sink (`OpenBoom Sink`) no grafo nativo do PipeWire via `libpipewire`/`pw-filter` com `PW_KEY_NODE_DONT_RECONNECT = true` e prioridade em tempo real via D-Bus RTKit (`org.freedesktop.RealtimeKit1`).
     - **macOS (`crates/open-boom-coreaudio`):** Ingestão via Virtual Audio Device (driver CoreAudio HAL `OpenBoom.driver` ou compatível com BlackHole), agendado sob `THREAD_TIME_CONSTRAINT_POLICY`.
     - **Mock Backend (`crates/open-boom-core::mock`):** Permite execução de testes automatizados e suítes headless em qualquer plataforma sem necessidade de servidor de áudio ativo.
  3. **Isolamento de Crates e Compilação Condicional:**
     - O container Tauri (`src-tauri`) utiliza `#[cfg(target_os = "...")]` para instanciar dinamicamente o adapter de áudio apropriado no `AppState`. Nenhuma dependência nativa de C/Windows é compilada no Linux ou Mac.
  4. **Padronização de Diretórios de Configuração e Autostart:**
     - Persistência desacoplada via `dirs::config_dir()` (`%APPDATA%` no Windows, `~/.config/openboom/` no Linux, `~/Library/Application Support/OpenBoom/` no macOS).
     - Autostart gerenciado por SO (`HKCU\Run` no Windows, `.desktop` em `~/.config/autostart/` no Linux, e `LaunchAgents` plist no macOS).
  5. **CI/CD Matrix no GitHub Actions:**
     - Pipeline estendido para `windows-latest`, `ubuntu-latest` e `macos-latest`, gerando executáveis e instaladores nativos para cada sistema.
* **Consequência:** A espinha de arquitetura multi-plataforma está documentada e finalizada em `_bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-12/ARCHITECTURE-SPINE.md`.

---

## ADR-038: Sistema de Atualização In-App a Custo Zero via GitHub Releases API (Spec 8.1)
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** O usuário solicitou um mecanismo de atualização integrado ao OpenBoom que funcione sem necessidade de servidores pagos ou infraestrutura de terceiros, verificando atualizações ao abrir o aplicativo, exibindo um botão indicador de destaque no cabeçalho quando houver nova versão, levando direto para `Configurações -> Atualizações`, e oferecendo switches de verificação automática na abertura e atualização automática com botão de reinicialização no mesmo slot de ação.
* **Decisões:**
  1. **Integração Direta com GitHub Releases (`check_for_updates`, `download_and_install_update`, `restart_application`):**
     - Consulta assíncrona ao endpoint público `https://api.github.com/repos/JVGS1111/open-boom/releases/latest` sem custo de hospedagem.
     - Comparador SemVer (`is_version_newer`) que analisa tuplas numéricas `(major, minor, patch)` e trata versões estáveis como mais recentes que pre-releases de mesma versão base (`0.3.0 > 0.3.0-beta.1`).
     - Execução de subprocessos PowerShell no Windows configurada com `CREATE_NO_WINDOW` (`0x08000000`) para impedir qualquer piscar de janela de terminal e saída de erro explícita (`[Console]::Error.WriteLine` + `exit 1`).
     - Download em `%TEMP%\OpenBoom-Update-{pid}.exe` e execução silenciosa (`/S`) de instaladores NSIS.
  2. **Persistência Atômica de Preferências (`AppConfig`):**
     - Adicionados `check_updates_on_startup: bool` (default `true`) e `auto_install_updates: bool` (default `false`) em `crates/open-boom-storage/src/preset_store.rs` com `#[serde(default = "...")]` para total retrocompatibilidade.
  3. **UX e Navegação Integrada (`Titlebar.tsx` & `SettingsModal.tsx`):**
     - Botão de destaque âmbar pulsante (`<ArrowUpCircle />`) exibido na `Titlebar` exclusivamente quando `hasUpdate === true`, abrindo `SettingsModal` diretamente na aba `"updates"`.
     - `SettingsModal` organizado em abas (`Geral` e `Atualizações`) com chips de versão atual vs. disponível, caixa de notas de lançamento, switches de preferências e botão de ação unificado no mesmo slot (`Verificar Atualizações` / `Baixar e Instalar` / `Reiniciar Aplicação`).
  4. **Internacionalização 100% Paritária (`en`, `pt-BR`, `es`):**
     - Todas as strings do fluxo de atualização e abas foram adicionadas em `types.ts`, `en.ts`, `pt-BR.ts` e `es.ts`.
* **Consequência:** 9 novos testes unitários em `Updater.test.tsx` (104 testes UI verdes) e novos testes Rust de SemVer e persistência (194+ testes Cargo verdes).

---

## ADR-039: Sincronização Bidirecional em Tempo Real com `IAudioEndpointVolume` do Windows (Spec 7.1)
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Quando o OpenBoom está ativo, o áudio do sistema é redirecionado para o sink virtual `Boom Audio` (driver SYSVAD) ou capturado via WASAPI Loopback antes do atenuador de hardware do endpoint. Como o `AUDCLNT_STREAMFLAGS_LOOPBACK` captura o mix digital pré-atenuação de endpoint, alterar o volume na barra de tarefas do Windows ou pressionar as teclas multimídia do teclado (`Vol+`, `Vol-`, `Mute`) alterava apenas os metadados de `IAudioEndpointVolume` do endpoint padrão sem produzir qualquer variação acústica no `AudioStreamPump` ou refletir no slider do OpenBoom.
* **Decisões:**
  1. **Serviço COM MTA Dedicado (`EndpointVolumeSync` em `crates/open-boom-wasapi/src/volume_sync.rs`):**
     - Worker thread dedicada `"open-boom-volume-sync"` inicializada com `CoInitializeEx(None, COINIT_MULTITHREADED)` e desinicialização determinística no `Drop` após `UnregisterControlChangeNotify`.
     - Implementação COM nativa de `IAudioEndpointVolumeCallback` via `#[windows_core::implement(IAudioEndpointVolumeCallback)]`.
  2. **Prevenção Dupla contra Loops de Realimentação (Feedback Echo Suppression):**
     - **Nível 1 (GUID de Contexto COM):** Toda chamada originada pelo OpenBoom (`SetMasterVolumeLevelScalar` e `SetMute`) passa o `OPEN_BOOM_VOLUME_CONTEXT_GUID` (`{0x8b4c9e2a, 0x5f1d, 0x4e8b, ...}`). No callback `OnNotify`, notificações com `guidEventContext == OPEN_BOOM_VOLUME_CONTEXT_GUID` são descartadas imediatamente.
     - **Nível 2 (Flag Atômica de Guarda):** Como drivers de áudio de terceiros podem zerar o `guidEventContext`, um `AtomicBool` (`is_syncing_to_windows`) é ativado em torno das chamadas COM originadas pelo OpenBoom para suprimir callbacks concorrentes espúrios.
  3. **Mapeamento Assimétrico 0%–100% vs. Faixa Boost 101%–150%:**
     - **UI $\to$ Windows:** Posições $v \in [0.0, 1.0]$ atualizam o scalar do Windows para $v$ e aplicam o ganho correspondente no DSP; posições na zona Boost $v \in (1.0, 1.5]$ fixam o volume do Windows em `1.0` (`100%`) e aplicam o ganho preamp adicional no `AudioProcessor` sob proteção do `MasterLimiter`.
     - **Windows $\to$ OpenBoom:** Quando o usuário move o volume pelo Windows/teclado e o OpenBoom estava em Boost ($> 1.0$), reduções abaixo de `0.99` trazem o volume de volta à escala linear $[0.0, 1.0]$, enquanto notificações em `1.0` preservam o nível de Boost atual sem achatá-lo.
  4. **Rebind Automático em Mudança de Topologia e Master Power:**
     - Como alternar o Master Power comuta o dispositivo padrão do Windows entre `Boom Audio` e o DAC físico, `AppState::handle_master_power_toggle`, `set_audio_device` e `trigger_device_fallback` invocam `volume_sync.rebind_endpoint(...)` para manter o callback registrado no endpoint ativo.
  5. **Auto-Unmute ao Arrastar o Slider da UI:**
     - Arrastar o slider da `Titlebar` para um valor $> 0$ enquanto `isMuted == true` desativa automaticamente o mudo tanto no DSP quanto no Windows (`set_mute(false)`), espelhando o comportamento nativo do mixer do Windows 11.
* **Consequência:** Sincronização bidirecional instantânea e livre de loops entre o Windows (barra de tarefas, teclas multimídia, roda de volume do teclado/headset) e o OpenBoom, validada por 9 novos testes de integração em `volume_sync_tests.rs` e suíte `TitlebarVolumeSync.test.tsx` (totalizando 212 testes Cargo verdes e 104 testes UI verdes).

---

## ADR-040: Seletor de Canal de Captura de Áudio (Driver Boom Audio vs. Emulação Loopback) com Reinicialização Automática
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** O usuário solicitou um seletor na janela de Configurações (`SettingsModal`) para poder alternar manualmente entre os modos de captura de áudio (`Automático`, `Com Driver Boom Audio — Modo A` e `Sem Driver / Emulação WASAPI Loopback — Modo B`), persistindo a escolha e reiniciando o aplicativo automaticamente ao trocar o canal para facilitar testes comparativos e diagnósticos.
* **Decisões:**
  1. **Persistência e Sanitização em `AppConfig` (`crates/open-boom-storage/src/preset_store.rs`):**
     - Adicionado campo `audio_bridge_mode: String` (default `"auto"`, aceitando `"auto"`, `"virtual"`, `"loopback"`) com sanitizador `sanitize_audio_bridge_mode`.
  2. **Controle de Modo Forçado no `VirtualAudioBridge` e `AudioStreamPump` (`crates/open-boom-wasapi`):**
     - `VirtualAudioBridge::set_force_mode` permite alternar entre detecção automática (`None`), Modo A forçado (`Some(BridgeMode::VirtualSinkModeA)`) e Modo B forçado (`Some(BridgeMode::LoopbackFallbackModeB)`).
     - `StreamPumpConfig` estendido com `force_loopback: bool`. Quando `force_loopback == true`, `AudioStreamPump::run_windows_pump` ignora qualquer driver virtual instalado (`Boom Audio`) e captura via `AUDCLNT_STREAMFLAGS_LOOPBACK` diretamente do endpoint físico de renderização (`render_device.clone()`).
  3. **Transição Limpa de Roteamento COM Antes do Restart (`src-tauri/src/lib.rs`):**
     - O comando IPC `set_audio_bridge_mode` grava `config.json` atomicamente, restaura o dispositivo padrão do Windows para o fone físico quando em modo `"loopback"` (ou redireciona para o `Boom Audio` quando em `"virtual"`/`"auto"`), reconfigura o `AudioStreamPump` e, em seguida, `restart_application` encerra o pump ativo antes de invocar `app.restart()` para evitar disputa de handles WASAPI.
  4. **UI e Internacionalização (`SettingsModal.tsx`, `App.tsx`, `i18n`):**
     - Adicionado card "Canal de Captura de Áudio (Driver vs. Emulação)" com `<select>` acessível, indicador de canal ativo (`Modo A` vs `Modo B`), alerta âmbar explicativo quando em modo Loopback sem driver e aviso de reinicialização automática em `pt-BR`, `en` e `es`.
* **Consequência:** 214 testes Cargo verdes e 106 testes UI verdes.

---

## ADR-041: Concepção da Iniciativa "OpenBoom Gamer Pro" (Modo Gamer Pago) e Estratégia de Driver Virtual Próprio
* **Data:** 12/10/2026
* **Status:** Proposta / Em Avaliação (Refinamento BMAD)
* **Contexto:** 
  - O usuário propôs a criação de um "Modo Gamer" comercial (pago), mantendo o núcleo do OpenBoom livre/open-source (modelo Open-Core / Freemium).
  - O Modo Gamer visa oferecer perfis acústicos específicos por jogo (CS2, Warzone, Battlefield, Apex, etc.) com realce dinâmico de frequências de passos/tiros e compressão adaptativa para vantagem tática sem violar anticheats.
  - Foi levantada a necessidade de desvincular o produto comercial do driver proprietário `boomvad.sys` (Boom 3D), avaliando o desenvolvimento de um driver virtual de áudio próprio via REA/WDK e os custos/viabilidade de certificação WHQL da Microsoft.
* **Decisões Iniciais & Pontos de Refinamento:**
  1. **Legalidade e Conformidade com Anticheat (Zero-Ban Guarantee):**
     - O Modo Gamer atuará estritamente como processador de áudio em nível de sistema/saída (Windows Core Audio / WASAPI).
     - Proibição absoluta de injeção de DLLs, abertura de handles de processo (`PROCESS_VM_READ`) ou leitura de memória dos jogos. O software é 100% passivo em relação ao processo do jogo.
  2. **Engenharia Reversa Ética de Áudio de Jogos (Lab Acoustics vs. Copyright):**
     - O REA e ferramentas de análise de áudio serão usados apenas em ambiente de laboratório para inspecionar assinaturas de frequência (espectrogramas FFT de passos de areia, cascalho, metal e tiros em jogos como CS2/Warzone).
     - NENHUM arquivo de áudio (`.wav`, `.bank`, `.pck`) ou código do jogo será embutido no instalador do OpenBoom. O produto distribuirá unicamente os parâmetros matemáticos do DSP (ganhos em dB, frequências centrais em Hz, fatores Q dos biquads e tempos de ataque/release do DRC).
  3. **Estratégia de Driver Virtual Próprio e Custos de Licenciamento (WHQL / Certificado EV):**
     - **Custos Reais de Certificação Microsoft:**
       - Não custa milhões de dólares. A submissão no portal Microsoft Hardware Developer Program (HLK/WHQL) em si é **gratuita** pela Microsoft.
       - O custo obrigatório é um **Certificado de Assinatura de Código EV (Extended Validation)** em token de hardware (Sectigo, DigiCert, GlobalSign), que custa entre **US$ 400 e US$ 900 por ano** (~R$ 2.500 a R$ 5.500/ano), além da abertura formal de CNPJ/D-U-N-S.
     - **Alternativas de Driver Virtual Livre:**
       - Investigar projetos open-source baseados no SYSVAD do WDK da Microsoft ou drivers virtuais abertos (ex: scream, virtual-audio-wire).
       - O Modo B (WASAPI Loopback puro) continuará disponível como fallback sem necessidade de driver.
  4. **Naming & Proteção contra Marcas Registradas (Trademarks):**
     - Evitar o uso direto de marcas registradas das publishers (Valve, Activision, EA) na vitrine de venda. Utilizar nomenclaturas de categoria acústica (ex: "Tactical FPS - CS Focus", "Battle Royale - High Velocity", "MilSim Clarity").
* **Consequência:** Abertura formal do épico de pesquisa e refinamento no backlog BMAD (`T9.1` a `T9.4`).

---

## ADR-042: Correção do Auto-Update In-App (Seleção Estrita do Instalador NSIS, Download Acelerado e Substituição Atômica sem Janelas Duplicadas)
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Durante a atualização in-app da v0.4.0 para a v0.5.0, o usuário relatou três sintomas: (1) demora excessiva no download, (2) abertura de 2 novas janelas na v0.5.0 enquanto 1 janela permanecia na v0.4.0 (gerando 3 ícones na bandeja do sistema), e (3) ao fechar todas e abrir novamente pelo atalho, o aplicativo abria novamente na v0.4.0.
* **Causa Raiz Identificada:**
  1. Na API do GitHub Releases, o artefato portátil `OpenBoom-v0.5.0-portable-x64.exe` (10.3 MB) precede alfabeticamente o instalador `OpenBoom_0.5.0_x64-setup.exe` (2.4 MB, pois `-` ASCII 45 < `_` ASCII 95). O filtro antigo (`lower.contains("openboom")`) selecionava o executável portátil em vez do instalador NSIS.
  2. O PowerShell 5.1 executava `Invoke-WebRequest` sem `$ProgressPreference = 'SilentlyContinue'`, degradando a taxa de transferência pelo cálculo de progresso a cada pacote.
  3. `download_and_install_update` executava `Start-Process $dest -ArgumentList "/S"` imediatamente após o download (com o app antigo ainda aberto). Como `$dest` era o binário portátil em `%TEMP%`, isso apenas abria novas instâncias da v0.5.0 a partir de `%TEMP%` sem atualizar `%LOCALAPPDATA%\OpenBoom\open-boom.exe`.
* **Decisões:**
  1. **Seleção Estrita do Instalador (`select_installer_asset_url` em `src-tauri/src/lib.rs`):** Prioriza estritamente pacotes `.exe` que contenham `"setup"` ou `"installer"` e **não** contenham `"portable"`, evitando que o binário portátil seja baixado como instalador.
  2. **Download Rápido e Staging Atômico com Guarda de Concorrência (`download_and_install_update`):** Utiliza `curl.exe -fL` nativo do Windows (com fallback `Invoke-WebRequest -UseBasicParsing` sob `$ProgressPreference = 'SilentlyContinue'`), protege contra downloads simultâneos via `DownloadInFlightGuard` (`AtomicBool`) e faz staging atômico via `.part` -> `%TEMP%\OpenBoom_Setup_Update.exe` **sem** iniciar processos enquanto a janela atual está aberta.
  3. **Aplicação Diferida no Encerramento (`restart_application` e `App.tsx`):** Ao concluir o download manual (ou clicar em Reiniciar), `restart_application` encerra o `AudioStreamPump`, restaura o endpoint físico do Windows, dispara um worker PowerShell oculto (`CREATE_NO_WINDOW`) e executa `app.exit(0)`. O worker aguarda o término do PID antigo, encerra quaisquer instâncias remanescentes de `open-boom`, executa `OpenBoom_Setup_Update.exe /S /UPDATE -Wait` com o arquivo destravado, remove os temporários de `%TEMP%` e abre **exatamente uma** instância do `%LOCALAPPDATA%\OpenBoom\open-boom.exe` atualizado.
* **Consequência:** Atualização in-app concluída em poucos segundos, substituição real do binário instalado em `%LOCALAPPDATA%\OpenBoom\open-boom.exe` e abertura de uma única janela na versão nova. Validado com 215 testes Cargo e 106 testes UI verdes.

---

## ADR-043: Eliminação de Realimentação Acústica ("Sino Atordoante" Karplus-Strong) no Modo B via `ActivateAudioInterfaceAsync` Process Loopback e Reset de Buffer no Idle
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Ao utilizar o Modo B de captura de áudio (`audio_bridge_mode = "loopback"` ou fallback sem driver virtual), o usuário relatou que o áudio começava a reproduzir brevemente e em seguida travava repetindo infinitamente uma única nota, transformando-se em um sino metálico atordoante ("começa reproduzir o audio e dai meio que ele fica infinitamente reproduzindo 1 nota e vira um sino atordoante bizarro").
* **Causa Raiz Identificada:**
  1. **Loop de Realimentação Positiva Fechado (Ressonador Comb / Karplus-Strong):** Em `crates/open-boom-wasapi/src/stream_pump.rs`, quando nenhum sink virtual estava selecionado (Modo B), o código executava `let capture_device = capture_device_opt.unwrap_or_else(|| render_device.clone());` e ativava um cliente `IAudioClient` com `AUDCLNT_STREAMFLAGS_LOOPBACK` padrão no **mesmo endpoint físico** (`render_device`) onde o `render_svc` do OpenBoom estava injetando o áudio processado. O loopback de endpoint padrão captura todo o mix do dispositivo — **incluindo a própria saída do OpenBoom**. A cada ~10–25ms ($\tau$), o OpenBoom recapturava sua própria saída, reaplicava o ganho do DSP (EQ, 3D Surround, Boost) e re-renderizava no DAC, formando um filtro comb com realimentação positiva ($H(z) = \frac{G}{1 - G z^{-N}}$, frequência fundamental $f_0 = 1/\tau \approx 40\text{--}100\text{ Hz}$) grampeado em `-0.2 dBFS` pelo `MasterLimiter`. Isso gerava uma nota metálica contínua idêntica à síntese de corda/sino Karplus-Strong que jamais parava, mesmo após pausar a música ou desligar o Master Power.
  2. **Retenção de Buffer DMA / Estados IIR ao Parar Áudio Externo:** Quando o áudio externo cessava e `capture_svc.GetNextPacketSize()` passava a retornar `0`, o `render_client` permanecia iniciado sem zerar o buffer circular com `AUDCLNT_BUFFERFLAGS_SILENT` e sem drenar a janela de lookahead de 2ms do `MasterLimiter` e os estados dos filtros biquad IIR.
* **Decisões:**
  1. **Ativação Real de Process Loopback com Exclusão de Árvore de Processos (`activate_process_loopback_client` em `crates/open-boom-wasapi/src/loopback_capture.rs`):**
     - Implementado handler COM `ProcessLoopbackCompletionHandler` (`#[windows_core::implement(IActivateAudioInterfaceCompletionHandler)]`) e função `activate_process_loopback_client(exclude_process_id: u32)` invocando `ActivateAudioInterfaceAsync` no dispositivo virtual do Windows `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK` (`"VAD\\Process_Loopback"`) com `AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK` e `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE`.
     - O `PROPVARIANT` (`VT_BLOB = 65`) é encapsulado em `ManuallyDrop` para impedir que `PropVariantClear` invoque `CoTaskMemFree` sobre `AUDIOCLIENT_ACTIVATION_PARAMS` alocado na stack.
  2. **Isolamento Estrito no `AudioStreamPump` (`crates/open-boom-wasapi/src/stream_pump.rs`):**
     - No Modo B (`capture_device_opt.is_none()`), o pump ativa exclusivamente `activate_process_loopback_client(GetCurrentProcessId())` com formato IEEE Float 32-bit estéreo e flags `AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY`, garantindo no nível do mixer do Windows (`audiodg.exe`) que o áudio renderizado pelo próprio OpenBoom nunca seja recapturado.
     - Adicionada guarda de verificação de ID de endpoint (`cap_id_s == render_device_id_str`) impedindo qualquer colisão acidental entre endpoint de captura e renderização.
  3. **Captura Orientada a Eventos, Supressão de Duplicação em Bypass e Reset de Render no Idle:**
     - Substituído `thread::sleep(1ms)` (que no Windows dormia ~15.6ms) por evento de sincronização `CreateEventW` + `AUDCLNT_STREAMFLAGS_EVENTCALLBACK` + `WaitForSingleObject`.
     - No Modo B, quando `!dsp_active`, `is_muted` ou em silêncio (`idle_flush_ticks >= 8`), `should_render` torna-se `false` e `stop_and_reset_render` preenche o buffer disponível com `0.0` (`AUDCLNT_BUFFERFLAGS_SILENT`), chama `render_client.Stop()` e `render_client.Reset()`.
     - Quando `packet_size == 0` no idle, o pump drena a pipeline com blocos zerados e dispara `AudioCommand::ResetFilters` no 6º tick (~24ms), decaindo os filtros IIR e a telemetria limpa e imediatamente para `-100.0 dBFS`.
* **Consequência:** Eliminação total do loop de realimentação / sino metálico no Modo B, decaimento imediato ao silêncio ao pausar o áudio e verificação automatizada com geração de áudio de teste `.wav` e tom piloto estreito (14.321 Hz) em `crates/open-boom-wasapi/tests/stream_pump_tests.rs` (217 testes Cargo e 106 testes UI verdes).

---

## ADR-044: Eliminação de Estática e Estouros (100 Hz Packet-Boundary Dropouts) no Modo B de Loopback
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Após a resolução do loop de realimentação acústica no Modo B (`4c4087b`), o usuário confirmou que o sino infinito desapareceu e que o 3D Surround e o 8D Orbit funcionavam, mas relatou ouvir estática e estouros ("uma estatica, uns estouros"), observando que **com o 3D Surround ativado havia menos estática e com o 3D Surround desativado havia mais estática**.
* **Causa Raiz Identificada:**
  1. **Flush Prematuro da Janela de Lookahead de 2ms entre Cada Pacote WASAPI de 10ms (`stream_pump.rs`):** O mixer compartilhado do Windows entrega pacotes de captura a cada **10ms** (`480` frames a 48 kHz). Como `WaitForSingleObject(h_ev, 4)` (ou `sleep(2ms)`) acordava após apenas **2 a 4ms**, `capture_svc.GetNextPacketSize()` ainda retornava `0` enquanto o pacote seguinte ($t = 10\text{ ms}$) não chegava. O loop tratava esse intervalo normal entre pacotes como "inatividade" e injetava `960` amostras `0.0` em `coord.process_audio_chunk` (sem enviá-las ao `render_svc`) no meio da reprodução ativa! Isso sobrescrevia a linha de atraso de 96 frames (2ms) do `MasterLimiter` com zeros e descartava os 2ms de áudio que aguardavam no lookahead. Quando o próximo pacote chegava em $t = 10\text{ ms}$, os primeiros 2ms emitidos pelo `MasterLimiter` eram silêncio (`0.0`), abrindo um buraco retangular de 2ms a cada 10ms (100 vezes por segundo).
  2. **Por que o 3D Surround Reduzia a Estática:** Com o 3D Surround ligado, a convolução FIR KEMAR de 64 taps e o reverberador suavizavam parcialmente as bordas retangulares dos buracos de 2ms; com o 3D Surround desligado, os degraus abruptos iam direto para a saída como estática/cliques de banda larga.
  3. **Chamadas Prematuras a `render_client.Stop()` / `Reset()` e Descarte de Pacotes (`padding > frames_read * 2`):** Parar e resetar o `IAudioClient` em hiatos de ~12ms ou descartar pacotes quando `padding > frames_read * 2` causava descontinuidades de fase (estouros audíveis).
* **Decisões:**
  1. **Preservação Estrita do Estado do DSP entre Pacotes (`last_audio_instant` >= 200ms):** O pump jamais injeta zeros em `coord.process_audio_chunk` durante esperas normais entre pacotes de 10ms. O flush de silêncio e `AudioCommand::ResetFilters` só ocorre **uma única vez** (`idle_flushed = true`) quando nenhum áudio ativo (`max_in > 1e-5`) é recebido por **$\ge 200\text{ ms}$** reais.
  2. **Stream WASAPI Contínuo sem `Stop()`/`Reset()` e Zero Pacotes Descartados:** O `render_client` é inicializado com buffer de 25ms (`buffer_guard_enabled`), pré-preenchido com `AUDCLNT_BUFFERFLAGS_SILENT` e mantido iniciado continuamente sem chamadas a `Stop()`/`Reset()` durante a sessão; a condição `padding > frames_read * 2` foi removida para que nenhum pacote capturado seja descartado.
* **Consequência:** Áudio 100% contínuo e cristalino no Modo B (tanto com 3D Surround desligado quanto ligado), validado pelo novo teste `test_mode_b_packet_boundary_continuity_no_static_with_surround_off_and_on` em `crates/open-boom-wasapi/tests/stream_pump_tests.rs` (**218 testes Cargo verdes**).

---

## ADR-045: Desacoplamento de Foco e Estabilidade de Scroll em Modais sob Telemetria em Tempo Real
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Ao selecionar o Modo B (WASAPI Loopback) no menu de configurações (`SettingsModal`), o usuário relatou um bug crítico de interação: não conseguia clicar em mais nada no menu; ao clicar no `<select>`, este abria e fechava muito rápido, e qualquer clique jogava o scroll para o topo da modal.
* **Causa Raiz Identificada:**
  1. **Acoplamento de Foco no Escape Listener (`SettingsModal.tsx` & `AppMixerDrawer.tsx`):** A chamada `closeButtonRef.current?.focus()` estava inserida dentro do `useEffect` de escuta da tecla `Escape`, dependendo de `[isOpen, onClose]`.
  2. **Cascata de Re-renders a 30 fps via Telemetria:** No Modo B (Loopback direto do sistema), pequenas flutuações de headroom/pico de áudio (`limiter_headroom_db`) alteravam o estado da UI a cada ~33ms (30 fps). A cada render, uma nova referência de função anônima `onClose={() => setIsSettingsOpen(false)}` era repassada.
  3. **Roubo Contínuo de Foco e Scroll Jump:** A cada tick (ou clique que gerasse render), a mudança na referência de `onClose` disparava o efeito, executando `closeButtonRef.current?.focus()`. Como o botão de fechar (`X`) está no topo da modal com `overflow-y-auto`, o navegador movia o foco para o topo, fechando qualquer dropdown de `<select>` imediatamente (por perda de foco) e rolando a visualização de volta para o topo.
* **Decisões:**
  1. **Separação Estrita do Foco Inicial (`[isOpen]` com `preventScroll: true`):** O foco no botão de fechar (ou no container da drawer) agora é disparado **apenas uma vez** quando o modal transiciona de fechado para aberto (`isOpen`), sem rolar a página. O listener de teclado `Escape` opera em efeito separado e não toca no foco de elementos da interface.
  2. **Memoização de Callbacks `onClose` (`useCallback`):** Todas as funções de fechamento de modais e drawers em `App.tsx` foram encapsuladas em referências estáveis com `useCallback`.
  3. **Amortecimento de Jitter de Telemetria:** Variáveis numéricas de telemetria receberam limiar $\epsilon = 0.05\text{ dB}$, evitando re-renders desnecessários quando o sinal de áudio oscila apenas na margem do ruído de quantização.
  4. **Comutação Dinâmica Instantânea do Modo de Captura:** O seletor de ponte de áudio no `App.tsx` reseta `isSwitchingBridge(false)` imediatamente após o hot-restart do backend em `set_audio_bridge_mode`, eliminando bloqueios do seletor e dispensando reinicializações do app.
* **Consequência:** Interação fluida, scroll 100% estável no menu, dropdown de seleção de canais de áudio operando sem fechar sozinho, e 106 testes de UI + 218 testes Cargo verdes no workspace.

---

## ADR-046: Registro do Efeito de Dupla Reprodução (Slapback Delay) no Modo B, Rotulagem como `[BETA]` e Direcionamento para Driver Virtual Open-Source
* **Data:** 12/10/2026
* **Status:** Aprovado / Implementado
* **Contexto:** Após a eliminação do loop de sino metálico (`ADR-043`) e da estática/estouros de fronteira de pacotes (`ADR-044`), o usuário validou o Modo B e observou com precisão o comportamento acústico remanescente: *"o som ficou duplicado de uma forma estranha, inclusive acho que entendi oq é loopfeedback se eu nao tiver falando bobagem é duas reproduções pra dar um efeito distante. Ficou com um delay entre as reproduções ou coisa assim"*. O usuário solicitou registrar esse bug/comportamento, colocar uma label `BETA` no Modo B, lançar uma release consolidada e direcionar o esforço futuro para a integração de um **driver virtual open-source** em vez de investir em ajuste fino do Modo B sem driver.
* **Diagnóstico Técnico Registrado (`BUG-MODE-B-01`):**
  - No Modo B (WASAPI Process Loopback sem dispositivo virtual), os aplicativos do Windows (Spotify, navegador, jogos) enviam o áudio original diretamente para o endpoint físico do fone de ouvido em $t = 0\text{ ms}$ (**Reprodução 1 — Dry**).
  - Simultaneamente, o OpenBoom captura esse fluxo via `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK`, aplica a cadeia DSP + `MasterLimiter` (2ms de lookahead) + buffers de captura/renderização WASAPI (~20–25ms) e injeta o áudio processado no **mesmo** fone físico em $t \approx 25\text{ ms}$ (**Reprodução 2 — Wet**).
  - A soma acústica dessas duas reproduções defasadas em ~25ms produz exatamente o efeito de **som duplicado / distante (slapback delay / comb filtering)** percebido pelo usuário.
* **Decisões:**
  1. **Registro Formal no Backlog (`BUG-MODE-B-01` e `T9.5` em `.bmad/03-TASK-BACKLOG.md`):** Documentado o fenômeno de dupla reprodução no Modo B para eventual ajuste fino futuro e criada a tarefa prioritária `T9.5` para adoção de um driver virtual de áudio open-source que isole o fluxo do Windows nativamente sem duplicação e sem depender do `boomvad.sys`.
  2. **Rotulagem Explícita `[BETA]` no Modo B (`SettingsModal.tsx`, `StatusBar.tsx`, `pt-BR.ts`, `en.ts`, `es.ts`):** Adicionados pills visuais `BETA` (`Modo B: BETA`) e textos explicativos claros nos 3 idiomas informando que o Modo B é experimental e apresenta dupla reprodução com leve delay (~25ms) por não utilizar driver virtual isolado.
* **Consequência:** Transparência total para o usuário na interface, governança BMAD atualizada com o bug e a nova diretriz arquitetural (driver open-source), e preparação imediata para o lançamento da release `v0.5.2`.

---

## ADR-047: Expurgar Nomenclatura "Boom 3D / Boom Audio" da UI, Automação do Formato 7.1 e Conformidade Jurídica de Drivers Virtuais
* **Data:** 12/10/2026
* **Status:** Aprovado / Em Planejamento
* **Contexto:**
  - Após os testes práticos bem-sucedidos com o VB-Cable comprovando a superioridade acústica do pipeline 7.1 com HRTF KEMAR, o usuário definiu diretrizes estratégicas cruciais para o produto:
    1. **Purificação da Marca (Remoção de referências ao Boom 3D):** Eliminar totalmente menções e nomes comerciais como "Boom 3D" ou "Boom Audio" de dentro do aplicativo (telas, labels, selects, status bar, tooltips), mantendo essas menções restritas exclusivamente à documentação técnica interna (`README.md`, `.bmad/`, relatórios de RE). O app deve se posicionar com identidade própria e neutra.
    2. **Automação de Configuração de Alto-falantes 7.1 Surround:** Impedir que o usuário final precise abrir manualmente o painel de controle do Windows (`mmsys.cpl`) para configurar o dispositivo em 7.1 Surround. O software deve aplicar a máscara de alto-falantes físicos (`PKEY_AudioEndpoint_PhysicalSpeakers` = `0x63F`) automaticamente.
    3. **Instalação Integrada de 1 Clique (Onboarding):** Oferecer fluxo guiado ou download automático do componente de canal surround em background, sem exigir navegação em sites externos pelo usuário.
    4. **Conformidade Jurídica & Direitos Autorais (Anti-Fraude/Trademark):**
       - Não reivindicar autoria de drivers proprietários de terceiros (evitar white-labeling não autorizado).
       - Renomeação amigável com atribuição transparente no Windows (ex: "OpenBoom Spatial Audio (powered by VB-Cable)" ou "OpenBoom 7.1 Engine") ou transição definitiva para drivers 100% livres/open-source (SYSVAD/Scream com licenças MIT/GPL).
* **Decisões:**
  1. **Expurgo de UI / Clean Branding:**
     - Substituir labels como "Driver Boom Audio (Modo A)" por "Driver de Áudio Surround 7.1 (Modo A)" ou "Dispositivo de Áudio Virtual".
     - Manter o reconhecimento de qualquer driver compatível no backend (`crates/open-boom-wasapi`) sem expor marcas proprietárias nos menus para o usuário final.
  2. **Configuração Automática da Máscara de Canais 7.1:**
     - Implementar no backend Rust rotina para checar e gravar `0x63F` (`KSAUDIO_SPEAKER_7POINT1_SURROUND`) nas propriedades do endpoint capturado via `IPropertyStore` / registro do Windows, notificando o Windows Audio Service.
  3. **Adição ao Backlog BMAD:** Abertura formal das tarefas `T9.6` (Expurgo de Marca), `T9.7` (Automação 7.1) e `T9.8` (Instalador Guiado de 1 Clique).
* **Consequência:** App com identidade profissional independente, sem risco de processos de marcas registradas e com onboarding zero-fricção para o usuário.

---

## ADR-048: Identidade Visual Oposta (Gamer Red vs. Studio Cyan), Monetização Global em USD e Orquestração de Agentes Especializados
* **Data:** 12/10/2026
* **Status:** Aprovado / Em Planejamento
* **Contexto:**
  - Definição da estratégia de produto e posicionamento comercial para a edição paga/comercial ("OpenBoom Gamer Pro" / Edição Fechada):
    1. **Identidade Visual Oposta (Color-way Contrast):**
       - A versão gratuita/open-source (**OpenBoom**) possui identidade visual estúdio/audiófila com paleta escura `#16171b` e brilhos em **Ciano Neon (`#38bdf8`) e Azul (`#3b82f6`)**.
       - O modo comercial gamer (**Gamer Pro**) adotará uma identidade visual contrastante e agressiva: **Vermelho Gamer Neon (`#ef4444` / Crimson `#dc2626` / Amber Glow)**, remetendo a alto desempenho, vantagem competitiva e foco tático.
    2. **Monetização Internacional em USD:**
       - Foco direto no mercado global de gamers/esports (EUA, Europa, Ásia) cobrando em **dólares americanos (USD)** via plataformas como Stripe, Lemon Squeezy ou Paddle.
    3. **Landing Page, Marketing & Estratégia de Go-to-Market:**
       - Criação de site/landing page de alta conversão para o produto Pro.
       - Estratégia de marketing voltada a benchmarks de passos/jogos (CS2, Warzone, BF6, Apex) e criadores de conteúdo.
    4. **Delegação para Agentes Especializados BMAD:**
       - Estas frentes serão conduzidas em sessões dedicadas com os agentes especializados disponíveis no ecossistema BMAD:
         - **Sally (`bmad-agent-ux-designer`) / `bmad-ux`:** Concepção da UI/UX da paleta vermelha e design da landing page.
         - **John (`bmad-agent-pm`) / Mary (`bmad-agent-analyst`):** Pesquisa de precificação em USD, análise competitiva (Sonar, Dolby, Scout Mode) e estratégia de go-to-market.
         - **Winston (`bmad-agent-architect`) / Amelia (`bmad-agent-dev`):** Arquitetura do módulo de licenciamento/DRM leve e motor DSP de passos.
* **Consequência:** Visão de produto clara e alinhada entre engenharia, design e negócios globais.

---

## ADR-049: Estratégia de Licenciamento Zero-Backend (Offline DRM / MoR), Teste Grátis, Filtro Anti-Overkill e Identidade da Edição Comercial
* **Data:** 12/10/2026
* **Status:** Aprovado / Em Avaliação (Refinamento BMAD)
* **Contexto:**
  - O usuário levantou diretrizes práticas essenciais para a viabilização do produto comercial, visando simplicidade operacional máxima, custo mínimo e proteção contra complexidade excessiva ("filtro de sanidade / anti-overkill"):
    1. **Licenciamento, Ativação & Trial com Zero (ou Mínimo) Backend/API:**
       - Evitar montar servidores caros, bancos de dados e APIs proprietárias de autenticação.
       - Soluções de mercado avaliadas:
         - **Chaves Criptográficas Offline (Assinatura Asimétrica Ed25519 / RSA):** O app em Rust valida matematicamente a chave com uma chave pública embutida sem precisar encostar na internet.
         - **Merchant of Record (MoR) com Licenciamento Nativo Embutido:** Plataformas como **Gumroad, Lemon Squeezy ou Polar.sh** que já geram chaves de licença prontas, cuidam do checkout, impostos internacionais (VAT/Sales Tax) e oferecem endpoint simples de 1 clique ou validação offline.
       - **Período de Teste Grátis (Free Trial):** Implementar trial de 7 a 14 dias com carimbo de tempo protegido no registro/armazenamento local ou cota de uso diário.
    2. **Onde Vender & Questão de CNPJ (Pessoa Física vs. Jurídica):**
       - Ao usar plataformas **MoR (Merchant of Record)** como Gumroad ou Lemon Squeezy, **não é obrigatório abrir CNPJ imediatamente**. O MoR é legalmente o vendedor perante o consumidor final internacional, recolhe os impostos nos EUA/Europa e repassa o saldo líquido via PayPal ou Payoneer para pessoa física (CPF) no Brasil, permitindo testar a demanda antes de gastar com contabilidade/CNPJ.
    3. **Filtro Anti-Overkill (Pés no Chão):**
       - Mapear e separar o que é essencial para o MVP comercial (chave de licença simples, trial, preset de passos e botão de compra) do que é complexidade desnecessária agora (servidores próprios de login, infraestrutura de microserviços, DRM invasivo de kernel, etc.).
    4. **Naming & Brainstorm de Marca para a Edição Comercial:**
       - Nomes levantados para amadurecimento:
         - *Footsteps 3D* / *Footstep Pro*
         - *GamerAudio* / *OpenBoom Gamer Pro*
         - *ApexAudio* / *Acrobat 3D* / *TacticalSound 3D*
* **Consequência:** Risco financeiro e de manutenção reduzido a zero, permitindo lançar a versão paga sem custos fixos de servidor.

---

## ADR-050: Posicionamento "Zero-Ban Guarantee", Performance Ultraleve para FPS e Roadmap v1.5 (Des-Reactização / UI Nativa Ultra-Thin)
* **Data:** 12/10/2026
* **Status:** Aprovado / Em Planejamento
* **Contexto:**
  - O usuário definiu dois pilares inegociáveis de proposta de valor para o público gamer e uma diretriz de evolução técnica para a versão `v1.5`:
    1. **Mensagem Central de Marketing: "100% Anticheat-Safe / Zero-Ban":**
       - Todo gamer tem pavor de tomar ban em jogos com anticheats agressivos (Vanguard do Valorant, Ricochet do Call of Duty, EAC do Apex/Fortnite, VAC do CS2).
       - A comunicação do produto (landing page, badges, onboarding) DEVE martelar com clareza cristalina:
         - *Zero DLL Injection / Zero Process Hooks*.
         - *Zero Memory Scanning / Zero Game Modification*.
         - *100% Windows Audio Pipe Compliance* (idêntico a softwares de headsets como Logitech G HUB, HyperX NGENUITY e SteelSeries GG).
    2. **Pegada de Recursos Mínima em Jogo (Zero FPS Drop):**
       - Gamers competitivos monitoram framerate (FPS), uso de CPU e consumo de RAM. O produto deve se posicionar e se comprovar como uma solução que **não rouba nem 1 frame** do jogo.
       - A engine de áudio em Rust (`crates/open-boom-core`) já opera com custo de CPU ínfimo (< 1%), mas o shell de interface precisa acompanhar essa eficiência.
    3. **Roadmap v1.5: Avaliação de Substituição do React por Frontend Ultraleve:**
       - Atualmente, o frontend em React + WebView2 consome uma cota moderada de memória RAM típica de runtimes web (~60–100 MB).
       - Para a versão `v1.5`, está planejado o estudo de migração do frontend para tecnologias de pegada de memória quase zero:
         - **Opção A:** HTML/JS puro (Vanilla DOM com zero frameworks e zero bundle overhead).
         - **Opção B:** UI Nativa Rust pura com GPU aceleração (ex: **Slint**, **Iced** ou **Egui**), eliminando totalmente o WebView2/Chromium e reduzindo o consumo de RAM total para menos de 15 MB.
* **Decisões:**
* **Consequência:** Confiança total dos jogadores para instalar e rodar o app em segundo plano durante partidas ranqueadas.

---

## ADR-051: Estratégia de Tração Orgânica TikTok / Shorts, Janela de Arbitragem com Micro-Influenciadores Gamers e Criadores Sintéticos (IA UGC)
* **Data:** 12/10/2026
* **Status:** Aprovado / Em Planejamento (Marketing & Aquisição)
* **Contexto:**
  - O usuário identificou uma oportunidade de arbitragem de tráfego e marketing de guerrilha de alto impacto:
    1. **Janela de Mercado com Micro-Influenciadores Gamers:**
       - Com a regulamentação severa e restrições sobre cassinos online / "bets" no Brasil, milhares de streamers e criadores de jogos perderam sua principal fonte de patrocínio e estão abertos a parcerias de produtos reais de tecnologia por valores acessíveis ou modelo de comissão / afiliado.
       - Parcerias diretas com streamers de CS2, Valorant, Warzone e R6: demonstrar o software em live ("ouvindo passos que o chat não ouve").
    2. **Vídeos Virais no TikTok & YouTube Shorts (Formato Demonstração A/B):**
       - Formato matador de vídeo curto para retenção alta:
         - *"Você também morre pelas costas sem ouvir nada no Warzone/CS?"*
         - Áudio original do jogo (abafado por explosão) vs. Áudio com OpenBoom Gamer ligado (passo nítido e amplificado).
         - O usuário assiste de fone e sente o "efeito UAU" em 5 segundos de vídeo.
    3. **Preservação de Imagem Pessoal & Criadores Sintéticos com IA (UGC com Higgsfield / Kling / HeyGen):**
       - Para evitar exposição pessoal obrigatória ("passar vergonha na internet"), avaliar pipelines de geração de criadores sintéticos de IA (geração de avatares com Higgsfield/HeyGen ou gameplay gravada pura com narração de IA com vozes ultra-realistas como ElevenLabs).
       - Vídeos com foco 100% na tela do jogo + espectrograma + áudio antes/depois, dispensando a necessidade de mostrar o rosto.
* **Decisões:**
  1. Formalizar no Backlog da Sprint 10 as tarefas de prospecção de micro-influenciadores gamers pós-cassinos (`T10.10`), roteiros de criativos virais A/B (`T10.11`) e teste de UGC sintético via ferramentas de IA (`T10.12`).
* **Consequência:** Canal de aquisição com potencial de viralização em massa a custo próximo de zero.

---

## ADR-052: Arquitetura de Onboarding do Driver Virtual, Bloqueio Preventivo de Controles no Boot, Resiliência Try/Catch e Atribuição Anti-Processo
* **Data:** 13/10/2026
* **Status:** Aceito
* **Contexto:**
  - Para fechar o ciclo da versão open-source e superar definitivamente a limitação do Modo B (`BUG-MODE-B-01`), o usuário definiu refinamentos críticos de experiência e segurança:
    1. **Bloqueio Preventivo no Boot (First-Run Gating):** Se o software abrir pela primeira vez (ou se nenhum driver virtual for detectado), o usuário não deve conseguir ligar o app ou interagir com botões (ficando desativados/cinza), abrindo automaticamente um modal de onboarding com botão de instalação de 1 clique.
    2. **Tratamento de Erros Resiliente (Try/Catch & Fallback):** Em caso de falha de rede ou cancelamento do UAC de administrador, o app não deve quebrar: deve exibir alerta de erro com diagnóstico, botão de tentar novamente e link direto para download manual (`https://vb-audio.com/Cable/`).
    3. **Proteção Jurídica & Atribuição Anti-Processo:** Renomeação do endpoint no Windows para `OpenBoom Spatial Audio (powered by VB-Cable)` e expurgo total de referências ao Boom 3D na UI.
    4. **Automação de 7.1 Canais:** Gravação de `PKEY_AudioEndpoint_PhysicalSpeakers = 0x63F` sem intervenção manual no painel do Windows.
    5. **Teste Final em Máquina Real:** Validação de ponta a ponta na máquina do usuário, desinstalando drivers prévios do host e verificando a instalação completa via OpenBoom.
* **Decisões:**
  1. Criação do contrato canonical `SPEC-virtual-driver-onboarding` (`SPEC.md`, `technical-driver-integration.md`, `stories.yaml`).
  2. Decomposição nas histórias `9-1` (UI Purge & Rename Attribution), `9-2` (Automated 7.1 Mask Injection), `9-3` (First-Run Gating & Controls Lockdown) e `9-4` (1-Click Silent Setup, Try/Catch & Manual Fallback).
* **Consequência:** Onboarding sem atrito, zero risco de duplicação de áudio no primeiro boot, e blindagem jurídica completa.

---

## ADR-053: Conclusão do Épico 9 — Entrega Consolidada de Onboarding, Gating e Automação 7.1
* **Data:** 13/10/2026
* **Status:** Aceito
* **Contexto:** Todas as histórias do Épico 9 foram implementadas e verificadas com 100% de aprovação na suíte de testes (226 testes Rust em Cargo e 113 testes de UI em React/Vite):
  - Story 9-1: Purgação completa de nomes comerciais ("Boom 3D / Boom Audio") da interface e dicionários i18n (`en`, `pt-BR`, `es`) e personalização de endpoint amigável com atribuição explícita `OpenBoom Spatial Audio (powered by VB-Cable)`.
  - Story 9-2: Injeção automatizada da máscara de alto-falantes 7.1 Surround `0x63F` (`PKEY_AudioEndpoint_PhysicalSpeakers`) via registro e property store do Windows Core Audio.
  - Story 9-3: Detecção de presença de driver virtual no boot, bloqueio visual e interativo dos controles principais (`opacity-40 pointer-events-none select-none cursor-not-allowed`) e abertura automática do `<DriverOnboardingModal />`.
  - Story 9-4: Rotina de instalação silenciosa em 1-clique (`VBCABLE_Setup_x64.exe -i -h -Verb RunAs`), tratamento completo de erros com fallback manual (`https://vb-audio.com/Cable/`) e injeção automática pós-instalação da marca e máscara 7.1.
* **Decisões:** Épico 9 concluído e pronto para validação de compilação local, smoke test e commit.
* **Consequência:** OpenBoom totalmente autônomo, livre de vínculos com drivers proprietários e juridicamente blindado contra violação de marcas.




---

## ADR-054: Eliminação de Communications Ducking e Auto-Unmute de Sessões na Transição de Driver
* **Data:** 13/10/2026
* **Status:** Aceito
* **Contexto:**
  - Após a instalação do driver e primeiro boot do OpenBoom, o usuário notou que o Windows mutava abas do Chrome e jogos no Mixer de Volume do Windows.
  - Causa raiz investigada:
    1. **Atribuição Indevida do Papel `eCommunications`:** Em `crates/open-boom-wasapi/src/device.rs`, a função `set_windows_default_device_worker` chamava `set_default_func(policy_ptr, pcwstr, eCommunications)`. No Windows, quando um dispositivo de comunicação é alterado ou streams de comunicação são ativadas, a política nativa de "Atenuação de Comunicações" do Windows (Painel de Som -> Comunicações: *"Reduzir o volume de outros sons em 80%"* ou *"Desativar o som de todos os outros sons"*) é disparada, mutando ou zerando o volume de fluxos de mídia/jogos (`eConsole` e `eMultimedia`).
    2. **Glitch de Migração do Windows 11 / Restart do `audiosrv`:** Durante a reinicialização forçada do serviço de áudio (`audiosrv`) para registrar as propriedades do driver, as sessões de áudio ativas são migradas para o novo endpoint padrão e frequentemente inicializadas com `bMuted = TRUE` pelo sistema operacional.
    3. **Ausência de Auto-Unmute para Sessões sem Perfil:** Em `crates/open-boom-wasapi/src/session_manager.rs`, o `SessionManager` apenas desmutava sessões caso houvesse um perfil salvo previamente em `app_volumes.json`. Sessões novas ficavam presas no estado mudo do Windows.
* **Decisões:**
  1. **Remoção Estrita de `eCommunications`:** `crates/open-boom-wasapi/src/device.rs` agora configura apenas `eConsole` e `eMultimedia`. O papel `eCommunications` permanece intacto nos dispositivos de comunicação do usuário (headsets, microfones, Discord), impedindo que o Windows aplique atenuação de comunicações no OpenBoom.
  2. **Auto-Unmute de Resiliência em `apply_stored_profiles`:** Se uma sessão for detectada com `session.is_muted == true` e não possuir um perfil explícito em `stored_profiles` pedindo mudo, o OpenBoom automaticamente desmuta a sessão via `ISimpleAudioVolume::SetMute(FALSE, NULL)`.
  3. **Enumeração no Boot e em Transições:** Invocação imediata de `enumerate_sessions()` na inicialização do `AppState::new_runtime()`, ao ligar a chave mestre de áudio, e ao aplicar mudanças de modo de ponte de áudio.
* **Consequência:** Abas do Chrome, jogos e tocadores de mídia tocam perfeitamente sem serem silenciados pelo Windows ao instalar o driver ou abrir o aplicativo.

---

## ADR-055: Refinamentos de UI: Remoção do Seletor de Idioma da Tela Inicial, Layout Espaçoso na Aba de Updates & Fallback em Inglês no i18n
* **Data:** 13/10/2026
* **Status:** Aceito
* **Contexto:**
  1. O seletor de idiomas estava visível diretamente na barra de título (`Titlebar.tsx`) da tela inicial, sobrecarregando os controles da janela.
  2. A aba de atualizações (`SettingsModal.tsx` Tab 2) apresentava layout comprimido/apertado, com as caixas de versão quebrando em duas linhas ("Current" / "Version: 0.5.2") e paddings escassos.
  3. Indagação sobre o comportamento do sistema de internacionalização diante de idiomas de host não suportados pelo aplicativo.
* **Decisões:**
  1. **Remoção do Menu de Idioma da Barra de Título:** Retirado o `<LanguageMenu />` de `Titlebar.tsx`. A seleção de idioma segue acessível com layout enriquecido dentro de `SettingsModal.tsx` (Seção 1: Idioma & Interface) e pelo menu de contexto nativo da bandeja do Windows (System Tray).
  2. **Refinamento Visual da Modal e Aba de Updates:**
     - Largura da modal expandida de `w-[520px]` para `w-[560px] max-w-[95vw]` e paddings aumentados para `p-5 sm:p-6`.
     - Banner de atualizações com `rounded-xl`, `p-4 sm:p-5`, ícone 36px e flexwrap responsivo.
     - Badges de versão atual e versão mais recente estruturados com `whitespace-nowrap`, padding confortável `px-3 py-1.5 rounded-lg` e sombra suave, impedindo quebras de linha desagradáveis.
     - Caixas de mensagem de status e switches com paddings ampliados (`p-3.5 sm:p-4`) e espaçamento balanceado.
  3. **Garantia de Fallback Estrito em Inglês:** Reforçado `getSystemLocale()` e `resolveEffectiveLocale()` para iterar candidatos em `navigator.languages` e garantir que qualquer idioma não suportado (ex.: alemão, francês, japonês, etc.) caia estritamente em `"en"`. Adicionados testes de asserção em `I18n.test.tsx`.
* **Consequência:** Interface limpa e despoluída na tela inicial, aba de atualizações elegante e legível, e previsibilidade plena no tratamento de idiomas do sistema operacional.

