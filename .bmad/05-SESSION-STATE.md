# OpenBoom - Session State

## Última Atualização
* **Data/Hora:** 13/10/2026 - 13:55
* **Sprint Ativa:** **Sprint 9 — Refinamentos de UI, Ajustes de Update & Robustez i18n (ÉPICO 9 100% CONCLUÍDO)**
* **Tarefa Atual:** Conclusão de todas as histórias do Épico 9: Story 9-5 (Aviso de Privacidade & Esclarecimento de Microfone) e Story 9-6 (Modo Inicial Desarmado Standby). Suíte de 228 testes Rust e 118 testes UI verdes. Mandatory Verification Gate aprovado. Pronto para commit, push e aguardar sinal do usuário para release.


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
29. **Implementação e Verificação Completa da Story 5.2 (`5-2-per-application-mixer-drawer-ui-individual-process-faders`):**
    - `src-ui/src/components/AppMixerDrawer.tsx`: Componente de gaveta deslizante Fluent Dark com linhas `app-mixer-row` de 44–48px, badges temáticos por aplicativo (Spotify, Discord, navegadores, jogos, players), nomes amigáveis de executáveis, indicador verde pulsante de áudio ativo (`bg-meter-green animate-pulse`), sliders de 0% a 150% com snap duplo-clique 100% (0.0 dB) e botões de mute desacoplados.
    - `src-ui/src/components/Titlebar.tsx`: Botão de abertura "App Volumes" no cabeçalho com ícone `Layers`, realce ativo ciano e atalho `Ctrl + Shift + M`.
    - `src-ui/src/App.tsx`: Gerenciamento de estado de sessões, despacho via Tauri IPC (`set_app_volume`, `set_app_mute`), polling dinâmico a cada 1000ms quando a gaveta está visível e atalho global de teclado `Ctrl + Shift + M`.
    - `src-ui/src/test/AppMixer.test.tsx` e `run.mjs`: 9 testes unitários de renderização, mapeamento perceptual de decibéis, faders e acessibilidade aprovados com 100% de sucesso (65 testes UI verdes no workspace).
    - Registro do ADR-029 em `.bmad/04-DECISION-LOG.md`, spec 5.2 marcada como `done` e `sprint-status.yaml` atualizado.
30. **Implementação e Conclusão Completa da Story 5.3 e Finalização do Epic 5 (`5-3-persistent-application-volume-preferences-automatic-session`):**
    - `crates/open-boom-core/src/types.rs`: Struct `AppVolumeProfile` (`process_name`, `volume`, `is_muted`) com serialização serde e construtor com clamp defensivo (0.0 a 1.5).
    - `crates/open-boom-storage/src/preset_store.rs`: Métodos atômicos `load_app_volumes`, `load_app_volume_list`, `save_app_volumes`, `save_app_volume_profile` e `delete_app_volume_profile` para `%APPDATA%\OpenBoom\app_volumes.json` utilizando staging `.tmp` e substituição atômica `MoveFileExW` / `fs::rename` (`AD-10`), com normalização insensível a maiúsculas/minúsculas e recuperação graciosa de JSON corrompido ou formato array.
    - `crates/open-boom-wasapi/src/session_manager.rs`: `SessionManager` atualizado com cache de perfis persistidos (`stored_profiles`), rastreamento de ciclo de vida de PIDs (`restored_pids`), e gancho de recuperação automática em `enumerate_sessions`: qualquer sessão recém-descoberta que coincida com um perfil memorizado tem seu volume e mudo restaurados automaticamente na thread COM MTA via `ISimpleAudioVolume::SetMasterVolume` e `SetMute`. Processos terminados têm seus PIDs podados para permitir restauração imediata em caso de reinicialização (`FR23`, `UJ-4`).
    - `src-tauri/src/lib.rs`: `AppState` inicializa `SessionManager` com os perfis carregados de `%APPDATA%\OpenBoom\app_volumes.json`; handlers IPC `set_app_volume` e `set_app_mute` atualizam o cache e salvam atomicamente no disco; comandos `get_app_volume_profiles` e `reset_app_volume_profile` expostos no `invoke_handler`.
    - `src-ui/src/components/AppMixerDrawer.tsx` e `src-ui/src/App.tsx`: Botão de reset rápido para perfis customizados (ícone `RotateCcw`), nota de rodapé garantindo persistência automática, e handler `handleResetAppVolume` no `App.tsx`.
    - **Verificação Completa de Testes:** 14 testes em `storage_tests.rs`, 6 testes em `session_manager_tests.rs`, 11 testes em `src-tauri` (174 testes Cargo verdes no workspace total), e 11 testes em `AppMixer.test.tsx` (67 testes de UI verdes no `npm test` do `src-ui`). Build Vite completado em 1.79s sem erros.
    - Registro do ADR-030 em `.bmad/04-DECISION-LOG.md`, spec 5.3 marcada como `done`, e status de `epic-5` marcado como `done` em `sprint-status.yaml`.
31. **Refinamento de Responsividade, Flexbox do Rack e Eliminação da Sobra do Rodapé:**
    - **Eliminação da Sobra do Rodapé:** Configurado `html, body, #root { height: 100%; width: 100%; }` em `index.html` e `tokens.css`, removido `justify-between` e `shadow-2xl` do container principal `App.tsx`. O rodapé agora fica 100% ancorado e rente à borda inferior da janela sem vazamento de fundo ou margens órfãs. Altura do `StatusBar` compactada para `h-8` (32px).
    - **Fim do Aperto ("Socado") no Layout:** Redesenhado o layout de `main`: o disco do 3D Surround agora ocupa largura fixa dedicada de `w-[280px] shrink-0`, enquanto todo o espaço horizontal restante (`flex-1 min-w-0`) é dedicado ao Equalizador e ao Rack de Efeitos. Cada card de efeito agora ganha > 124px de largura (aumento de +63%), dando respiro total aos títulos, switches e sliders.
    - **Geometria Mínima da Janela:** Aumento da resolução mínima para **960 × 580 px** e padrão para **1000 × 620 px** (`tauri.conf.json`, `App.tsx`, `preset_store.rs`), garantindo que Titlebar, Rack e Dock de telemetria funcionem sem qualquer sobreposição.
    - **Física Polar e Animação do 3D Surround Dial:** Removido `transition-all duration-75` do path SVG do arco ciano (`SoundstageDial.tsx`), eliminando defasagem e descolamento do arco em relação ao beacon branco em movimentos rápidos do mouse; corrigido limiar de snap polar da dead-zone inferior de 60° (ponto médio 330°), impedindo saltos abruptos para 100% ao mover próximo do 0%.
