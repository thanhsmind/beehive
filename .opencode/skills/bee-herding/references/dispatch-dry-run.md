# Dispatch dry-run — recorded proof

> Note: this recording was made in a different repo. The
> workspace/pane ids, PBI ids, `--wt--` slugs and the `docs/history/<feature>/`
> paths quoted below are that repo's, kept verbatim because the *method* — a
> cold agent reaching the right decision from external state alone — is what
> this proves, not the specific rows. They do not name files in this repo.

Three recorded runs, verbatim, proving the cold-agent decision this skill's
design rests on: a fresh `bee-herding` invocation, with no
memory of any earlier iteration, must reach the *right* decision from external state alone —
"nothing to dispatch" when nothing is dispatchable, and a fail-closed refusal when a row is
unsafe or unparseable. Every command below was actually run against that
repo's live state; nothing here is a description of what would happen.

## Run (a) — full dispatch role, `--dry-run`, against this repo's real state

Executed by `worker-proof` from its assigned worktree
(`/home/you/proj/repo--wt--agent-pane-orchestration`), following
`.claude/skills/bee-herding/SKILL.md` section by section. One honest deviation is
recorded up front rather than glossed over: **§0 of SKILL.md requires the role to run from
the MAIN checkout** and treats a cwd containing `--wt--` as a fatal misconfiguration that
stops the iteration before any dispatch logic runs. This session's cwd is exactly that kind
of path (`git rev-parse --show-toplevel` → `/home/you/proj/repo--wt--agent-pane-orchestration`).
A genuine cold iteration invoked from here would stop at §0 and report the anomaly, full
stop. Because this run's job is to prove the §5 dispatchable-set *decision*, the
transcript below deliberately continues past §0 anyway, using reads that are correct
regardless of cwd (the bee cell/worktree-grant store is keyed off `main_root`, not the
caller's cwd; `herdr` talks to the live workspace directly). This is flagged as a scope
choice for this proof only, not a claim that a real unattended iteration would reach §5 from a
worktree checkout — it would not, and should not.

### §1 — Self-identify

```
$ herdr pane current --current
```
Real result: `pane_id: w7:pN`, `tab_id: w7:t1`, `workspace_id: w7`, `label: "PBI-043"`.

`label` is not `dispatch`. Under `--dry-run`, §1 says report the rename rather than perform
it: **would run** `herdr pane rename w7:pN dispatch` — not run. The current label
(`"PBI-043"`) is this worker session's own bee-assigned pane identity from an unrelated
mechanism (the swarming session naming), not evidence of a bootstrapped cockpit pane —
recorded as-is rather than reframed to look tidier.

### §2 — Gate-bypass check

```
$ .bee/bin/bee status --json
```
Real result (relevant field): `"gate_bypass_level": "full"`. `full` clears the `>= full`
bar from §2 — the role may proceed to build a dispatchable set.

### §3 — Find the chat pane

`herdr pane layout --current` was tried first and returned a *different* workspace
(`w3`/`w3:t6`/`w3:p6`) than `pane current --current`'s `w7` — a real discrepancy between how
these two subcommands infer "current" in this environment, recorded rather than silently
worked around. Resolved instead with the pane id already in hand:

```
$ herdr pane layout --pane w7:pN
```
Real result: 4 panes in tab `w7:t1`:
- `w7:pP` — `rect.x=36, rect.y=1`
- `w7:pM` — `rect.x=153, rect.y=1`
- `w7:pJ` — `rect.x=36, rect.y=37`
- `w7:pN` — `rect.x=153, rect.y=37` (self)

Leftmost excluding self: `x=36` ties between `pP` (`y=1`) and `pJ` (`y=37`) → break tie on
smallest `y` → **chat pane = `w7:pP`**.

Note: this workspace has not been bootstrapped with the cockpit tab/pane labels — that
bootstrap was separate work, in progress concurrently with
this run. `w7:pP` is, in reality, another live bee worker's session pane, not a purpose-built
chat pane. The §3 resolution *rule* was still exercised correctly against real geometry; the
result is not claimed to be a real chat pane in a real cockpit.

### §4 — Occupancy and anomalies

```
$ herdr tab list --workspace w7
```
Real result: two tabs — `w7:t1` (label `"1"`, 4 panes), `w7:t5` (label `"2"`, 1 pane).
Neither is labelled `runtime` (again: cockpit labels not yet bootstrapped in this
workspace). §4's fallback fires: "the one tab that is not your own `tab_id`". Own `tab_id`
is `w7:t1`, so the fallback **runtime tab = `w7:t5`**.

```
$ herdr pane list --workspace w7   (filtered to tab w7:t5)
```
Real result: one pane, `w7:pQ`, `agent_status: "done"`, **no `label` field at all**.

Per §4: unlabelled → anomaly candidate, cannot be attributed to a PBI, does not count toward
`occupied_count`. `occupied_count = 0`. A slot is free (well under the cap of 4).

Anomaly dedup check, as §4 requires before sending anything:
```
$ herdr pane read w7:pP --source recent --lines 200
```
Real result: genuine scrollback of an unrelated human/agent conversation about PBI-032
documentation gaps — no prior mention of `w7:pQ` or this anomaly reason anywhere in it. So
this is a fresh anomaly, not a repeat. Under `--dry-run`, the line that would be sent via
`herdr pane send-text w7:pP "..."` is printed instead:

> `dispatch: anomaly — pane w7:pQ (tab w7:t5) is unlabelled, cannot be attributed to a PBI`

No `herdr pane send-text` call was actually made.

### §5 — Build the dispatchable set — the core of this proof

**(a) Ready — reverse index.**
```
$ grep -rn '^\*\*Backlog:\*\*' docs/history/*/CONTEXT.md
```
Real result: exactly one line —
```
docs/history/<upstream-feature>/CONTEXT.md:8:**Backlog:** PBI-043
```
Inverted, the reverse index is `{ PBI-043: agent-pane-orchestration }` and nothing else.
**No other PBI in `docs/backlog.md` has a `CONTEXT.md` at all**, so every other row fails
condition (a) outright, regardless of its Status column — a human has to run `bee-shaping` (Explore)
on an item before the dispatcher can see it as "ready". This is the readiness condition working as designed, not
a gap: the dispatcher must never invent scope for an item nobody has explored yet.

**(b) `in-flight`.** Reading `docs/backlog.md`'s Status column row by row: only `PBI-043` is
`in-flight`. Every other row is `proposed` (e.g. PBI-044, PBI-042, PBI-041, PBI-040,
PBI-039, PBI-032... ) or `done`. Combined with (a), `PBI-043` is the *only* row that can even
reach conditions (c)/(d).

**(c) No worktree grant.**
```
$ .bee/bin/bee worktree list --json
```
Real result:
```json
{
  "grants": {
    "herdr-gateway--wt--agent-pane-orchestration": true,
    "herdr-gateway--wt--self-update-merge-config": true
  },
  "main_root": "/home/you/proj/repo"
}
```
The key `herdr-gateway--wt--agent-pane-orchestration` ends with
`--wt--agent-pane-orchestration` → a grant already exists for PBI-043's slug
(`agent-pane-orchestration`). **PBI-043 fails condition (c).**

**(d) Zero cells.**
```
$ .bee/bin/bee cells list --feature agent-pane-orchestration --json
```
Real result: 10 cells, not zero — `agent-pane-orchestration-1..4` dropped,
`-5..7` capped, `-8` and `-9` claimed (the run recording this proof among
them), `-10` open. **PBI-043 fails condition (d) too**, for the same underlying reason as
(c): this feature is mid-execution right now, and this very proof is part of that execution.

**§5 conclusion: zero rows survive all four conditions. The dispatchable set is empty.**

### §6/§7 — Nothing to classify, nothing to rank

With no candidates surviving §5, §6's lane classifier has nothing to run against and §7 has
nothing to rank or announce. Per SKILL.md: "If nothing survives §5/§6 ... end the iteration
quietly." This iteration is not perfectly silent — §4 found one real anomaly (`w7:pQ`) and
reported it (printed, under `--dry-run`) — but it dispatches **nothing to dispatch** this
iteration, and stops before §8. No `bee worktree new`, `herdr pane split`, or
`herdr agent start` was run.

### Why the real dispatchable set is empty today — read this as expected, not a bug

Two independent, structural reasons, both already surfaced in §5:

1. **Only one backlog item has ever been promoted through exploring.** `docs/backlog.md`
   carries ~20 `proposed` rows (PBI-044 down to PBI-039 and others) that are completely
   invisible to condition (a) until a human runs `bee-shaping` (Explore) on each one — that is what
   creates its `docs/history/<slug>/CONTEXT.md` and is the only thing that can flip its
   Status to `in-flight`. The dispatcher has no mechanism, and must have none, to decide
   readiness on its own; that decision belongs to a human running exploring per item first.
2. **The one item that passed (a) and (b) — PBI-043, this feature — is already under way.**
   It holds a worktree grant and ten cells (this proof run among them) precisely because
   dispatch already happened for it, by a human, before this automated dispatcher existed.
   Conditions (c) and (d) exist to stop the dispatcher from double-starting work that is
   already running — seeing them fire here is the mechanism working, not failing.

A future reader seeing an empty dispatchable set should conclude: *nobody has explored a new
item since PBI-043, and PBI-043 itself is already claimed* — not that the reverse index, the
grant check, or the cell check is broken.

## Runs (b) and (c) — classifier level, against `.bee/tmp/` fixtures

> **The transcript below is a verbatim historical record — do not run its commands.**
> Two things have changed since it was recorded. The classifier now lives in the bee
> binary as `.bee/bin/bee herding classify-lane <PBI-ID>` (R6a retired the standalone
> Node script), and it reads the event-sourced PBI fold rather than a markdown backlog
> file, so the `--backlog <file>` flag and the `"row text"` wording in the outputs below
> no longer exist. What the runs prove — pass, hard-gate refusal, and unreadable-record
> refusal, with textually distinct reasons — is unchanged and still holds against the
> current verb.

The full dispatch role's §5 also needs a history root (`docs/history/*/CONTEXT.md`) and a
cells store that no isolated fixture can provide — a fixture backlog row with no matching
`CONTEXT.md` and no cells would just fail (a)/(d) trivially, telling us nothing about the
*classifier*. So runs (b)/(c) instead exercise exactly the layer that fixtures can isolate:
the lane-safety filter, run directly against two throwaway
backlog files under `.bee/tmp/` (not `docs/backlog.md` — nothing there was touched).

### Fixture files (exact contents, reproducible)

`.bee/tmp/fixture-ordinary.md`:
```markdown
# Fixture backlog — ordinary row (used by agent-pane-orchestration-9's classifier proof)

| ID | PBI | Status | Ghi chú |
|---|---|---|---|
| FX-001 | Add a small caption tweak to the switcher card footer text so it wraps at 2 lines instead of 1. | in-flight | Fixture row: no mode-gate risk flags anywhere in the text. |
```

`.bee/tmp/fixture-refusals.md`:
```markdown
# Fixture backlog — refusal pair (used by agent-pane-orchestration-9's classifier proof)

| ID | PBI | Status | Ghi chú |
|---|---|---|---|
| FX-002 | Rework the authentication flow for admin login so tokens rotate on every request. | proposed | Fixture row: hard-gate flag (auth) present in the description. |
| FX-003 |  | proposed |  |
```

### Run (b) — ordinary row, expect `lane_safe: true`

```
$ bee herding classify-lane FX-001 --backlog .bee/tmp/fixture-ordinary.md
```
Verbatim output:
```json
{"pbi":"FX-001","lane":"small","hard_gate_flags":[],"lane_safe":true,"reason":"no mode-gate risk flags matched in row text"}
```
`lane_safe: true`, lane named (`"small"`) — matches expectation.

### Run (c) — hard-gate row + empty-text row, expect `lane_safe: false` for both, distinct reasons

```
$ bee herding classify-lane FX-002 --backlog .bee/tmp/fixture-refusals.md
```
Verbatim output:
```json
{"pbi":"FX-002","lane":"high-risk","hard_gate_flags":["auth"],"lane_safe":false,"reason":"hard-gate flag matched: auth"}
```

```
$ bee herding classify-lane FX-003 --backlog .bee/tmp/fixture-refusals.md
```
Verbatim output:
```json
{"pbi":"FX-003","lane":"high-risk","hard_gate_flags":[],"lane_safe":false,"reason":"row for FX-003 has empty or unparseable text (no description or notes found)"}
```

Both fixture rows in `fixture-refusals.md` come back `lane_safe: false`, and their reasons
are textually distinct:
- `FX-002`: `"hard-gate flag matched: auth"` — the hard-gate branch (§6's "no hard-gate
  flag" bar), caught by the row's own text (`authentication`), fail-closed by design.
- `FX-003`: `"row for FX-003 has empty or unparseable text (no description or notes
  found)"` — the fail-closed branch for a row the classifier cannot read at all, not a
  content match. This is a different failure mode entirely: an unclassifiable row, per the
  script's own contract ("anything the rules cannot classify with confidence ... returns
  `lane_safe:false`"), must never come back safe — and it does not.

Together with run (b), that is 3 recorded classifier JSON objects, 2 refusals with distinct
reasons, and 1 pass — the full set this proof exists to demonstrate.

## What this proves, and what it does not

**Proves:** a cold agent following SKILL.md's literal steps, reading only external state
(git, bee's cell/worktree-grant store, the live herdr workspace, and the classifier script),
reaches "nothing to dispatch" on this repo's real backlog today, for reasons traceable to
the four dispatchability conditions rather than a flaw in the reverse index, the grant check, or the cell
check — and reaches fail-closed, distinctly-reasoned refusals on both a hard-gate row and an
unparseable row at the classifier level.

**Does not prove:** the cockpit layout (this workspace was not bootstrapped with
`chat`/`dispatch`/`merge` labels — that bootstrap happened
separately), or the `--pane current --current` vs `--pane layout --current` discrepancy
noted in §3, which is an environment quirk observed here, not something this proof's scope
covers investigating further.

---

## Superseded: the reverse-index method used above

Run (a) above resolved the PBI→slug map by grepping `^\*\*Backlog:\*\*` across
`docs/history/*/CONTEXT.md`, and concluded that no other PBI had a CONTEXT.md
at all. **Both are wrong, and it was measured:** exactly one
of 24 CONTEXT.md files carried that line — this feature's own, hand-written —
so the index only ever matched its own author, while **14** other backlog rows
resolve to a CONTEXT.md that exists.

The correct lookup is different: the slug comes from
the backlog row's own `` Feature `<slug>` `` annotation, which 15 rows already
carry. See the dispatch role's §5(a).

The transcript is kept because what it proves is still true and still the point:
a cold agent, reading only external state, reached the correct decision and
changed nothing. Only its lookup step is stale. Do not copy the grep above.
