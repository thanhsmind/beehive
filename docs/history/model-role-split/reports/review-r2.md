# Review — model-role-split (session `model-role-split-r2`)

**Scope, frozen at create.** `0075e2e0..7b14a72f` — 36 commits, 83 files,
7653 insertions, 1735 deletions. Baseline is the merge-base with main
rather than the branch point, deliberately: main was merged into this
branch mid-flight and its own 40 commits are neither this feature's work
nor unreviewed. 30 cells included, all capped; 21 carry
`behavior_change` and their evidence was verified at the preflight door.

**Panel.** Four reviewers in parallel — code-quality, architecture,
security, test-coverage — each handed the frozen diff, `CONTEXT.md` and
`plan.md`, and nothing else. None saw session history, so none inherited
the orchestrator's assumptions. **No reviewer filed a P1 alone.** Both P1s
below are promotions earned by independent corroboration, and both were
then *executed* rather than left as traces.

A note on the panel's own limits, recorded because it changes what the
verdict is worth: the test-coverage reviewer's first pass could not run
its central method — the write guard denied every mutation target
because it was dispatched from the main checkout. It reported the
blockage rather than working around it, and was re-dispatched from
inside the worktree. Everything marked *executed* comes from that second
pass.

---

<!-- bee:not-a-deferral: everything below is FINDINGS PROSE — it describes defects the panel found and, where a defect is about deferral itself, says so as an observation. A review report never defers work by contract: it does not repair, decide, or close, and its P2/P3 findings leave as backlog rows (all 19 filed 2026-08-25). No line below promises a later delivery. -->

## Spec axis — does it do what the locked decisions promised

### P1-A · A cell declaring `role: "ceiling"` takes the session model and no budget counter sees it

Found by security and code-quality independently, then executed.

`prepare.rs:941` reads `else if escalated_cell || tier_token ==
ESCALATION_WORD`. On the `from_role` path `tier_token` **is the cell's own
declared role string**, so the arm fires before `resolve_role_named` is
reached: `Resolved::Inherit`, `channel: session-model`, no `model` param.
Meanwhile `cell_is_escalated` (`validate.rs:98`) reads only `escalate ==
true` or the legacy `tier == "ceiling"`, so the same record is invisible
to all three counters that enforce D5.

Executed on a seven-cell feature with six such cells:

```
model=None  channel=session-model  prompt=[bee-tier: ceiling]
cell_is_escalated=false
set_escalation on the seventh: ALLOWED — ration silent at 86% escalated
```

Two things make this a door rather than an oversight. `cells/tests.rs`
**asserts** that `role: "ceiling"` is accepted at add time, and
`models.rs:530-538` carries a comment stating the opposite of what
ships — that such a cell "warns and falls through like any other".

D5 locks the ration as "unchanged in force"; `plan.md:187` rates it HIGH
with the proof "the 40 percent refusal fires on the flag exactly as on
the tier value". It does — and is bypassable around it. No test connects
add-time acceptance to the dispatch interception.

### P2 · `[bee-tier: advisor]` passes the guard on a host with no advisor, and inherits the session model

`guard.rs:55-59` inserts every `slot_for_kind` slot into `known_roles`
unconditionally, `advisor` included, configured or not. `classify_marker`
returns `Marker::Role`, the unconfigured-role refusal never fires, and the
dispatch falls through `Resolved::Budget` to `allow("marker", …)` with no
model param — verbatim the outcome the refusal text at
`model_guard.rs:152-163` exists to prevent. `bee dispatch prepare --role
advisor` **refuses** on the same host: one question, two doors, two
answers, the defect D1 collapsed the parsers to remove.

### P2 · The escalation ration's denominator changed, and no decision records it

D5 locks the ration "unchanged in force". The delivered code moved from
`ceiling / tiered` to `escalated / the feature's cells`. The change is
**necessary** — a tier-shaped denominator reads 0 once `role` is
required — and all three sites argue it. The defect is the record: three
code sites cite `store b39d045f`, which is this branch's own commit sha
dressed as a decisions-store id. `grep -c b39d045f .bee/decisions.jsonl`
returns 0. This feature logged decisions for five smaller deviations and
missed the one touching a locked decision's arithmetic.

### P2 · D9's backfill has never been applied to the live store

Measured: 543 cells, **532 roleless**, 20 still carrying `tier:
"ceiling"`, 0 escalate flags, 6 traces still on the retired `tier_reason`
key. The verb ships and is idempotent; the migration *is* the decision.
No backlog row or decision defers it. Behaviour is not red —
`cell_is_escalated` still reads the legacy spelling — but D5 keeps "its
persisted reason" in force, and six traces hold it under a key the
handbook no longer publishes.

### P3 (spec)

- A well-formed `models.<rt>.ceiling` is accepted, ignored and never
  mentioned: `models.rs:624` returns `Inherit` without reading it,
  `role_slot_display` skips it, `validate_models_config` passes a plain
  string. Decision 0015 is right to ignore the key; ignoring it silently
  is the class this feature exists to close.
- `docs/artifacts/bee-workflow.html:242` still teaches `bee cells tier`,
  retired in all four directions.
- `role` cannot be updated and is not named as frozen: `UPDATE_FIELDS`
  has 15 entries including `lane`, no `role`, and no
  `update_frozen_hint` arm. The sole model selector is the one required
  field with no revision door.
- Rendered agent files never ask for `code`/`read`. On claude the
  dispatch's model param covers it; on **opencode the agent file is the
  enforcement**, so a configured `models.opencode.code` is silently
  ignored.

---

## Standards axis — is the code well made

### P1-B · `cells backfill-roles` scans outside its lock, so a concurrent write is silently reversed

Found by code-quality, demonstrated by test-coverage, raised from P2.

