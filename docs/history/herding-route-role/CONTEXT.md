# Herding Route Role — Context

**Feature slug:** herding-route-role
**Date:** 2026-09-08
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

The bee-herding control loop gains a fourth cold role, `route`, that reads one
finished worktree per iteration, starts a reviewer on a different agent for it,
turns a CHANGES verdict into a cell in that worktree's own lane and hands the
brief to that worktree's already-open coder pane (D4), and stops cold on
anything it cannot classify — and it ends there: `route` never merges, never
picks a PBI, never **starts** a coder or creates a pane, and never touches main.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The control loop takes a **fourth role, `route`**, beside `dispatch`, `merge` and `supervisor`. Its finish signal is a **finished worktree** — the same four conditions `role-merge.md` already tests: phase `compounding-complete`, zero cells open or claimed, a clean tree, and `HEAD` exactly on `wt/<slug>`. One cold iteration acts on exactly **one** finished worktree, then exits. `.bee/result-inbox/` markers stay outside this role's scope. (decision `a685d557`) | The cockpit's dispatched coders reach `compounding-complete`; none of them ever writes a result-inbox marker, so the inbox alone would leave the loop open. The four-condition test already exists and is already proven in the merge role — one home, not a second definition of "finished". |
| D2 | A **named, scoped carve-out** to rule `agents-review-user-invoked`. Inside the herding cockpit **only**, and only while **both** the owner enable marker (`.bee/tmp/bee-herding.enable`) is present **and** `gate_bypass` is `full` or `total`, `route` may start a reviewer over a finished worktree with no human ask. The reviewer **must** run on a different `herding.agents` entry than the agent that produced the work. Everywhere else — every ordinary session, every lane, `bee-reviewing`, Gate 3 — independent review stays the human's door, unchanged. (decision `8388df3e`, touches `565e68d0` and `b34fdea9`) | The two arming conditions are the same pair `role-dispatch.md` §2 and §5 already refuse below, so the carve-out cannot arm itself. The different-agent requirement is new law: bee today has **no** rule against a model reviewing its own output, and `tier_role_list("review")` falls through to `generation` when the review slot is unset. |
| D4 | `route` may deliver a CHANGES follow-up brief into the **producing coder's still-open pane**. It still never *starts* a coder and never creates a pane; it may hand work to a pane already open and idle on the worktree it just reviewed. **This amends the Feature Boundary line below**, on the owner's answer of 2026-09-08. | Without it D3's cell has no actor at all: writing the cell makes the worktree unfinished, so `merge` skips it permanently (`role-merge.md:89-90`) and `dispatch` refuses it for holding a grant (`role-dispatch.md:243`), while `route` starts no coder. The coder's pane is still open — merge is the only thing that closes it (`role-merge.md:157-161`) — and it is the actor the source fleet uses. Found by two hat seats independently; escalated rather than designed. |
| D3 | The two bad-outcome paths are **asymmetric**. A review verdict of **CHANGES** becomes work the **same** worktree takes ahead of anything new: `route` writes the findings as a cell in that feature's lane, and that cell is served before any other ready cell in the lane. A **BLOCKED** worker **stops the routing cold** and reports to the human — never restarted, never re-briefed, never handed to a different agent. An outcome `route` cannot classify is **reported and left standing**, never dropped. (decision `4a395ea7`) | CHANGES is mechanical work with a named author and a named diff, so the loop can carry it. BLOCKED is a decision the agent could not make — restarting it asks the same agent the same unanswerable question. |

### Agent's Discretion

- The **shape of the reviewer's brief** and how its verdict is recorded, within
  D2's different-agent constraint and bee's existing `bee reviews` store.
- **How the priority cell is marked** so the lane serves it first (D3), within
  the existing `bee cells` and lane machinery — no new store.
- Whether `route` runs on its own interval or shares the dispatch interval.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| finished worktree | A granted worktree meeting all four of D1's conditions at the moment `route` reads it. Never inferred from a pane's `agent_status`, which is not evidence anywhere in this cockpit. |
| routing | Reading one finished outcome and taking exactly one consequent action on it. Not dispatching (that starts new backlog work) and not merging (that lands work in main). |
| different agent | A different key in the `herding.agents` registry than the one recorded for the work's producing dispatch. Not "a different model string" — the registry key is the unit bee can actually check. |
| CHANGES | A review verdict asking for named repairs before the work can land. Distinct from BLOCKED, which is a question the worker could not answer. |

## Specific Ideas And References

- The user's source is the herdr "25 agents, one lead" fleet model (a thread and
  two slides). Its routing rules — DONE routes to a reviewer on another harness,
  CHANGES routes to a followup lane the coder gets first, an unrouted outcome
  goes to a retry lane rather than the void — are the discipline this feature
  adapts. The fleet's own scripts are unpublished and are **not** being ported.
