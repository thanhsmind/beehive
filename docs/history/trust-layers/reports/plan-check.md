# Plan check — trust-layers (hat wave synthesis, one round)

Seats returned: 3 of 3 (facts-gaps, alternatives, user-impact). None dropped.
Reports: `.bee/mailbox/job-1794553554638-420881-1/report-1.md`,
`.bee/mailbox/job-1794553556127-421091-1/report-1.md`,
`.bee/mailbox/job-1794553557623-421637-1/report-1.md` (main checkout).
No seat ran a test: each was read-only.

```text
PLAN CHECK
Work: trust-layers, slice 1 (trl-1 .. trl-5)

STRUCTURE
BLOCKERS (all closed before the gate):
- Cell completeness / trl-1: "recorded" in commands_notices is string-only
  (notices.rs:214), so a list-valued commands.test reads as not recorded and
  case (b) contradicted case (c). Fix: D5 rules the list form as recorded;
  claims 15 and 16 added; the matrix row now expects an empty result.
- Cell completeness / trl-2: the stored entry is a struct built by literals
  in handlers_close.rs, drivers/close.rs and work.rs. Fix: files added; D6
  puts the field on the entry and the letter item.
- Claims / unrowed: "the regen chain reads the command registry" had no
  support. Fix: removed; the serial order rests on the shared build alone.
WARNINGS (applied):
- trl-3 anchored on the count verb. Re-anchored on build_digest and
  digest_summary_line; prose_load is a top-level key (D7).
- Absent sources read as null, and bee's lines are apart from the
  project's (D7).
- Marker-word match, lockfile runner, run-before-offer, record the
  owner's answer (D5).
- The reviewing-skill sentence had no real target and was a second home.
  Dropped (D6).
- The registry file is hand-edited by a compact JSON round-trip. Carried
  into trl-2.
- The worktree onboard report lists runtime files. Carried into trl-4.
- trl-5 named no onboarding target. Named.

CELLS  (reviewed: 5)
CRITICAL FLAGS: none after the fixes above.
MINOR FLAGS: trl-1, trl-2, trl-3 — a name-filtered proof passes on zero
  matches; each action now requires the matched test count.
CLEAN CELLS: trl-4, trl-5

SUMMARY: The shape (five serial cells, one slice) stands. Three blockers
were plan defects, not design defects, and each is closed by a logged
ruling (D5, D6, D7) and a cell edit.
```

## Dismissed

- Count nested AGENTS.md, Copilot, Gemini and Cursor rule files — outside
  D4's fixed list of four files; no request text asks for it.
- A status or doctor surface that prints the prose load in a host — not
  asked for; named under Known limits in delivery.md.
- A pointer from the backlog row text to the craft page — the promotion
  tree is where a row is taken, and it gets the pointer.
- Detection below the repo root and for further ecosystems — silence is
  safe; left as an open choice of trl-1.
- A lesson that names its sources — the lesson key and shape stay as
  locked; the source is on the entry and in the letter.
- trl-4 depending on trl-2 only — the serial order is a build edge for
  every cell, so the wider deps stay.
- Echo `source` casing variants and repeated flags as separate cases —
  the set is exact and lower-case; one refusal test covers off-set values.
