# OpenBoom - Session State

## Última Atualização
* **Data/Hora:** 11/10/2026 - 14:45
* **Sprint Ativa:** Epic 4 CONCLUÍDO (`done`) — `Psychoacoustic Audio Sweeteners & Dynamic Effects Rack`. Próximo no backlog: **Epic 5: Windows Per-Application Audio Session Mixer & Volume Routing**.
* **Tarefa Atual:** Conclusão formal do Epic 4 (Stories 4.1 a 4.5, ADR-027, 140 testes Cargo e 56 testes UI verdes).

## O que foi concluído nesta sessão
1. Localização e engenharia reversa do Boom 3D instalado no sistema (identificado como app MSIX em .NET/WPF com driver SYSVAD `boomvad.sys`, motor DSP C++ SSE e banco AutoEq).
2. Criação do repositório dedicado em `C:\Users\joaov\Documents\GitHub\open-boom` e inicialização do Git.
3. Implantação completa da estrutura do Harness BMAD em `.bmad/` (Briefing, Arquitetura, Milestones, Task Backlog, Decision Log, Session State).
4. Vinculação explícita ao repositório `decompilator` e ao servidor MCP `rea` em todas as diretrizes do projeto.
5. Execução completa do workflow `/bmad-prd` em Fast Path (`prd.md` e `addendum.md` finalizados).
6. Execução completa do workflow de UX Design (`DESIGN.md`, `EXPERIENCE.md`, mockup interativo).
7. Execução completa do workflow de Arquitetura Técnica (`ARCHITECTURE-SPINE.md`, ADR-007).
8. Execução e conclusão do workflow `/bmad-create-epics-and-stories` (5 Épicos, 24 Histórias de Usuário).
9. Execução e conclusão do workflow `/bmad-sprint-planning` (`sprint-status.yaml` gerado e validado).
10. **Implementação e Revisão Completa da Story 1.1 (`1-1-project-workspace-scaffolding-desktop-shell-window`):**
    - Scaffold do workspace Cargo unificado com `crates/open-boom-core`, `crates/open-boom-wasapi`, `crates/open-boom-storage`, `src-tauri` e `src-ui`.
    - Registro de ADR-008 e aprovação da revisão em 4 camadas com 10 testes unitários.
11. **Implementação e Verificação Completa da Story 1.2 (`1-2-driverless-wasapi-audio-ingest-output-engine`):**
    - `crates/open-boom-wasapi/src/format_converter.rs`: Normalizador de formatos de entrada (IEEE Float32, PCM16, PCM24, PCM32) com zero alocações e sanitização de NaN.
    - `crates/open-boom-wasapi/src/resampler.rs`: Reamostrador cúbico de Hermite (Catmull-Rom de 4 pontos) com continuidade $C^1$, tolerância a epsilon e suporte a 7.1 surround.
    - `crates/open-boom-wasapi/src/mmcss.rs`: Guardião RAII `MmcssGuard` com priorização "Pro Audio" e monitor de orçamento de callback.
    - `crates/open-boom-wasapi/src/loopback_capture.rs` e `render_client.rs`: Captura e renderização WASAPI em modo usuário sem drivers de kernel.
    - `crates/open-boom-wasapi/src/coordinator.rs`: Coordenador garantindo latência de ida e volta < 15ms.
    - Registro de ADR-009 e aprovação com 31 testes.
