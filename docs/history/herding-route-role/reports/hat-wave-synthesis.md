# Hat wave — plan-step synthesis, herding-route-role

Date: 2026-09-08. Five seats, three models, one draft (`plan.md` revision 1).
Seats: `hat-facts-gaps` (opus), `hat-risks` (fable), `hat-alternatives` (opus),
`hat-value` (agy-flash pane), `hat-user-impact` (agy-flash pane).
No seat was dropped; all five returned inside the ceiling.

This wave is this feature's plan check **and** its high-risk advisor consult
(absorption, decision `b34fdea9`).

## Verdict

**Revision 2 is owed.** Revision 1 does not survive the wave: one seat broke the
plan's own SMALLER PATH answer with evidence, and three independent seats found
the same unnamed race. Twelve blockers were accepted, four rejected on evidence,
and one blocker is escalated to the owner because resolving it would widen the
role's authority past the boundary CONTEXT.md itself sets.

## Accepted blockers, and what each changed

| # | Seat | Finding | Change in revision 2 |
|---|---|---|---|
| B1 | alternatives | **`WorkerRow.agent` is derivable and must not be added.** Inside the cockpit the producing agent is one config key: `role-dispatch.md:418-432` builds every worker's argv from `herding.agent_command` and never varies it per dispatch, and `resolve_agent_command_for_runtime` (`wave.rs:402-417`) resolves that same entry cold with `agent: None`. Worse, the field would fail closed on every row already on disk — S2 would route nothing until a fresh dispatch. | The field, the `record-worker --agent` flag, the `role-dispatch.md` caller change, the fail-closed-on-absent branch and its two tests are all **deleted**. The producing agent is resolved through the existing precedence. Claim 16's conclusion was wrong and is retired; the SMALLER PATH check is re-run, not amended. |
| B2 | facts-gaps · risks · user-impact (three seats, independently) | **`route` and `merge` race on the same finished set.** Both read the identical four conditions on independent 60 s loops; nothing holds `merge` off a worktree under review, so a merge can land work whose review is about to say CHANGES, and can delete the directory the reviewer is reading. | Revision 2 adds an **in-review marker** — `.bee/tmp/bee-herding.review.<slug>`, mirroring the existing `.bee/tmp/bee-herding.red.<slug>` pattern `role-merge.md:97-105` already honors. `merge` skips a marked worktree; `route` clears it when the verdict lands. |
| B3 | facts-gaps · risks | **A reviewer in flight is an undefined state.** `route` is cold; between "reviewer started" and "verdict recorded" the worktree still meets all four conditions, so every tick re-enters the same cell and re-dispatches. `reviews status` cannot distinguish it either — an open session reads `in review` forever (`reviews.rs:697-704`). | The same in-review marker carries the worktree's HEAD sha and the reviewer's pane id, which makes "already routed at this head" a durable predicate a cold tick can read. A dead reviewer pane is detected by the pane not being live, and clears the marker. |
| B4 | facts-gaps | **The APPROVE verdict has no defined outcome.** The most common verdict appears nowhere in revision 1. | Defined: a clean review clears the marker, announces `<slug> reviewed clean by <agent> — ready for your merge`, and stops. `route` never merges. |
| B5 | risks | **CHANGES has no home in the verdict vocabulary.** `DECISION_STATUSES` is `["pending", "blocked", "approved"]` (`reviews.rs:69`); CHANGES maps to none of them. | Mapping recorded: CHANGES → `decision.status = "blocked"` plus the findings appended through `--kind finding`; clean → `"approved"`. Written down rather than left to the implementer. |
| B6 | facts-gaps | **"A different `herding.agents` entry" has no selection rule and no empty case.** | Rule recorded: prefer the agent the `review` model-role names; if that equals the producer, take the first registry entry that differs; if the registry holds only the producer, refuse and say so. |
| B7 | facts-gaps · value | **The occupancy question CONTEXT.md deferred to planning was never answered.** | Answered: a reviewer is a worker pane, is recorded to the ledger like any spawn, and therefore counts toward the four-slot cap. `route` refuses at `>= 4` exactly as dispatch does. |
| B8 | user-impact | **Reviewer and coder panes cannot be told apart.** Coder panes are labelled `<slug>` and stay open until merge closes them (`role-merge.md:157-161`), so a reviewer on the same worktree would collide. | Reviewer panes are labelled `<slug>-review`. `merge`'s cleanup gains that label. |
| B9 | user-impact | **Guard refusals are silent.** Revision 1 mandated a silent exit and named no message for a failed marker, bypass level, agent or cap check. | Every refusal announces once, through the scrollback dedup the cockpit already uses (`role-dispatch.md:98-113`). |
| B10 | user-impact | **A BLOCKED stop leaves nothing durable and nothing actionable.** | The stop writes `.bee/tmp/bee-herding.blocked.<slug>`, prints the reason, and sets the `waiting-on` mark. |
| B11 | user-impact · facts-gaps | **`route` cannot see a blocked coder at all** — a blocked worker's worktree is not finished, and finished is the only trigger. D3's BLOCKED path had no source. | Scope reading recorded: under D1's trigger the only outcome `route` ever reads is a **reviewer's**, so D3's BLOCKED means a reviewer that came back blocked. A blocked *coder* is already dispatch's §4 anomaly scan. Flagged to the owner as a narrowing of D3's words. |
| B12 | risks | **`cells add` has no root flag**, so `route` must run it inside the worktree, and the written cell dirties that tree. | Named in the protocol, and the tool surface is written to permit it. |

