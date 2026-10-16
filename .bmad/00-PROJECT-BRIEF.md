# OpenBoom - Project Brief

## 1. Visão Geral
**OpenBoom** é um aplicativo desktop open-source para Windows focado em aprimoramento de áudio em tempo real, equalização e áudio espacial 3D, inspirado no Boom 3D.

## 2. Objetivos Principais
* **Áudio Espacial 3D (Virtual Surround):** Posicionamento tridimensional em fones de ouvido utilizando HRTF (Head-Related Transfer Function) e simulação de canais 5.1/7.1 com controle individual por alto-falante virtual.
* **Equalizador Multibanda & AutoEq:** Equalizador gráfico/paramétrico (10 a 31 bandas) e suporte integrado ao banco de dados AutoEq (calibração para milhares de fones de ouvido).
* **Efeitos de Áudio em Tempo Real:**
  * **Ambiente (Reverb / Stage Widening):** Simulação de palco acústico e reflexões espaciais.
  * **Fidelidade (Harmonic Exciter):** Realce de agudos e harmônicos sutis para maior clareza.
  * **Modo Noturno (Dynamic Range Compression):** Compressão para nivelar picos e sons baixos.
  * **Espacial (Spatial Widening):** Alargamento estéreo e controle de crosstalk.
  * **Tonalidade (Pitch Shifting):** Controle de afinação/tom.
  * **Bass Boost / Sub-Bass:** Reforço de frequências graves.
* **Mixer por Aplicação:** Controle independente do volume de programas em execução via Windows Audio Session API.
* **Interface Moderna e Fluida:** Tema Dark elegante, controles circulares táteis, visualização de áudio em tempo real e experiência de uso idêntica à referência.

## 3. Filosofia de Desenvolvimento
* **Zero Bloat / Zero Telemetria:** Foco estrito em processamento de sinal limpo e leve.
* **Baixa Latência:** Buffer mínimo sem introduzir clipping ou delay perceptível em reprodução de mídia e streaming.
* **Acessibilidade:** Sem necessidade de certificados proprietários de drivers no kernel; aproveitamento de APIs modernas de áudio do Windows (WASAPI / APO / Loopback).