12. **Implementação e Verificação Completa da Story 1.3 (`1-3-lock-free-pod-command-dispatcher-master-power-control`):**
    - `crates/open-boom-core/src/dsp/pipeline.rs`: Motor `AudioProcessor` com consumo lock-free da fila circular SPSC de structs POD `AudioCommand` (`AD-1`, `NFR2`), com drenagem sem alocações e suporte a `SetMasterPower`, `SetMasterVolume`, `SetMute` e `SetBypass`.
    - Passthrough Bit-Perfect com 0% CPU (`FR27`, `NFR5`): Cópia direta `output.copy_from_slice(input)` quando `master_power == false`, preservando exatidão de bits com zero aritmética de DSP.
    - `crates/open-boom-wasapi/src/coordinator.rs`: Integração de `AudioProcessor` e fila SPSC no `AudioEngineCoordinator`, expondo telemetria dinâmica (`dsp_active: bool`).
    - `crates/open-boom-wasapi/src/mmcss.rs`: Implementação de `Send` e `Sync` para `MmcssGuard`.
    - `src-tauri/src/lib.rs`: Integração do coordenador no `AppState` de Tauri e comando `get_telemetry` sincronizado dinamicamente.
    - `src-ui/src/components/MasterPowerSwitch.tsx`: Componente de chave mestre medindo estritamente 48px × 26px com brilho ciano LED neon (`rgba(56, 189, 248, 0.35)`), animação suave e acessibilidade plena (teclado Space/Enter e ARIA `role="switch"`).
    - `src-ui/src/components/Titlebar.tsx` e `src-ui/src/App.tsx`: Chave mestre integrada ao HUD e despacho de comando via Tauri IPC.
    - Suíte de 42 testes passando no workspace com zero warnings e build Vite do frontend concluído em 2.16s.
    - Registro de ADR-010 no `.bmad/04-DECISION-LOG.md`.
    - Atualização formal de `sprint-status.yaml`, `03-TASK-BACKLOG.md` e spec de Story 1.3 (`status: done`).
13. **Execução e Aprovação do Workflow `/bmad-correct-course`:**
    - Identificada a limitação fundamental do WASAPI Loopback puro em evitar duplicação acústica (slapback echo) e restrição de formato estéreo no DAC físico.
    - Investigação e comprovação de engenharia reversa no Boom 3D instalado: presença e uso do driver PortCls `boomvad.sys` (SYSVAD) como sink mudo padrão do sistema operacional, e da engine nativa `BoomEngine.dll` (`CreateAudioProcessor`, `CreateSession`, `AVBoomAudioProcessor`, `AVFIRFilterSSE`) para processamento e re-injeção na saída física.
    - Elaboração, ratificação e aprovação formal do **Sprint Change Proposal** (`_bmad-output/planning-artifacts/sprint-change-proposal-2026-10-10.md`).
    - Registro do **ADR-011** no `.bmad/04-DECISION-LOG.md`.
    - Atualização da topologia do sistema em `.bmad/01-ARCHITECTURE.md` para Ingestão Dual-Mode (Virtual Audio Device Primário + Fallback Loopback).
    - Inclusão da **Story 1.8** (`1-8-virtual-audio-device-bridge-rea-reverse-engineering`) em `sprint-status.yaml`.
14. **Implementação e Verificação Completa da Story 1.4 (`1-4-master-volume-boost-control-device-selection`):**
    - `crates/open-boom-core/src/types.rs`: Struct `AudioDeviceInfo` (`id`, `name`, `is_default`) serializável.
    - `crates/open-boom-wasapi/src/device.rs`: `AudioDeviceManager` com enumeração MMDeviceAPI e nomes amigáveis via property store, além de modo sintético isolado para testes headless.
    - `crates/open-boom-wasapi/src/render_client.rs` e `coordinator.rs`: Métodos `switch_device` e `handle_device_lost` com garantia de comutação sub-500ms e fallback para console endpoint padrão do Windows.
    - `src-tauri/src/lib.rs`: Comandos IPC `get_audio_devices`, `set_audio_device` e `trigger_device_fallback`.
    - `src-ui/src/components/DeviceSelector.tsx`: Popover dropdown acessível com badge de endpoint padrão, indicador online e navegação por teclado.
    - `src-ui/src/components/NotificationBanner.tsx`: Alerta flutuante com auto-dismiss para notificação de desconexão e recuperação de hardware.
    - `src-ui/src/components/Titlebar.tsx`: Slider de volume master 0% a 150% com taper perceptual $p^2$ (-12.0 dB em 50%), boost preamp (+2.6 dB em 135%) e snap instantâneo para 100% (0.0 dB) via duplo clique.
    - Suíte de 60 testes passando no workspace com zero warnings e build Vite do frontend concluído em 2.06s.
    - Registro de ADR-012 no `.bmad/04-DECISION-LOG.md`.
    - Atualização formal de `sprint-status.yaml`, `03-TASK-BACKLOG.md` e spec de Story 1.4 (`status: done`).
