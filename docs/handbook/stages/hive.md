# Stage: hive (`bee-hive`)

**Purpose** — The bootstrap meta-skill and router. It orients the session, verifies
onboarding, classifies the request into a lane/mode, routes to the next skill, and
presents and protects the human approval gates.

**When it runs** — First, in every bee session, and again after any context
compaction. Re-entered whenever a routing or mode-gate decision is needed.

## Inputs
- **`bee orient`** — the one command that answers where the work is: phase,
  feature, mode, gates, bypass level, the decisions in force, ready cells,
  blockers, and exactly one recommended next step (`{action, skill, command}`).
  It supersedes prose reading-orders — the packet *is* the context assembly.
  Orient is for routing, starting, or resuming; a plain question is already
  answered by the session preamble.
- The session preamble (phase, mode, feature, gate states, cell/PBI counts, the
  critical-patterns digest, recent decisions, the intent anchor) — read it, don't
  re-fetch it.
- [`state.json`](../register.md#beestatejson), [`onboarding.json`](../register.md#beeonboardingjson),
  [`HANDOFF.json`](../register.md#beehandoffjson), [`config.json`](../register.md#beeconfigjson).
- `bee status --json` — only when about to *route work*.
- `docs/knowledge/index.md` (critical patterns) or `docs/history/learnings/critical-patterns.md`.

## Outputs
- A routing decision. The lifecycle is **shape → plan → swarm → capture**, and
  `bee orient`'s `next.skill` names the current stop; reviewing, researching,
  grooming, and herding are out-of-band, on explicit request only.
- Onboarding mutations (via `bee onboard --repo-root <root> --apply`).
- Gate presentations. hive owns no feature artifacts of its own.

## Gate
Presents all three verbatim but structurally owns none — it is the presenter and
enforcer. Gate 1 "Decisions locked. Approve CONTEXT.md before planning?" · Gate 2
"Work shape is ready. Approve before current-work preparation?" — folding the old
standalone execution gate into the same approval, shape and execution together via
`bee gate --merge` · Gate 3 (P1>0) "P1 findings block merge. Fix before
proceeding?" / (P1=0) "Review complete. Approve merge?".

## State touched
Reads [`state.json`](../register.md#beestatejson),
[`onboarding.json`](../register.md#beeonboardingjson),
[`HANDOFF.json`](../register.md#beehandoffjson),
[`config.json`](../register.md#beeconfigjson) (bypass level, declared commands).
Writes onboarding state and gate approvals (`bee gate`).

## Key rules
- **Gates are never skipped, batched, or self-approved** — including go mode and
  headless — except the opt-in `gate_bypass` level in `config.json`. Setting the
  level is itself never a gate approval: state the level's row (what auto-approves,
  what still stops) and log a one-line audit decision in the same turn. `full` and
  `total` need the user's explicit instruction.
- **Classification is mechanical** — count risk flags and product files; never by
  vibe. Uncertainty resolves *downward* into more ceremony, never upward into less.
- **The hook is a safety net, not the authority** — an unblocked write is not an
  approved write. Route through hive *before* touching source, every time.
- **Never build on a red base** — a red result is the next work item, never a base.
  `cells claim` enforces it: a claim against a recorded red run refuses unless it
  carries `--fix-first "<reason>"`.
- **A handoff is adopted only at a fresh-session boundary.** `state handoff adopt`
  refuses from a resumed or compacted session, and never adopts a `pause` record —
  those are presented to the user, who decides. A session with no recorded start
  source warns and proceeds.
- **The capture queue turns into a blocker on its own.** At 10 pending stubs, or one
  older than 7 days, `bee orient` moves the queue out of its offer line and into
  `work.blockers[]`.
- **Onboarding is a three-step contract**: missing or stale → `bee onboard
  --repo-root <root> --json`; `changes_needed` → summarize, get approval, re-run
  with `--apply`, never silently; `blocked_*` → zero mutations, surface `versions`.
  Do not continue until it reports `up_to_date`.
- **A freshly onboarded project has craft but no knowledge of its own.**
  `.bee/expertise/` arrives full; the project's knowledge layer arrives empty, and
  its first debt is the orientation entry — written by reading the tree, never by
  asking the user to describe their own repo. `bee-capturing` owns that write.
- **Multisession etiquette** — coordinate through lanes, claims, and holds, never
  around them. Code-touching feature work goes to its own worktree
  (`bee worktree new` / `bee worktree merge`); docs, a solo tiny, and release work
  stay in the main checkout.

## Source
`skills/bee-hive/SKILL.md` + `references/{routing-and-contracts, gates-and-delegation, scout-and-ticks, onboarding, go-mode}.md`
