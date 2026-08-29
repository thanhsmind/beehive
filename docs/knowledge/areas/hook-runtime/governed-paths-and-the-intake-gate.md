---
type: bee.area
title: "Hook Runtime — governed paths, the always-writable set, and the intake gate"
description: "Which write targets escape the active feature's gate routing and which never do, why the always-writable set only ever shrinks, why a finished feature's leftover approvals are not what decides whether the next source write is allowed, how many phases require that approval today, why a phase value the workflow does not recognize is now refused instead of silently allowed, how a value left by a retired phase is translated rather than left to trip that refusal, why an approved plan document stops accepting direct edits until a revision is stamped, and why the always-writable set is now two lists — a gated-phase list without blanket docs/, and an unchanged intake list — instead of one shared constant, why a session bound to no lane is now judged against the lane record its own live claim names instead of the control-root default, and why an intake refusal aimed at such a session names binding it as the remedy."
timestamp: 2026-08-14
bee:
  id: hook-runtime-governed-paths-and-the-intake-gate
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: [8ed35504 (write-guard always-writable set shrinks), c2c46488 (the intake gate fires in every terminal state; approvals never outlive the feature that earned them), "validation-diet D3/D13 (docs/history/validation-diet/CONTEXT.md, 2026-07-28)", "hook-teeth D1/D7 (docs/history/hook-teeth/CONTEXT.md, 2026-08-04 — the approved plan document is frozen by the write guard itself, resolved lane-record-first; every flip lands red-first)", "traceable-runs D1/D6 (docs/history/traceable-runs/CONTEXT.md, 2026-08-14 — a file-touching request is gated at every lane including docs, and the mandatory flow scopes to writes, code and docs alike; trun-5 splits the always-writable set so a gated-phase docs/ write outside docs/history/ actually refuses, closing the accidental hole D1/D6 named)", "edd92ac9 (slp-followup-gaps D1/D2, 2026-08-29 — an unbound session's acting write record is resolved from its own live claim before the control-root default answers, and the intake refusal aimed at such a session names binding it as the remedy)"]
  sources: ["bee-footprint D2 (cell footprint-2, 2026-07-12)", "docs/specs/hook-runtime.md#B11", "docs/specs/hook-runtime.md#B12", "docs/specs/hook-runtime.md#R11", "docs/specs/hook-runtime.md#R12", "docs/specs/hook-runtime.md#P8", "validation-diet cells vd-1/vd-2 (traces in .bee/cells/, reports docs/history/validation-diet/reports/vd-1.md,vd-2.md, 2026-07-28 — the gated phase set narrowed to two, the write guard's unrecognized-phase fall-through flipped from silently allowing to refusing, and a saved value left by the retired phase translated on read)", "hook-teeth cell bh-1 (trace .bee/cells/bh-1.json, 2026-08-04 — plan-document freeze deny, feature resolved from the path, lane-aware gate state; write_guard slice 93 passed)", "traceable-runs cell trun-5 (trace .bee/cells/trun-5.json, capped 2026-08-14 — guards.rs/checks.rs/paths.rs/hook_local.rs/tests.rs, red-first retargeting two pre-existing tests that pinned the old shared-list behavior)", "slp-followup-gaps cell sfg-1 (commit 9809d34e, 2026-08-29 — write_guard/checks.rs claim-derived record plus the shared lane_record_from helper, store.rs session_claimed_features, paths.rs session_bind_remedy_line, write_guard/tests.rs)"]
  authoritative_for: "hook-runtime: which write targets are governed and which are always writable"
  applied_at: [skills/bee-hive/references/routing-and-contracts.md]
---

# Hook Runtime — governed paths, the always-writable set, and the intake gate

The previous concept covers what the guard can read; this one covers what it
governs. Two questions decide every write: is the target inside the small,
deliberately shrinking set of locations that need no gate routing at all, and is
there active work whose gates could authorise it in the first place. The second
question is answered from the workflow's state, never from approvals a closed
feature left behind.

## Data Dictionary

| Element | Meaning |
|---|---|
| always-writable location | A small named set of locations a write may target without the active feature's gate routing, because the content is machine-local and disposable — today: the workflow's own state/log directory and, inside it, a dedicated subfolder for disposable experiment work. Removing a location from this set only tightens governance; adding one is a deliberate, reviewed decision. |
| gated phase | A phase in which a source write outside the always-writable set is refused until the workflow’s approval is granted; today exactly two of the phases that precede that approval carry this requirement. |