32. **Profissionalização de Release, CI/CD GitHub Actions, Artefatos .EXE e Branding do README:**
    - **Banner Oficial de Alta Fidelidade:** Arte gerada com iluminação cinematográfica, ondas ciano neon e HUD holográfico salva em `assets/banner.png`.
    - **README Completo e Profissional:** Documentação técnica completa com badges dinâmicos, resumo de arquitetura sem drivers proprietários, tabela de atalhos globais de teclado, instruções de compilação passo a passo e tabela comparativa de artefatos de download (Instalador NSIS vs Executável Portátil).
    - **Licenciamento Open-Source:** Inclusão formal de arquivos `LICENSE` (MIT) e `LICENSE-APACHE`.
    - **Pipeline CI/CD GitHub Actions (`.github/workflows/`):**
      - `release.yml`: Acionado no push de tags `v*.*.*` (ou manual via `workflow_dispatch`). Compila em runner `windows-latest`, executa a suíte de 174 testes Cargo + 67 testes UI, gera o instalador NSIS `OpenBoom_x.x.x_x64-setup.exe` e o executável standalone portátil `OpenBoom-vX.Y.Z-portable-x64.exe`, calcula checksums SHA-256 (`SHA256SUMS.txt`) e publica automaticamente no GitHub Releases com notas de versão geradas.
      - `ci.yml`: Validação contínua em PRs e pushes para `main`.
    - **Automação de Tag de Versão:** Script PowerShell `scripts/release.ps1` e comando `npm run release:tag` para validação de integridade e criação segura de tags anotadas.

33. **Correção Definitiva de Estalos de Sub-Graves ("Tiros no Som") e Audio Buffer Stability Guard:**
    - **Diagnóstico & Causa Raiz:** Faixas com sub-graves densos e alta compressão/RMS (ex: *M.O.O.N - Dust*, Rock moderno) causavam estalos devido a: (1) buffer WASAPI de 10ms sofrendo underrun por micro-jitter do agendador do Windows, (2) contenção transitória de lock pela telemetria da UI a cada 60ms copiando dry audio, e (3) modulação espúria de ganho a 100 Hz no limitador terminal durante a passagem por zero de ondas de 50 Hz.
    - **Correções no Motor de Áudio (`open-boom-core` e `open-boom-wasapi`):**
      - Peak-Hold dinâmico de 5ms (240 amostras a 48 kHz) no limitador lookahead, mantendo o envelope estável através de cruzamentos por zero de sub-graves sem distorção harmônica.
      - Buffer Guard WASAPI estendido para 25ms (`250_000` hns), com loop de escrita de até 25 retries e `spin_loop()`, eliminando perda de pacotes e quedas para áudio dry.
      - Comando `AudioCommand::SetBufferGuard(bool)` e IPC Tauri `set_buffer_guard` com reinicialização a quente (`hot-restart`) do stream pump.
    - **Interface de Usuário (`SettingsModal.tsx` & `App.tsx`):**
      - Switch opcional "Audio Buffer Stability Guard" na modal de configurações, ativado por padrão (`buffer_guard_enabled: true`).
      - Badge ciano "Proteção Ativa: Zero Estalos & Transientes Suaves" quando habilitado.
      - Banner de alerta vermelho proeminente ("Aviso de Risco Acústico & Fadiga Mecânica") exibido caso o usuário desative a proteção (modo 10ms agressivo).
      - Persistência atômica das preferências em `%APPDATA%\OpenBoom\config.json`.
    - **Verificação Completa de Testes:** 175 testes Cargo verdes no workspace total e 71 testes React/Vite verdes no frontend (incluindo `SettingsModal.test.tsx`). ADR-031 registrado no `.bmad/04-DECISION-LOG.md`.
35. **Refinamento Responsivo, Proporção 60/40 e Estabilidade de Layout na Interface:**
    - **Proporção Vertical 60/40 no Rack Direito:** O painel direito foi reestruturado com Flexbox proporcional exato: o card de Equalização (`EqualizerPreview`) ocupa `flex-[6] min-h-0` (60% da altura vertical) e o strip de módulos de efeitos (`EffectsRackStrip`) ocupa `flex-[4] min-h-0` (40% da altura vertical), eliminando o aspecto excessivamente espichado dos cards de efeitos e dando destaque adequado ao display do EQ.
    - **Aumento da Geometria Mínima e Padrão da Janela:** Tamanho mínimo configurado para **1000 × 600 px** e padrão para **1040 × 640 px** (`tauri.conf.json`, `App.tsx`, `preset_store.rs`, e asserções sincronizadas em `lib.rs`), prevenindo compressão excessiva em telas HD.
    - **Eliminação de Layout Shift no Headroom do StatusBar:** Aplicado `shrink-0`, `whitespace-nowrap` e `min-w-[56px] tabular-nums text-right` no badge de Headroom, eliminando quebras de linha durante a oscilação de telemetria a cada 60ms.
    - **Física Polar Robusta do SoundstageDial:** Adicionado clamping na dead-zone angular inferior e rastreamento contínuo em `intensityRef`, impedindo que movimentos rápidos do mouse saltem acidentalmente de 0% para 100%.
    - **Verificação Completa de Qualidade:** Build do frontend (`npm run build`) concluído com 0 erros, todos os 78 testes de UI passando e todos os testes Cargo passando.

36. **Suporte Completo a Internacionalização (i18n: pt-BR, en, es) e Lançamento da Versão v0.2.0:**
    - **Arquitetura de i18n Tipada e Nativa:** Desenvolvido sistema em Contexto React sem dependências externas (`src-ui/src/i18n/`), com contrato TypeScript estrito (`TranslationDictionary`).
    - **Dicionários Completos:** Dicionários com 100% de paridade entre Inglês (`en`), Português do Brasil (`pt-BR`) e Espanhol (`es`), abrangendo todos os componentes do app.
    - **Detecção do Sistema por Padrão:** Opção `"system"` inspeciona o idioma do host (`navigator.language`) e define automaticamente `pt-BR`, `es` ou `en`.
    - **Acessibilidade & Menus de Troca:**
      - Popover dropdown na Titlebar com ícone de Globo e badges de idioma (`LanguageMenu.tsx`).
      - Grade segmentada acessível na modal de preferências (`SettingsModal.tsx`).
      - Submenu nativo de bandeja do sistema no Windows (System Tray em `src-tauri/src/lib.rs` emitindo `tray-language-changed`).
    - **Persistência Atômica:** Preferência de idioma salva em `%APPDATA%\OpenBoom\config.json` via campo `language: Option<String>` em `AppConfig`.
    - **Suíte de Testes & Verificação de Qualidade:**
      - Nova suíte de testes de i18n (`src-ui/src/test/I18n.test.tsx`) validando mapeamento de locales, paridade estrita dos dicionários e renderização de provedor e badges.
      - 78 testes de UI do frontend 100% verdes no `npm test`.
      - 175 testes Cargo 100% verdes no workspace.
      - Build de produção (`npm run build`) validado sem erros.
    - **Lançamento & Versionamento:** Bump de versão para `0.2.0` no `package.json`, `src-ui/package.json`, `Cargo.toml`, e `tauri.conf.json`.

