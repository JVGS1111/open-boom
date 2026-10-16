# PRD Quality Review — OpenBoom

## Overall verdict
OpenBoom's PRD is an exceptionally well-engineered and technically grounded specification that establishes a clear path to replacing proprietary audio bloatware with a transparent, high-performance open-source pipeline. Its core functional requirements, audio thread constraints, SIMD performance budgets, and reverse-engineering insights are authoritative and testable. However, it carries two significant risks: an architectural loopback-feedback paradox between the driverless WASAPI claim and Windows 10 compatibility (§8 vs §10 OQ-1), and an MVP scope contradiction where core User Journey UJ-3 relies on features deferred to Phase 2 (§2.3 vs §6.2).

---

## Decision-readiness — adequate
The PRD makes courageous, unambiguous architectural choices in several critical areas, notably rejecting kernel drivers (`boomvad.sys` clones) and Windows Audio Processing Objects (APOs) in favor of user-mode WASAPI Loopback to guarantee zero BSODs and eliminate WHQL signing friction (detailed in §5 and `addendum.md` §1). The counter-metrics in §7 (SM-C1, SM-C2) clearly establish that audio continuity and phase coherence must not be sacrificed for microsecond latency or artificial stereo width.

However, the PRD dodges a make-or-break technical decision in §10 OQ-1 regarding how WASAPI loopback will capture system audio without causing an infinite acoustic feedback loop. In standard WASAPI loopback, capturing the default playback endpoint while rendering processed audio back to that same physical endpoint causes the loopback stream to recursively re-capture the processed output. While OQ-1 suggests `AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS` as a remedy, this API is only available on Windows 10 build 20348+ (Windows 11 / Server 2022). Yet §8 mandates support for Windows 10 version 1903+ (build 18362). Leaving this unresolved leaves engineers without a validated capture architecture for the stated OS baseline. Furthermore, OQ-2 leaves the core desktop frontend stack (Avalonia .NET AOT vs Rust/Tauri) open without trade-off scoring, and no `[NOTE FOR PM]` callouts are present to highlight decision milestones.

### Findings
- **critical** Unresolved WASAPI Loopback Feedback Loop & OS Compatibility Paradox (§8, §10 OQ-1, §4.1 FR-2) — The PRD commits to driverless loopback capture on Windows 10 (1903+), but acknowledges in OQ-1 that capturing the default render device creates an infinite feedback loop unless process loopback (`AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS`) is used. Because process loopback requires Windows 10 build 20348+, Windows 10 builds 18362–19045 will require a virtual audio device, violating the driverless premise for those OS versions. *Fix:* Resolve OQ-1 decisively: either elevate the minimum OS requirement to Windows 11 / Windows 10 build 20348+ for pure driverless operation, or specify a bundled open-source virtual audio cable bridge fallback specifically for pre-20348 Windows 10 builds.

---

## Substance over theater — strong
The PRD is remarkably free of filler, corporate buzzwords, and decorative theater. The 4 Jobs-To-Be-Done in §2.1 (Spatial Audio Enthusiast, Headphone Audiophile, Media Consumer, Multitasking Power User) map directly to 4 distinct, concrete functional pillars (3D Surround HRTF, AutoEq biquad engine, Night Mode compressor, and Per-App WASAPI mixer). There is zero persona theater; each persona drives specific functional requirements and real-world acceptance criteria.

The non-functional requirements in §8 avoid generic boilerplate ("must be fast and reliable") in favor of strict, domain-accurate real-time audio engineering constraints: real-time `AVRT` / `Pro Audio` thread priorities, absolute ban on heap allocations (`malloc` / `new`) in the real-time audio callback to prevent GC/lock pauses, callback execution budgets (<60% buffer time), SSE2/AVX vector intrinsics, and exact resource ceilings (<40 MB RAM, <2.0% CPU, <15ms latency). The reverse-engineering findings in `addendum.md` §2 (`BoomEngine.dll`, `AVFIRFilterSSE`, channel azimuth geometry) substantiate that the team has verified real assembly behavior rather than speculating.

### Findings
*(None — strong dimension)*