## Behaviors & Operations

**B11 — A repo-root disposable-experiment location is no longer
always-writable.** Trigger: a write targeting the former repo-root
disposable-experiment location. What blocks it: the same gate routing that
governs any other source path — the active feature's phase and gate state —
exactly as for a path outside the always-writable set; nothing exempts this
location anymore. What changes: this location moves from always-writable to
governed, strictly shrinking the always-writable set by one entry; disposable
experiment work itself continues unblocked, but now inside the workflow's own
always-writable directory, under a dedicated subfolder that location's
existing allowance already covers. Side effects: the close-time nudge's own
always-writable set shrinks identically, so a write left in the old location
is flagged there too, not only by the write guard. What actors observe: the
assistant sees the same corrective deny/allow outcome it would see writing to
any other governed source path; the human owner sees no new prompt — the
location simply stopped being an exception (bee-footprint D2).

**B12 — No active work means no source writes — a finished feature is not an
open door.** Trigger: any write to a governed path while the workflow sits in a
terminal state — either *nothing has started yet* or *the last feature has
closed* (workflow-state: the two terminal states, and the only two from which a
new feature may start). What blocks it: the intake gate, which denies the write
and names the terminal state it fired on, telling the assistant to route the
request through the workflow first. What is still allowed: the always-writable
set plus the knowledge locations (docs, plans, the workflow's own directory) —
the closing steps of a feature, spec sync and learning capture, must keep
writing after that feature closes. Why the state and not the gates decide this:
a closed feature leaves its approvals **behind it**, still recorded as
approved. Reading approvals alone, a finished feature is indistinguishable from
an approved one, so the guard reads the state — the phase — and the moment work
is no longer active the door is shut regardless of what the last feature was
allowed to do. Escape hatch: unchanged — a repository may disable the intake
gate entirely in its configuration, and doing so disables it for both terminal
states alike, never one but not the other (decision c2c46488).

**B13 — The gated set of phases lost the one entry a retired feasibility phase
used to hold (validation-diet D3, 2026-07-28).** Trigger: a source write while
the workflow sits in one of the phases that require approval before a write
outside the always-writable set proceeds. What blocks it: exactly the
existing behavior, unchanged in mechanism — an unapproved write is refused.
What changed: the gated set itself is one entry smaller, because the phase
that used to sit between the two that remain no longer exists at all — its
own reality check moved earlier, into the shape-drafting step of the phase
right before it. Nothing about which paths are governed, what unblocks
them, or how the always-writable set behaves changes; only the count of
phases carrying the approval requirement drops from three to two.

**B14 — A phase value matching none of the workflow’s real phases is now
refused outright, not silently passed through (validation-diet D13,
2026-07-28).** Trigger: a source write checked against a phase value that
matches none of the workflow’s real phases — a broken or stale record, never
a phase the workflow can actually be in. What changed: previously such a
value fell through every branch of the write check and reached an implicit
allow, so a broken record gated nothing at all, silently. Now the same
situation is refused outright, naming the unrecognized value. What stays
unaffected: every phase the workflow can actually reach keeps its own
behavior exactly as before — the two gated phases (B13), the states a new
feature may start from, the mid-execution state, and the four phases after
approval that were never given a dedicated branch of their own (independent
review, spec-sync, learning-capture, and housekeeping) all still write
freely once reached; a repository with no saved state, or one the workflow
cannot read, resolves to the very first phase, which has always had its own
branch and is untouched by this change. What actors observe: an assistant
working against a genuinely broken state record now sees a clear refusal
instead of writes silently going ungated.

**B15 — A saved phase value left over from the retired feasibility phase is
translated before the new refusal ever sees it (validation-diet D13,
2026-07-28).** Trigger: a repository whose saved state still names the
phase that used to sit between the two gated phases, written before that
phase was retired. What happens: reading the state translates that value,
transparently, to the phase that absorbed its role — before either the
approval check or B14’s refusal evaluates it, so it is treated as the
perfectly ordinary recognized phase it now is. What this avoids: without
the translation, such a repository would either be permanently unable to
leave the retired phase (only an explicit phase change moves it, and an
unrecognized value refuses that too) or would fall straight into B14’s new
refusal and find every write blocked. What actors observe: an existing
repository resumes exactly where it left off, still gated the same as it
always was, never bricked and never silently ungated.

**B27 — An approved plan document is frozen by the guard itself, not by
convention (hook-teeth D1, 2026-08-04).** Trigger: any write whose target is a
feature's plan document — the guard recognises exactly that one filename under
that one feature-history location, and derives the feature's name from the path
itself, so no caller has to declare which feature it is editing. Any other
document in that folder, including the locked-decisions document, and any
deeper path never reach this check. What blocks it: the shape approval recorded
for that same feature. The guard reads the feature's own lane record first and
takes its shape gate as final; only when no lane record exists does it fall back
to the default record, and only when that record names the same feature — a
record about some other feature is no opinion, never approval. What each actor
observes: before shape approval the plan document is an ordinary writable
document; after it, a direct edit is denied and the denial names the two ways
forward — stamp a plan revision, which reopens the document, or withdraw the
shape approval and redraft. The check fires ahead of record resolution and hold
checks, and does not depend on the workflow's phase.

**B33 — The always-writable set is now two lists, not one, because a blanket
`docs/` write at a GATED phase was a real hole (traceable-runs trun-5,
2026-08-14).** Trigger: a source write while the workflow sits in a gated
phase (`exploring` or `planning`, execution unapproved) versus a write at
idle or a terminal phase. What changed: the single shared
`GATE_ALLOWED_PREFIXES` constant split into `GATE_ALLOWED_PREFIXES_GATED`
(`.bee/`, `docs/history/`, `plans/`, `AGENTS.md` — no blanket `docs/`) and
`GATE_ALLOWED_PREFIXES_INTAKE` (unchanged: `.bee/`, `docs/`, `plans/`,
`AGENTS.md`). The gated-phase boundary and its "Allowed now" message now
consult the gated list; the idle/terminal intake gate and the
git-bookkeeping arm keep consulting the intake list; the worktree-first
exemption in `hook_local.rs` stays on the intake list on purpose, to
preserve its exact prior behavior — it independently exempts every `*.md`
already, so it was never the enforcement point for this hole. What actually
changed for a caller: at a gated phase, a write under `docs/history/`
(bee's own brief/plan for the active feature) is still always allowed; a
write to any other `docs/` path — a spec, a stray note — now refuses until
execution is approved, exactly like any other governed source path. At idle
or a terminal phase, blanket `docs/` is untouched. This closes the gap D1/D6
named: the docs lane escaped the gate boundary by accident (the constant
was shared, not by any considered exemption for docs work specifically),
not by design.

**B34 — An unbound session's acting record is resolved from its own live
claim before the control-root default record answers (slp-followup-gaps D1,
cell sfg-1, 2026-08-29).** Trigger: any governed write, or any git command the
guard evaluates, from a session that HAS a record of its own and carries no
non-empty lane binding. What changed: before the default record answers, the
guard asks that session's own live claims which feature it is working under — a
claim names a cell, the cell names a feature, and that feature's lane record
becomes the acting record, carried under a third provenance value (`claim`)
beside `default` and `lane`. The merge that builds a lane record out of the
lane file now lives in one helper both arms call, so a record resolved by
declaration and a record resolved by claim are byte-identical. Why this is not
a guess: a claim is a fact the store already holds, written by the claim verb
under this same session id. Why it was needed: a dispatched worker that was
never bound was judged against a record about some other feature, and at a
terminal phase it lost every source write and every commit it legitimately
owed its own lane. What is deliberately narrow — every one of these conditions
only ever removes the derivation, never widens it: it fires only for a session
that has a record and no lane (a call with no session id, and a session the
store has no record of, both fall straight to the default, the latter because
the bind verb itself refuses such a session); it reads only claims THIS session
owns; it needs exactly ONE distinct claimed feature, so two or more are
ambiguous and resolve to nothing; an expired claim, an unreadable or non-object
claim, a claim whose cell record is missing, and a cell naming no feature each
contribute nothing (a claim carrying no usable ttl or no parseable timestamp
reads as active, never as expired); the feature name must be a plain id, and its
`.bee/lanes/<feature>.json` must exist, parse as an object, and carry a matching
`feature` key — missing, corrupt, or naming a different feature resolves to
nothing. Every one of those failures is SILENT: the derived path never raises
the declared-lane path's typed lane refusals, because those belong to a session
that declared a lane and got it wrong. What actors observe: a worker holding
exactly one claim writes and commits under the lane it was actually handed; a
lane-bound session, a sessionless call, and a session holding no claim behave
byte-identically to before. One consumer groups `claim` with `default` on
purpose — the workspace-ownership check: the derivation says WHICH LANE the
session works under, never who owns the checkout, so that check keeps firing for
exactly the sessions it fired for before.

**B35 — An intake refusal aimed at an unbound session names binding it, beside
the standing FIX line (slp-followup-gaps D2, cell sfg-1, 2026-08-29).**
Trigger: an intake-gate refusal — a governed write, `git push`, a bookkeeping
git verb reaching outside its allowed paths, or an unrecognised git
subcommand — where the acting record came from the DEFAULT record AND the
refusing session has a record of its own carrying no lane. What changed: the
refusal keeps everything it said (the phase, the blocked action, the
per-call-site extra sentence, the standing FIX line) and adds one more remedy
line: this session is bound to no lane, so the gate judged it against the
control-root default record rather than the lane it is working under — bind it
to that lane, or claim its cell, and retry. Why the standing line was the wrong
remedy for this caller: it tells them to route the request through the
workflow, but the work IS routed; only the session is unattached to it. Who
never sees the line: a lane-bound session, a claim-derived one (B34), a call
with no session id, and a session the store has no record of — each already has
its own correct answer, and the last could not act on the advice anyway.

## Business Rules

<!-- rule: hook-runtime-docs-lane-allowlist -->
- R33 — The write-guard allowlist has two prefix sets: `GATE_ALLOWED_PREFIXES_GATED` (`[".bee/", "docs/history/", "plans/", "AGENTS.md"]`) at a gated phase (`exploring`/`planning`, execution unapproved) with no blanket `docs/`; and `GATE_ALLOWED_PREFIXES_INTAKE` (`[".bee/", "docs/", "plans/", "AGENTS.md"]`) at idle or a terminal phase keeping blanket `docs/` (traceable-runs trun-5, D1/D6, 2026-08-14).
<!-- /rule -->

- R11 — The write guard's always-writable set no longer includes the
  repo-root disposable-experiment location; that work now lives inside the
  workflow's own already-writable directory, under a dedicated subfolder. The
  set of ungoverned writable locations only shrinks from this change, never
  grows; the session-close nudge's allowed-path set shrinks identically
  (bee-footprint D2).

- R12 — The intake gate fires in **every** terminal state, not merely the
  never-started one: a source write is governed whenever no feature is active,
  including immediately after a feature closes with its approvals still on
  record. Approvals belong to the feature that earned them and never outlive
  it; the active state, not the recorded approvals, decides whether the door is
  open (decision c2c46488).

- R13 — Every idle-gate/write-policy config read inside the guard follows the
  **resolved controlRoot**, never the raw `root` parameter: on a
  companion-mounted path the two name different projects' `.bee/config.json`,
  and reading `root`'s made the idle gate judge a different project's phase
  than the containment check had just resolved in the same call (GH #83, live
  incident 2026-07-27). All three call sites — the terminal-phase idle-gate
  branch, the write-policy-mode read, and the bash-command guard's own idle
  read — were aligned in one pass; half-fixing recreates the bug on the
  remaining path (gh-fix-batch cell gfb-2, 2026-07-28).

- R14 — The gated set — phases requiring approval before a source write
  outside the always-writable set — now holds exactly two entries; the
  retired feasibility phase’s membership disappeared with the phase itself
  (validation-diet D3).

- R15 — A phase value the workflow does not recognize as any of its real
  phases is refused at the write check; every phase the workflow can
  actually produce — gated or not — is unaffected, including the four
  phases after approval with no dedicated branch and the very first phase a
  repository with no or unreadable state resolves to (validation-diet D13).

- R16 — A saved phase value naming the retired feasibility phase is read as
  the phase that absorbed its gate, so an existing repository is neither
  locked out of progressing nor left with its writes silently ungated
  (validation-diet D13).

- R26 — A feature's plan document accepts direct edits only while its shape gate
  is unapproved; afterwards the only sanctioned paths are stamping a plan
  revision or withdrawing the approval, and the gate that decides this belongs
  to the feature named in the path, resolved lane-record-first with a
  mismatched default treated as silence (hook-teeth D1, cell bh-1, 2026-08-04).

- R27 — The two harness-owned surfaces exempt from the outside-the-worktree
  refusal — the agent's memory root and its scratchpad — are matched at the
  name the harness actually creates, not an idealized one. On a shared
  temporary area the scratchpad root carries the calling user's account
  identifier, and containment compares whole path segments, so a root written
  without that identifier matches nothing: the exemption silently misses the
  only surface it exists for and an ordinary scratchpad write is refused for
  being outside the tree. The exemption stays scoped to the calling user's own
  root — never a prefix match that would sweep in a neighbour's — and a
  per-user temporary area needs no identifier at all (cell hsa-1, measured
  2026-08-06).

- R34 — A session that has its own record but no non-empty `lane` resolves its
  acting write record from its OWN live claims before the control-root default
  record: exactly one distinct claimed feature, whose `.bee/lanes/<feature>.json`
  exists, parses as an object, and carries a matching `feature` key, yields that
  lane record under provenance `claim`. Another session's claim, an expired
  claim, an unreadable or non-object claim, a claim whose cell record is missing
  or names no feature, two or more distinct features, a feature name that is not
  a plain id, and a missing, corrupt or mismatched lane record each resolve to
  nothing, and the default record answers exactly as before. The derived path is
  silent by construction — it never raises the declared-lane path's typed lane
  refusals — so it is neither more permissive nor more restrictive than the lane
  the worker was legitimately handed; a lane-bound session, a sessionless call,
  and a session holding no claim are unchanged to the byte. The
  workspace-ownership check reads `claim` exactly as it reads `default`: the
  derivation names a lane, never a checkout owner (slp-followup-gaps D1, cell
  sfg-1, 2026-08-29).

- R35 — The intake refusal names binding the session as a remedy exactly when
  the acting record came from the default record AND the refusing session has a
  record carrying no lane. A lane-bound session, a claim-derived one, a call
  with no session id, and a session the store has no record of never see that
  line — the last because the bind verb refuses a session it has no record of,
  which would make the advice a dead end (slp-followup-gaps D2, cell sfg-1,
  2026-08-29).

## Pointers (implementation)

- Always-writable set (B33/R33): `GATE_ALLOWED_PREFIXES_GATED` and
  `GATE_ALLOWED_PREFIXES_INTAKE` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs` — the gated
  list is `.bee/`, `docs/history/`, `plans/`, `AGENTS.md`; the intake list is
  unchanged: `.bee/`, `docs/`, `plans/`, `AGENTS.md`. The gated-phase
  boundary and idle/terminal-intake consumers are named per-list in
  `write_guard/checks.rs`; the legacy `packages/bee/lib/guards.mjs` this
  Pointer used to name no longer exists in this repo (Node fully retired) —
  session-close-nudge parity is now whatever the Rust port's own nudge path
  reads, not a separate `NUDGE_ALLOWED` mirror.
- Gated set, unrecognized-phase refusal, and legacy-phase translation:
  `is_gated_phase` and the phase dispatch's final branch in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs` and `paths.rs`.

- Harness allowlist (R27): `HarnessRoots::from_bases` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs` — the memory
  root `<home>/.claude/projects/`, `<temp>/claude`, and on unix
  `<temp>/claude-<libc::getuid()>`; consulted only at the containment failure
  sites via `harness_allowlisted_target`. Tests:
  `gh1_the_uid_suffixed_scratchpad_is_exempt_and_a_sibling_uid_is_not`,
  `gh1_harness_scratchpad_write_is_exempt_for_write_and_bash` in
  `write_guard/tests.rs`. Provenance: cell `hsa-1`, commit 2857bc8d.

- Plan-document freeze (B27/R26): `plan_freeze_feature` and
  `plan_freeze_shape_approved` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:159-206`, fired
  from `check_write` at `checks.rs:333-343` ahead of `resolve_write_record` and
  the hold checks. Lane precedence mirrors `resolve_write_record`'s own. The
  deny text names `bee state plan-rev bump --lane <feature>` as the reopen.
  Red-first per hook-teeth D7: the path-resolution tests landed failing before
  the deny wired in. Evidence: trace `.bee/cells/bh-1.json` (write_guard slice
  93 passed, 0 failed, 2026-08-04).

## Open Gaps

- Recorded tradeoff (bee-footprint P3): the workflow's disposable-experiment
  subfolder is both always-writable and excluded from version control, so its
  contents never appear in a change listing. This is deliberate, not a defect
  — but a reviewer must not read a clean change listing as proof that nothing
  was staged in that location; confirming its contents requires looking at
  the location itself.
