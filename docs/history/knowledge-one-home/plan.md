---
artifact_contract: bee-plan/v1
mode: standard
---

# Plan: Knowledge one-home

Mode: `standard` — 3 risk flags: covered-contract-change (cap and gate
refusals change behavior existing tests assert), public-contracts
(frontmatter schema, CLI output), multi-domain (knowledge, cells, gate,
decisions, skills).
Why this is the least workflow that protects the work: two new refusal
doors in bee's own control plane need a frozen shape and a review wave;
anything smaller would let the doors drift the way the rules they guard
already did.

## Requirements (from CONTEXT.md)

- D1: fix at write time; `knowledge context` ranking untouched.
- D2: conflict detection at plan time, before the merged gate.
- D3: ownership map in area frontmatter; plan predicts affected
  skills/specs per cell; cap diffs reality against the map and REFUSES
  (never warns) without a recorded reason. (3ea7500a)
- D4: one home per rule (AGENTS.md discipline / area spec mechanism);
  `applied_at` outbound list; bee computes the update list; cap refuses
  an untouched listed file; rule ids, copies cite them; `knowledge check`
  flags id-less rule blocks, one id in two bodies, dangling targets;
  capture stub answers the skill question. (27e55095)
- D5: bee derives conflict candidates from the plan; verdict per
  candidate; `gate --merge` refuses unverdicted candidates; plan-rev bump
  resets. (efd6cbaa)

## Discovery

Static scout of the crate (no build run), file:line under
`packages/bee-rs/crates/bee/src/`:

- Cap refusals are prose `Err(Fail::Thrown(String))` inside
  `cap_cell_from_flags` (`verbs/cells/handlers_close.rs:148`); the
  regen obligation (`verbs/cells/obligation.rs:160`) is the shape to
  copy but runs at add/update, never at cap. Cap reads `--files` only;
  the commit diff is read by `head_commit_numstat`
  (`verbs/cells/finish_support.rs:663`) and pushed to warnings — an
  advisory, finish-only.
- Gate: `high_risk_advisor_refusal` (`verbs/state_group/set_gate.rs:588`)
  is checked twice under `exec_component && approved` (`:701`, `:711`).
  `plan_rev` lives on the workflow record
  (`verbs/workflow_store/record.rs:35`); gates are stamped with it and
  a bump flips the projection (`projections.rs:25`) — no explicit reset
  needed, a stamped rev that no longer matches reads as unapproved.
- `knowledge check`: `PROFILE_REQUIRED` is a key-path table
  (`verbs/knowledge/frame.rs:164`); dangling-target checks at
  `verbs/knowledge/check.rs:543-590` are the pattern for new codes.
  Frontmatter parser `verbs/knowledge/frontmatter.rs:259`.
- Decisions: `conflict_candidates(active, text, tags, exclude)`
  (`verbs/decisions/read.rs:454`) and `count_term_hits` (`:436`) are
  reusable as-is.
- Help text is hand-edited `src/generated/registry_payload.json`
  (compiled in via `registry.rs:13`); `bee dev regen` does not touch it;
  `tests/registry_contracts.rs` pins it; catalog counts in
  `src/catalog.rs` move with it.
- Precedent: the instruction-layer fence
  (docs/knowledge/areas/okf-profile/the-instruction-layer-fence.md) —
  three hand audits found 7, 2, 6 gaps with the chain green; the fix was
  a machine check. D4's copy check generalizes that fence from one
  retired layer to every homed rule.

## Approach

Recommended path: four demoable phases, each one door. Phase 1 makes the
data exist and checkable (D4 schema, ownership map, rule ids); Phase 2
puts the cap door on it (D3, D4 cap refusal); Phase 3 puts the gate door
on it (D5); Phase 4 migrates the 12 inventoried rules and the stale help
text through the new doors, proving them on real cases.

Agent's-discretion values (CONTEXT.md), fixed here for planning:

