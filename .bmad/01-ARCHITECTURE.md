# OpenBoom - Architecture Specification

## 1. Topologia do Sistema

```
                      +-----------------------------------+
                      |   OpenBoom Frontend (Desktop UI)  |
                      |   Dark Theme Fluent / Controls    |
                      +-----------------+-----------------+
                                        | IPC / State
                                        v
+-------------------------------------------------------------------------+
|                        OpenBoom Audio Engine Core                       |
|                                                                         |
|  [ Ingestão de Áudio ]                                                  |
|     ├─ WASAPI Loopback Capture (Dispositivo Padrão / Virtual Cable)     |
|     └─ IAudioSessionManager2 (Mixer por aplicação individual)           |
|                                                                         |
|  [ Pipeline de DSP em Tempo Real (Buffers PCM Float32) ]               |
|     ├─ 1. Pré-Amp & Gain Staging                                        |
|     ├─ 2. 3D Spatializer (HRTF Convolution / Virtual 5.1/7.1 Matrix)    |
|     ├─ 3. Equalizer Multibanda (Filtros Biquad Peaking/Shelving)        |
|     ├─ 4. AutoEq Headphone Profile Calibration                          |
|     ├─ 5. Dynamic Effects:                                              |
|     │    ├─ Ambience (Stereo Delay & Convolution Reverb)                |
|     │    ├─ Fidelity (Odd/Even Harmonic Distortion Exciter)             |
|     │    ├─ Night Mode (Fast Attack / Soft Knee Dynamic Compressor)     |
|     │    ├─ Spatial (Mid/Side Stereo Expander & Crossfeed)              |
|     │    └─ Pitch Shifter (Phase Vocoder / Delay Modulation)            |
|     └─ 6. Master Limiter (Lookahead Peak Limiter para evitar clipping)  |
|                                                                         |
|  [ Saída de Áudio ]                                                     |
|     └─ WASAPI Shared / Exclusive Low-Latency Render (Dispositivo Físico)|
+-------------------------------------------------------------------------+
```

## 2. Camadas do Projeto

### A. UI Layer
* **Tema Visual:** Paleta escura `#16171b` com detalhes em `#3b82f6` (azul), `#ec4899` (rosa/magenta) e cinza suave `#8e95a5`.
* **Componentes Chave:**
  * `MasterSwitch`: Botão deslizante com transição luminosa.
  * `VolumeSlider`: Slider com display numérico e mute rápido.
  * `Surround3DDial`: Dial circular interativo com visualização dos alto-falantes espaciais (Front Left, Center, Front Right, Surround Left, Surround Right, Rear Left, Rear Right, LFE).
  * `EffectToggleStrip`: Barra com os 5 efeitos principais com alternância rápida de estado.
  * `HeadphoneProfileCard`: Busca rápida e aplicação dos perfis AutoEq.
  * `EqualizerCard`: Seletor de presets e visualização gráfica de curvas de frequência.

### B. DSP Engine
* Processamento em buffers de 32-bit Floating Point (`float[]`).
* Taxa de amostragem padrão: 48 kHz / 44.1 kHz.
* Latência alvo: < 15ms.

### C. Persistência e Configuração
* Armazenamento de presets e configurações do usuário em formato JSON limpo.
* Cache local da base AutoEq para consulta instantânea offline.
