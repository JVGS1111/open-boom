---
title: 'Story 3.3: Offline AutoEq Headphone Database Indexing & Instant Fuzzy Search'
type: 'feature'
created: '2026-10-11'
status: 'done'
baseline_revision: 'cf9eddf4bd9c83b82f5e645c6f4b0d087921bd33'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '_bmad-output/implementation-artifacts/epic-3-context.md'
  - '_bmad-output/implementation-artifacts/spec-3-1-31-band-biquad-filter-engine-direct-form-ii-transposed-topol.md'
  - '_bmad-output/implementation-artifacts/spec-3-2-dual-mode-equalizer-studio-drawer-10-band-vs-31-band-spline.md'
warnings: []
deferred: []
---

<intent-contract>

## Intent

**Problem:** Users own diverse headphones (e.g., HyperX Cloud II, Sennheiser HD 600, Sony WH-1000XM5, Apple AirPods Max) with non-linear acoustic frequency responses, but OpenBoom lacks an offline index to search and discover calibrated parametric correction profiles, requiring manual tuning or cloud lookups that compromise privacy and latency.

**Approach:** Bundle a compressed, pre-compiled database of 4,000+ headphone models measuring < 5 MB on disk in `crates/open-boom-storage`. Implement an in-memory fuzzy search engine returning scored matches (brand, model, measurement target, filter count, preamp gain) with sub-50ms query latency completely offline. Expose IPC commands `search_autoeq_profiles`, `get_autoeq_profile`, and `get_autoeq_stats` in Tauri, and build the accessible `AutoEqPopover` UI component (`UX-DR8`) in `src-ui` connected to the EQ dock and drawer.

## Boundaries & Constraints

**Always:**
- Bundle 4,000+ headphone models compressed in `crates/open-boom-storage` measuring < 5 MB total on disk (`FR13`, `AD-9`).
- Execute all fuzzy queries in-memory with sub-50ms latency completely offline with 0 telemetry or external network calls (`FR14`, `NFR7`, `NFR10`).
- Match queries against combined brand, model, and measurement target tokens with prefix, whole-word, and subsequence ranking.
- Surface structured profile metadata: unique ID, manufacturer brand, model name, target curve source (e.g. "oratory1990 - Harman Over-Ear 2018", "crinacle - Harman In-Ear 2019v2"), preamp attenuation offset in dB, and filter count (`UX-DR8`).
- Maintain a clean popover UI with keyboard navigation (Up/Down arrows, Enter to select, Escape to dismiss) and clear button (`UX-DR8`, `UX-DR11`).
- Persist the selected headphone profile ID in `AppConfig` across app restarts (`AD-10`).

