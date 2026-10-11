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
- [ ] **T1.6**: Construção da barra de efeitos (Cards com toggles de Ambiente, Fidelidade, Modo Noturno, Espacial, Tonalidade).
- [ ] **T1.7**: Construção dos seletores de Headphone EQ (AutoEq) e Equalizador (Presets).
- [X] **T1.8**: Criação dos painéis expansíveis / modais (Modal de calibração 7.1 Soundstage com nós orbitais, faders de trim ±6 dB, atalho global Ctrl+Shift+S e arrasto angular - Stories 2.3 e 2.4).
- [X] **T1.9**: Lock-Free POD Command Dispatcher (`AudioProcessor`, SPSC ring buffer sem alocações, bit-perfect passthrough quando desligado e telemetria sincronizada).
- [X] **T1.10**: Comutação A/B Instantânea com Alinhamento de Fase (Crossfade equal-power 5ms, retenção do lookahead 2ms, zero pops/clicks, atalho Ctrl+Shift+B e status Hardware Passthrough - Story 1.6).
- [X] **T1.11**: Persistência Atômica de Configurações, Geometria da Janela e Inicialização no Windows (PresetStore com staging .tmp e replace MoveFileExW em %APPDATA%\OpenBoom\config.json, StartupManager em HKCU\...\Run sem UAC, menu de contexto da bandeja do sistema e SettingsModal acessível - Story 1.7).
- [X] **T1.12**: Ponte de Dispositivo Virtual & Engenharia Reversa REA (VirtualAudioBridge dual-mode Modo A/B, heurísticas de detecção de endpoints virtuais, recuperação de invalidation sub-500ms, relatório técnico REA em .bmad/evidence/rea-boom-engine-reverse-engineering-report.md, telemetria using_virtual_bridge e badge no StatusBar - Story 1.8).

### Sprint 2: Motor de Áudio & Dispositivos (Marco 2)
- [X] **T2.1**: Módulo de descoberta de dispositivos de saída e entrada (MMDeviceAPI enumeration, dynamic switching < 500ms, fallback recovery e popover selector - Story 1.4).
- [X] **T2.2**: Pipeline de captura e reprodução de baixa latência (WASAPI Loopback Mode B/A, Resampler Hermite C1 48 kHz, MMCSS Pro Audio, RenderClient).
- [ ] **T2.3**: Integração do controle de volume do sistema e mixer por aplicativo.
- [X] **T2.4**: Motor de Convolução Binaural 7.1 KEMAR HRTF & Downfold Matrix (64 taps SSE2 SIMD, buffer circular duplo de 128 amostras, ITD 0.2-0.7ms, pinna notch 6 kHz, crossfade paramétrico e orçamento de desempenho < 25% - Story 2.1).
- [X] **T2.5**: Reposicionamento Angular Espacial de Alto-Falantes Virtuais & Interpolação HRTF Contínua (arraste polar orbital, cálculo em tempo real de coeficientes HRTF KEMAR sem dropouts de áudio, display de coordenadas azimutais e reset para ITU-R BS.775 - Story 2.4).
- [X] **T2.6**: Roteamento Inteligente Automático via COM IPolicyConfig & Dual-Mode DAC Bridge (chaveamento automático para Boom Audio no Master Power ON, retorno transparente ao HyperX no Master Power OFF/Exit, retry loop anti-jitter e filtragem de sinks virtuais na UI - ADR-020).
- [ ] **T2.7**: Calibração da Matriz de Upmix Estéreo & Curva Perceptual do Dial 3D (Refinamento do ganho de ambiência lateral L-R e transição wet/dry para realce imediato de presença antes do Epic 4 - ADR-021).

### Sprint 3: Motor DSP & Efeitos de Som (Marco 3)
- [X] **T3.1**: Motor do Equalizador Multibanda (31 bandas ISO 1/3-oitava, RBJ peaking biquads, Q=4.31, Direct Form II Transposed, suavização sub-buffer 32 amostras, estabilidade estrita de pólos, comandos POD SPSC - Story 3.1).
- [X] **T3.2**: Algoritmo de espacialização 3D com HRTF (Convolução KEMAR 64 taps por canal, interpolação angular dinâmica, ganho e mute por canal - Stories 2.1, 2.3, 2.4).
- [X] **T3.5**: Equalizer Studio Drawer Dual-Mode & Visualizador Spline (Drawer expansível via Ctrl+Shift+E e Expand EQ, toggle 10/31 bandas preservando ganhos, faders 180px com snap magnético ±0.3 dB, 5x precisão Shift, reset duplo-clique e canvas Bezier spline ciano - Story 3.2).
- [X] **T3.6**: Indexação Offline da Base AutoEq & Motor de Busca Fuzzy Instantâneo (Base comprimida com 4.200 modelos medindo 0.20 MB < 5 MB em open-boom-storage, busca fuzzy in-memory < 1ms com ranking por relevância, comandos IPC Tauri search_autoeq_profiles / get_autoeq_profile / get_autoeq_stats, popover AutoEqPopover UX-DR8 com navegação por teclado e chips no preview/drawer, persistência atômica no AppConfig - Story 3.3).
- [ ] **T3.3**: Módulo de efeitos: Reverb, Exciter de harmônicos, Compressor DRC e Stereo Widener.
- [X] **T3.4**: Master Limiter anti-clipping (Lookahead 2ms -0.2 dBFS, brickwall ceiling clamp, 0% digital clipping, medidor de headroom dinâmico e telemetria pulsante - Story 1.5).

### Sprint 4: AutoEq & Presets (Marco 4)
- [X] **T4.1**: Parser e indexador dos dados AutoEq (4.200 perfis, filtros paramétricos biquad e compensação automática de preamp - Story 3.3).
- [ ] **T4.2**: Integração dos perfis de fone diretamente na cadeia de equalização paramétrica desacoplada (Story 3.4).
- [ ] **T4.3**: Gerenciamento de presets de gênero de fábrica e presets customizados do usuário (Story 3.5).