15. **Implementação e Verificação Completa da Story 1.5 (`1-5-master-lookahead-peak-limiter-telemetry-metering`):**
    - `crates/open-boom-core/src/types.rs`: Extensão de `TelemetrySnapshot` com `limiter_headroom_db: f32`, `sample_rate: u32` e `bit_depth: u32` com valores padrão de alta fidelidade.
    - `crates/open-boom-core/src/dsp/limiter.rs`: `MasterLimiter` com atraso lookahead de 2ms (96 frames a 48 kHz), pré-detecção de picos na janela circular, envelope transparente (ataque 0.2ms, release 50ms) e teto brickwall inviolável cravado em -0.2 dBFS ($10^{-0.2/20} \approx 0.9772372$) com sanitização de NaN/Inf, garantindo matematicamente 0% de clipping digital e saída $\le -0.1\text{ dBFS}$ sob qualquer nível de volume boost (150%) ou transientes patológicos (+20 dBFS).
    - `crates/open-boom-core/src/dsp/pipeline.rs`: Cálculo dinâmico de headroom $(-0.2 - \text{peak\_dbfs})\text{.clamp}(0.0, 20.0)$ em `AudioProcessor::telemetry_snapshot`, proteção de amostras ímpares contra bypass do limitador, e propagação contínua de metadados.
    - `crates/open-boom-wasapi/src/coordinator.rs`: Propagação contínua de sample rate (48 kHz) e bit depth (32-Bit Float) através de `AudioEngineCoordinator::telemetry_snapshot`.
    - `crates/open-boom-core/tests/pipeline_tests.rs`: Testes unitários para atraso lookahead de 96 frames (2ms), teto brickwall sob 150% boost (+3.52 dB), transiente patológico (+20 dBFS / amplitude 10.0), e cálculo exato de headroom (5.8 dB em -6 dBFS, 20 dB no silêncio/passthrough).
    - `crates/open-boom-wasapi/tests/coordinator_tests.rs`: Teste de integração de telemetria cobrindo sample rate, bit depth e headroom.
    - `src-ui/src/components/StatusBar.tsx`: Renderização contínua de sample rate (`48 kHz`), bit depth (`32-Bit Float`), latência medida (`<15 ms`), medidor dinâmico de headroom (`Headroom: +11.8 dB` / `>20 dB` / `0.0 dB`), e badge de limitador pulsante âmbar/vermelho com sombra neon exibindo redução exata em decibéis (`LIMITER: -0.2 dBFS (-X.X dB)`).
    - `src-ui/src/App.tsx`: Polling otimizado para 60ms (~16.6 Hz) para visualização em tempo real sem travamentos e ligação de headroom e metadados ao dock de status.
    - Suíte de 66 testes passando no workspace com zero warnings e build Vite do frontend concluído em 2.19s.
    - Registro de ADR-013 no `.bmad/04-DECISION-LOG.md`.
    - Atualização formal de `sprint-status.yaml`, `03-TASK-BACKLOG.md` e spec de Story 1.5 (`status: done`).
16. **Implementação e Verificação Completa da Story 1.6 (`1-6-phase-continuous-instant-a-b-bypass`):**
    - `crates/open-boom-core/src/types.rs`: Definição de `CROSSFADE_SAMPLES = 240` (5ms a 48.000 Hz).
    - `crates/open-boom-core/src/dsp/pipeline.rs`: Adicionado `crossfade_pos: f32`, interpolação equal-power $\cos/\sin$ ($g_A^2 + g_B^2 = 1.0$), e roteamento de sinal dry com preservação estrita do atraso lookahead de 2ms (96 frames) e teto brickwall do `MasterLimiter`, garantindo zero saltos de fase e zero cliques/pops.
    - `crates/open-boom-wasapi/src/coordinator.rs`: Adicionados métodos de conveniência `is_bypassed()` e `crossfade_pos()`.
    - `crates/open-boom-core/tests/pipeline_tests.rs`: Testes unitários para rampa de crossfade de 240 frames, invariante de equal-power, retenção de atraso lookahead de 2ms no modo bypass, estabilidade contra chattering rápido (100 toggles com $\Delta y$ delimitado), e precedência de passthrough bit-perfect com restauração do estado de crossfade ao desligar/ligar master power (70 testes passando no workspace).
    - `src-ui/src/App.tsx`: Registrado listener global de atalho de teclado `Ctrl + Shift + B` e despacho via Tauri IPC.
    - `src-ui/src/components/StatusBar.tsx`: Renderização de `Hardware Passthrough (Bypassed)` em estilo ardósia suave (`text-text-muted`) quando bypass ativo, badge de limiter refletindo bypass, e tooltip do botão atualizado com atalho.
    - Build do frontend React/Vite concluído sem erros em 1.73s.