The verb reads every cell to build its plan, takes the `cells-archive`
lock **only at `:1607`**, then writes each whole object back. The lock
guards the write phase alone; the scan is unlocked, and every write is a
full-object write of a stale clone.

Demonstrated on a 40 000-cell store, with a real operator door:

```
disarm (cells escalate --off) committed to disk at t=25ms, ok=true
backfill finished                                     t=1.007s
on disk afterwards: escalate=true   ← the operator's write, reversed
```

The doc block at `:1523-1531` claims "a concurrent `cells
update`/`cap`/`claim` cannot half-write behind this pass". It does not
half-write: it fully writes and is fully overwritten. A budget-checked
`--on` was correctly refused by the lock at 319 ms, which is why the
defect is invisible to casual testing — only writers completing inside
the scan phase are lost. Code-quality additionally traced that a `bee
close` archiving a feature during the scan is unblocked, and the write
then recreates the archived cell as a live duplicate.

P1 rather than P2 because the loss is silent, the verb has not yet run
against the live store, and this repository's default is concurrent
sessions — main gained commits from a sibling twice during this feature's
own development.

### P2 · The two-agents ambiguity refusal is keyed on the literal `"generation"`, so `code` walks past it

`model_guard.rs:760` guards on `t == "generation"`. Every freshly
onboarded host now configures `code`, which aliases to `generation`, so
`[bee-tier: code]` skips the check and is **repaired onto `bee-gather`**,
the read-only agent. An execution dispatch then dies later at the write
guard, with the audit line recording the wrong agent. The test that should
catch it iterates the table's keys, so no alias spelling is exercised.

### P2 · The pre-roles migration window is unpinned against a null-valued asked role — the shape bee ships

Executed: narrowing `host_opted_into_roles` from `.is_some()` to
"non-null" left the **entire suite green** (2654 passed). That mutation
reopens the window permanently on every fresh host, because
`.bee/config-sample.json` seeds codex as `"code": null, "read": null`.
The window is this feature's one deliberate exception to "never resolves
silently", and its boundary is unguarded in precisely the configuration
bee itself writes.

### P2 · `cell_role_list` duplicates its head, doubling the warn

`cell_role_list("code")` returns `["code", "code", "generation"]`. On a
half-migrated host the warn fires twice for one dispatch, and the first
copy reads `falling through to "code"` — the name it just warned about.

### P2 · The worker-registry refusal mixes both vocabularies in one sentence

`workers.rs:78`: *invalid tier "…" — the value records the ROLE … FIX:
pass the cell's own role, e.g. `--tier code`*. The persisted key staying
`tier` is argued well; the **flag** is a separate surface with no on-disk
record and was not renamed. Compounding it, `handlers_close.rs:370`
dropped the flag from its FIX entirely, so bee no longer teaches how to
record the role at all.

### P3 (standards)

- The `[bee-tier: …]` role name is unbounded and control-char-permissive
  into stderr and `dispatch.jsonl`; `description` is truncated to 120
  chars, `tier` is not. Bounded in practice to log noise and bidi
  spoofing, not command injection.
- Published chain steps are never checked against what the guard will
  admit, so bee can hand out an instruction its own guard refuses.
- A **fifth** hand-written agent-to-role table, inside a FIX line this
  feature edited — and it omits `bee-build`, the only write-capable agent.
- The runtime list in status rendering was patched per instance: the
  missing-codex fix added a third hand-written block rather than looping
  `RUNTIMES`, and its own comment names the defect it then reproduces.
- `default_models(rt)` is rebuilt per list entry — three `Map`
  allocations per name, on every dispatch and every status render.
- `known_roles` skips the unknown-runtime coercion `resolve_role_named`
  applies; latent only, since both callers pass gated literals.
- `defaults_apply_without_config`: three of four assertions are
  tautological against the built-in defaults; the fourth catches a seed
  drop by accident. The same indistinguishable-fixture defect this
  feature already shipped once and fixed, not carried into `onboard`.
- Two of `plan.md`'s Test-matrix scale probes are absent with no recorded
  skip; a third is honestly declined in-test.
- The `mrs-8` fixture-sweep claim went stale two cells later: 85 raw
  fixtures in `drivers/tests.rs` carry no role, so the bulk of the
  dispatch-envelope suite never traverses the D3 cell-role read.
- The semantic-gate test names its four classes as a local literal.
  Adding a fifth semantic class to `CHAIN_ADVANCE_ON` was caught — by the
  **sibling** test, not the one named for the property.

---

## What the panel confirmed as sound

Stated because a review that lists only defects misreports the work.

- **The two central structural claims hold.** `normalize_models` and
  `resolve_role` exist exactly once; the guard calls through; the third
  copy is a thin wrapper. `CLAUDE_TIERS`, `CODEX_TIERS`,
  `MODEL_NORMALIZE_SLOTS`, `MODEL_VALIDATE_SLOTS` and all four
  `MODEL_TIERS` copies are gone. No fifth legality list survives.
- **The safety property binds.** The fixture rotation off `default_models`
  means a tail regressing to the built-ins fails *at the fixture*, and the
  codex variant covers the resolves-nothing direction. A real
  anti-recurrence device, not a re-assertion.
- **No security exposure.** No secret reaches a log or terminal; the chain
  is data-only and never joined into a command line; the migration verb
  builds its whole plan before any write and cannot be steered by record
  content; role names reach no path, process or shell.
- **The ration is pinned at both boundaries** — exactly 40 percent
  allowed, 43 percent refused.
- **One test function was deleted repo-wide, with a named replacement.**
  No proof weakening by deletion.

<!-- /bee:not-a-deferral -->

23 finding(s) — P1 2, P2 7, P3 14 · axis: spec 8, standards 15.
