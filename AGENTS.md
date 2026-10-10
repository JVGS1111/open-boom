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