37. **Implementação do Motor DSP de Modulação Azimutal Orbital 8D & Rotação Contínua 7.1 em Sub-Blocos (Story 6.1):**
    - **Tipos de Domínio e Contratos POD SPSC (`crates/open-boom-core/src/types.rs`):**
      - Definição do enum `OrbitDirection` (`Clockwise = 0`, `CounterClockwise = 1`, `PingPong = 2`) com derivações `Copy`, `POD` e `Serialize`/`Deserialize`.
      - Comando POD `AudioCommand::SetOrbit8D { enabled, direction, period_sec, depth }` para transferência lock-free na fila circular SPSC.
      - Extensão de `TelemetrySnapshot` com `orbit_phase_deg: f32` e `orbit_enabled: bool`.
    - **Motor de Órbita Contínua (`crates/open-boom-core/src/dsp/spatializer_3d.rs`):**
      - Acumulador de fase em dupla precisão (`f64`) para eliminação estrita de drift por arredondamento ao longo de milhões de amostras.
      - Recálculo dinâmico de coeficientes HRTF KEMAR a cada 32 frames de áudio (`ORBIT_SUB_BLOCK_SAMPLES = 32`, ~0.67ms a 48 kHz), economizando 97% de CPU e mantendo passo angular de ~0.12° (imperceptível sob limiar MAA).
      - Deslocamento azimutal rígido da constelação virtual 7.1: $\theta_k(t) = \text{wrap}_{[-180, 180]}(\text{speaker\_azimuth}[k] + \theta_{\text{orbit}}(t) \cdot \text{orbit\_depth})$.
      - Isolamento do canal LFE (`CH_LFE` inalterado com HRTF mono estático) contra desequilíbrio e fadiga de fase nos sub-graves.
      - Transição angular suave sem saltos ao comutar direção (`sync_progress_to_direction`).
      - Restauração suave dos HRTFs estáticos nominais ao desativar o modo 8D (`restore_static_hrtfs`).
    - **Integração no Processador de Áudio (`crates/open-boom-core/src/dsp/pipeline.rs`):**
      - Despacho de comando `AudioCommand::SetOrbit8D` com ativação automática do spatializer 3D e preenchimento contínuo na telemetria.
    - **Suíte de Testes Dedicada (`crates/open-boom-core/tests/orbital_tests.rs`):**
      - 13 testes de integração cobrindo rotação horária (+360° em 12s), anti-horária (-360° em 12s), pêndulo senoidal ($\pm 180^\circ \sin(2\pi t / T)$ sem descontinuidades), cadência estrita de 32 frames, isolamento LFE, profundidade orbital, sanitização de NaN/Inf, reversão contínua de direção, restauração ao desativar e orçamento de processamento em tempo real.
    - **Verificação de Qualidade Global:**
      - 188 testes Cargo passando no workspace com 100% de sucesso e zero warnings.
      - ADR-034 registrado no `.bmad/04-DECISION-LOG.md`.

38. **Implementação do Botão Rápido 8D Orbit & Indicador Trajetória Neon no SoundstageDial (Story 6.2):**
    - **Projeção Polar e Mapeamento de Azimute Acústico (`SoundstageDial.tsx`):**
      - Função `azimuthToCartesian(azimuthDeg, r, cx, cy)` mapeando coordenadas acústicas ($0^\circ = \text{Norte/Frente}$, $+90^\circ = \text{Leste/Direita}$, $-90^\circ = \text{Oeste/Esquerda}$, $\pm 180^\circ = \text{Sul/Traseira}$) perfeitamente à pista de raio $R = 84\text{ px}$ do dial SVG ($192 \times 192\text{ px}$) com sanitização rigorosa de NaN e Inf.
      - Cometa orbital neon desenhado com núcleo branco quente, disco ciano neon (`#06b6d4`), halo pulsante e 3 partículas de rastro desvanecente ($\theta - 7^\circ, \theta - 14^\circ, \theta - 21^\circ$), acompanhado por anel orbital guia pontilhado.
      - Subtítulo do botão usinado central comutado para `"8D ORBIT ACTIVE"` em destaque ciano neon.
    - **Botão Rápido Acessível e Reorganização do Strip de Controles (`SoundstageDial.tsx`):**
      - Botão pill `"8D Orbit"` com ícone `<Orbit />` animado (6s de rotação), `aria-pressed`, `role="button"`, suporte a teclado (Enter/Space) e brilho neon ciano (`shadow-[0_0_12px_rgba(56,189,248,0.45)]`).
      - Grid de controles balanceado em 2 linhas (Linha 1: Active/Bypassed e 8D Orbit; Linha 2: Calibrate Soundstage), eliminando qualquer aperto ou overflow na largura de 280px.
    - **Comando IPC Tauri e Despacho Lock-Free (`src-tauri/src/lib.rs`):**
      - Comando `set_orbit_8d(enabled, direction, period_sec, depth)` exposto no Tauri e registrado em `generate_handler!`, despachando `AudioCommand::SetOrbit8D` pela fila SPSC lock-free com defaults calibrados (Clockwise, 12.0s, 0.85 depth).
      - Teste de integração `test_set_orbit_8d_command_dispatch` validando consumo na fila SPSC.
    - **Sincronização de Telemetria e Auto-Engajamento (`App.tsx`):**
      - Taxa de polling de `get_telemetry` aumentada para 33ms (~30.3 fps), sincronizando o cometa em tempo real a $\ge 30\text{ fps}$.
      - Auto-engajamento do spatializer 3D ao ligar 8D orbit com surround desativado.
    - **Internacionalização e Suíte de Testes UI (`SoundstageDial.test.tsx`):**
      - Chave `orbit8D` registrada em `types.ts`, `en.ts`, `pt-BR.ts` e `es.ts` mantendo 100% de paridade nos dicionários.
      - Nova suíte de testes `SoundstageDial.test.tsx` com 7 asserções cobrindo projeção polar, sanitização, renderização do cometa, acessibilidade, handlers e contenção na `viewBox="0 0 192 192"`.
    - **Verificação de Qualidade Global:**
      - 193 testes Cargo passando no workspace com 100% de sucesso e zero warnings.
      - 85 testes UI passando no `npm test` do `src-ui`.
      - Build Vite de produção concluído em 1.82s com zero erros.
      - ADR-035 registrado no `.bmad/04-DECISION-LOG.md`.

