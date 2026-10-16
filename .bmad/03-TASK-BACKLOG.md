# OpenBoom - Task Backlog

## Status Legend
* `[ ]` Não iniciado (Todo)
* `[/]` Em progresso (In Progress)
* `[X]` Concluído (Done)
* `[!]` Bloqueado (Blocked)
* `[-]` Descartado (Discarded)

---

### Sprint 1: Fundação do Projeto & UI Shell (Marco 1)
- [X] **T1.1**: Criação do repositório `open-boom` e inicialização do Git.
- [X] **T1.2**: Implantação da estrutura do Memory Harness BMAD (`.bmad/`).
- [X] **T1.3**: Scaffold da aplicação desktop (UI base, tema escuro, layout e responsividade).
- [X] **T1.4**: Construção do Header (Master Power Switch 48x26 com brilho ciano, Slider de Volume com valor numérico e controles de saída).
- [X] **T1.5**: Construção do Dial Circular 3D Surround (knob rotativo 192px/144px com varredura 300°, física polar atan2, pips ciano 25/50/75%, snap duplo-clique 50%, 8 satélites e launcher de calibração - Story 2.2).
- [X] **T1.6**: Construção da barra de efeitos (5-Effect Quick Rack Strip: Ambience, Fidelity, Night Mode, Spatial e Pitch com badge "Soon" Fase 2 - Story 4.1).
- [X] **T1.7**: Construção dos seletores de Headphone EQ (AutoEq) e Equalizador (Presets) (`AutoEqPopover.tsx` e `PresetSelectorPopover.tsx` - Stories 3.3 e 3.5).
- [X] **T1.8**: Criação dos painéis expansíveis / modais (Modal de calibração 7.1 Soundstage com nós orbitais, faders de trim ±6 dB, atalho global Ctrl+Shift+S e arrasto angular - Stories 2.3 e 2.4).
- [X] **T1.9**: Lock-Free POD Command Dispatcher (`AudioProcessor`, SPSC ring buffer sem alocações, bit-perfect passthrough quando desligado e telemetria sincronizada).
- [X] **T1.10**: Comutação A/B Instantânea com Alinhamento de Fase (Crossfade equal-power 5ms, retenção do lookahead 2ms, zero pops/clicks, atalho Ctrl+Shift+B e status Hardware Passthrough - Story 1.6).
- [X] **T1.11**: Persistência Atômica de Configurações, Geometria da Janela e Inicialização no Windows (PresetStore com staging .tmp e replace MoveFileExW em %APPDATA%\OpenBoom\config.json, StartupManager em HKCU\...\Run sem UAC, menu de contexto da bandeja do sistema e SettingsModal acessível - Story 1.7).
- [X] **T1.12**: Ponte de Dispositivo Virtual & Engenharia Reversa REA (VirtualAudioBridge dual-mode Modo A/B, heurísticas de detecção de endpoints virtuais, recuperação de invalidation sub-500ms, relatório técnico REA em .bmad/evidence/rea-boom-engine-reverse-engineering-report.md, telemetria using_virtual_bridge e badge no StatusBar - Story 1.8).

### Sprint 2: Motor de Áudio & Dispositivos (Marco 2)
- [X] **T2.1**: Módulo de descoberta de dispositivos de saída e entrada (MMDeviceAPI enumeration, dynamic switching < 500ms, fallback recovery e popover selector - Story 1.4).
- [X] **T2.2**: Pipeline de captura e reprodução de baixa latência (WASAPI Loopback Mode B/A, Resampler Hermite C1 48 kHz, MMCSS Pro Audio, RenderClient).
- [X] **T2.3**: Integração do controle de volume do sistema e mixer por aplicativo (Concluído formalmente no Epic 5 via Stories 5.1, 5.2 e 5.3).
- [X] **T2.4**: Motor de Convolução Binaural 7.1 KEMAR HRTF & Downfold Matrix (64 taps SSE2 SIMD, buffer circular duplo de 128 amostras, ITD 0.2-0.7ms, pinna notch 6 kHz, crossfade paramétrico e orçamento de desempenho < 25% - Story 2.1).
- [X] **T2.5**: Reposicionamento Angular Espacial de Alto-Falantes Virtuais & Interpolação HRTF Contínua (arraste polar orbital, cálculo em tempo real de coeficientes HRTF KEMAR sem dropouts de áudio, display de coordenadas azimutais e reset para ITU-R BS.775 - Story 2.4).
- [X] **T2.6**: Roteamento Inteligente Automático via COM IPolicyConfig & Dual-Mode DAC Bridge (chaveamento automático para Boom Audio no Master Power ON, retorno transparente ao HyperX no Master Power OFF/Exit, retry loop anti-jitter e filtragem de sinks virtuais na UI - ADR-020).
- [X] **T2.7**: Calibração da Matriz de Upmix Estéreo & Curva Perceptual do Dial 3D (Refinamento do ganho de ambiência lateral L-R e transição wet/dry para realce imediato de presença antes do Epic 4 - ADR-021).

