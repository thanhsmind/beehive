# vt-1 — Mandatory ak-style per-step tick contract

**Status:** [DONE]

**Outcome:** Rewrote `skills/bee-hive/references/routing-and-contracts.md`'s "Progress
ticks" section into the full per-step visibility law: every perceivable pipeline step
emits exactly one short chat line, on by default, in the user's work language. Bypass
silences questions, never ticks; a red/refusal is never quiet-able. Added the fixed
glyph format (`▸`/`✓`/`✗`/`⚡`) and one merged tick catalog (20 events, each with a
worked example) — route recorded, gate passed/auto-approved, cells created, worker
dispatched, `[DONE]`/`[BLOCKED]` received, cell capped, fix cell opened, feature verify
started/green/RED, feature-verify recorded, barrier paid, knowledge synced, learnings
compounded, feature closed, plus the prior cap-seam/slice/re-lane/PR rows folded in —
one list only. Narrowed the silence switches: `quiet` (whole stream, never the red
line) and `ship_visibility: "off"` (PR ticks only, not the whole stream). Ship-visibility
PR wiring left byte-identical. Updated `provenance.md`'s silent-bookkeeping row to match.

**Files touched:**
- `skills/bee-hive/references/routing-and-contracts.md`
- `skills/bee-hive/references/provenance.md`

**Commit:** `587727c7` — docs(step-ticks): vt-1 — mandatory ak-style per-step tick contract

**Verify:** Not run per R82 (main-verifies) — capped `--feature-verify-pending`. Census
patterns in `packages/bee/tests/test_misc.mjs` that pin `routing-and-contracts.md`
(AO14 execution-worker class, native-Codex ordered-wait contract) were checked BY GREP
against sections this cell did not touch — unaffected.

Full trace/evidence: `.bee/cells/vt-1.json`.