- Frontmatter, flat keys under `bee:` (the parser refuses nested maps
  other than `bee:` — `frontmatter.rs:371 unsupported_map`, and a
  `{`-leading scalar — `:159`; dotted keys pass `key_re_ok`,
  `frame.rs:166`): `owns.code: [..]`, `owns.skills: [..]`,
  `owns.tests: [..]` on the area's `overview.md` concept only (one map
  per area); `applied_at: [..]` on any concept that homes a rule. All
  four keys join `BEE_KEY_ORDER` (`frame.rs:148`, a fixed-size const
  used by `emit_entries`, `frontmatter.rs:96`) so the parse→emit
  round-trip stays byte-stable. Three areas have no `overview.md`
  (`feedback-digest`, `performance-log`, `verify-pipeline` carry only the
  generated, reserved `index.md`); Phase 1 authors a `bee.area`
  `overview.md` for each so every area can carry its map. Paths are
  repo-relative, trailing-`*` globs allowed, resolved like
  `required_context` targets (bundle first, then repo root,
  `check.rs:549-561`); with no repo root, out-of-bundle targets are not
  graded and the report says so.
- Rule id and block: an explicit marker pair in the home body —
  `<!-- rule: <area>-<slug> -->` … `<!-- /rule -->`. A copy is any line
  elsewhere carrying `(rule: <id>)`. No prose heuristic: an unmarked
  block is never guessed at; the check grades markers and references
  only. AGENTS.md homes use the same marker.
- New `knowledge check` codes, graded as profile ERRORS in the default
  run (exit non-zero without `--strict` — a warning-only grade would be
  the advisory door D3 rejects): `dangling_applied_at`, `dangling_owns`,
  `duplicate_rule_home` (one id marked in two files),
  `unknown_rule_ref` (`(rule: x)` with no home), `applied_at_unlinked`
  (a listed file carries no `(rule: <id>)` for that rule),
  `owns_missing` (an area `overview.md` without any `owns.*` key —
  a separate walk over area overviews, NOT an entry in
  `PROFILE_REQUIRED` (`frame.rs:164`), which applies to every concept).
  Exempt trees for the copy scan: `docs/history/`, `docs/discovery/`,
  `docs/specs/`, `.bee/`. The verify chain already runs `knowledge
  check` on every green run (instruction-layer fence, same area).
- Cap-time diff source: union of `head_commit_numstat` (authority) and
  `--files`; when no commit is resolvable, `--files` alone. This is a
  NEW git call on the plain `cells cap` path — today only `finish`
  shells to git (`handlers_close.rs:447-467`); the root resolves per
  feature via `commit_trailer_history_root` (`:461`). Escape:
  `--sync-ack "<reason>"` on cap/finish, stored to `trace.sync_ack`,
  mirrored as a cap deviation line so `knowledge promote` can mine it.
- Cell prediction field: `affects_skills: [..]` and `affects_specs:
  [..]` (flat arrays; `[]` means "none") — required on add for EVERY
  lane (D3: "per cell"), validated in validate.rs.
- Decision-log trigger (D1): `decisions log` resolves its `--tags` and
  areas to the rules homed there and prints their `applied_at` union as
  `update_obligations` in its output — the push at the moment a rule
  settles; enforcement stays at cap through `owns`/`applied_at`.
- Conflict verdicts: stored on the workflow record as
  `conflict_review: {plan_rev, candidates: [{id, kind: decision|rule,
  verdict: compatible|conflicts|retires-prior, note}]}` via a new verb
  `bee plan conflicts --derive` (writes candidates, empties verdicts) and
  `bee plan conflicts --verdict <id>=<verdict>`; `gate --merge` refuses
  when `conflict_review` is absent, its `plan_rev` ≠ current, or any
  candidate lacks a verdict (D5 verbatim). A recorded `conflicts`
  verdict does NOT refuse by itself; the gate output lists it so the
  user approves with eyes open.
- Capture: `bee capture add` gains `--skill-answer "<changed|not: why>"`,
  required when the stub's area `owns.skills` is non-empty.

Rejected alternatives:

