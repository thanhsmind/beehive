PLAN CHECK
Work: agent-shell-rename, slice 1 (asr-1, asr-2)

Seats: hat-facts-gaps (opus, dispatch 810a472d), hat-user-impact (opus,
dispatch 661be5a0), hat-alternatives run on the advisor slot (fable,
dispatch 958ef924; recorded deviation 73d9eae8 — the worktree config still
pinned the seat to a retired GPT pane).

STRUCTURE
BLOCKERS (all fixed in plan.md before the gate):
- coverage/key-link: a ROLE_AGENTS row alone never reaches bee-elder for a
  seat role (prepare.rs:2354-2355 keeps the resolved seat as marker_role;
  guard.rs:289 canonicalises only read/code) — fix: an explicit
  "advisor" => "bee-elder" arm in the prepare.rs pinned_type match; claims
  12 and 13 added. All three seats found this independently.
- cell completeness: asr-1 truth "no old name under packages/" was
  unreachable within its files — generated/registry_payload.json and
  verbs/reviews.rs added; truth rewritten as an exact rg line.
- cell completeness: asr-2 verify scanned four docs/knowledge/areas files
  it did not own — added, plus the onboarding concept and six
  docs/product-description files.
WARNINGS (applied):
- asr-2 listed eight files with no hit (README, INSTALL, expertise,
  bee-hive SKILL/go-mode/routing, bee-reviewing) — dropped.
- apply.rs already deletes remove_*_agent_file items (apply.rs:505, claim
  14); the action now says so and only asks for the replacement name in
  the printed line.
- Discovery wording on the guard's seat-marker path tightened; the
  general-purpose repair is extended to a seat marker carrying
  [bee-kind: advisor].
- onboard/skills.rs dropped from asr-1 (design-skill hits only).
- Herding advisor panes gain the bee-elder body prefix — recorded as
  intended, test-matrix row added, asr-2 records it in the herding concept.
- Retired-name habits: rename hint on bare-denied and a new
  retired-agent-type deny under a marker.
- Regen at wave close must run the freshly built binary (tables are
  compiled in).
- bee-elder does not render on OpenCode without team.opencode.advisor —
  named in Discovery.
- site guide: giao-viec.html becomes the one home of the six names;
  tu-vung.html links there; "four" and "only writer" claims fixed.

CELLS (reviewed: 2)
CRITICAL FLAGS: none after the edits above.
MINOR FLAGS: asr-1 is large (one atomic rename; include_str! paths make a
split red in between — hat-alternatives confirmed no real dependency to
split on).
CLEAN CELLS: asr-2

SUMMARY: Shape kept; three blockers and nine warnings folded into
plan.md. Both cells are disjoint and run in one parallel wave; the leader
owes one regen with the fresh binary at wave close.