17. **Implementação e Verificação Completa da Story 1.7 (`1-7-application-preferences-window-geometry-system-startup-persi`):**
    - `crates/open-boom-storage/src/preset_store.rs`: `AppConfig` com `#[serde(default)]` e suporte a todos os parâmetros de sessão, persistência atômica via staging `config.<pid>.tmp` e `MoveFileExW` / `fs::rename` atômico em `%APPDATA%\OpenBoom\config.json`, resolução robusta de diretório e recuperação com fallback gracioso para `AppConfig::default()` na presença de JSON ausente ou corrompido (`FR29`, `AD-10`).
    - `crates/open-boom-storage/src/startup.rs`: `StartupManager` gerenciando chaves de inicialização em `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run` sem requerer privilégios administrativos (UAC) e com modo mock para testes e isolamento headless.
    - `crates/open-boom-storage/tests/storage_tests.rs`: Suíte de 6 testes unitários cobrindo persistência atômica, recuperação contra corrupção, campos parciais, resolução de diretório e ciclo de vida de registro de inicialização.
    - `src-tauri/src/lib.rs`: Exposição dos comandos IPC `load_app_config`, `save_app_config`, `get_launch_on_startup` e `set_launch_on_startup`, restauração de coordenadas de janela no setup, e menu nativo de bandeja do sistema estendido (`FR28`) com Master Power, Mudo, Presets ("Harman Acoustic", "Bass Boost", "Vocal Clarity") e Sair, com despacho via dispatcher e sincronização de eventos com a UI.
    - `src-ui/src/components/SettingsModal.tsx`: Modal Dark Fluent com toggle de inicialização com o Windows, badge de persistência atômica NTFS e dismiss por tecla Escape (UX-DR11).
    - `src-ui/src/components/Titlebar.tsx`: Botão de Settings (ícone engrenagem) abrindo modal de preferências.
    - `src-ui/src/App.tsx`: Restauração automática de preferências e volume no carregamento, auto-save com debounce de 250ms e captura de posição da janela, e listeners para eventos da bandeja do sistema.
    - Suíte de 78 testes passando no workspace com zero warnings e build Vite do frontend concluído em 1.68s.
    - Registro de ADR-015 no `.bmad/04-DECISION-LOG.md`.
    - Atualização formal de `sprint-status.yaml`, `03-TASK-BACKLOG.md` e spec de Story 1.7 (`status: done`).
