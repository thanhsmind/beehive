# Advisor consult — pi-result-mailbox high-risk gate (dispatch f7229ea9, fable, 2026-08-30)

CONSULT VERDICT: proceed-with-conditions —
1. Fence-escape handling for inline reports (or path-only injection) — D5 is not met without it.
2. Rename the delivery guarantee to at-least-once with job-id dedupe; test replay-after-restart explicitly.
3. Marker write ordered before pane spawn; grace re-probe (or explicit missing-report marker) after result parse.
4. D7 caveat replacement text names the residual limits (at-least-once async, live-session drain, sync path primary).

Axis verdicts: (1) contract honesty red-flag narrow — brief-text ordering unenforced, add explicit expected-but-missing surfacing; (2) exactly-once red-flag on the claim, green on the mechanism — pi-peer is at-least-once; marker-before-spawn closes the silent-never-delivered window; (3) injection safety red-flag — a fixed fence around attacker-writable markdown is escapable; path-only injection recommended; (4) D7 lift green-flag conditioned on honest replacement text; (5) no structural refusal.

All four conditions are folded into plan.md rev 2 (path-only injection, at-least-once + dedupe naming, pre-spawn marker + stale/missing report_note, caveat replacement text) together with reviewer a56966b0's four P1 fixes.