- Warn-only cap door — rejected by D3 verbatim.
- A separate `docs/knowledge/ownership.md` — two homes for ownership.
- Prose heuristic for rule blocks — re-creates guessing (CONTEXT.md).
- Regenerating `registry_payload.json` — no generator exists; out of
  scope, hand-edit with the drift test as the net.

Risk map:

| Component | Risk | Proof needed |
|---|---|---|
| frontmatter parser + PROFILE_REQUIRED | LOW | knowledge tests green; `bee knowledge check` on live bundle |
| ownership map content for 15 areas | MEDIUM (wrong globs = false refusals) | `check` zero `owns_missing`; dry cap against three recent cells shows no false refusal |
| cap door | HIGH (every cap passes through it) | cells tests green + new refuse/ack/no-commit cases; existing cap cases unchanged |
| gate precondition | MEDIUM | set_gate tests green + new absent/stale/unverdicted/conflicts cases |
| migration of 12 rules | MEDIUM (prose churn) | `check --strict` green; parity of pointers |

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 Schema + map | `owns.*` / `applied_at` parsed, emitted, and graded; rule markers + refs graded; six new check codes; `overview.md` authored for the three areas lacking one; ownership map written for all 15 areas; three rules from ticket 004 get ids + `applied_at` as the check's first real input — #1 delegation threshold (the contradiction), #2 write-guard allowlist (drift vs code), #4 cap proof line (nine copies) | Nothing downstream can compute a list without the data | `bee knowledge check` on the live bundle reports the new codes; a deliberately duplicated rule id is flagged | Phases 2, 3, 4 |
| 2 Cap door | `affects_*` fields on cells (validate.rs); `decisions log` prints `update_obligations`; cap/finish derive the obligation from `owns` + `applied_at` against the commit diff; refuse with remedy text; `--sync-ack` escape; help text + catalog updated | The data exists; the door is the behavior change the user asked for first | cap a cell that touched `verbs/cells/*.rs` without `skills/bee-swarming/**` → refused naming the skill; same cap with `--sync-ack` → capped with the reason on the trace | Phase 4 runs through it |
| 3 Gate door | `bee plan conflicts --derive/--verdict`; `conflict_review` on the workflow record; `gate --merge` precondition; help text + catalog | Independent of phase 2; needs phase 1's `applied_at` for rule candidates | `gate --merge` refused with the unverdicted list; after verdicts, approved; `plan-rev bump` → refused again | Phase 4 runs through it |
| 4 Migration | The 12 rules of ticket 004: ids, homes, copies reduced to one line + pointer, the two contradictions resolved, `cap --help` text fixed; bee-capturing / bee-planning / bee-swarming skill text updated to the new fields; capture `--skill-answer` | The doors exist; this is their first real use and the proof they bite | `check --strict` green on the bundle; `rg "(rule: "` shows every copy pointing home | Feature close |

Current slice to prepare: Phase 1.

## Test matrix

Triad, smallest demonstrating size; each cell's writer judges existing
coverage first (`verbs/knowledge/tests.rs:1242` dangling_required_context,
`verbs/cells/tests.rs:896` regen obligation, `set_gate.rs:1271` merged
advisor precondition) and authors only the gap.

- Happy: concept with valid `owns` + `applied_at` → no finding; cap with
  all listed files touched → capped; gate with all verdicts → approved.
- Edge: glob in `owns` matching zero files (warn `dangling_owns`); rule
  id marked once, referenced from three copies (no finding); cap with no
  resolvable commit (falls back to `--files`); `plan-rev bump` after
  verdicts (gate refuses again); exempt tree carrying a stale copy (no
  finding).
- Error: same id marked in two files (`duplicate_rule_home`); `(rule:
  x)` with no home (`unknown_rule_ref`); cap touching owned code with
  the owned skill untouched and no ack (refused, names the skill); cap
  with blank `--sync-ack` (refused); gate with a `conflicts` verdict
  (refused naming the decision id).

## Out of scope

- `knowledge context` ranking changes (D1).
- A generator for `registry_payload.json`.
- Migrating rules beyond the 12 in ticket 004; later drifts are caught
  by the check, not by this feature.
- Release checklist gaps (already fixed separately).
