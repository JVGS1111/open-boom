# OpenBoom - Milestones & Roadmap

## Marco 1: Protótipo de Interface e Componentes Visuais (Foco Imediato)
- [ ] 1.1 Scaffold da aplicação de UI com tema escuro idêntico à referência.
- [ ] 1.2 Implementação da barra superior (Master Switch, Volume Slider com indicador numérico, botões Saída/Aplicações).
- [ ] 1.3 Implementação do Dial Circular do 3D Surround (knob rotativo com marcador angular e botão de configuração).
- [ ] 1.4 Implementação da faixa horizontal de efeitos (`Ambiente`, `Fidelidade`, `Modo Noturno`, `Espacial`, `Tonalidade`).
- [ ] 1.5 Implementação do Seletor de Headphone (campo de busca e tag com o modelo ativo).
- [ ] 1.6 Implementação do Seletor de Equalizador e dropdown de presets.
- [ ] 1.7 Criação das telas de detalhe / modais:
  * Painel do Equalizador completo (sliders de bandas com ganho em dB).
  * Painel de Configuração dos Falantes 3D (posicionamento e volume por canal).
  * Painel do Mixer por Aplicação (volume individual por programa aberto).

## Marco 2: Motor de Áudio & I/O no Windows
- [ ] 2.1 Enumeração e monitoramento de dispositivos de áudio ativos via Core Audio / MMDeviceAPI.
- [ ] 2.2 Captura de áudio do sistema (WASAPI Loopback ou roteamento de dispositivo padrão).
- [ ] 2.3 Renderização de áudio em tempo real com buffer circular (jitter buffer de baixa latência).
- [ ] 2.4 Controle de volume e muting mestre e por aplicativo.

## Marco 3: Pipeline DSP & Efeitos em Tempo Real
- [ ] 3.1 Implementação do Equalizador Gráfico/Paramétrico (Filtros Biquad IIR Peaking, Low-Shelf, High-Shelf).
- [ ] 3.2 Implementação do 3D Spatializer (HRTF / Convolução espacial estéreo para fones).
- [ ] 3.3 Implementação do efeito Ambiente (Convolução / Algoritmo de Reverb Schroeder-Moorer).
- [ ] 3.4 Implementação do efeito Fidelidade (Exciter harmônico e excitação de agudos).
- [ ] 3.5 Implementação do Modo Noturno (Dynamic Range Compressor / AGC).
- [ ] 3.6 Implementação de Espacial (Mid/Side stereo widen) e Tonalidade (Pitch shift).
- [ ] 3.7 Master Peak Limiter para proteção contra distorção e clipping.

## Marco 4: Integração com AutoEq & Presets
- [ ] 4.1 Carregamento e indexação dos dados de calibração do AutoEq (fones HyperX, Sennheiser, Sony, etc.).
- [ ] 4.2 Aplicação automática das curvas de equalização corretivas.
- [ ] 4.3 Criação, edição e exportação de presets personalizados pelo usuário.

## Marco 5: Polimento, Empacotamento e Executável
- [ ] 5.1 Redução da latência para < 10ms.
- [ ] 5.2 Inicialização com o Windows e minimização para a bandeja do sistema (System Tray).
- [ ] 5.3 Empacotamento standalone do executável.
