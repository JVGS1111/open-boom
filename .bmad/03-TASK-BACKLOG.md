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
- [/] **T1.3**: Scaffold da aplicação desktop (UI base, tema escuro, layout e responsividade).
- [ ] **T1.4**: Construção do Header (Master Switch, Slider de Volume com valor numérico e controles de saída).
- [ ] **T1.5**: Construção do Dial Circular 3D Surround (knob rotativo com indicador de posição e botão expansor).
- [ ] **T1.6**: Construção da barra de efeitos (Cards com toggles de Ambiente, Fidelidade, Modo Noturno, Espacial, Tonalidade).
- [ ] **T1.7**: Construção dos seletores de Headphone EQ (AutoEq) e Equalizador (Presets).
- [ ] **T1.8**: Criação dos painéis expansíveis / modais (Equalizador com sliders de frequência, posicionamento 3D).

### Sprint 2: Motor de Áudio & Dispositivos (Marco 2)
- [ ] **T2.1**: Módulo de descoberta de dispositivos de saída e entrada.
- [ ] **T2.2**: Pipeline de captura e reprodução de baixa latência.
- [ ] **T2.3**: Integração do controle de volume do sistema e mixer por aplicativo.

### Sprint 3: Motor DSP & Efeitos de Som (Marco 3)
- [ ] **T3.1**: Motor do Equalizador Multibanda (10 e 31 bandas).
- [ ] **T3.2**: Algoritmo de espacialização 3D com HRTF.
- [ ] **T3.3**: Módulo de efeitos: Reverb, Exciter de harmônicos, Compressor DRC e Stereo Widener.
- [ ] **T3.4**: Master Limiter anti-clipping.

### Sprint 4: AutoEq & Presets (Marco 4)
- [ ] **T4.1**: Parser dos arquivos TSV/JSON do AutoEq.
- [ ] **T4.2**: Integração dos perfis de fone diretamente na cadeia de equalização.
- [ ] **T4.3**: Gerenciamento de presets (salvar, carregar, resetar).