39. **Implementação do Painel Completo de Parâmetros Orbitais 8D no Modal de Calibração & Persistência Atômica (Story 6.3):**
    - **Extensão do Esquema `AppConfig` com Retrocompatibilidade (`preset_store.rs`):**
      - Adicionados campos `orbit_enabled: bool` (default `false`), `orbit_direction: OrbitDirection` (default `Clockwise`), `orbit_period_sec: f32` (default `12.0`) e `orbit_depth: f32` (default `0.85`) com atributos `#[serde(default = "...")]` e `AppConfig::default()`.
      - Re-exportado `OrbitDirection` em `open_boom_storage`.
      - Testes em `storage_tests.rs` validando que JSONs legados sem chaves orbitais desserializam para defaults e que configurações 8D customizadas são salvas e restauradas atomicamente.
    - **Internacionalização Rigorosa com 100% de Paridade (`types.ts`, `en.ts`, `pt-BR.ts`, `es.ts`):**
      - Dicionários estendidos com todas as chaves de seletor direcional, rótulos de sliders de período e profundidade, tooltips e acessibilidade ARIA.
    - **Painel de Controle no `SoundstageConfiguratorModal.tsx`:**
      - Card dedicado com seletor direcional de 3 segmentos (Horário, Anti-Horário, Pêndulo) com realce ciano neon.
      - Slider de período de revolução de 2.0s a 30.0s com passo 0.5s, duplo-clique no slider e no badge para resetar em 12.0s e display `XX.Xs`.
      - Slider de profundidade orbital de 0% a 100% com passo 1%, duplo-clique no slider e no badge para resetar em 85% e display `XX%`.
      - Controles habilitados para pré-configuração direta de parâmetros antes da ativação do modo 8D, garantindo UX flexível.
      - Acessibilidade WAI-ARIA completa (`role="radiogroup"`, `role="radio"`, `aria-checked`, `aria-label`, `aria-valuetext`).
      - Localização estrita de todos os elementos, incluindo `resetTrims` no footer do modal com 100% de paridade (pt-BR, en, es).
    - **Integração de Estado e Despacho em `App.tsx`:**
      - Sincronização em tempo real de `orbitDirection`, `orbitPeriodSec` e `orbitDepth`, restauração em `load_app_config` com despacho de parâmetros no boot para o motor de áudio e auto-engajamento de surround se o modo 8D estiver ativo.
      - Correção crítica no `handleToggleOrbit`: validação com `typeof targetEnabled === "boolean"` prevenindo que eventos React `MouseEvent` do HUD corrompam o estado e impeçam o desligamento do modo 8D.
      - Sanitização em tempo de execução de `handleOrbitDirectionChange` com fallback seguro para `"Clockwise"`.
      - Auto-save debounced em 250ms em `save_app_config` e repasse para o modal.
    - **Suíte de Testes UI (`SoundstageConfiguratorModal.test.tsx`):**
      - 10 testes unitários cobrindo renderização, botões de direção com semântica de rádio e classes ativas `bg-accent-cyan/25`, sliders com acessibilidade ARIA, toggle, callbacks, pre-configuração e duplo-clique de reset.
    - **Sanitização Defensiva no Backend (`preset_store.rs`):**
      - `load_config` sanitiza e faz clamp defensivo de `orbit_period_sec` em `[2.0, 30.0]` e `orbit_depth` em `[0.0, 1.0]` contra valores corrompidos ou NaN.
    - **Verificação de Qualidade Global:**
      - 194 testes Cargo passando no workspace com 100% de sucesso e zero warnings.
      - 95 testes UI passando no `npm test` do `src-ui` (10 suítes de teste).
      - Build Vite de produção validado em 2.27s com zero erros.
      - ADR-036 registrado no `.bmad/04-DECISION-LOG.md`.
      - Épico 6 marcado como `done` no `sprint-status.yaml`.

40. **Validação de Execução ao Vivo, Lançamento da Versão v0.3.0 & Publicação no GitHub:**
    - **Teste de Inicialização do Executável (`open-boom.exe`):** Compilação de produção (`cargo build --release -p open-boom`) concluída em 26.6s. Teste de execução ao vivo via PowerShell confirmando que o processo inicia de forma saudável, sem falhas de apartamentos COM, sem crashes de acesso e com alocação e carregamento perfeitos.
    - **Versionamento & Bump para v0.3.0:** Versão incrementada para `0.3.0` em `Cargo.toml`, `package.json`, `src-tauri/tauri.conf.json`, `src-ui/package.json` e `Cargo.lock`.
41. **Execução e Conclusão do Workflow de Arquitetura Multi-Plataforma (`/bmad-architecture`):**
    - Executado workflow em Fast Path para definição da arquitetura de suporte estável a **Linux (PipeWire)** e **macOS (CoreAudio)**.
    - Definido o padrão Hexagonal (Ports and Adapters) desacoplando `open-boom-core` (DSP puro) das APIs nativas de áudio via trait `AudioHostBackend` e trait `SessionManager`.
    - Especificadas as decisões arquiteturais `AD-37` (Trait Abstraction & Adapters), `AD-38` (PipeWire Graph Node & RTKit no Linux), `AD-39` (CoreAudio HAL Virtual Device no macOS), `AD-40` (Session Mixer Multiplataforma), `AD-41` (Resolução de Diretórios e Autostart) e `AD-42` (Matrix CI/CD GitHub Actions).
    - Linter `lint_spine.py` executado com 0 avisos, revisões de Tech Stack e Adversarial concluídas e arquivadas em `reviews/`.
    - Documento `ARCHITECTURE-SPINE.md` finalizado em `_bmad-output/planning-artifacts/architecture/architecture-open-boom-2026-10-12/`.
    - Registro do **ADR-037** no `.bmad/04-DECISION-LOG.md`.
