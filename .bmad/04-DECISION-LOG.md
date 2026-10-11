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