18. **Implementação e Verificação Completa da Story 1.8 (`1-8-virtual-audio-device-bridge-rea-reverse-engineering`):**
    - `crates/open-boom-core/src/types.rs`: Extensão do contrato de telemetria com `using_virtual_bridge: bool` em `TelemetrySnapshot`.
    - `crates/open-boom-wasapi/src/device.rs`: Métodos `is_virtual_device_name` e `find_virtual_device` com heurísticas de identificação de virtual audio adapters (Boom Audio, VB-Audio Cable, VAC, VoiceMeeter, etc.) e suporte a dispositivos sintéticos para isolamento determinístico. Detecção validada em tempo real com o dispositivo físico `Boom Audio` instalado no host.
    - `crates/open-boom-wasapi/src/virtual_bridge.rs`: Componente `VirtualAudioBridge` com suporte a Modo A (Virtual Sink) e Modo B (Loopback Fallback com exclusão de árvore de processos), garantindo zero eco acústico e comutação graciosa sub-500ms sob invalidação de hardware (`AUDCLNT_E_DEVICE_INVALIDATED`).
    - `crates/open-boom-wasapi/src/coordinator.rs`: Integração do `VirtualAudioBridge` ao `AudioEngineCoordinator`, sincronização com `telemetry_snapshot()` e métodos de ciclo de vida e comutação instantânea.
    - `crates/open-boom-wasapi/tests/virtual_bridge_tests.rs`: 6 testes unitários cobrindo heurísticas de nomes, detecção de Modo A, fallback para Modo B, invalidation em tempo de execução e telemetria do coordenador.
    - `.bmad/evidence/rea-boom-engine-reverse-engineering-report.md`: Relatório completo de engenharia reversa REA documentando os pontos de entrada exportados de `BoomEngine.dll` (`CreateAudioProcessor`, `CreateSession`, `GetEqualizerPresetGains`), estruturas de classes internas (`AVBoomAudioProcessor`, `AVCSessionBase`, `Choreographer`), vetorização SIMD de `soundtouch::FIRFilterSSE`, buffer FIFO (`AVFIFOProcessor`), arquitetura de driver SYSVAD `boomvad.sys` e camada gerenciada `Audio.dll` (`IsBoom`, `IsBoomDeviceStereo`, `SwitchToBoom` via `IPolicyConfigVista`).
    - `src-ui/src/components/StatusBar.tsx`: Renderização do badge de topologia de captura no dock de status (`Virtual Sink (Echo-Free)` em verde ou `Loopback Mode` em âmbar) com ícone `Radio`.
    - `src-ui/src/App.tsx`: Polling de telemetria atualizado para capturar `using_virtual_bridge` e repassar ao `StatusBar`.
    - Suíte de 84 testes passando no workspace com zero warnings e build Vite do frontend concluído em 1.71s.
    - Registro de ADR-016 no `.bmad/04-DECISION-LOG.md`.
    - Atualização formal de `sprint-status.yaml` (Epic 1 agora 100% `done`), `03-TASK-BACKLOG.md` e spec de Story 1.8 (`status: done`).
19. **Implementação e Conclusão Completa do Epic 2: 3D Surround Soundstage & Spatial Calibration (Stories 2.1, 2.2, 2.3, 2.4):**
    - **Story 2.1 (`2-1-binaural-7-1-hrtf-convolution-engine-down-folding-matrix`):** Motor `Spatializer3D` em `crates/open-boom-core` com tabelas KEMAR HRTF 64 taps, atraso ITD (0.2–0.7ms), pinna rear notch a 6 kHz, buffers circulares duplos de 128 amostras, convolução SIMD SSE2 (`dot_product_64_sse`), upmix estéreo Mid/Side e downfold binaural com < 25% de orçamento de CPU.
    - **Story 2.2 (`2-2-interactive-circular-3d-surround-dial-control`):** `SoundstageDial.tsx` com diâmetro 192px/144px, varredura de 300°, física polar $\text{atan2}$, pips ciano em 25/50/75%, snap duplo-clique 50%, controle de teclado ARIA slider, 8 satélites e botão de calibração de soundstage.
    - **Story 2.3 (`2-3-8-channel-virtual-speaker-configurator-soundstage-modal`):** `SoundstageConfiguratorModal.tsx` com perspectiva top-down do ouvinte, anéis orbitais concêntricos, 8 nós virtuais com alternância de mudo e traço diagonal vermelho, faders de ganho relativo $\pm 6.0\text{ dB}$ (resolução 0.1 dB), atalho global `Ctrl + Shift + S`, persistência em `AppConfig` (`speaker_muted`, `speaker_gains_db`, `speaker_azimuths_deg`).
    - **Story 2.4 (`2-4-spatial-virtual-speaker-angular-repositioning`):** Arraste polar dos nós ao longo dos anéis orbitais com captura de ponteiro (`setPointerCapture`), supressão de clique acidental de mudo para arrastos $> 4\text{ px}$, badge flutuante de coordenadas azimutais $\theta$, interpolação dinâmica e contínua dos coeficientes KEMAR entre 9 âncoras horizontais sem dropouts ou estalos, e botão "Reset Speaker Positions" para ângulos nominais ITU-R BS.775.
    - 95 testes unitários e de integração verdes no workspace Cargo com zero warnings e build Vite concluído em 1.68s.
    - Registro de ADR-017 no `.bmad/04-DECISION-LOG.md`.
    - Atualização de `sprint-status.yaml` (`epic-2: done`), `03-TASK-BACKLOG.md` e specs de Story 2.1 a 2.4 (`status: done`).
