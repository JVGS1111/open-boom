# OpenBoom - Agent Guidelines

## Memory Harness Protocol (BMAD)
1. **Always read `.bmad/05-SESSION-STATE.md`** at the beginning of any work session to understand current progress and active sprint.
2. **Consult `.bmad/01-ARCHITECTURE.md` and `.bmad/02-MILESTONES.md`** before planning or executing new features.
3. **Update `.bmad/03-TASK-BACKLOG.md` and `.bmad/05-SESSION-STATE.md`** whenever a task is started, progressed, or finished.
4. **Log important technical tradeoffs** in `.bmad/04-DECISION-LOG.md`.

## Development Rules
* Do not introduce heavy dependencies or proprietary kernel drivers.
* Maintain clean separation between the UI layer, DSP audio pipeline, and system audio adapters.
* Keep the UI responsive, low-latency, and true to the dark modern aesthetic.

## Decompilation Tools & REA MCP Integration
* **Sister Decompilation Repository:** `../decompilator` (sibling repo on the same machine).
* **Active MCP Server:** The **REA (Reverse Engineer Anything)** MCP server (`rea`) is registered globally in the Antigravity environment and available to the agent in every session.
* **Mandatory for Reverse Engineering:** Whenever there are questions about:
  - Audio buffer formats, system calls, or virtual audio drivers (`boomvad.sys`);
  - Internal DSP algorithms (`AVBoomAudioProcessor`, `AVFIRFilterSSE`, HRTF and EQ routines);
  - Method signatures and audio data structures;
  The agent MUST use REA MCP tools (`batch_decompile`, `inspect_native_instruction`, `read_bytes`, `procedure_assembly`, etc.) and consult artifacts from the sibling decompilator repository to extract real evidence and guide implementation without guesswork.

## Git & Release Policy (Version Batching)
* **Commit & Push ao Finalizar Tarefas:** Ao concluir uma tarefa, feature ou correção, o agente deve realizar o `git commit` e fazer `git push` das alterações para a branch no GitHub.
* **Proibição de Release Automático por Fix:** **NUNCA** faça bump de versão (`Cargo.toml`, `package.json`, `tauri.conf.json`) nem crie/publique tags de release (`v*.*.*`) automaticamente para cada tarefa ou fix individual. O projeto adota o lançamento de **pacotes de atualização (batches)** consolidados.
* **Aviso de Aguardo de Sinal:** Sempre que finalizar uma tarefa e subir o código para o Git, informe explicitamente ao usuário que o código foi enviado e que **está aguardando o sinal explícito dele para gerar uma nova versão/release**.
