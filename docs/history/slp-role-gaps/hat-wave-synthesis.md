# slp-role-gaps — plan check (hat wave synthesis)

Seats: hat-facts-gaps, hat-alternatives, hat-user-impact (opus each, native). No seat dropped. Dispatch ids: 65416fa9 (facts-gaps), f6ab4ce9 (alternatives), 6819e90b (user-impact).

PLAN CHECK
Work: slice 1, all nine decisions

STRUCTURE
BLOCKERS: none. All 11 claim rows matched their anchors verbatim.
WARNINGS:
- Decision coverage (D8): leader-side home missing → fixed: rig-6 owns `skills/bee-swarming/SKILL.md` as a pointer to the one Result Formats home.
- Cell completeness (D7): no inbox writer, no baseline field, the open cwd question unanswered → fixed: the notice uses `append_message` (kind `progress`); baseline `diff_fingerprint` / `diff_fingerprint_since` on the job record; `cwd` and `read_only` read from the job record (claims 12-14).
- Cell completeness (D1/D2 docs): the envelope concept had no owner → fixed: rig-3 owns `the-run-verb-and-worker-outcomes.md`; rig-5 moved to `overview.md`.
- Truth table (D9): post-loop observation would discard a finished result; stop vs archive unsaid → fixed: refuse only while polling, stop never archive (claim 15).
- Truth table (D7): both stall sites, reason field, recovery, read-time firing → fixed in the settled list and rig-5 action.
- Truth table (D2): native strict vs herding lenient partial → fixed: herding emits `stopped_missing: true`.
- Prose claims "D6 is a prompt step" and "regen per barrier" → the first is now stated in the settled list; the second rests on `bee dev regen`'s own help (render-skill-trees runs first).

CELLS (reviewed: 6)
CRITICAL FLAGS: none after the fixes above.
MINOR FLAGS: rig-6 and rig-7 prove with the instruction-law suite plus the barrier regen; no stronger docs proof exists.
CLEAN CELLS: rig-1, rig-2, rig-3, rig-5, rig-6, rig-7

ACCEPTED SHAPE CHANGES
- A1: `record_dissent` keeps its signature; rig-1 adds an evidence-taking sibling and never touches `run.rs`.
- A2: the planned rig-4 merged into rig-3 (both edited `result_envelope`); three serial waves became two.
- A3: refuse reuses the `herding interrupt` stop command; no new stop path.
- User-impact: refusal and notice texts carry fix lines; native `reaction` matches case-insensitively; the four-layer report is omitted when nothing runnable ships; worker-cell.md gets one short block.

SUMMARY: No blocker. The plan now names every D7 and D9 edge the seats found, cuts one serial wave, and gives each refusal a fix line.

Dismissed:
- W6 (drop `registry_payload.json` from rig-1, let regen write it) — dismissed: the payload is declared by hand in the same change as the served flag (claim 16), regen does not write it.
- User-impact "make herding reaction strict too" (implied by the one-field-two-behaviors note) — dismissed: D1 locks lenient herding parsing; the gap is surfaced with `stopped_missing` instead.
- User-impact "falsifier is prompt-only, nothing checks it" — dismissed: D4 asks for a prompt rule, not a door; a checker would be new scope.
- A5 "drop worker-details.md and the dissent concept" — partly taken: worker-details.md dropped; the dissent concept stays because D3 changes the verb's behavior and that concept is its one home.