20. **Correção de Conflito de Concorrência COM MTA/STA no Windows (ADR-018) & AudioStreamPump / Roteamento Inteligente (ADR-019, ADR-020, ADR-021):**
    - Isolamento de chamadas COM (`enumerate_windows_devices` e `set_windows_default_device`) em worker threads dedicadas, eliminando `RPC_E_CHANGED_MODE`.
    - `AudioStreamPump` em tempo real sob agendamento MMCSS "Pro Audio" com comutação automática para `Boom Audio` no Master Power ON e restauração imediata ao dispositivo físico (`HyperX Cloud II Wireless`) no OFF/Exit.
21. **Implementação e Verificação Completa da Story 3.1 (`3-1-31-band-biquad-filter-engine-direct-form-ii-transposed-topol`):**
    - `crates/open-boom-core/src/dsp/biquad.rs` e `graphic_eq_stage.rs`: 31 bandas ISO 1/3-oitava (20 Hz a 20 kHz), RBJ peaking EQ ($Q=4.31$), Direct Form II Transposed, estabilidade Schur-Cohn ($|\text{pole}| < 1.0$), suavização de 32 amostras (~0.67ms) e THD+N < 0.005%.
22. **Implementação e Verificação Completa da Story 3.2 (`3-2-dual-mode-equalizer-studio-drawer-10-band-vs-31-band-spline`):**
    - `src-ui/src/components/EqualizerStudioDrawer.tsx`: Gaveta acelerada por hardware (`Ctrl + Shift + E`), toggle 10-Band vs 31-Band preservando curvas, faders de 180px com precisão 5x (`Shift`), detent magnético $\pm 0.3\text{ dB}$, reset duplo-clique e canvas Bezier spline ciano (`FR10`, `FR12`, `UX-DR7`).
23. **Implementação e Verificação Completa da Story 3.3 (`3-3-offline-autoeq-headphone-database-indexing-instant-fuzzy-sea`):**
    - `crates/open-boom-storage/assets/autoeq_database.bin.gz` (209 KB < 5 MB) com 4.200 perfis de fones e IDs 100% únicos, busca fuzzy in-memory em 0.04ms (< 50ms) e `AutoEqPopover.tsx` acessível (`FR13`, `FR14`, `UX-DR8`).
24. **Implementação e Verificação Completa da Story 3.4 (`3-4-decoupled-parametric-autoeq-stage-auto-gain-preamp-compensat`):**
    - `ParametricStage` até 12 biquads (`Peaking`, `LowShelf`, `HighShelf`) posicionado antes do `GraphicEqStage`, atenuação automática de pré-amp $A_{\text{preamp}} = -\max(0, \max_i(G_i))$ e comandos SPSC lock-free (`FR15`, `AD-6`).
25. **Implementação e Verificação Completa da Story 3.5 (`3-5-genre-factory-presets-custom-preset-persistence`) & Conclusão do Epic 3:**
    - `crates/open-boom-storage/src/preset_store.rs`: Struct `EqPreset`, 15 presets de fábrica calibrados (`Flat`, `Bass Boost`, `Treble Boost`, `Acoustic`, `Classical`, `Electronic`, `Gaming`, `Hip-Hop`, `Jazz`, `Movie`, `Pop`, `Rock`, `Vocal`, `Lounge`, `Deep`), campo `active_preset_name` em `AppConfig`, e CRUD atômico de presets customizados em `%APPDATA%\OpenBoom\presets\*.json` via `.tmp` + `fs::rename` (`MoveFileExW`) com sanitização de slugs e proteção de presets de fábrica (`FR11`, `AD-10`).
    - `src-tauri/src/lib.rs`: Comandos IPC `get_eq_presets`, `save_custom_eq_preset`, `rename_custom_eq_preset` e `delete_custom_eq_preset`.
    - `src-ui/src/components/PresetSelectorPopover.tsx`, `EqualizerPreview.tsx`, `EqualizerStudioDrawer.tsx` e `App.tsx`: Seletor de presets de 1 clique, detecção de estado modificado com asterisco (`Rock *`) quando os faders divergem $> 0.05\text{ dB}$, criação/renomeação/exclusão de presets customizados e independência total com o estágio AutoEq (`EXPERIENCE.md §5.2`, `AD-6`).
    - `crates/open-boom-storage/tests/storage_tests.rs` e `src-ui/src/test/PresetSelector.test.tsx`: 122 testes Cargo passando no workspace, 47 testes de UI passando e build Vite concluído em 1.76s.
    - Registro de ADR-025 em `.bmad/04-DECISION-LOG.md` e marcação de `epic-3: done` em `sprint-status.yaml`.