### Sprint 3: Motor DSP & Efeitos de Som (Marco 3)
- [X] **T3.1**: Motor do Equalizador Multibanda (31 bandas ISO 1/3-oitava, RBJ peaking biquads, Q=4.31, Direct Form II Transposed, suavização sub-buffer 32 amostras, estabilidade estrita de pólos, comandos POD SPSC - Story 3.1).
- [X] **T3.2**: Algoritmo de espacialização 3D com HRTF (Convolução KEMAR 64 taps por canal, interpolação angular dinâmica, ganho e mute por canal - Stories 2.1, 2.3, 2.4).
- [X] **T3.5**: Equalizer Studio Drawer Dual-Mode & Visualizador Spline (Drawer expansível via Ctrl+Shift+E e Expand EQ, toggle 10/31 bandas preservando ganhos, faders 180px com snap magnético ±0.3 dB, 5x precisão Shift, reset duplo-clique e canvas Bezier spline ciano - Story 3.2).
- [X] **T3.6**: Indexação Offline da Base AutoEq & Motor de Busca Fuzzy Instantâneo (Base comprimida com 4.200 modelos medindo 0.20 MB < 5 MB em open-boom-storage, busca fuzzy in-memory < 1ms com ranking por relevância, comandos IPC Tauri search_autoeq_profiles / get_autoeq_profile / get_autoeq_stats, popover AutoEqPopover UX-DR8 com navegação por teclado e chips no preview/drawer, persistência atômica no AppConfig - Story 3.3).
- [X] **T3.3**: Módulo de efeitos psicoacústicos e rack dinâmico (Epic 4: Reverb Schroeder-Moorer/FDN, Exciter de harmônicos 2º/3º, Compressor DRC soft-knee com diálogo boost e Stereo Widener Mid/Side com sub-bass centering - Stories 4.2, 4.3, 4.4, 4.5).
- [X] **T3.4**: Master Limiter anti-clipping (Lookahead 2ms -0.2 dBFS, brickwall ceiling clamp, 0% digital clipping, medidor de headroom dinâmico e telemetria pulsante - Story 1.5).

### Sprint 4: AutoEq & Presets (Marco 4)
- [X] **T4.1**: Parser e indexador dos dados AutoEq (4.200 perfis, filtros paramétricos biquad e compensação automática de preamp - Story 3.3).
- [X] **T4.2**: Integração dos perfis de fone diretamente na cadeia de equalização paramétrica desacoplada (Estágio ParametricStage de até 12 biquads Peaking/LowShelf/HighShelf com RBJ cookbook e Schur-Cohn, atenuação automática de preamp anti-clipping, cascata antes do EQ gráfico 31 bandas sem sobrescrever curvas do usuário, comandos SPSC lock-free e IPC Tauri - Story 3.4).
- [X] **T4.3**: Gerenciamento de presets de gênero de fábrica e presets customizados do usuário (15 presets calibrados de fábrica FR11, detecção de estado sujo com asterisco `Rock *`, persistência atômica CRUD em `%APPDATA%\OpenBoom\presets\*.json` via `.tmp` + `MoveFileExW`, sanitização de slugs e componente `PresetSelectorPopover.tsx` - Story 3.5).