42. **Implementação e Verificação Completa do Sistema de Auto-Update In-App a Custo Zero (Spec 8.1 / ADR-038):**
    - **Persistência Atômica (`crates/open-boom-storage/src/preset_store.rs`):** Adicionados `check_updates_on_startup: bool` (default `true`) e `auto_install_updates: bool` (default `false`) em `AppConfig` com retrocompatibilidade total via `#[serde(default = "...")]`.
    - **Comandos Tauri IPC (`src-tauri/src/lib.rs`):** Implementados `check_for_updates`, `download_and_install_update` e `restart_application` consumindo a API pública do GitHub Releases (`https://api.github.com/repos/JVGS1111/open-boom/releases/latest`), com comparador SemVer sensível a pre-releases (`is_version_newer`), subprocessos PowerShell silenciosos (`CREATE_NO_WINDOW = 0x08000000`) sem janela piscando, saída de erro explícita (`exit 1`) e download temporário isolado por PID (`OpenBoom-Update-{pid}.exe`).
    - **Indicador na Barra de Título (`src-ui/src/components/Titlebar.tsx`):** Botão âmbar pulsante com ícone `<ArrowUpCircle />` exibido quando `hasUpdate === true`, abrindo diretamente `Configurações -> Atualizações`.
    - **Painel de Atualizações (`src-ui/src/components/SettingsModal.tsx`):** Navegação por abas (`Geral` e `Atualizações`), chips de versão atual vs. disponível, caixa de notas de versão, switches de checagem na abertura e atualização automática, e botão de ação unificado no mesmo slot (`Verificar Atualizações`, `Baixar e Instalar` e `Reiniciar Aplicação`).
    - **Internacionalização & Testes (`src-ui/src/i18n/` & `Updater.test.tsx`):** Paridade 100% entre `en`, `pt-BR` e `es`; 104 testes de UI verdes e 194+ testes Cargo verdes.
43. **Implementação e Verificação Completa da Sincronização Bidirecional de Volume com o Windows (Spec 7.1 / ADR-039):**
    - **Serviço COM MTA `EndpointVolumeSync` (`crates/open-boom-wasapi/src/volume_sync.rs`):** Thread dedicada `"open-boom-volume-sync"` gerenciando `IAudioEndpointVolume` e callback `IAudioEndpointVolumeCallback` (`#[windows_core::implement(IAudioEndpointVolumeCallback)]`), com supressão dupla de feedback loop (`OPEN_BOOM_VOLUME_CONTEXT_GUID` + `is_syncing_to_windows: AtomicBool`), caches atômicos lock-free (`cached_volume_bits`, `cached_muted`), resolução de `"default"` para o endpoint de console do Windows e modo sintético para testes headless.
    - **Integração no Backend Tauri (`src-tauri/src/lib.rs`):** `AppState::dispatch_audio_command` centraliza a atualização de `AudioCommand::SetMasterVolume` e `AudioCommand::SetMute`, sincronizando a fila SPSC de áudio e o volume escalar do Windows (`v.clamp(0.0, 1.0)`), emitindo eventos `windows-volume-changed` para o frontend quando o volume/mute do Windows é alterado externamente (barra de tarefas ou teclas de mídia do teclado), e executando `rebind_endpoint` em `handle_master_power_toggle`, `set_audio_device` e `trigger_device_fallback`.
    - **Sincronização Reativa na UI (`src-ui/src/App.tsx` e `src-ui/src/components/Titlebar.tsx`):** Listener para `windows-volume-changed` atualizando `masterVolume` e `isMuted` sem re-invocar `set_master_volume` (prevenindo loops IPC), e desbloqueio automático de mudo (`onToggleMute`) ao arrastar o slider de volume para $> 0$ enquanto mutado.
    - **Verificação Completa de Testes:** 9 testes em `crates/open-boom-wasapi/tests/volume_sync_tests.rs`, testes de despacho em `src-tauri/src/lib.rs` e suíte `src-ui/src/test/TitlebarVolumeSync.test.tsx`. Total global: **212 testes Cargo passando (0 falhas)**, **104 testes UI passando (0 falhas)** e build de produção Vite concluído em 1.90s.
44. **Implementação e Revisão Completa do Seletor de Canal de Captura de Áudio (Driver Boom Audio vs. Emulação Loopback) com Reinicialização Automática (Spec 1.9 / T8.2 / ADR-040):**
    - **Artefato de Implementação BMAD (`_bmad-output/implementation-artifacts/spec-1-9-audio-capture-bridge-mode-selector.md`):** Spec completa com `<intent-contract>`, `I/O & Edge-Case Matrix`, `Code Map`, `Tasks & Acceptance`, `Review Triage Log` (12 achados das 4 lentes de review: 1 high, 7 medium, 4 low — 100% patcheados) e `Auto Run Result`, além de atualização de `_bmad-output/implementation-artifacts/sprint-status.yaml`.
    - **Persistência Atômica (`crates/open-boom-storage/src/preset_store.rs`):** Adicionado `audio_bridge_mode: String` (`"auto"`, `"virtual"`, `"loopback"`) em `AppConfig` com sanitizador `sanitize_audio_bridge_mode` e testes de regressão em `storage_tests.rs`.
    - **Controle Dinâmico de Bridge e StreamPump (`crates/open-boom-wasapi`):** Adicionados `VirtualAudioBridge::set_force_mode`, `AudioEngineCoordinator::set_bridge_force_mode`, `AudioEngineCoordinator::bridge_force_mode` e `StreamPumpConfig::force_loopback` para permitir ignorar drivers virtuais (`Boom Audio`) e forçar captura WASAPI Loopback direto no dispositivo físico (Modo B) ou forçar Virtual Sink (Modo A) com fallback seguro quando o driver não está instalado no host.
    - **Comandos Tauri IPC, Inicialização Unificada e Restauração Limpa antes do Restart (`src-tauri/src/lib.rs`):** Construtor unificado `AppState::with_options(store, startup, auto_route)` (roteando o endpoint padrão do Windows antes de iniciar o `AudioStreamPump`), comandos `set_audio_bridge_mode` e `get_audio_bridge_mode`, sincronização de `audio_bridge_mode` em `save_app_config`, e chamada incondicional a `restore_physical_default_device()` e parada do `AudioStreamPump` em `restart_application` antes de `app.restart()`.
    - **UI e Internacionalização (`SettingsModal.tsx`, `App.tsx`, `i18n`):** Card dedicado nas configurações gerais com `<select>` acessível para os 3 modos, badge permanente do canal ativo (`Mode A` ou `Mode B`), alerta âmbar de eco no Modo B sem driver, guarda de early-return e rollback em caso de erro de IPC, reinicialização automática ao trocar o canal e paridade 100% em `pt-BR`, `en` e `es`.
    - **Verificação Completa de Testes & Smoke Test Local:** **214 testes Cargo passando (0 falhas)**, **106 testes UI passando (0 falhas)**, build Vite em 1.90s e smoke launch local de `open-boom.exe` verificado com sucesso (`PID=18752`, `Responding=True`).