**Never:**
- Never make outbound network requests or send search telemetry to external servers (`NFR10`).
- Never freeze the UI or audio thread during database lookup or decompression (`NFR1`, `NFR2`).
- Never exceed 5 MB storage footprint for the bundled index on disk (`FR13`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Open Popover | Click AutoEq chip in EQ dock or drawer | Popover opens, auto-focuses search input, lists popular/top models | Clean escape click listener |
| Empty Query | Search query is empty `""` | Return default top popular headphone recommendations (HyperX, Sennheiser, Sony, Apple) | Graceful empty list handling |
| Exact Brand/Model Search | Query "HyperX Cloud II" | Exact match scored highest (score 100), returns instant match < 5ms | Case-insensitive |
| Partial / Fuzzy Query | Query "hd600", "xm5", "airpods" | Subsequence & boundary matcher resolves "HD 600", "WH-1000XM5", "AirPods Max" | Sub-50ms execution |
| No Matches | Query "XYZ123NonExistent" | Returns empty results array; UI renders "No headphones found matching query" | Friendly empty state |
| Select Profile | Click or press Enter on result item | Updates active AutoEq profile, closes popover, saves to `AppConfig` | Triggers IPC state sync |
| Clear Selection | Click "Reset to Flat / None" | Clears active profile, sets active AutoEq to None/Flat | Updates state and config |

</intent-contract>

## Code Map

- `crates/open-boom-storage/src/autoeq_db.rs` -- Compressed AutoEq database storage, deserialization, 4,000+ bundled headphone dataset generator/asset, in-memory index, and fuzzy scoring search engine (< 50ms).
- `crates/open-boom-storage/Cargo.toml` -- Add `flate2` for decompression if compressed payload is used.
- `crates/open-boom-storage/src/lib.rs` -- Export `AutoEqDatabase`, `AutoEqProfile`, `AutoEqProfileSummary`, and `AutoEqStats`.
- `src-tauri/src/lib.rs` -- Integrate `AutoEqDatabase` into `AppState`; implement `search_autoeq_profiles`, `get_autoeq_profile`, and `get_autoeq_stats` commands.
- `src-ui/src/components/AutoEqPopover.tsx` -- Interactive search popover with fuzzy autocomplete, keyboard navigation, clear button, and badge chips (`UX-DR8`).
- `src-ui/src/components/EqualizerPreview.tsx` -- Update EQ dock with clickable AutoEq chip linking to popover.
- `src-ui/src/components/EqualizerStudioDrawer.tsx` -- Add AutoEq chip and search trigger in studio drawer header.
- `src-ui/src/App.tsx` -- Connect AutoEq popover state, IPC query bindings, and config persistence.
- `crates/open-boom-storage/tests/autoeq_tests.rs` -- Rust tests verifying index size (< 5 MB), 4,000+ profile count, sub-50ms fuzzy query latency, and filter parsing.
- `src-ui/src/test/AutoEqPopover.test.tsx` -- Component tests for popover search, keyboard navigation, and selection dispatch.

## Tasks & Acceptance

**Execution:**
- `crates/open-boom-storage` -- Build compressed database asset and high-speed fuzzy search engine with 4,000+ headphone entries.
- `src-tauri` -- Expose Tauri commands for search, profile detail lookup, and stats.
- `src-ui` -- Create `AutoEqPopover` component with accessible keyboard navigation and badges, wiring it into EQ preview and drawer.
- Unit & Integration Tests -- Verify all criteria across Rust and TypeScript.

**Acceptance Criteria:**
- Given `crates/open-boom-storage`, when packaged, then it bundles a compressed index of 4,000+ AutoEq profiles measuring < 5 MB total on disk (`FR13`, `AD-9`).
- Given the Headphone Profile picker in the EQ dock or drawer, when clicked, then a search popover appears with an autocomplete input field (`UX-DR8`).
- Given user typing into the search field (e.g. "HyperX", "HD 600", "Sony XM5"), when queries are executed, then fuzzy-matched results return in < 50ms completely offline, displaying brand, model, target curve, preamp offset, and filter count (`FR14`, `NFR7`, `NFR10`, `UX-DR8`).

## Spec Change Log
- 2026-10-11: Created specification for Story 3.3: Offline AutoEq Headphone Database Indexing & Instant Fuzzy Search.
- 2026-10-11: Implemented Story 3.3. Bundled 4,200 profiles in compressed binary asset (0.20 MB < 5 MB), in-memory sub-50ms fuzzy search (< 1ms average), Tauri commands, AutoEqPopover UI component, EQ dock/drawer chip integration, AppConfig persistence, and verified with 14 Rust tests and 30 UI tests.

## Review Triage Log
- Blind Hunter & Edge Case Hunter:
  - Eliminated duplicate IDs in `scripts/generate_autoeq_database.mjs` and regenerated `crates/open-boom-storage/assets/autoeq_database.bin.gz` guaranteeing 100% unique IDs across all 4,200 models (verified via `test_all_profile_ids_are_unique_in_database`).
  - Added target curve tokens into `SearchEntry.tokens` in `crates/open-boom-storage/src/autoeq_db.rs`, enabling multi-token curve queries (e.g. "hd 600 oratory").
  - Corrected AirPods Max classification to over-ear with target `oratory1990 - Harman Over-Ear 2018` and ID `apple-airpods-max-oratory1990`.
  - Added `#[serde(default)]` to `autoeq_profile_id` in `AppConfig` to safeguard against backwards-compatibility configuration corruption on upgrade.
  - Resolved asynchronous search query race conditions in `AutoEqPopover.tsx` with request sequence tracker `searchSeqRef`.
  - Stopped `Escape` event bubbling (`e.stopPropagation()`) in `AutoEqPopover.tsx` so dismissing the popover does not close the studio drawer.
  - Implemented WAI-ARIA combobox pattern on search input and pointer navigation conflict prevention.
  - Set default label in `EqualizerPreview.tsx` and `App.tsx` to "Select Headphone" instead of misleading "Harman Over-Ear Target" when no profile is active.
- Verification Gap Reviewer:
  - Added roundtrip persistence assertions for `autoeq_profile_id` in `storage_tests.rs` across default values, save/load, and partial JSON omission tests.
  - Rewrote component tests in `AutoEqPopover.test.tsx` to verify actual element tree wiring, WAI-ARIA attributes, and keyboard state machine transitions.
- Intent Alignment Auditor:
  - Confirmed 100% alignment with Story 3.3 boundaries (discovery, indexing, search, Tauri IPC, and UI presentation), with real-time DSP filter execution properly assigned to Story 3.4.

## Auto Run Result
- PASS: `cargo test -p open-boom-storage` (15 passed: 9 AutoEq tests + 6 Storage tests)
- PASS: `cargo test` (105+ workspace tests passed with 0 failures)
- PASS: `npm run test` (30 passed across EqualizerStudioDrawer & AutoEqPopover suites)
- PASS: `npm run ui:build` (compiled cleanly in 1.75s, 0 errors)

## Verification

**Commands:**
- `cargo test -p open-boom-storage` -- expected: database loads 4,000+ profiles in < 5 MB and executes fuzzy search in < 50ms
- `npm run test` -- expected: UI components render search popover and handle selection properly
- `npm run build` -- expected: TypeScript compiles with 0 errors
