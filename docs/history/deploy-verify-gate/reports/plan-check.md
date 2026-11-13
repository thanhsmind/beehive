# Plan check — deploy-verify-gate (hat wave synthesis, one round)

Seats returned: 3 of 3 (facts-gaps, alternatives, user-impact). None dropped.
Reports (main checkout): `.bee/mailbox/job-1794558283052-1240333-1/report-1.md`,
`.bee/mailbox/job-1794558284438-1240520-1/report-1.md`,
`.bee/mailbox/job-1794558285907-1241140-1/report-1.md`.
No seat ran a test: each was read-only.

```text
PLAN CHECK
Work: deploy-verify-gate, slice 1 (dvg-1 .. dvg-3)

STRUCTURE
BLOCKERS (closed before the gate):
- Key links / dvg-3: it pointed at a section of
  host-project-artifacts.md that does not exist. Fix: dvg-3 adds one
  short section beside the static-checks section.
WARNINGS (applied, ruled in D7):
- No owner for a control script without drive --full; running
  control-bee drive --full exited 1. bee-verify-upkeep now adds the
  entry points; the template gives exit 2 its own message.
- The notice's both/either mismatch with D3; the notice now fires when
  any existing deploy file lacks the path.
- The retired-key exception was unlocked; dropped.
- The skip flag clashed with the post-deploy Verify name; renamed
  --skip-product-drive, and it never overrides a red drive.
- A project-owned release script would be edited; replay now asks.
- The replay check of drive --full is a read, never a full run.
- dvg-1 wrote docs outside its file list; it writes none now.
- dvg-2 had no executable proof of its stop branches; it now runs the
  template against fake control scripts and is judged as a behaviour
  change.

CELLS  (reviewed: 3)
CRITICAL FLAGS: none after the fixes above.
MINOR FLAGS: dvg-1 — a name-filtered proof passes on zero matches; the
  action requires the matched count.
CLEAN CELLS: dvg-3

SUMMARY: The three-cell shape stands. One blocker and eight warnings
were plan gaps; D7 rules on each and the cell texts carry the fixes.
```

## Dismissed

- Remove the dvg-1 → dvg-2 edge — it is the shared-build serial edge in
  one worktree, named under Waves.
- Add `drive` entry points to beehive's own control-bee — beehive has
  no deploy record and its release runs the smoke; named in limits.
- Gate deploys triggered by CI or recorded outside the two paths — not
  asked for; named in limits.
