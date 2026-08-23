---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-08-23
---

# Plan: tmux Herding Cockpit

Mode: `standard` — 2 risk flags: cross-platform, external-systems (tmux)
Why this is the least workflow that protects the work: every change sits
behind the phase-1 seam (`PaneTransport`) and one config key; the cockpit's
safety rules (interlock, human merge, enumerated allowlist) are untouched.

## Requirements (from CONTEXT.md)
- D1: full cockpit on tmux behind `herding.transport`.
- D2: roles and bootstrap use only transport-neutral bee pane verbs.
- D3: tmux mapping — session=workspace, window=tab, title=label, caller's pane=chat.
- D4: waves/occupancy select by the same key; one classifier in `fleet`.
- Inherited tmux-herding-transport D1–D5.

## Discovery
- Phase 1 left `PaneTransport` (14 methods) with `RealHerdr`/`RealTmux` and one construction helper `transport_for_run(main_root)` (`run.rs`). The cockpit needs six more pane operations (send-text, rename, list with labels, tab list/focus, pane-id by label) — a second trait keeps the phase-1 fakes untouched.
- The cockpit's herdr vocabulary is 14 verbs over 59 markdown lines plus 19 script calls (`rg "herdr [a-z]+ [a-z-]+" skills/bee-herding`).
- `fleet` is a library crate (integration tests possible); `bee` is a binary crate (in-module tests only). `bee` depends on `fleet`, so the shared classifier moves DOWN into fleet (D4).
- `allowed_tools_for(role)` is a byte-copied constant (`control_loop.rs:212`); the tmux arm swaps `Bash(herdr:*)` for `Bash(tmux:*)` and nothing else.

## Approach
Recommended: (1) transport switch for occupancy + allowlist; (2) `fleet::backend::tmux::TmuxBackend` + classifier move; (3) `bee herding pane …` verb group over `CockpitTransport`; (4) bootstrap on bee verbs; (5) role docs on bee verbs; (6) knowledge/skill/regen sync.

Rejected:
- A tmux column beside every herdr line in the role docs — two vocabularies for a cold agent (D2).
- Bootstrap as a Rust verb — bash on bee verbs is smaller and keeps the dry-run shape.

Risk map:
| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| pane verbs | MEDIUM | new CLI surface the roles depend on | in-module tests per verb with both fakes; JSON shape pinned |
| role-doc rewrite | MEDIUM | 59 lines; a missed line breaks a cold role | `rg "herdr " skills/bee-herding/references` → 0 command hits; dry-run role walk |
| fleet TmuxBackend | LOW | mirrors HerdrBackend with a stub binary | `fleet/tests/tmux_backend.rs` |
| bootstrap | MEDIUM | live pane creation | `--dry-run` prints the bee verb lines; live proof at uat |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 (current) | occupancy + allowlist on tmux; `TmuxBackend`; pane verb group; bootstrap + role docs on bee verbs; knowledge sync | the whole cockpit is one slice — a half-converted role doc is worse than none | `herding.transport=tmux` → bootstrap opens cockpit + runtime windows in the caller's session; dispatch role spawns a worker pane; merge role lands it | uat |

Current slice: six cells `thc-1` … `thc-6`. Smaller-path check: a waves-only slice was offered and refused by the user (D1); within the full scope, the bootstrap stays bash (no Rust verb) and the phase-1 trait is untouched (a second trait) — both the cheaper paths. Review wave: one bee-review pass before the gate.

## Test matrix
- Happy: tmux key → `occupancy` counts `list-panes -a` ids; `wave` builds `TmuxBackend`; each pane verb emits the same JSON keys on both fakes; `allowed_tools_for` tmux arm carries `Bash(tmux:*)`; bootstrap `--dry-run` prints bee verb lines only.
- Edge: key absent → every output byte-identical to today (existing tests green); `pane-id --label x` with no match → typed not-found; `tab-focus` on tmux = `select-window`.
- Error: tmux missing → occupancy falls back (`source: fallback`); a pane verb on a bad pane id → non-zero with the argv named; illegal key → refusal naming both values.
- Existing coverage: `wave.rs` occupancy tests (:1698-1723), `control_loop.rs` tests, `fleet/tests/herdr_backend.rs`, `run.rs` fakes, `tmux.rs` tests.

## Out of scope
- Native Windows tmux; orchestrator HANDOFF; changes to the enable interlock, the merge gesture, or the permission posture.