### Sprint 5: Windows Per-Application Audio Session Mixer & Volume Routing (Marco 2 / Epic 5)
- [X] **T5.1**: Windows Core Audio Session Manager & Thread Dedicada COM MTA (Isolamento de `IAudioSessionManager2` e `ISimpleAudioVolume` em worker thread COM MTA, canais de mensageria mpsc, inspeção segura de executáveis, poda de sessões expiradas, modo sintético headless e comandos IPC Tauri - Story 5.1).
- [X] **T5.2**: Per-Application Mixer Drawer UI & Faders Individuais por Processo (Gaveta lateral de mixer no HUD, linhas `app-mixer-row` de 44px com ícone de processo, indicador pulsante de áudio, fader 0%-150% e mute, despacho via Tauri IPC - Story 5.2).
- [X] **T5.3**: Persistência Atômica de Volumes por Aplicação & Recuperação Automática de Sessões (Armazenamento em `%APPDATA%\OpenBoom\app_volumes.json` via staging atômico `.tmp`, restauração automática na inicialização e recuperação quando novos processos iniciam reprodução - Story 5.3).
- [X] **T5.4**: Audio Buffer Stability Guard & Peak-Hold Anti-Glitch Limiter (Buffer WASAPI estendido de 25ms contra jitter do Windows, 5ms peak-hold contra intermodulação em sub-graves, switch nas preferências com banner de risco acústico em vermelho e persistência atômica - ADR-031).
- [X] **T5.5**: Suporte a Internacionalização (i18n: pt-BR, en, es), Menu de Idiomas e Detecção Automática do Sistema Operacional (Dicionários completos sem dependências externas, seletor na barra de título, modal de configurações e menu de bandeja nativo do Windows, persistência atômica no AppConfig e lançamento v0.2.0 - ADR-033).

### Sprint 6: 8D Orbital Audio (Som Rotativo 360° & Trajetória Dinâmica)
- [X] **T6.1**: Motor DSP de Modulação Azimutal Orbital & Rotação Contínua 7.1 em Sub-Blocos (Extensão de `Spatializer3D` com cálculo dinâmico $\theta_k(t)$ a cada 32 frames, interpolação suave KEMAR e zero alocações na heap - CAP-1, ADR-032).
- [X] **T6.2**: Controle de Orientação e Velocidade Angular (Suporte a Sentido Horário, Anti-Horário e Pêndulo; período de 2s a 30s; comando `AudioCommand::SetOrbit8D` lock-free - CAP-2, CAP-3).
- [X] **T6.3**: Botão Rápido 8D Orbit & Indicador Trajetória Neon no SoundstageDial (Toggle 8D no HUD, comando Tauri IPC `set_orbit_8d`, projeção polar azimutal e cometa orbital neon $\ge 30\text{ fps}$ via telemetria - Story 6.2, ADR-035).
- [X] **T6.4**: Painel Completo de Parâmetros Orbitais no Modal de Calibração & Persistência Atômica (Controles detalhados de direção, sliders de período e profundidade no SoundstageConfiguratorModal e persistência em `AppConfig` - Story 6.3, ADR-036).

### Sprint 7: Integração com Controle de Volume Mestre do Windows (Epic 7)
- [X] **T7.1**: Sincronização Bidirecional em Tempo Real com `IAudioEndpointVolume` do Windows (Worker thread COM MTA dedicada `open-boom-volume-sync`, callback `IAudioEndpointVolumeCallback::OnNotify`, supressão de eco por GUID `OPEN_BOOM_VOLUME_CONTEXT_GUID` + flag atômica `is_syncing_to_windows`, rebind automático em troca de Master Power/dispositivos, preservação da faixa Boost 101%–150%, auto-unmute em arrasto do slider da UI e eventos IPC `windows-volume-changed` - Spec 7.1, ADR-039).