45. **Correção Definitiva do Auto-Update In-App: Seleção Estrita do Instalador NSIS, Download Acelerado via `curl.exe` e Substituição Atômica sem Janelas Duplicadas (T8.3 / ADR-042):**
    - **Causa Raiz Identificada:** Na API do GitHub Releases, o artefato portátil `OpenBoom-v0.5.0-portable-x64.exe` (10.3 MB) precede alfabeticamente o instalador `OpenBoom_0.5.0_x64-setup.exe` (2.4 MB). O filtro anterior (`lower.contains("openboom")`) baixava o binário portátil para `%TEMP%` (de forma lenta por `Invoke-WebRequest` sem `$ProgressPreference = 'SilentlyContinue'`) e o executava com `/S` enquanto a v0.4.0 seguia aberta, abrindo janelas da v0.5.0 a partir de `%TEMP%` sem atualizar `%LOCALAPPDATA%\OpenBoom\open-boom.exe`.
    - **Correção no Backend (`src-tauri/src/lib.rs`) e Frontend (`src-ui/src/App.tsx`):**
      - `select_installer_asset_url` prioriza estritamente pacotes `.exe` contendo `"setup"` ou `"installer"` e exclui `"portable"`.
      - `download_and_install_update` utiliza `curl.exe -fL` (com fallback PowerShell sob `$ProgressPreference = 'SilentlyContinue'`), guarda de concorrência `DownloadInFlightGuard` (`AtomicBool`) e staging atômico via `.part` -> `%TEMP%\OpenBoom_Setup_Update.exe` sem abrir processos enquanto o app está aberto.
      - `restart_application` encerra o `AudioStreamPump`, restaura o endpoint físico do Windows, inicia um worker oculto (`CREATE_NO_WINDOW`) que aguarda o encerramento do PID antigo, finaliza processos remanescentes, executa `OpenBoom_Setup_Update.exe /S /UPDATE -Wait`, limpa `%TEMP%` e abre uma única instância do `%LOCALAPPDATA%\OpenBoom\open-boom.exe` atualizado.
    - **Verificação Completa & Atualização Local:** **215 testes Cargo passando (0 falhas)**, **106 testes UI passando (0 falhas)**, binário instalado em `%LOCALAPPDATA%\OpenBoom\open-boom.exe` atualizado para a v0.5.0 corrigida e smoke test local verificado.
46. **Correção Definitiva do Loopback Mode (Modo B) contra Realimentação Acústica ("Sino Atordoante" Karplus-Strong) e Retenção de Nota no Idle (T8.4 / ADR-043):**
    - **Causa Raiz Identificada:** Em `crates/open-boom-wasapi/src/stream_pump.rs`, o Modo B abria `AUDCLNT_STREAMFLAGS_LOOPBACK` padrão no mesmo endpoint físico (`render_device.clone()`) onde o `render_svc` do OpenBoom injetava o áudio processado. O OpenBoom recapturava sua própria saída a cada ~10–25ms, reaplicava o ganho do DSP e re-injetava no DAC, criando um ressonador comb de realimentação positiva (síntese Karplus-Strong de sino metálico em `-0.2 dBFS`) que tocava 1 nota infinitamente mesmo após pausar o áudio externo.
    - **Implementação de Process Loopback Real (`crates/open-boom-wasapi/src/loopback_capture.rs` e `stream_pump.rs`):**
      - Implementado `ProcessLoopbackCompletionHandler` (`#[windows_core::implement(IActivateAudioInterfaceCompletionHandler)]`) e `activate_process_loopback_client(exclude_process_id: u32)` via `ActivateAudioInterfaceAsync` com `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK` (`"VAD\\Process_Loopback"`) e `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE`, excluindo a árvore de processos do próprio OpenBoom (`GetCurrentProcessId()`) diretamente no mixer do Windows (`audiodg.exe`).
      - Adicionado `AUDCLNT_STREAMFLAGS_EVENTCALLBACK` (`CreateEventW` + `WaitForSingleObject`), supressão de renderização duplicada em bypass/mute/silêncio no Modo B, parada e limpeza do buffer circular (`AUDCLNT_BUFFERFLAGS_SILENT` + `render_client.Stop()` + `render_client.Reset()`) e despacho de `AudioCommand::ResetFilters` no idle para decair filtros IIR e telemetria para `-100.0 dBFS`.
    - **Testes com Geração de Áudio Real (`crates/open-boom-wasapi/tests/stream_pump_tests.rs`):**
      - Gerador de arquivo `.wav` PCM 16-bit e tom piloto estreito (14.321 Hz) validando >34 dB de rejeição da própria saída no Modo B e decaimento imediato ao silêncio (`<= -90 dBFS`) após o término do áudio (**217 testes Cargo verdes e 106 testes UI verdes**).
47. **Eliminação de Estática e Estouros (100 Hz Packet-Boundary Dropouts) no Modo B de Loopback (T8.4 / ADR-044):**
    - **Causa Raiz Identificada:** Entre pacotes consecutivos de 10ms do WASAPI, `WaitForSingleObject(h_ev, 4)` (ou `sleep(2ms)`) acordava antes da chegada do próximo pacote (`GetNextPacketSize() == 0`) e injetava `960` amostras `0.0` em `coord.process_audio_chunk` no meio da reprodução ativa, zerando a janela de lookahead de 2ms (96 frames) do `MasterLimiter` 100 vezes por segundo (gerando mais estática com 3D Surround desligado e menos quando suavizado pelo filtro FIR do 3D Surround), além de chamar `render_client.Stop()`/`Reset()` prematuramente e descartar pacotes quando `padding > frames_read * 2`.
    - **Correção (`crates/open-boom-wasapi/src/stream_pump.rs` & `tests/stream_pump_tests.rs`):**
      - O flush de silêncio e `AudioCommand::ResetFilters` agora só é disparado uma única vez (`idle_flushed = true`) quando nenhum áudio ativo é recebido por $\ge 200\text{ ms}$ (`last_audio_instant.elapsed() >= Duration::from_millis(200)`), preservando 100% da continuidade de fase e do buffer de lookahead de 2ms entre pacotes.
      - O `render_client` utiliza buffer de 25ms (`buffer_guard_enabled`), permanece iniciado sem `Stop()`/`Reset()` abruptos durante a sessão e jamais descarta pacotes capturados.
      - Adicionado teste de regressão `test_mode_b_packet_boundary_continuity_no_static_with_surround_off_and_on` validando continuidade senoidal sem buracos de 2ms e sem degraus nas fronteiras de pacotes tanto com 3D Surround desligado quanto ligado (**218 testes Cargo verdes e 106 testes UI verdes**).
