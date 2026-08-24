---
type: grilling
status: open
claimed-by: (unclaimed)
blocked-by: (none) — unblocked by 001
---

## Question

Which new roles earn their place — meaning each has a real dispatch site
that would select it, not just a name on a config screen?

**Re-framed by ticket 001's answer (decision `8dad7c2e`).** A role is
reachable by two independent paths, and the candidates below do not all
want the same one:

- **Cell tier** — the work item carries `tier:` and `--kind cell`
  honours it (`validate.rs:29`, `prepare.rs:731-745`). Fits a role that
  describes *how big the work is*.
- **Dispatch kind** — the caller passes `--kind` and the door resolves
  the slot (`prepare.rs:31-40`). Fits a role that describes *what job
  the worker does*.

So each candidate now needs two answers, not one: does it earn a slot,
and which path reaches it? A role that wants a cell tier costs nothing
at the door; a role that wants a kind costs a `DISPATCH_KINDS` entry, a
`slot_for_kind` arm, and a `dispatch_kind_for_tier` arm.

Candidates, with the site that would use each:

- **`tiny`** — reads as a **cell tier**, not a kind: it names work
  size. A `tiny` cell may run inline on the session model
  (AGENTS.md, "From small up, cells run through dispatched workers … a
  tiny cell may run inline"). Inline means the ceiling, the priciest
  model, for the cheapest work. Strongest candidate.
- **`judge`** — reads as a **dispatch kind**: it names a job. The
  goal-check judge tier (`bee-hive`, "§ Goal-check
  judge tier") currently has no slot of its own.
- **`plan`** — reads as a **dispatch kind**. Planning research
  dispatches; today they take the
  generation slot by default (decision 0023's aux-dispatch line).
- **`commit`** — path unclear; ask which it is. Commit-message and
  scribe writing; mechanical, and the
  cheapest tier would do.

For each: is there a dispatch site, and does routing it separately
change the model actually chosen? A role whose answer is "it would
resolve to the same model as `generation`" is a config knob with no
effect and should not ship.

## Settled 2026-08-24 — decision `06e49368`

The structural half of this ticket is answered. Roles fall through (a
consumer names an ordered list; an unset name yields to the next) and
the role set is open (any name in `models.<runtime>` is legal; the
guard asks "is this configured", not "is this one of four words"). The
cost of publishing a role therefore falls to roughly one name.

Two consequences for the candidates below:

- The ticket's own admission test — *does a real dispatch site select
  it* — is retired. It was a symptom of a resolver that refused. A role
  now earns its place by being **nameable**.
- `plan` and `commit`, ruled out above for having zero dispatch sites,
  are no longer ruled out on that ground. Zero sites now means a role
  nobody currently asks for, which costs one name. Whether it should
  still ship is a publishing question, not a reachability one.

**What stays open in this ticket:** which names bee publishes as its
default role set, and what each one means. That is the map's remaining
destination.

## The ticket's own test is challenged by the source read

This ticket tests a candidate by "does a real dispatch site select it".
The xia read of `~/Projects/refs/oh-my-pi` (docs/history/research/oh-my-pi-model-roles-distill.md)
shows that test belongs to bee's *refusing* resolver, not to roles as
such. Upstream, a consumer names an ordered list of roles
(`["commit","smol",...ALL]` — `commit/model-selection.ts:46`) and an
unset role simply falls through. A role there earns its place by being
**nameable**, and costs one array entry when nobody configures it. If
bee adopts fallthrough, the question this ticket asks changes from "how
many roles" to "which names do we publish".

## Verified findings (2026-08-24, two gather reads)

Applying the ticket's own test — *a real dispatch site that would select
it* — to each candidate:

- **`tiny` — not a role at all; a missing derivation.** bee already
  carries both vocabularies and never joins them. Lanes:
  `LANES = ["tiny","small","standard","high-risk","spike"]`
  (`verbs/cells/validate.rs:27`). Tiers:
  `MODEL_TIERS = ["extraction","generation","ceiling"]`
  (`validate.rs:29`). No code maps one to the other — `dispatch prepare`
  never reads `lane` at all. Lane drives four other things only (context
  budget `session_preamble/mod.rs:118-120`, uat applicability `uat.rs:54`,
  the worktree-first carve-out `status_full/topology.rs:269`, and a
  must_haves requirement `validate.rs:146`). Meanwhile a cell's `tier` is
  never derived: `normalize_new_cell` (`validate.rs:272-297`) does not
  touch it, `cells add` has no `--tier` flag (`handlers_write.rs:236`
  accepts only file/stdin/dry-run), and the only mutator is the explicit
  `bee cells tier` verb (`handlers_close.rs:1114`, `:1141`). So an
  untierred `tiny` cell resolves through `slot_for_kind("cell")` to
  `generation` — the priciest configured worker model, for the cheapest
  lane. The `tiny`-may-run-inline rule is real but lives at the *cap*
  door, not the dispatch door (`handlers_close.rs:348-373`).
- **`judge` — one real dispatch site, already tiered.** The rule home is
  `gates-and-delegation.md:190-193`: a pinned `bee-review` dispatch on
  the review tier, returning `judge-verdict/1`, recorded by
  `bee cells judge-record`. The verdict contract is Rust
  (`verbs/cells/judge.rs:226-345`); the *dispatch* is prose. One site
  (`swarming-reference.md:202`), plus three pointer lines. Independence
  is already a stated want — `gates-and-delegation.md:191` prefers a
  model differing from the builder's and records
  `model_independence: "same-model"` when it does not — and the review
  slot is already separately configurable.
- **`plan` — zero dispatch sites.** The single planning dispatch is a
  *reviewer* on the review tier (`planning-reference.md:285`), and
  research delegation names a skill, not a kind
  (`bee-planning/SKILL.md:55`). `skills/bee-planning/` names no tier
  anywhere. The aux-dispatch rule this candidate rests on
  (decision `0023`) was never landed: it lists five skills, three of
  which no longer exist.
- **`commit` — zero dispatch sites.** Commit text is written by the
  execution worker inline (`worker-details.md:157-163`) or is a
  hardcoded Rust string (`verbs/worktree/phases.rs:221`, `:356`). No
  model is selected for it anywhere.

Two facts about the door itself, which bound any answer here:

- **Two of the four existing kinds are never passed.** Repo-wide the
  only literal uses are `--kind gather` and `--kind cell`; `reviewer`
  and `advisor` occur only inside the placeholder
  `<cell|gather|reviewer|advisor>`. Reviewing dispatches via a plain
  prompt template with no `prepare` call
  (`reviewing-reference.md:17-25`); the advisor is resolved by the
  *worker* through `resolveAdvisor` (`swarming-reference.md:145-147`,
  `worker-details.md:233`). bee's door already carries roles no caller
  asks for — adding more is the same defect ticket 001 named.
- **`slot_for_kind` ends in a catch-all `_ => "advisor"`**
  (`prepare.rs:34-40`). Any kind added to `DISPATCH_KINDS` without its
  own arm silently resolves the **advisor** slot. This is a build
  hazard for ticket 001's `--kind extract` and for every role added
  here.

Related dead field, same shape as ticket 001: **`effort` is configured,
displayed, and dropped.** The `{model, effort}` leaf parses
(`models.rs:167-181`) and the preamble renders `model:effort`
(`model_guard.rs:338-341`), but every `Resolved::Model` site at the door
destructures `{ model, .. }` (`prepare.rs:800`, `:1050`, `:1063`). Only
the codex `native` branch emits it, as `reasoning_effort`
(`prepare.rs:898-899`); herding never sees it. On the claude runtime bee
prints an effort it does not send.

## Constraint from the map

Whatever the count, each role costs: an entry in `CONFIGURABLE_SLOTS`
(`models.rs:37`), a branch in two independent parsers (`models.rs:318`,
`model_guard.rs:442`), and an entry in two hand-maintained guard tier
lists (`model_guard.rs:192-193`). See ticket 004.

## Answer

(open)