---

## Strategic coherence — adequate
The PRD possesses a compelling core thesis: delivering the visual tactile polish and psychoacoustic enhancement of Boom 3D without kernel drivers, latency penalties, account walls, or telemetry. Feature prioritization and success metrics (§7) reinforce this thesis, particularly the pairing of SM-1 (<15ms latency) and SM-2 (zero BSODs) with counter-metrics SM-C1 (do not sacrifice buffer stability for latency) and SM-C2 (do not sacrifice phase coherence for stereo width).

However, an internal strategic disconnect exists between the MVP scope definition and the primary user journeys. In §6.2, Night Mode (FR-18) and Pitch Shifter (FR-20) are explicitly declared "Out of Scope for MVP (Phase 2 & Beyond)". Yet §2.3 elevates Night Mode to one of the four headline user journeys (UJ-3: "Elena Watches Late-Night Cinema Without Volume Jumps"), and the ASCII UI mockup in §4 shows Night Mode active in the primary rack. If Night Mode is deferred to Phase 2, UJ-3 cannot be satisfied in MVP, creating ambiguity for downstream release planning and sprint goal definitions.

### Findings
- **high** MVP Scope Collision: Headline Journey UJ-3 Relies on Deferred Phase 2 Feature (§2.3 UJ-3, §4.5 FR-18, §6.1, §6.2) — UJ-3 is framed as a primary core user journey centered on Night Mode (FR-18), but §6.2 designates Night Mode as Phase 2. Additionally, the §4 UI mockup includes Night Mode in the primary rack, while FR-18 itself lacks a `[Phase 2]` badge. *Fix:* Either promote Night Mode (FR-18) into the MVP scope (§6.1) since dynamic range compression is standard in audio sweetening suites, or mark UJ-3 explicitly as `[Phase 2 Target]` and clarify that the MVP UI rack omits the Night Mode button or renders it as a disabled/Phase 2 badge.

---

## Done-ness clarity — strong
Acceptance criteria across all 29 Functional Requirements are defined with quantitative, testable consequences. Rather than relying on vague adjectives like "system responds gracefully" or "high sound quality," each FR specifies explicit thresholds:
- **FR-1:** Device enumeration within 500ms;
- **FR-4:** A/B bypass switching within <10ms buffer without pops/clicks;
- **FR-5:** 0.2–0.7ms ITD and spectral notch for rear channels;
- **FR-9:** 31 bands, ±12.0 dB at 0.1 dB resolution, RBJ biquad formulas, THD+N < 0.005%;
- **FR-13 & FR-14:** 4,000+ headphone database bundled in <5 MB offline with <50ms fuzzy search latency;
- **FR-15:** Preamp attenuation matching positive peak gain (e.g. +6.5 dB peak -> -6.5 dBFS preamp);
- **FR-24:** 2ms lookahead peak limiter clamping at -0.2 dBFS;
- **FR-26:** Exact hex color values (`#16171b`, `#3b82f6`, `#ec4899`).

The only slight ambiguity lies in the slider parameter definitions for FR-17 and FR-18.

### Findings
- **low** DSP Slider Parameter Ranges Underspecified for Fidelity and Night Mode (§4.5 FR-17, FR-18) — While FR-16 (Ambience), FR-19 (Spatial), and FR-20 (Pitch) explicitly specify their slider units and modulation ranges (0–100%, 0–100%, ±12 semitones), FR-17 (Fidelity) does not specify its slider scale, and FR-18 (Night Mode) mentions both a toggle and an "intensity slider" without defining whether the slider modulates threshold (dB), compression ratio, or wet mix. *Fix:* Define the exact range and units for the Fidelity slider (e.g., 0–100% excitation drive) and specify the control parameter for the Night Mode slider (e.g., Compression Threshold from -30 dBFS to 0 dBFS or 0–100% leveling intensity).

---

## Scope honesty — adequate
The PRD is admirably transparent about Non-Goals (§5), firmly ruling out kernel drivers, telemetry/accounts, VST hosting, microphone ANC, and mobile ports. Phase boundaries in §6 distinguish Phase 1 MVP from Phase 2.