26. **Correção de Crash `0xc0000005` (`STATUS_ACCESS_VIOLATION`) no Startup COM `IPolicyConfig` e Ativação Live do `AudioStreamPump` (ADR-026):**
    - Identificada a causa-raiz do `STATUS_ACCESS_VIOLATION` durante `npm run dev`: em `AudioDeviceManager::set_windows_default_device_worker` (`crates/open-boom-wasapi/src/device.rs`), os objetos COM `unk: IUnknown` e `_policy_guard: IUnknown` sofriam `Drop` (`IUnknown::Release()`) **após** `CoUninitialize()` ter encerrado o apartamento COM e descarregado `AudioSes.dll` da thread worker isolada.
    - Encapsulado o ciclo de vida dos ponteiros COM em closure interna antes de `CoUninitialize()`, alinhado com `Audio.CPolicyConfigVistaClient.SetDefaultDevice` (`eConsole`, `eMultimedia`, `eCommunications`) inspecionado via REA MCP em `Boom3D\Audio.dll`.
    - Corrigido `allow_synthetic: !auto_route` em `src-tauri/src/lib.rs` para ativar `run_windows_pump` em `AppState::new_runtime()`, e adicionado suporte multi-canal seguro (1ch/2ch/7.1ch) em `crates/open-boom-wasapi/src/stream_pump.rs`.
27. **Implementação e Conclusão Completa do Epic 4: Psychoacoustic Audio Sweeteners & Dynamic Effects Rack (Stories 4.1 a 4.5):**
    - **Story 4.1 (`4-1-5-effect-quick-rack-strip-ui-component`):** Componente `EffectsRackStrip.tsx` no `src-ui` com grid de 5 cartões (Ambience, Fidelity, Night Mode, Spatial, Pitch), switches de bypass por módulo com brilho ciano neon, mini-sliders verticais com Cascadia Code monospace, tooltips técnicos e boundary de Pitch Fase 2 com badge "Soon" e bloqueio de despacho para o DSP core (`FR19`, `FR20`, `UX-DR6`, `PRD §6.2`).
    - **Story 4.2 (`4-2-ambience-reverb-module-schroeder-moorer-fdn-dsp`):** `AmbienceProcessor` (`crates/open-boom-core/src/dsp/ambience.rs`) com topologia Schroeder-Moorer / FDN: 4 filtros comb com amortecimento passa-baixa de 1 polo (delays de 1116 a 1356 amostras), 2 all-pass diffusers em série (delays de 225 e 556 amostras, ganho 0.5), early reflections cruzadas assimétricas, descorrelação estéreo, interpolação suave de parâmetros (~200 amostras) e zero alocações na heap (`FR21`).
    - **Story 4.3 (`4-3-fidelity-psychoacoustic-harmonic-exciter-module`):** `FidelityProcessor` (`crates/open-boom-core/src/dsp/fidelity.rs`) com isolamento de altas frequências (> 3.2 kHz) via Butterworth HP de 2ª ordem, DC-blocker de 1 polo, síntese de 2º harmônico ($x^2$) e 3º harmônico ($x^3$), saturação suave tanh-like, filtro air shelf (> 10 kHz), latência zero absoluta (0.0 ms) e bypass bit-idêntico (`FR22`).
    - **Story 4.4 (`4-4-night-mode-dynamic-range-compressor-drc`):** `NightModeProcessor` (`crates/open-boom-core/src/dsp/night_mode.rs`) com compressor de faixa dinâmica soft-knee: ataque rápido de 2.5 ms (< 5 ms), liberação de 120 ms, atenuação de picos barulhentos em até -18 dB, boost de diálogo de baixo nível em até +6 dB, gate de silêncio a -55 dBFS impedindo amplificação de ruído/hiss, e ganho estéreo simétrico (`FR23`).
    - **Story 4.5 (`4-5-spatial-stereo-expander-crossfeed-matrix`):** `SpatialProcessor` (`crates/open-boom-core/src/dsp/spatial.rs`) com matriz Mid/Side ($M = 0.5(L + R), S = 0.5(L - R)$), separação passa-baixa de 1 polo a 200 Hz para isolar sub-graves, centralização mono de graves ($S_{low} \times 0.0$), expansão de largura das frequências médias/agudas ($1.0 + 1.2 \times \text{amount}$) e invariante matemático de compatibilidade mono perfeita (sinal puramente mono $L=R \implies S=0 \implies$ saída bit-idêntica à entrada) (`FR20`).
    - **Pipeline e Integração de Sistema:** `AudioProcessor` (`crates/open-boom-core/src/dsp/pipeline.rs`) atualizado com `Stage 4: SweetenersRack` entre Equalizador Gráfico (Stage 3) e Master Limiter (Stage 5); comando lock-free `AudioCommand::SetSweetener` consumido via SPSC queue; persistência atômica dos 5 efeitos em `%APPDATA%\OpenBoom\config.json` via `AppConfig` (`crates/open-boom-storage/src/preset_store.rs`).
    - **Verificação de Testes:** 17 testes de integração/DSP em `sweeteners_tests.rs` (140 testes Cargo verdes no workspace) e 9 testes de UI em `EffectsRack.test.tsx` (56 testes UI verdes no suite `npm test` do `src-ui`).
    - Registro de ADR-027 em `.bmad/04-DECISION-LOG.md`, specs 4.1 a 4.5 marcadas como `done` e sprint atualizado.
