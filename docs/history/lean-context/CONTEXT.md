# Lean Context — Context

Asked: "b" (2026-11-04), choosing to build items 1 and 3 of the token
reflection now: a handoff mark that fires on a 1M window, and a cheaper
model for docs workers.

## Feature Boundary

In scope: D1 and D2. Out of scope: fewer round trips, a smaller session
preamble, and trimmed tool output (items 2, 4, 5 of the reflection).

## Locked Decisions

- **D1 (9bf28c33).** The prompt-context hook adds one line when the
  latest request context reaches `handoff_context_tokens` in
  `.bee/config.json` (default 150000; 0 turns it off). The line names the
  token count and tells the agent to finish the step, write a
  planned-next handoff, and continue in a fresh session. The doctrine
  mark becomes about 150k tokens or 65% of the window, whichever comes
  first, and every feature close is also a handoff point.
- **D2 (88109b52).** `team.claude.docs` runs on sonnet. Review, plan,
  test and judge roles stay on opus.

## Evidence

- Session 5b57a891: 888 main calls, average context 303k, peak 505k, no
  compaction; 133.8M of 134.5M main tokens were cache reads.
- All 7 workers of that session ran opus at high effort, including
  prose-only skill edits.

## Outstanding Questions

None.