48. **Registro do Efeito de Dupla Reprodução (Slapback Delay) no Modo B (`BUG-MODE-B-01`), Rotulagem `[BETA]` na UI, Direcionamento para Driver Virtual Open-Source (`T9.5`) e Lançamento `v0.5.2` (T8.5 / ADR-046):**
    - **Diagnóstico Registrado (`BUG-MODE-B-01`):** No Modo B (sem driver virtual isolando a saída do Windows), o usuário ouve a soma do áudio seco original enviado pelos aplicativos ao fone físico ($t = 0\text{ ms}$) com a reprodução processada pelo DSP do OpenBoom ($t \approx 25\text{ ms}$), gerando o efeito de som duplicado/distante.
    - **UI & Internacionalização (`SettingsModal.tsx`, `StatusBar.tsx`, `pt-BR.ts`, `en.ts`, `es.ts`):** Adicionados pills visuais `BETA` (`Modo B: BETA`) e textos explicativos claros nos 3 idiomas marcando o Modo B como experimental.
    - **Diretriz Estratégica (`T9.5`):** Pausado o ajuste fino do Modo B em favor da integração futura de um driver virtual de áudio open-source para Windows.
49. **Execução e Conclusão do Workflow `/bmad-spec` para Onboarding de Driver Virtual & Automação 7.1 (`spec-virtual-driver-onboarding`):**
    - Criado o contrato canonical do BMad em `_bmad-output/specs/spec-virtual-driver-onboarding/` com `.memlog.md`, `SPEC.md`, `technical-driver-integration.md` e `stories.yaml`.
    - Formalizada a conformidade jurídica anti-processo: renomeação do endpoint no Windows com atribuição transparente `OpenBoom Spatial Audio (powered by VB-Cable)`.
    - Formalizada a automação da máscara de alto-falantes 7.1 Surround via injeção de `PKEY_AudioEndpoint_PhysicalSpeakers = 0x63F` (`KSAUDIO_SPEAKER_7POINT1_SURROUND`).
    - Formalizado o expurgo definitivo de marcas comerciais "Boom 3D / Boom Audio" da UI.
    - Formalizado o bloqueio preventivo no boot: se não houver driver compatível, controles da UI ficam desativados/cinza (`opacity-40 pointer-events-none`) e modal de onboarding abre automaticamente.
    - Formalizado o tratamento de erro em try/catch na instalação silenciosa com alerta explicativo, botão de tentar novamente e link direto para download manual (`https://vb-audio.com/Cable/`).
    - Protocolo de teste final em máquina real estabelecido: desinstalação prévia de drivers VB-Audio do host e validação de ponta a ponta via OpenBoom.
    - Decomposição em 4 histórias de usuário: `9-1` (UI Purge & Rename Attribution), `9-2` (Automated 7.1 Mask Injection), `9-3` (First-Run Gating & Controls Lockdown) e `9-4` (1-Click Silent Setup, Try/Catch & Manual Fallback).
50. **Implementação, Verificação e Conclusão Integral do Épico 9 via `bmad-build-auto`:**
    - **Story 9-1 (`spec-9-1`):** Purgação total de marcas "Boom 3D / Boom Audio" da UI e dicionários (`en`, `pt-BR`, `es`); normalização e renomeação do endpoint para `OpenBoom Spatial Audio (powered by VB-Cable)` via registro e comando Tauri `customize_virtual_device_endpoint`.
    - **Story 9-2 (`spec-9-2`):** Injeção automatizada da máscara de alto-falantes 7.1 Surround `0x63F` (`PKEY_AudioEndpoint_PhysicalSpeakers`) no registro do Windows Core Audio e comando Tauri `configure_virtual_endpoint_speaker_mask`.
    - **Story 9-3 (`spec-9-3`):** Detecção de driver no boot (`check_virtual_driver_status`), bloqueio e desativação visual dos controles interativos (`opacity-40 pointer-events-none select-none cursor-not-allowed`) e abertura automática do `<DriverOnboardingModal />`.
    - **Story 9-4 (`spec-9-4`):** Rotina de 1 clique (`install_virtual_driver`), execução UAC silenciosa com `-i -h`, resiliência try/catch para cancelamento de UAC e falhas de rede, link de download manual oficial (`https://vb-audio.com/Cable/`), e injeção automática pós-instalação da marca e máscara 7.1.
    - **Verificação Geral:** 226 testes Rust verdes em Cargo e 113 testes de UI em React/Vite verdes (0 falhas).

## O que está em andamento agora
* **Épico 9 100% Concluído:** Stories 9-1, 9-2, 9-3, 9-4, 9-5 e 9-6 finalizadas.
* **Próxima Etapa:** Mandatory Final Verification Gate (compilação do binário Tauri, smoke launch de `open-boom.exe`, git commit e git push, aguardando sinal do usuário para release).

## Bloqueios / Pendências
* Nenhum bloqueio. Sistema ultra-estável com 228 testes Cargo verdes e 118 testes UI verdes.