### Sprint 8: Experiência de Distribuição & Auto-Update In-App
- [X] **T8.1**: Mecanismo de Auto-Update In-App a Custo Zero via GitHub Releases (Verificação automática na abertura do app, badge âmbar pulsante na Titlebar levando direto para `Configurações -> Atualizações`, painel completo com notas de versão, switches de checagem ao iniciar e instalação automática, e botão de `Reiniciar Aplicação` no mesmo slot de ação - Spec 8.1, ADR-038).
- [X] **T8.2**: Seletor Manual de Modo de Captura de Áudio (`audio_bridge_mode`: `auto` | `virtual` | `loopback`) em `SettingsModal.tsx` com Persistência Atômica e Reinicialização Automática do App (Permite alternar e testar ao vivo entre o driver virtual `Boom Audio` Modo A e a emulação sem driver WASAPI Loopback Modo B, restaurando o endpoint físico do Windows antes do restart - ADR-040).
- [X] **T8.3**: Correção do Seletor de Assets, Download Rápido via `curl.exe` e Instalação Atômica sem Janelas Duplicadas no Auto-Update In-App (`select_installer_asset_url` priorizando `-setup.exe` sobre `-portable-x64.exe`, staging atômico em `%TEMP%\OpenBoom_Setup_Update.exe` com guarda `DownloadInFlightGuard` e execução diferida `/S /UPDATE` após encerramento do processo antigo em `restart_application` - ADR-042).
- [X] **T8.4**: Correção Definitiva do Loopback Mode (Modo B) contra Realimentação Acústica ("Sino Atordoante" Karplus-Strong), Retenção de Nota no Idle e Estática de Fronteira de Pacotes (`activate_process_loopback_client` via `ActivateAudioInterfaceAsync` com `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK` e `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE`, preservação contínua do buffer de lookahead de 2ms entre pacotes de 10ms e testes automatizados com geração de áudio `.wav` - ADR-043, ADR-044).
- [X] **T8.5**: Classificação do Modo B (Emulação Loopback Sem Driver) como `[BETA]` na UI e Registro do Bug de Duplicação Acústica / Slapback Delay (Adição de badge e rótulo `[BETA]` em `SettingsModal.tsx`, `StatusBar.tsx` e locales `pt-BR`/`en`/`es`, documentando que sem um driver virtual isolando a saída do Windows, o sistema reproduz o som original seco em $t=0\text{ ms}$ somado à reprodução processada pelo DSP em $t\approx 25\text{ ms}$ no mesmo fone físico - ADR-045).

---

### Backlog de Bugs Conhecidos & Ajustes Finos Futuros (Modo B `[BETA]`)
- [ ] **BUG-MODE-B-01 (Duplicação Acústica / Delay entre Duas Reproduções no Modo B `[BETA]`)**: No Modo B (`audio_bridge_mode = "loopback"` sem driver virtual), como os aplicativos renderizam diretamente no endpoint físico do Windows ($t = 0\text{ ms}$) e o OpenBoom captura esse fluxo via WASAPI Process Loopback, processa no DSP (2ms lookahead + 10–25ms de buffer WASAPI) e re-injeta no mesmo fone físico ($t \approx 25\text{ ms}$), o usuário ouve **duas reproduções simultâneas com leve delay entre elas** (efeito de som duplicado / distante / slapback). **Decisão Estratégica:** Pausar o ajuste fino desse modo (mantendo-o rotulado como `[BETA]`) e priorizar a adoção/integração de um **driver de áudio virtual open-source** (`T9.5`) que isole a captura nativamente sem duplicação.

---

### Sprint 9: Driver Virtual, Resiliência e Usabilidade
- [X] **T9.6**: Expurgar Referências "Boom 3D / Boom Audio" da UI e Renomear Endpoint para `OpenBoom Spatial Audio (powered by VB-Cable)` (Story 9-1 / ADR-047 / ADR-052).
- [X] **T9.7**: Automação da Máscara de Alto-Falantes Físicos 7.1 Surround no Windows (`PKEY_AudioEndpoint_PhysicalSpeakers = 0x63F`) (Story 9-2 / ADR-047 / ADR-052).
- [X] **T9.8**: Bloqueio Preventivo no Boot & Modal de Onboarding (Gating com controles cinza/desativados se não houver driver) (Story 9-3 / ADR-052).
- [X] **T9.9**: Instalação Silenciosa de 1 Clique, Resiliência Try/Catch, Link Manual de Fallback & Smoke Test na Máquina Host (Story 9-4 / ADR-052).
- [X] **T9.10**: Refinamentos de UI: Remoção de Seletor de Idioma da Tela Inicial, Padding Confortável na Aba de Updates & Fallback em Inglês para Idiomas de Sistema Não Suportados (ADR-055).
- [X] **T9.11**: Aviso de Privacidade e Esclarecimento de Acesso a Microfone / Interceptação de Som no Onboarding e Configurações (Story 9-5 / ADR-056).
- [X] **T9.12**: Modo Inicial Desarmado (Standby / Nenhum Modo Selecionado) no Boot Sem Driver para Impedir Ativação Involuntária do Modo B Bugado (Story 9-6 / ADR-056).
- [-] **T9.1**: [DESCARTADO - ADR-057] Estudo de Viabilidade e Custos de Driver Virtual Próprio (Descartado definitivamente; adotado o VB-Audio Virtual Cable oficial com automação de 1 clique, certificação WHQL e conformidade anti-cheat).
- [ ] **T9.2**: Curvas de Equalização e Otimização para Jogos & Mídia (Presets para jogos de ação, FPS e filmes sem violar anticheats).
- [ ] **T9.3**: Compliance e Governança Open-Source (Garantia de conformidade de licenciamento MIT/Apache-2.0, guias de contribuição e proteção de marcas).