However, the Assumptions mechanism suffers from incomplete roundtrip indexing. While Section 11 lists 4 assumptions, none of them are tagged inline at the requirements where they originated. Conversely, line 168 introduces `[ASSUMPTION: Phase 2]` on FR-8 (Custom HRTF SOFA loading), which is completely absent from the Section 11 Assumptions Index. Furthermore, deferred Phase 2 requirements (FR-18 Night Mode, FR-20 Pitch Shifter) are presented inline in §4.5 without `[Phase 2]` or `[DEFERRED]` tags, obscuring the fact that they are excluded from MVP.

### Findings
- **medium** Assumptions Index and Inline Tag Roundtrip Mismatch (§4.2 FR-8, §4.5 FR-18, FR-20, §11) — The Section 11 Assumptions Index contains 4 entries (`[ASSUMPTION: WASAPI Pipeline]`, `[ASSUMPTION: Phase 2 Effects]`, `[ASSUMPTION: Headphone DB Packaging]`, `[ASSUMPTION: Default HRTF Profile]`), none of which appear inline in §4. Conversely, the inline tag `[ASSUMPTION: Phase 2]` on FR-8 does not exist in Section 11. FR-18 and FR-20 also lack inline phase annotations. *Fix:* Establish a strict 1:1 roundtrip between inline tags and the Section 11 index. Tag FR-2, FR-5, FR-8, FR-13, FR-18, and FR-20 with their matching assumption/phase identifiers.

---

## Downstream usability — strong
The PRD is structured cleanly for downstream consumption by UX designers, software architects, and story authoring workflows:
- **Glossary (§3):** Comprehensive, providing 15 clear, domain-specific definitions that are consistently used throughout the requirements.
- **Identifiers:** Contiguous, unique identifiers without gaps or collisions (FR-1 through FR-29, UJ-1 through UJ-4, SM-1 through SM-6, SM-C1, SM-C2).
- **Protagonists:** All four User Journeys have named protagonists carrying distinct contexts (Alex / Headphones, Marcus / Spatial Audio, Elena / Cinema, David / Multitasker).
- **Traceability:** Functional Requirements clearly indicate which UJs they realize (e.g. FR-1 realizing UJ-1, UJ-2, UJ-3), and Success Metrics explicitly map to the FRs they validate.

The single downstream hazard is the aforementioned scope collision where downstream teams writing stories for MVP Phase 1 might mistakenly attempt to implement Night Mode (FR-18) to fulfill UJ-3.

### Findings
*(Covered under Strategic Coherence finding on UJ-3)*

---

## Shape fit — strong
The PRD's shape is an ideal fit for an open-source technical audio software product. It strikes the correct balance between high-level user value (visual dials, headphone correction, late-night leveling) and low-level DSP/systems engineering constraints (real-time threads, WASAPI loopback, SIMD math, biquad filter formulas). The decision to split architectural options and reverse-engineering findings into `addendum.md` preserves narrative clarity in `prd.md` while giving technical implementers exact azimuth configurations and DLL references.

### Findings
*(None — strong dimension)*

---

## Mechanical notes
1. **Assumptions Index Roundtrip:** Failure between Section 11 and Section 4. Tag names do not match (`[ASSUMPTION: Phase 2]` in FR-8 vs `[ASSUMPTION: Phase 2 Effects]` in §11), and 4 index entries are missing their inline callouts.
2. **OS Build Number Continuity:** Section 8 targets Windows 10 version 1903 (build 18362), while Section 10 OQ-1 identifies process loopback APIs that only exist on Windows 10 build 20348+.
3. **Cross-References:** All FR, UJ, and SM cross-references resolve cleanly with 0 dangling pointers.
4. **Glossary Term Consistency:** All 15 glossary terms are used verbatim across the document without synonym drift (e.g., "3D Surround Dial" and "AutoEq" are used consistently throughout).
5. **UI Mockup Parity:** The ASCII layout in §4 accurately mirrors the feature breakdown across the master engine, dial, effects rack, and AutoEq equalizer.