28. **Implementação e Verificação Completa da Story 5.1 (`5-1-windows-core-audio-session-manager-dedicated-com-mta-thread`):**
    - `crates/open-boom-core/src/types.rs`: Struct `ProcessAudioSession` (`process_id`, `name`, `volume`, `is_muted`, `is_active`) com suporte a serialização `serde`.
    - `crates/open-boom-wasapi/src/session_manager.rs`: `SessionManager` operando em thread worker dedicada (`"open-boom-session-mgr"`) inicializada com `CoInitializeEx(None, COINIT_MULTITHREADED)`, comunicando via canais `std::sync::mpsc`.
    - Integração de `IAudioSessionManager2`, `IAudioSessionEnumerator`, `IAudioSessionControl2` e `ISimpleAudioVolume`, inspeção segura de executáveis via `OpenProcess` e `QueryFullProcessImageNameW` com liberação rigorosa de handles, tratamento de sons do sistema (`IsSystemSoundsSession()`), poda de sessões expiradas (`AudioSessionStateExpired`), e modo sintético determinístico (`with_forced_synthetic` e `with_synthetic_fallback`).
    - `src-tauri/src/lib.rs`: `SessionManager` integrado ao `AppState` de Tauri e comandos IPC `get_audio_sessions`, `set_app_volume` e `set_app_mute` expostos no `invoke_handler`.
    - `crates/open-boom-wasapi/tests/session_manager_tests.rs` e `src-tauri/src/lib.rs`: 5 testes unitários e teste de integração em `src-tauri` passando com sucesso (146 testes Cargo e 56 testes UI verdes).
    - Registro do ADR-028 em `.bmad/04-DECISION-LOG.md`, spec 5.1 marcada como `done` e `sprint-status.yaml` atualizado.

## O que está em andamento agora
* **Story 5.1 Concluída com Sucesso (`done`)!**
* **Próximo no Backlog:** **Story 5.2: Per-Application Mixer Drawer UI & Individual Process Faders** (`5-2-per-application-mixer-drawer-ui-individual-process-faders`).

## Bloqueios / Pendências
* Nenhum bloqueio ativo. Todos os 146 testes Cargo e 56 testes UI estão passando com 100% de sucesso.
