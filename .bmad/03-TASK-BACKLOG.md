# OpenBoom - Task Backlog

## Status Legend
* `[ ]` Não iniciado (Todo)
* `[/]` Em progresso (In Progress)
* `[X]` Concluído (Done)
* `[!]` Bloqueado (Blocked)

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
- [/] **T6.1**: Motor DSP de Modulação Azimutal Orbital & Rotação Contínua 7.1 em Sub-Blocos (Extensão de `Spatializer3D` com cálculo dinâmico $\theta_k(t)$ a cada 32 frames, interpolação suave KEMAR e zero alocações na heap - CAP-1, ADR-032).
- [ ] **T6.2**: Controle de Orientação e Velocidade Angular (Suporte a Sentido Horário, Anti-Horário e Pêndulo; período de 2s a 30s; comando `AudioCommand::SetOrbit8D` lock-free - CAP-2, CAP-3).
- [ ] **T6.3**: Integração de UI / HUD do Modo 8D (Toggle 8D, seletores de direção e velocidade, visualização do cometa/beacon orbital $\ge 30\text{ fps}$ e persistência no `AppConfig` - CAP-4, CAP-5).