54. **Implementação e Conclusão das Stories 9-5 e 9-6 (Épico 9 Finalizado):**
    - **Story 9-5 (`T9.11`):** *Aviso de Privacidade & Esclarecimento de Acesso a Microfone*:
      - Adicionadas chaves `privacyNoticeTitle` e `privacyNoticeDesc` com paridade de 100% nos dicionários `en.ts`, `pt-BR.ts` e `es.ts`.
      - Inserido banner visual de privacidade com ícone `ShieldCheck` em `DriverOnboardingModal.tsx` e banner informativo no seletor de modo em `SettingsModal.tsx`.
      - Testes adicionados em `DriverOnboarding.test.tsx` e `SettingsModal.test.tsx`.
    - **Story 9-6 (`T9.12`):** *Modo Inicial Desarmado Standby (No Audio Bridge Default Mode B)*:
      - Em `crates/open-boom-wasapi/src/stream_pump.rs`, adicionado loop de Standby seguro em `run_windows_pump` quando `!config.force_loopback && capture_device_opt.is_none()`.
      - Em `src-tauri/src/lib.rs`, `AppState::with_options`, `apply_audio_bridge_mode`, `set_audio_device`, `trigger_device_fallback` e `set_buffer_guard` agora checam `let should_start_pump = force_loopback || virtual_id.is_some()`. Sem driver virtual e sem modo B forçado, `pump` permanece `None` e o dispositivo padrão físico do Windows permanece intacto.
      - Corrigida a inicialização de `SessionManager` em `AppState::with_options` usando modo sintético forçado quando `auto_route == false` para isolamento total e determinismo de testes unitários.
      - No frontend, adicionado badge `Standby (Driver Required)` em `StatusBar.tsx` via prop `isStandby`, e chaves `standbyMode` e `standbyModeTooltip` em `en`, `pt-BR` e `es`.
      - Testes unitários adicionados em `crates/open-boom-wasapi/tests/stream_pump_tests.rs`, `src-tauri/src/lib.rs` e `src-ui/src/test/DriverOnboarding.test.tsx`.
    - **Resultado da Verificação:** Todos os 228 testes Cargo do workspace aprovados; todos os 118 testes de UI aprovados; build do frontend concluído em 1.90s.

51. **Eliminação de Communications Ducking e Auto-Unmute de Sessões na Transição de Driver (ADR-054):**
    - **Causa Raiz Identificada:** Em `crates/open-boom-wasapi/src/device.rs`, a chamada a `set_default_func(policy_ptr, pcwstr, eCommunications)` vinculava o driver virtual ao papel de comunicações do Windows, disparando a política nativa de "Atenuação de Comunicações" do Windows (*"Reduzir o volume de outros sons em 80%"* ou *"Desativar o som de todos os outros sons"*), mutando abas do Chrome e jogos ao instalar o driver ou abrir o app. Além disso, a reinicialização forçada do `audiosrv` migra sessões no Windows 11 frequentemente marcando-as como `bMuted = TRUE`.
    - **Soluções Implementadas:**
      - Removida a atribuição do papel `eCommunications` em `crates/open-boom-wasapi/src/device.rs`, restringindo estritamente a `eConsole` e `eMultimedia`.
      - Implementado auto-unmute de resiliência em `crates/open-boom-wasapi/src/session_manager.rs`: se uma sessão ativa de processo estiver mutada e não possuir um perfil explícito em `stored_profiles` pedindo mudo, o OpenBoom a desmuta imediatamente via `ISimpleAudioVolume::SetMute(FALSE, NULL)`.
      - Adicionado teste de regressão `test_session_manager_auto_unmutes_unprofiled_muted_sessions` em `session_manager_tests.rs`.
      - Adicionada enumeração imediata de sessões no boot em `AppState::new_runtime()`, ao alternar o Master Power, e ao trocar o modo de ponte de áudio.
      - Adicionada chamada a `fetchAudioSessions()` no hook de montagem inicial do frontend (`App.tsx`).
    - **Verificação Geral:** 227 testes Rust verdes em Cargo, 113 testes de UI em React/Vite verdes, compilação de release e NSIS (`OpenBoom_0.5.2_x64-setup.exe`) concluídas com sucesso.
52. **Refinamento de UI: Remoção do Seletor de Idioma da Tela Inicial, Layout da Aba de Updates & Fallback em Inglês:**
    - **Remoção de Idioma da Tela Inicial:** Removido o `<LanguageMenu />` de `src-ui/src/components/Titlebar.tsx`. A seleção de idioma permanece totalmente acessível na modal de preferências (`SettingsModal.tsx`) com a grade visual completa e no menu de contexto nativo da bandeja do Windows (System Tray).
    - **Refinamento de CSS na Tela de Atualizações (`SettingsModal.tsx`):**
      - Aumento da largura da modal para `w-[560px] max-w-[95vw]` e paddings internos (`p-5 sm:p-6`, `p-4 sm:p-5`, `p-3.5 sm:p-4`).
      - Redesenho responsivo do banner de versão com `whitespace-nowrap`, padding generoso `px-3 py-1.5` nas caixas de "Versão Atual" e "Última Versão", evitando quebras de linha e texto espremido.
      - Cartões de status e toggles com bordas arredondadas `rounded-xl`, espaçamento balanceado e correção de caractere de bullet point.
    - **Garantia de Fallback em Inglês (`src-ui/src/i18n/index.tsx`):**
      - `getSystemLocale()` e `resolveEffectiveLocale()` reforçados para iterar candidatos de locale e retornar estritamente `"en"` sempre que o idioma do sistema não for suportado (ex.: alemão, francês, japonês, chinês, russo, etc.).
      - Adicionados testes de regressão em `src-ui/src/test/I18n.test.tsx` cobrindo locales não suportados.
    - **Verificação Geral:** 226 testes Rust verdes, 113 testes UI verdes, build do frontend e do binário Rust compilados sem erros, smoke launch do executável validado em tempo real com processo saudável.

53. **Extensão do Épico 9: Stories 9-5 (Aviso de Privacidade sobre Microfone) e 9-6 (Modo Inicial Desarmado/Standby) (ADR-056):**
    - **Registro de Novas Histórias em `stories.yaml`, `sprint-status.yaml` e `03-TASK-BACKLOG.md`:**
      - **Story 9-5 (`T9.11`):** *Aviso de Privacidade e Esclarecimento de Acesso a Microfone / Interceptação de Som*: Mensagem informativa clara em `DriverOnboardingModal.tsx`, `SettingsModal.tsx` e dicionários i18n explicando que o alerta de "Acesso ao Microfone" do Windows/antivírus decorre do agrupamento de APIs de loopback sob a permissão genérica de microfone, garantindo que o OpenBoom processa estritamente o áudio do PC e nunca grava a voz do usuário.
      - **Story 9-6 (`T9.12`):** *Modo Inicial Desarmado (Standby / Nenhum Modo Selecionado) no Boot Sem Driver*: Impede que o app caia automaticamente no Modo B (Loopback Fallback com duplicação acústica `BUG-MODE-B-01`) no primeiro boot sem driver. O app inicializa desarmado com o áudio do Windows intocado e o motor desligado até que o driver seja instalado ou o usuário ative conscientemente o Modo B nas configurações.
      - **Backlog Pós-1.0 (`T11.3`):** Investigação de Nomenclatura PnP de Dispositivo Virtual no Windows Shell movida para a Sprint 11 sem bloquear a release 1.0.