---

### Sprint 10: Roadmap v1.5 — Performance Ultra-Thin & Avaliação de Substituição do React
- [ ] **T10.1**: Auditoria de Consumo de Recursos (Profiling de memória RAM do WebView2 vs. Engine Rust, identificando oportunidades de reduzir a pegada de background para < 20 MB).
- [ ] **T10.2**: Estudo de Viabilidade para Des-Reactização da Interface (Avaliar migração da camada de apresentação para HTML/JS Vanilla sem frameworks ou UI nativa Rust via Slint/Iced/Egui acelerada por hardware, eliminando dependência de browser engines pesadas - ADR-050).
- [ ] **T10.3**: Investigação de Nomenclatura PnP de Dispositivo Virtual no Windows Shell pós-1.0 (Explorar injeção em interfaces de dispositivo de baixo nível ou INF customizado para customizar o rótulo do driver sem quebrar assinaturas de driver ou caches do audiosrv - ADR-056).

---

### Sprint 12: Portabilidade Cross-Platform — macOS CoreAudio (Epic 10)
- [X] **T12.1**: Crate `open-boom-coreaudio` — Adaptador CoreAudio HAL de Captura & Render (Story 10-1 / ADR-058). Trait hexagonal `AudioHostBackend` em `open-boom-core`, fronteira `AudioHal` com FFI CoreAudio real + `MockHal` determinístico headless, anéis SPSC Float32 entre os IOProcs e o pump, prioridade de tempo real `pthread_setschedparam(SCHED_RR)`, vetorização NEON (64 taps HRTF) em Apple Silicon com equivalência escalar testada, enumeração/troca do endpoint padrão (CFString + `CFRelease`) e job CI `macos-latest`.
- [X] **T12.2**: Gating de Plataforma macOS — Isolamento `#[cfg]` & Switch de Backend Tauri (Story 10-2 / ADR-059). Isolamento condicional `[target.'cfg(target_os = "windows")'.dependencies]` vs `[target.'cfg(target_os = "macos")'.dependencies]`, resolução de diretório `PresetStore::default_dir()` para `~/Library/Application Support/OpenBoom/` no macOS, stubs interoperáveis `CoreAudioSessionManager` e `CoreAudioVolumeSync`, compilação local e smoke launch aprovados no macOS.
- [X] **T12.3**: Onboarding de Driver BlackHole-16ch em 1 Clique no macOS (Story 10-3). Detecção de status de driver virtual, download via Homebrew cask API com fallback CDN, instalação elevada via osascript com restart de coreaudiod, configuração de layout 7.1 surround (`kAudioChannelLayoutTag_MPEG_7_1_A`), modal de onboarding com link para BlackHole no macOS e testes 100% verdes.
- [X] **T12.4**: Roteamento de Áudio Virtual no macOS (Modo A — BlackHole Default Sink) (Story 10-4). BlackHole-16ch como sink padrão do sistema via CoreAudio HAL (`kAudioHardwarePropertyDefaultOutputDevice`), captura 8 canais discretos 7.1 surround com ITU-R BS.775 downfold, renderização binaural estéreo e restauração do endpoint físico em sub-500ms no shutdown/power off.
- [X] **T12.5**: Mixer de Sessões por Aplicação no macOS Gated por OS (Story 10-5). Detecção dinâmica de versão do macOS via sysctl `kern.osproductversion`, gating de `AudioHardwareCreateProcessTap` no macOS 14.2+ (Sonoma) e degradação suave com aviso amigável no macOS 12–14.1 com zero crashes.
- [X] **T12.6**: Autostart no macOS via launchd LaunchAgent (Story 10-6). Gerenciamento atômico de ~/Library/LaunchAgents/com.openboom.launcher.plist com launchctl load/unload.
- [X] **T12.7**: Sincronização Bidirecional de Volume de Sistema no macOS (Story 10-7). Property listeners em kAudioHardwareServiceDeviceProperty_VirtualMasterVolume sem loops de feedback.
- [X] **T12.8**: Empacotamento macOS .app / .dmg & CI/CD GitHub Actions + Auto-Update (Story 10-8). Bundles universais macOS, create-dmg, CI runner macos-latest e tauri-plugin-updater v2.x.





