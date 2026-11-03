# Pi stage dispatch — hat wave synthesis

**Feature:** pi-stage-dispatch
**Date:** 2026-09-15
**Wave:** plan step, 3 seats (standard lane, 8 product files)
**Plan decision:** `15109d96`

## Seats

| Seat | Transport | Result |
|---|---|---|
| hat-facts-gaps | native (opus) | returned: 4 BLOCKER, 10 WARNING |
| hat-alternatives | native (opus) | returned: PASS with changes |
| hat-user-impact | herding agy-flash, job `job-1789462139790-623745-1` | **DROPPED** — still reading at the 10-minute ceiling (launched 08:48:59 UTC, pane closed 08:58 UTC, no result file). Its questions are named in "Open for the live run" below. |

## PLAN CHECK

Work: slice 1, cells psd-1..psd-8

### STRUCTURE

BLOCKERS (all fixed in plan.md before the gate):

- Coverage/key links — the state.json fallback is live for every runtime; the first draft removed it. Evidence `prepare.rs:1814`. Fix: kept the fallback, added the granted-worktree step before it (psd-1, claim 19).
- Cell completeness — onboarding has its own label list without `pi`. Evidence `onboard/templates.rs:352`. Fix: psd-4 files and claim 4.
- Cell completeness — psd-5 regen writes `.claude-plugin/skills`, `.codex-plugin/skills`, `.opencode/skills`, not in files. Fix: added, and the negative check covers all four other trees.
- Cell completeness — psd-4 missed `onboard/templates.rs`. Fix: added.

WARNINGS (applied):

- psd-4 named the wrong regen step for `.agents/skills`; corrected, and the Codex plugin tree strips Pi blocks.
- The bash `PI_SESSION_ID` equal to the drain token is unproven by unit tests; psd-6 takes the token from the harness session, psd-8 proves it live, Open Question added.
- Late-seat drop had no test; psd-6 asserts a non-done result still shows its seat row.
- psd-5 verify `PATH` applied only to the first command; now exported.
- psd-8 verify could not prove a live run; now checks the feature evidence file.
- psd-6 named a stub pane transport that does not exist; now names the real helpers and notes a dry run writes no marker (claim 11).
- psd-2 would rewrite comments that name which Pi docs were read; now only the plain version label.
- psd-3 changes a prompt another checker reads; verify widened to `prompts`.
- The "PI_SESSION_ID empty" matrix row proved nothing new; now asserts the note wording.
- `advisor-ref record` refuses inside a granted worktree; psd-1 now serves it (claim 17).
- `--seat` and the ceiling cap reached Claude and Codex; now Pi only.

### ALTERNATIVES (applied)

- Seat block carries the role name and the Hat wave pointer, never the configured description (stale second copy; path now skill-relative for host repos).
- No render sidecar schema change.

### CELLS (leader cold-pickup self-check, reviewed: 8)

CRITICAL FLAGS: none after the fixes above.
MINOR FLAGS: psd-1 finds the advisor-ref refusal site by search (`rg -n emit_unsupported_root …/state_group`) instead of a named line — the call site was not opened before the gate.
CLEAN CELLS: psd-2, psd-3, psd-4, psd-5, psd-6, psd-7, psd-8

SUMMARY: The shape survived the wave. Two design picks changed (keep the fallback; seat block without description), one step joined scope (advisor-ref from a worktree), and the remaining control-plane verbs became a named slice 2.

## Open for the live run (from the dropped seat's questions)

psd-8 observes each and records what the Pi user and leader see:

- a seat that misses the budget, and a result that arrives after it was dropped, or twice;
- `PI_SESSION_ID` empty;
- a Codex reader of `On Pi:` text;
- the one plain state line the human sees while the wave runs.

## Dismissed

- Reuse `--nickname` as the seat name (alternatives) — dismissed: the nickname already names the worker in its ack file; one word would carry two meanings.
- `.pi/settings.json` exclusions to make a `.pi/skills` tree work (alternatives, unverified) — dismissed: the docs do not say exclusions apply to `.agents/skills` discovery, and the shared-tree render needs no host setting.
