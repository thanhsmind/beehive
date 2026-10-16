# SLP Detector — hat wave synthesis

Date: 2026-10-15. Seats: hat-facts-gaps, hat-alternatives, hat-user-impact
(three, standard lane, 7 product files). Budget: all three returned inside
the ceiling; none dropped.

PLAN CHECK
Work: slp-detector current slice (det-1..det-4)

STRUCTURE
BLOCKERS (facts-gaps, 1, fixed):
- Key links / membership: the changed set built from a diff against HEAD
  loses committed paths (workers commit mid-job, bee-build.md:3), loses
  untracked files, and nothing recorded a base commit on the job
  (run.rs:5131 only on the done result). Fix applied: D3 refinement
  32492fc4 — base_sha recorded at spawn (run.rs created_at sites), else
  captured at the first tick; untracked files included; .bee/ and
  docs/decisions/ excluded; claims 26–29 added; det-2 rewritten; test
  matrix rows added (mid-round commit, untracked test file, old job).
WARNINGS (applied):
- Allowlist owner: det-3 owns the SUPERVISOR_ALLOWED_TOOLS line (named in
  its action, files and truths).
- Ceiling per round: rule measures from continued_at when present
  (run.rs:4936, claim 29).
- Token rule reads the last finished round; the new config key gets a
  docs/config-reference.md row in det-4 (claim 36).
- Boundary matcher: declared_covers from the cells judge (claim 31); the
  src/b/d/e.rs case is in the matrix.
- Contract walk: hoisted reads through contract_status_over as
  handlers_write.rs:1286 does (claim 32).
- Cell loader: crate::verbs::cells::read_cell (claim 30).
- registry_payload.json: hand-edited in the shape of the precedent cells
  (herding.notify, herding.fork); the registry test pins it.
- Tick home: RealArgvProvider::spec_for, not resolve_iteration which tests
  call directly (alternatives seat; claim 33).

USER IMPACT (applied):
- `--events` legal on every record kind, judged read from both stores, so an
  event acted on by an intervention is judged in the same tick (D4
  refinement e3de051a).
- Silence keeps --signal none; the row's signal is independent of the
  events it names.
- `No detector events.` on an empty store (claim 37).
- Status carries counts plus `unjudged`, so a count that only grows is
  visible as unjudged work.
- Heartbeat runs the tick in the finally block with a 30 s timeout, so a
  failed broker tick never hides it (claim 35).
- Point keys for path rules are `<signal>-<job>-<8 hex>` to fit the slug
  rule.

CELLS (reviewed: 4, leader cold-pickup pass)
CRITICAL FLAGS: none after the rewrite.
MINOR FLAGS: det-1 — the regen also rewrites the five prompt copies that
det-4 edits again; accepted (one more regen is cheap).
CLEAN CELLS: det-2, det-3, det-4

DISMISSED
- alternatives (a1, a2): fold det-4's prompt work into det-1 and its concept
  edits into det-3 — dismissed: prose belongs to the docs role, and det-1 is
  already the largest code cell; a second regen costs seconds.
- user-impact: shared-worktree attribution of a sibling's path — not solved;
  named as a limit in the D3 refinement and in the prompt paragraph the
  supervisor reads before speaking.
- user-impact: advisor-nudge limited to the three poor-work signals — out of
  scope; the four new names ride observation and mailbox rows.

SMALLER PATH: PASS after the review — four cells, one per contract surface,
no cheaper shape that keeps the docs role on the prose.

SUMMARY: One real blocker on the changed-set source, fixed in the plan and
locked as a refinement; every claims row verified byte-exact by the
facts-gaps seat; the remaining findings were applied as plan edits or
dismissed with a reason above.