## The one escalation

**Nobody runs the CHANGES cell.** Found independently by `hat-facts-gaps` and
`hat-risks`, and verified against the code:

- `route` writes the cell into the finished feature's lane.
- `merge` now skips that worktree permanently — it fails condition 2, and
  `role-merge.md:89-90` says "A granted worktree failing the test is ordinary
  work in progress… skip it silently".
- `dispatch` will not start a worker either — `role-dispatch.md:243` refuses any
  PBI whose feature already holds a worktree grant.
- `route` never starts a coder (CONTEXT.md's own boundary).

So D3's "the same worktree takes it ahead of anything new" has no actor. The
coder's pane is still open and idle at that moment (`role-merge.md:157-161`
makes merge the only thing that closes it), which is the natural actor and is
exactly what the source fleet does — but handing it the brief means `route`
writes into a live worker's pane, an authority CONTEXT.md's boundary denies.

This is a conflict between a locked decision and the feature boundary, with no
clean answer inside either. It is escalated to the owner rather than designed
here (`bee-principle-never-invent-behavior-neither-side-has`).

## Rejected on evidence, recorded rather than dropped

| Seat | Suspicion | Why it failed |
|---|---|---|
| alternatives | `bee supervisor record` as D3's "reported and left standing" | `--target-session` is required and a cold tick has no session id to address |
| alternatives | `bee cells dissent-verdict` as the verdict grammar | Different subject — a cell's dissent, not a review outcome |
| alternatives | A slug→review-session index | `bee reviews status --feature <slug>` already derives it (claim 21 stands) |
| alternatives | The fourth role itself — the component it was most suspicious of | Survives the deletion test: each registration site carries a per-role *fact*, not a pass-through. Delete the role and every fact has to be expressed once somewhere else |

## Accepted warnings

- **One document, not two.** There is no `role-supervisor.md` — the supervisor
  ships `supervisor-prompt.md` alone, and it says so in its own words: "no skill
  document carries your procedure. This file is the whole contract"
  (`supervisor-prompt.md:8-10`). `route-prompt.md` becomes the whole contract;
  `role-route.md` is dropped.
- **Two slices, not three.** The plan's own risk map grades both S3 rows LOW, and
  claim 20 already dissolved the ordering mechanism. No risk boundary sits on the
  S2/S3 line, so S3 folds into S2.
- **S1 shrinks.** Its "and to which different agent" clause was a would-do
  report, not a capability. S1 announces which granted worktrees are finished and
  unrouted — a real capability on day one.
- **`gate_bypass: total` is untested** in the arming matrix; added.
- **"Exactly one is acted on" has no selection rule** — a cold role can starve the
  same worktree every tick; a rule is recorded.
- **Claim 19 misread the write path.** `worker_row_to_json` and
  `worker_row_from_json` are hand-rolled (`wave_ledger.rs:103-118`, `:138-150`);
  the serde derive is not the write path. Moot once B1 removes the field, but the
  claim is retired rather than left standing.
- **Row 18's anchor was off by one** — `wave_ledger.rs:81` is `pane_id`, not
  `worktree`. Moot once B1 removes the row.
- **Three prose claims never became rows** — the `gate_bypass` half of D2's
  arming law, the grant-row shape, and the scrollback dedup. All three are rows
  in revision 2.
- **Claim 20 read the wrong path for the serve side** — `read.rs:166-171` is the
  list sort; `claim-next` selects through `handlers_select.rs`. The claim is
  re-anchored and narrowed to what it actually shows.
- **`agent` could be erased by a later append** under the same `wave_id`. Moot
  once B1 removes the field.
- **A producer no longer in the registry** reads as "different" by string
  compare, and a rename passes falsely. Covered by B6's refusal rule.
- **Value's slice critique** — S1 and S2 have little standalone value and the
  feature earns its cost only when the loop closes. Accepted as true and
  answered by folding three slices into two.
