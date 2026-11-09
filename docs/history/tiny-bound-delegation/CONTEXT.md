# Tiny bound and team delegation — Context

**Feature slug:** tiny-bound-delegation
**Date:** 2026-11-09
**Shaping session:** complete (spec-drop sup-20261109-7c3a from waggledance@44009e1; gate_bypass full, every gray area below is the leader's named guess)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Make `tiny` mean genuinely tiny, and make that bound a check, not only
prose. A tiny job is one hand-authored file (plus mechanical companions
such as a version bump or regen output), rewrites no existing test, and
needs one cheap proof. Everything larger is `small` or above and runs
through dispatched team workers. The feature ends before any change to
the inline-versus-dispatched A/B for real tiny cells
(`docs/decisions/ab-tiny-protocol.md`) and before any change to how
`small` and larger lanes are sized.

Trigger: in anphabe-goglbe (2026-11-09, bee 2.79.1) a leader classified
"hide resolved comments by default" as tiny because the core edit was one
line, then ran it inline: 6 files, 5 rewritten browser-drive steps, 4 red
runs. The size was read from the core edit, not from the blast radius.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The owner prefers team delegation for all work that is not genuinely tiny: each role runs a different configured model, so dispatched work is more effective. The leader keeps deciding, gating, synthesis and the human conversation. (d7959afc) | Doctrine text names this as the reason for the delegation default. |
| D2 | `tiny` = exactly one hand-authored file, plus mechanical companions only (a version or cache-bust bump, regen output, the release manifest); no existing test is rewritten; one cheap proof. A second hand-authored file of any kind — a test (new or rewritten), a spec or knowledge doc beside code, a second source file — makes the work `small` or above. The `small` trigger (0–1 flags, ≤3 product files) does not change. | For the tiny bound only, a hand-authored doc beside code counts; R14 (lane-ceremony-v3 D6) still keeps docs out of the small and larger caps. Replaces the old tiny trigger "≤2 product files". |
| D3 | From `small` up, execution goes through `bee dispatch prepare` to the configured role; inline execution outside tiny needs a recorded named reason. This is already enforced at cap (`handlers_close.rs:482-507`, `--inline-reason`); the feature changes only the doctrine text that explains it (names D1 as the reason) and adds no second escape. | Chesterton's fence: the tiny carve-out at cap stays, because the A/B (decision e9c23b0a) measures inline tiny on purpose. |
| D4 | Mechanical check at cap: a `tiny` cell capped with no registered execution worker whose counted `files_changed` is more than one is refused unless `--inline-reason "<why>"` is given. Counted = every path except `.bee/**` and `docs/history/**`. The refusal names the reroute: `bee route --set --lane small …`, then dispatch the next cell with `bee dispatch prepare --claim`, or `--inline-reason` for a mechanical companion. A dispatched tiny cell is not refused. | A refusal with a named escape, not a warning: the trigger leader ran under gate_bypass full and would not have stopped for a warning. |
| D5 | Mechanical check at route: `bee route --set --lane tiny --files N` (and so `bee feature start`) refuses N > 1, naming `--lane small`. | The route record is the classification moment; refusing there catches the mistake before any edit. |

### Agent's Discretion

- Exact refusal wording, as long as it names the counted files, the bound,
  the reroute and the `--inline-reason` escape.
- Where the counting helper lives, as long as there is one helper.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| hand-authored file | A file a person or agent writes by judgment: source, test, spec, knowledge doc, skill text. |
| mechanical companion | A file changed only as a direct consequence of the hand-authored one: a version or `?v=` bump, regen output, the release manifest. |
| counted file | For the D4 check: any `files_changed` path outside `.bee/**` and `docs/history/**`. |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/doctrine-layer/lane-and-working-discipline.md | confirmed | Holds R13, R14 and R16a, the tiny trigger and file-cap rules this feature amends. |
| docs/knowledge/areas/workflow-state/cells-completion-judge-and-archive.md | suspect | The cap path where the D4 check lands. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs` | Cap-time lane checks; the registered-worker refusal and `--inline-reason` escape (lines 482-507). |
| `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs` | `bee route --set` validation of lane and `--files` (lines 564-660). |
| `docs/decisions/ab-tiny-protocol.md` | Why inline tiny stays legal. |
