---
title: 'Privacy Notice & Audio Interception Microphone Permission Clarification'
type: 'feature'
created: '2026-10-13'
status: 'done'
baseline_revision: '20bca779570c8a6142bc07f6337674854c0ef93e'
review_loop_iteration: 0
followup_review_recommended: false
context: []
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** When OpenBoom captures internal audio for DSP processing (via virtual device or loopback), Windows and antivirus software alert users that OpenBoom is "accessing the microphone" because Windows Core Audio groups all audio capture under generic microphone permissions. This causes alarm that user voice is being recorded.

**Approach:** Add clear, reassuring privacy notices with a shield icon in `DriverOnboardingModal`, `SettingsModal`, and all i18n dictionaries (`en`, `pt-BR`, `es`) explaining that Windows groups loopback capture under microphone permissions, that OpenBoom processes solely internal playback audio for spatialization/EQ, and never captures, records, or transmits voice.

## Boundaries & Constraints

**Always:**
- Maintain 100% i18n dictionary parity across `en`, `pt-BR`, and `es`.
- Use Fluent Dark theme design consistent with existing modals (`bg-[#1a1b20]`, `border-border-subtle/40`, cyan/shield accents).
- Render accessible semantic markup (`role="note"` or informational section with clear contrast).
- Include comprehensive unit tests verifying presence, localization, and accessibility.

**Never:**
- Alter audio capture pipelines or DSP buffers.
- Introduce heavy external dependencies or external privacy tracking.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Onboarding Modal open | `DriverOnboardingModal` open | Privacy disclaimer card rendered with shield icon and explanation | Visible and accessible |
| Settings Modal general tab | `SettingsModal` open | Privacy notice banner rendered under Audio Bridge section | Visible and accessible |
| Language change | Switch locale between pt-BR, en, es | Privacy strings dynamically translate with zero untranslated fallback | 100% dictionary coverage |
| Small / mobile viewport | Narrow viewport width | Notice gracefully wraps without overflow or clipped text | Responsive flexbox/grid layout |

</intent-contract>

## Code Map

- `src-ui/src/i18n/types.ts` -- Define privacy notice keys in `driverOnboarding` and `settings`.
- `src-ui/src/i18n/locales/en.ts` -- English privacy text.
- `src-ui/src/i18n/locales/pt-BR.ts` -- Portuguese privacy text.
- `src-ui/src/i18n/locales/es.ts` -- Spanish privacy text.
- `src-ui/src/components/DriverOnboardingModal.tsx` -- Embed privacy card with shield icon and clear explanation.
- `src-ui/src/components/SettingsModal.tsx` -- Embed privacy notice banner in Audio Bridge section.
- `src-ui/src/test/DriverOnboarding.test.tsx` -- Unit tests for onboarding privacy card.
- `src-ui/src/test/SettingsModal.test.tsx` -- Unit tests for settings privacy notice banner.
- `src-ui/src/test/I18n.test.tsx` -- Dictionary parity tests for privacy keys.

## Tasks & Acceptance

**Execution:**
- `src-ui/src/i18n/types.ts` -- Add `privacyNoticeTitle` and `privacyNoticeDesc` to `driverOnboarding` and `settings` interfaces.
- `src-ui/src/i18n/locales/en.ts` -- Add English translations explaining Windows microphone permission grouping and zero-recording guarantee.
- `src-ui/src/i18n/locales/pt-BR.ts` -- Add Portuguese translations.
- `src-ui/src/i18n/locales/es.ts` -- Add Spanish translations.
- `src-ui/src/components/DriverOnboardingModal.tsx` -- Add styled privacy card with `ShieldCheck` icon.
- `src-ui/src/components/SettingsModal.tsx` -- Add styled privacy notice card in Audio Capture Bridge section.
- `src-ui/src/test/DriverOnboarding.test.tsx` -- Add tests verifying privacy card exists and displays localized explanation.
- `src-ui/src/test/SettingsModal.test.tsx` -- Add tests verifying settings privacy notice exists.
- `src-ui/src/test/I18n.test.tsx` -- Verify 100% key parity across all 3 languages.

**Acceptance Criteria:**
- Given `DriverOnboardingModal` is open, when viewed, then a privacy notice card with a shield icon explaining the Windows microphone permission classification is visible.
- Given `SettingsModal` is open to the general tab, when scrolled to the Audio Bridge section, then a privacy notice explaining the microphone permission grouping is visible.
- Given any supported locale (`pt-BR`, `en`, `es`), when selected, then all privacy texts display in the selected language with 100% dictionary completeness.

## Spec Change Log

<!-- Empty until review loopback -->

## Review Triage Log

- Verifying Dictionary Parity: Passed 100% across en.ts, pt-BR.ts, es.ts.
- Verifying Accessibility: `role="note"` semantic markup and clear contrast applied.
- Verifying UI Tests: 116 tests passing in `run.mjs` (3 new tests added).
- Verifying TypeScript Compilation: 0 errors in `tsc && vite build`.

## Design Notes

- The privacy notice uses a subtle `bg-[#14151a]` / `bg-surface-elevated/60` container with a cyan shield icon (`ShieldCheck` from `lucide-react`) and text sized `text-[11px]` to provide clear reassurance without overwhelming the action buttons.

## Verification

**Commands:**
- `npm --prefix src-ui test` -- expected: All UI tests pass including new privacy notice tests.
- `npm --prefix src-ui run build` -- expected: TypeScript build succeeds with 0 errors.
- `cargo test --workspace` -- expected: All Cargo tests pass.

## Auto Run Result

Status: done
Completed At: 2026-10-13T13:39:30
Findings: 0 blockers, 0 warnings. Full test suite passing (227 Cargo tests, 116 UI tests).