- `hmans/beans` was read at commit `99260bf1` and contributes nothing to this
  feature: it is an issue tracker, with no lead, reviewer or router.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — the cold, stoppable
  poll-act loop. `Role::parse` (`:72-74`), per-role default interval (`:93-94`)
  and the per-role `--allowedTools` surface (`:277`) are the three places a
  fourth role registers.
- `skills/bee-herding/references/role-merge.md:50-95` — the four finished
  conditions, already written and already proven. D1 reuses this test; it must
  not be restated in a second place.
- `bee reviews create` / `record` / `status` — a frozen review scope, a findings
  append, and derived coverage labels. The reviewer's verdict has a home already.
- `bee cells add` and the lane records under `.bee/lanes/` — the followup cell of
  D3 has a store already.
- `.bee/wave-ledger.jsonl` and `bee herding record-worker` — where a dispatch
  records the agent it used, which is what D2's different-agent check reads.

### Established Patterns

- **A cold role reads live, decides once, reports, exits.** All three existing
  roles follow it; `route` is a fourth instance, not a new pattern.
- **Announce into the chat pane with scrollback dedup** —
  `bee herding pane read <chat> --lines 200` before `send-text`, so a repeated
  condition is stated once, not once per tick (`role-dispatch.md:98-105`).
- **Refuse rather than guess** — `bee herding occupancy`'s `fallback` source ends
  the iteration instead of dispatching on an unknown count. D3's unclassifiable
  outcome takes the same posture.
- **A closed verdict set with a required reason** — `bee cells dissent-verdict`
  accepts exactly `accept|reject|escalate`, each needing `--reason`.

### Integration Points

- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — role registration,
  interval, tool surface.
- `skills/bee-herding/SKILL.md` — "The three roles" becomes four; the role
  boundary paragraph gains `route`'s own "never" list.
- `skills/bee-herding/references/` — a new `role-route.md`, and
  `operational-invariants.md` gains D2's carve-out as a named safety boundary.
- `AGENTS.md` — the `agents-review-user-invoked` line needs D2's scope named
  beside it, since a rule with an unnamed exception is a rule that misleads.
- `docs/knowledge/areas/bee-herding/overview.md` — "the three-role cockpit" is
  its title claim and its body; it must sync when behavior changes.

## Canonical References

- `docs/history/research/herdr-fleet-lead-xia.md` — the distill this feature came
  from: the dependency matrix, what bee already had, and the two rows this
  feature closes.
- `skills/bee-herding/references/role-dispatch.md` — the reference shape a role
  protocol takes, and the arming conditions D2 reuses.
- `skills/bee-herding/references/operational-invariants.md` — where the cockpit's
  safety boundaries are recorded in full.

## Outstanding Questions

### Resolve Before Planning

- None. D1–D3 close every product question this feature raised.

### Deferred To Planning

- [ ] **Does `route` spend an occupancy slot?** A reviewer is a live pane, and
      the cap is 4. Planning must decide whether a reviewer counts against the
      same cap as a coder, or holds a slot of its own — reading
      `herding/wave.rs` occupancy and the ledger's row shape answers it.
- [ ] **How does `route` learn which agent produced the work?** The wave ledger
      records a spawn's agent, but the join from a worktree slug back to its
      dispatch row has not been traced. D2's different-agent check depends on it.
- [ ] **Where the reviewer's verdict is written**, and how `route` reads it back
      on the *next* cold iteration — the role holds no memory between ticks, so
      the verdict must be durable and findable from the worktree alone.
- [ ] **What marks a cell "served first" in its lane.** `claim-next` orders
      pipelines by backlog rank; whether a cell can outrank its siblings inside
      one lane needs `handlers_select.rs` read before D3's priority is designed.
- [ ] **Interval and tool surface for the role** — dispatch and merge run at
      60 s, supervisor at 900 s. Routing's rate follows from how long a review
      actually takes, which no run has measured yet.

## Deferred Ideas

Out-of-scope ideas captured during shaping. Not lost, not planned.

- **A harness-health guard** — probe a `herding.agents` entry for credit before
  dispatching to it, so a spent agent never idles a seat. Real, and named in the
  research brief, but it belongs to `dispatch`, not to `route`.
- **Artifact-liveness measurement** — commits since dispatch, last-commit age and
  report mtime, with a flag threshold. The doctrine is already written in
  `role-dispatch.md` §4; only the numbers are missing. A separate item.
- **Making `.bee/result-inbox/` runtime-neutral** — today only the Pi extension
  drains it. D1 puts it outside this feature deliberately.
- **Raising the 4-slot cap** — the cap has a named reason and needs its own
  evidence, not this feature's.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
