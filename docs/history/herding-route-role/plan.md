---
artifact_contract: bee-plan/v1
mode: high-risk
plan_rev: 2
---

# Plan: Herding Route Role

Mode: `high-risk` — 5 risk flags: audit-security, public-contracts, multi-domain, covered-contract-change, proof-weakening
Why this is the least workflow that protects the work: D2 and D4 both widen what an unattended agent may do without the human, which is the one axis bee's own scars say to widen only under a named, owner-armed exception — that earns the full ceremony even though the code is one new enum variant and one marker file.

**Revision 2.** The plan-step hat wave broke revision 1: five seats returned twelve accepted blockers, one of which (`WorkerRow.agent` is derivable, not missing) contradicted revision 1's own SMALLER PATH answer with evidence, and three seats independently found the same unnamed `route`/`merge` race. Full synthesis, including the four suspicions rejected on evidence: `reports/hat-wave-synthesis.md`.

## Requirements (from CONTEXT.md)

- **D1** — The control loop takes a fourth role, `route`, beside `dispatch`, `merge` and `supervisor`. Its finish signal is a **finished worktree**: phase `compounding-complete`, zero cells open or claimed, a clean tree, `HEAD` on `wt/<slug>`. One cold iteration acts on exactly one finished worktree, then exits. `.bee/result-inbox/` stays out of scope. (`a685d557`)
- **D2** — A named, scoped carve-out to `agents-review-user-invoked`. Inside the herding cockpit only, and only while **both** the owner enable marker exists **and** `gate_bypass` is `full` or `total`, `route` may start a reviewer with no human ask. The reviewer runs on a **different** `herding.agents` entry than the agent that produced the work. Everywhere else the rule is unchanged. (`8388df3e`)
- **D3** — CHANGES becomes a cell in that feature's lane, served ahead of anything else in the lane. BLOCKED stops the routing cold and reports. An unclassifiable outcome is reported and left standing, never dropped. (`4a395ea7`)
- **D4** — `route` may deliver a CHANGES follow-up brief into the **producing coder's still-open pane**. It still never *starts* a coder and never creates a pane; it may hand work to a pane already open and idle on the worktree it just reviewed. This amends CONTEXT.md's boundary line, on the owner's answer of 2026-09-08. Without it, D3's cell has no actor at all (see § The escalation this revision answers).

## Load-bearing claims

Labels: `read` = the author opened that file at that line; `ran` = the author executed that command and holds its output; `guessed` = inferred. No `guessed` row survives the gate. Evidence is a verbatim byte substring of the anchored lines; multi-line evidence joins lines with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A role is registered by parsing its name in one closed `match`, so a fourth arm is the whole parse change. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:72-75` | `"dispatch" => Some(Role::Dispatch),` / `"merge" => Some(Role::Merge),` / `"supervisor" => Some(Role::Supervisor),` / `_ => None,` |
| 2 | Each role names itself through a second closed `match`, which the prompt-file path is built from. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:81-83` | `Role::Dispatch => "dispatch",` / `Role::Merge => "merge",` / `Role::Supervisor => "supervisor",` |
| 3 | Each role carries its own default tick interval, so `route` picks its own without touching the cockpit's 60 s. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:93-94` | `Role::Dispatch \| Role::Merge => DEFAULT_INTERVAL,` / `Role::Supervisor => SUPERVISOR_DEFAULT_INTERVAL,` |
| 4 | The role's opening prompt is found by name on disk — a new role needs a new `<role>-prompt.md` and no path code. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:439` | `let leaf = format!("{}-prompt.md", role.as_str());` |
| 5 | A role's tool surface is a per-(role, transport) closed `match`, so `route` gets an enumerated surface of its own on both transports. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:273` | `fn allowed_tools_for(role: Role, kind: TransportKind) -> &'static str {` |
| 6 | The supervisor is the precedent for a role whose surface is enumerated verb by verb rather than taking the whole-CLI wildcard. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:299` | `const SUPERVISOR_ALLOWED_TOOLS: &str = "Bash(.bee/bin/bee status:*),\` |
| 7 | The `--role` refusal names the legal set in prose, so a fourth role must be added there too or the error lies. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:153` | `format!("unknown role '{v}' (expected dispatch, merge or supervisor)")` |
| 8 | The missing-role message names the set a third time. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:195` | `let role = role.ok_or_else(\|\| "--role dispatch\|merge\|supervisor is required".to_string())?;` |
| 8b | The usage line names it a fourth time. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:225` | `"Usage: bee herding control-loop --role dispatch\|merge\|supervisor [--main-root PATH] \` |
| 9 | Existing tests assert the three-role set by name, so adding a fourth is a covered-contract change, not an additive one. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:916-918` | `fn unknown_role_names_supervisor_among_the_expected_ones() {` / `let err = Options::parse(&["--role", "observer"]).unwrap_err();` / `assert!(err.contains("supervisor"), "{err}");` |
| 10 | The four finished-worktree conditions D1 reuses already exist, written once, in the merge role's protocol. | read | `skills/bee-herding/references/role-merge.md:73-76` | `1. \`phase\` is \`compounding-complete\`;` / `2. zero cells open or claimed;` / `3. a clean tree (\`git status --porcelain\` empty);` / `4. \`HEAD\` is exactly \`wt/<slug>\`.` |
| 11 | Nothing in bee forbids a model reviewing its own output, and an unset `review` slot falls through to the coding slot — so D2's different-agent rule is new law. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:666-668` | `pub(crate) fn tier_role_list(slot: &str) -> Vec<&str> {` / `    if slot == "review" \|\| slot == "read" {` / `        return vec![slot, "generation"];` |
| 12 | An agent cannot push at all, so nothing in this feature can reach a remote however the routing goes wrong. | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:859` | `"git push is outward-facing and is never exempted from this gate, regardless of what it would push. ",` |
| 13 | The review store's decision vocabulary is a closed set of three — so D3's CHANGES needs an explicit mapping, not a new status. | read | `packages/bee-rs/crates/bee/src/verbs/reviews.rs:69` | `const DECISION_STATUSES: [&str; 3] = ["pending", "blocked", "approved"];` |
| 14 | The enable interlock, half of D2's arming law, is a single durable marker with one reader. | ran | `.bee/bin/bee herding interlock --help` | `Report the dispatch loop's owner-enable interlock — {enabled, marker, main_root} — read before dispatch may build any dispatchable set.` |
| 14b | The `gate_bypass` level, the **other** half of D2's arming law, is already a live per-iteration refusal in the sibling role — so `route` copies a check, it does not invent one. | read | `skills/bee-herding/references/role-dispatch.md:41-42` | `This role may only pick up work when \`gate_bypass_level\` is exactly \`full\` or` / `\`total\`. At \`off\` or \`normal\`: build nothing, classify nothing, spawn nothing —` |
| 15 | **The four-slot cap is prose, not code** — so `route`'s obligation to respect it is a line in its own contract, exactly like dispatch's, and no enforcement code is owed. | read | `skills/bee-herding/references/role-dispatch.md:89-91` | `and hold it against the cap exactly as before: the cap is 4; at \`>= 4\` no` / `  slot is free — still run the anomaly scan below, but do not build or` / `  announce a dispatch decision (§6-7).` |
| 16 | **The producing agent is derivable, not missing.** The cockpit builds every worker's argv from one config key and never varies it per dispatch — so no new recorded field is owed. | read | `skills/bee-herding/references/role-dispatch.md:420-424` | `**The trailing agent argv is config-driven, not hard-fixed prose.** Read` / `\`.bee/config.json\`'s \`herding.agent_command\` from MAIN: a non-empty array` |
| 17 | That same key is resolved cold by an existing resolver with a fixed precedence, so `route` reads the producing agent the way the dispatcher chose it. | read | `packages/bee-rs/crates/bee/src/herding/wave.rs:412-417` | `    if let Some(name) = generation_slot_herding_agent(cfg, runtime) {` / `        return resolve_from_registry(&registry, name);` / `    }` / `    if let Some(name) = cfg.get("herding").and_then(\|h\| h.get("agent_command")).and_then(Value::as_str) {` |
| 18 | **A durable per-slug marker that a sibling role honors is an existing pattern**, not a new mechanism — the in-review marker copies the red-stop marker exactly. | read | `skills/bee-herding/references/role-merge.md:237` | `\| Red-stop marker, check before merging \| \`ls .bee/tmp/bee-herding.red.<slug>\` — exists → skip this worktree, say nothing (§4) \|` |
| 19 | **The coder's pane is still open and idle** when `route` finds a finished worktree — merge is the only thing that ever closes it. That pane is D4's actor. | read | `skills/bee-herding/references/role-merge.md:157-161` | `it: \`bee herding pane close <pane_id>\`. This is the **only** circumstance in which` / `this role closes a pane — it frees the slot dispatch's §4 occupancy count` / `watches next.` |
| 20 | **Writing a cell makes the worktree unfinished, and merge then skips it forever** — which is why D4 exists and why the fix cell must reach an actor in the same tick. | read | `skills/bee-herding/references/role-merge.md:89-90` | `anomaly — skip it silently and let a later invocation find it once it settles.` |
| 21 | **`dispatch` will never pick that worktree up either** — it refuses any feature already holding a grant. | read | `skills/bee-herding/references/role-dispatch.md:243-245` | `- **(c) No worktree grant.** From \`bee worktree list --json\`, a grant exists` / `  for \`<slug>\` when any \`grants\` key ends with \`--wt--<slug>\`. One exists →` / `  already under way; skip.` |
| 22 | The cold read-back path for a verdict needs no new store — the candidate ledger row carries the feature slug. | read | `packages/bee-rs/crates/bee/src/verbs/reviews.rs:1226` | `entry.insert("feature".into(), Value::String(js_trim(&feature).to_string()));` |
| 23 | A cell is an untyped JSON object validated dynamically, so the CHANGES cell goes through the same `cells add` door as any other — no schema change. | read | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:230` | `for field in ["id", "feature", "title", "action", "verify"] {` |
| 24 | **`cells add` takes no root flag**, so `route` must write the cell from inside the worktree — and that write dirties the tree, which is the block D4's hand-off must beat in the same tick. | ran | `.bee/bin/bee cells add --help` | `      --file (str) — Path to a cell JSON file. Required unless --stdin is set.` |
| 25 | **Announce-once-with-scrollback-dedup is an existing cockpit discipline**, so B9's non-silent refusals reuse it rather than adding state. | read | `skills/bee-herding/references/role-dispatch.md:100-101` | `` `bee herding pane read <chat_pane_id> --lines 200` first, and `` / `  only send the line if that scrollback does not already carry it —` |
| 26 | **The supervisor ships ONE document, and says so** — so `route` ships `route-prompt.md` alone, with no `role-route.md` beside it. | read | `skills/bee-herding/references/supervisor-prompt.md:8-10` | `Unlike the dispatch and merge roles of this same loop, no skill document` / `carries your procedure. This file is the whole contract. Read it, do the four` / `steps, stop.` |
| 27 | Inside one lane, `claim-next` selects by pipeline backlog rank and pipeline creation time — **not** by any per-cell priority. D3's "served first" therefore rests on the lane being empty, never on ordering. | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_select.rs:777` | `            // (cell, rank, created_at_ms) — the sort keys, built in pool order.` |

## Discovery

Inspected `control_loop.rs` end to end for how a role registers: eight closed sites plus one prompt file resolved by name. The supervisor role is the working precedent, and it ships one document, not two (claim 26).

Four read-tier gathers and then a five-seat hat wave ran over revision 1. The wave changed the shape in four ways, each on evidence:

1. **The new ledger field is deleted.** Revision 1 added `WorkerRow.agent` because nothing records the producing agent. True of the ledger, false of the cockpit: the dispatcher builds every worker from one config key and never varies it (claims 16–17), so the producer is readable cold at zero schema cost — and the field would have failed closed on every row already on disk, routing nothing until a fresh dispatch.
2. **An in-review marker is added.** Three seats independently found `route` and `merge` racing on the same four conditions, and the same gap makes a reviewer-in-flight indistinguishable from a finished-and-unrouted worktree on the next cold tick. One durable per-slug marker answers both, copying a pattern merge already honors (claim 18).
3. **D4 was escalated to the owner and answered.** See below.
4. **Three slices became two, and two documents became one.**

## The escalation this revision answers

`route` writes the CHANGES cell; the worktree then fails condition 2, so `merge` skips it forever (claim 20) and `dispatch` refuses it (claim 21), while `route` itself never starts a coder. D3's "the same worktree takes it" had no actor. The coder's pane is still open and idle (claim 19) — the natural actor, and the one the source fleet uses — but handing it a brief is a power CONTEXT.md's boundary denied. That conflict was not resolved here: it went to the owner, who chose the hand-off. **D4** records it, and the boundary is amended rather than quietly stretched.

## Approach

**Recommended path.** Add `route` as a fourth control-loop role, built the way `supervisor` was built (claims 1–9, 26): a closed arm in each registration site, one `route-prompt.md` that is the whole contract, and an enumerated tool surface **narrower than dispatch's and wider than supervisor's**. D1's finished test is read from its one existing home (claim 10), never restated. D2's two arming conditions are the interlock and bypass checks the sibling role already runs (claims 14, 14b), so the carve-out cannot arm itself. The producing agent is resolved through the existing config precedence (claims 16–17), not recorded anew. One new artifact exists: `.bee/tmp/bee-herding.review.<slug>`, carrying the reviewed HEAD sha and the reviewer's pane id, copying the red-stop marker pattern (claim 18) — it is what stops `merge` mid-review, what makes "already routed at this head" readable by a cold tick, and what a human deletes to undo a wrong route.

**The four verdict paths, all four defined.**

| Verdict | Recorded as | `route` then |
|---|---|---|
| clean | `decision.status = "approved"` (claim 13) | clears the marker, announces `<slug> reviewed clean by <agent> — ready for your merge`, stops. `route` never merges. |
| CHANGES | `decision.status = "blocked"` plus findings via `--kind finding` | writes the cell from inside the worktree (claim 24), hands the brief to the coder's open pane (D4, claim 19), clears the review marker, announces both acts |
| BLOCKED (the reviewer came back blocked) | `decision.status = "blocked"` | writes `.bee/tmp/bee-herding.blocked.<slug>`, prints the reason, sets the `waiting-on` mark, stops cold |
| unclassifiable | nothing | announces once and leaves the worktree exactly as it found it |

**Rejected alternatives.**
- *A standalone `bee herding route --once` verb* — the control loop already owns the loop, interval, ceiling and tool surface; D1 names a role.
- *Routing off `.bee/result-inbox/`* — D1: no cockpit coder writes a marker there.
- *`WorkerRow.agent`* — rejected on evidence by the wave (claims 16–17): derivable, and it would fail closed on every existing row.
- *A per-cell priority field for D3* — rejected on evidence (claim 27): the lane is empty when the cell is written, so ordering never arises.
- *A slug→review-session index* — the candidate ledger already carries the slug (claim 22).
- *`bee supervisor record` as the "reported and left standing" channel* — rejected by the wave: `--target-session` is required and a cold tick has no session id.
- *`bee cells dissent-verdict` as the verdict grammar* — different subject; `reviews record --kind decision` is the home.
- *A second document (`role-route.md`)* — rejected on evidence (claim 26): the precedent ships one.
- *Letting `route` merge on a green review* — merge stays a human gesture.

**SMALLER PATH check, re-run after the wave (revision 1's PASS was broken by claim 16).** Is there a cheaper shape that still honors D1–D4? No, and the wave is the evidence: it deleted the ledger field, the second document and one slice, and every survivor now maps one-to-one onto a locked decision. Remove the role and D1 is unimplemented; remove the in-review marker and D2's reviewer races merge and re-dispatches itself every tick; remove the cell-plus-hand-off and D3 and D4 are unimplemented. The one component the wave was most suspicious of — the fourth role itself — survived its own deletion test: each registration site carries a per-role *fact*, not a pass-through. PASS.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| The carve-out's arming check (D2) | **HIGH** — a carve-out that arms itself is the whole danger | Five cases: no marker · marker + `off` · marker + `normal` · marker + `full` · marker + `total`. Each refuses or permits exactly once. |
| The different-agent rule (D2) | **HIGH** — silently reviewing with the producing agent is worse than not reviewing | Same-agent pairing refuses by name; a producer absent from the registry refuses (never "different by string compare"); a one-entry registry refuses and says so |
| The hand-off to the coder pane (D4) | **HIGH** — a new write into a live agent's pane | The brief goes only to a pane labelled with the reviewed slug, only after the cell is written, and never to a pane classifying Blocked (the pane verb already refuses that) |
| The in-review marker | MEDIUM — the whole race fix rests on it | Merge skips a marked worktree; a second `route` tick on a marked slug does nothing; a dead reviewer pane clears it; a human deleting it restores the pre-route state |
| The finished-worktree read (D1) | MEDIUM — a false "finished" routes live work | One test per failing condition, and one that the conditions are read from their single home |
| The tool surface | MEDIUM — a wildcard here is a router with merge rights | Forbidden-token test mirroring `SUPERVISOR_FORBIDDEN_TOOL_TOKENS` |
| The CHANGES cell (D3) | LOW — claim 27 removed the ordering risk | The cell lands in the finished feature's lane and `claim-next` returns it |
| The BLOCKED stop (D3) | LOW — stopping is the safe direction | A durable marker, a reason, a `waiting-on` mark, and no dispatch after |

## Shape

**Feature outcome.** With the cockpit enabled and the owner's bypass at `full`, a worktree that finishes gets reviewed by a different agent without the human relaying it; a clean review announces itself ready to merge; a CHANGES verdict becomes the next thing that worktree's own coder works on; a blocked reviewer and anything unclassifiable stop and reach the human with something durable to act on.

**Repo-reality basis.** The control loop already runs three cold roles on this pattern (claims 1–9); the finished test, the review store, the cell door, the interlock, the bypass refusal, the durable-marker pattern, the announce-once dedup and the still-open coder pane all already exist (claims 10, 13, 14, 14b, 18, 19, 23, 25). Nothing here builds a subsystem, and after the wave nothing here changes a schema.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | The role exists, cold and bounded | Nothing else can be true until `--role route` parses, ticks and refuses correctly | S1 | role parse/name/interval/usage tests; forbidden-token test on the tool surface |
| E2 | Reading finished, unrouted worktrees | D1's finish signal, read from its one home, minus what is already under review | S1 | one test per failing condition; a marked worktree reads as not-routable |
| E3 | The carve-out, the reviewer, the race fix | D2 — the authority widening, the different-agent law, and the marker that keeps merge off | S2 | five arming tests; same-agent and absent-producer refusals; merge skips a marked worktree |
| E4 | The four verdict paths | D3 and D4 — including the clean path revision 1 never defined | S2 | one test per row of the verdict table; no dispatch after BLOCKED |
| E5 | Doctrine kept honest | A rule with an unnamed exception misleads its readers | rides S1–S2 | parity checks: SKILL.md, `operational-invariants.md`, `role-merge.md`, AGENTS.md and the knowledge area each name what shipped |

**Slice queue.**

- **S1 — the walking skeleton (current slice).** `--role route` parses, ticks on its own interval, carries an enumerated tool surface, and reads the granted worktrees that are **finished and not already under review**, announcing them once with the scrollback dedup. It dispatches nothing and writes no cell. A real capability on day one, no stub, no would-do report. Ships `route-prompt.md` and SKILL.md's three-roles→four.
- **S2 — the carve-out, the reviewer, and the four verdicts.** Depends on S1. The two arming conditions with their announced refusals, the producing-agent resolution, the different-agent selection and its refusals, the in-review marker, the reviewer dispatch on a `<slug>-review` pane, all four verdict paths including D4's hand-off, and the BLOCKED marker plus `waiting-on`. Ships the AGENTS.md carve-out sentence, `operational-invariants.md`'s new boundary, `role-merge.md`'s two additions (honor the review marker; close the `-review` pane), and the knowledge-area sync.

**Current slice to prepare: S1.**

## Test matrix

High-risk: probes per applicable dimension of `references/edge-dimensions.md`, judged against existing coverage first — `control_loop.rs`'s test module already pins the three-role set and the supervisor's surface, and those tests are the pattern each new one follows.

| Dimension | Probe (S1 unless noted) |
|---|---|
| Boundary | `--role route` parses and names itself `route`; `--role rout` refuses and the message names all four |
| Empty / absent | zero granted worktrees; every granted worktree unfinished — one tick, no announcement, clean exit |
| Malformed input | a worktree whose `bee orient` cannot be read classifies as **not** finished |
| Concurrency | two finished worktrees in one tick — exactly one is acted on, and the selection rule is stated so the other is not starved forever |
| Idempotence | a worktree already carrying `.bee/tmp/bee-herding.review.<slug>` at the same HEAD is not announced or routed a second time |
| Authority | the tool surface carries no `Write`, no `Edit`, no `Bash(git`, and no `Bash(.bee/bin/bee:*)` wildcard |
| Failure direction | every unreadable input classifies as not finished; a refusal is always announced, never silent |
| Cross-platform | the tool surface and the interval resolve identically on the tmux transport |
| Arming (S2) | five cases: no marker / marker + `off` / marker + `normal` / marker + `full` / marker + `total` |
| Identity (S2) | same-agent pairing refuses by name; a producer absent from the registry refuses; a one-entry registry refuses and says which |
| Race (S2) | `merge` skips a worktree carrying the review marker; a dead reviewer pane clears the marker instead of stranding it |
| Verdicts (S2) | one probe per row of the verdict table, including the clean path and the unclassifiable path |
| Hand-off (S2) | the CHANGES brief reaches only the pane labelled with the reviewed slug, only after the cell is written |
| Side effects (S2) | after a BLOCKED verdict: no dispatch, no hand-off, a durable marker, and a `waiting-on` mark |

## Open Questions

- **OQ5 — the tick interval is unmeasured.** Dispatch and merge run at 60 s, supervisor at 900 s. Routing's honest rate follows from how long a review actually takes, which no run in this repo has measured. S1 ships a stated default and records in `route-prompt.md` that the number is a starting guess, not a measurement.
- **OQ6 — D3's "BLOCKED worker" is read narrowly, and the owner has not confirmed the narrowing.** Under D1's trigger the only outcome `route` ever sees is a *reviewer's*; a blocked **coder** never produces a finished worktree and is already dispatch's §4 anomaly scan. Revision 2 implements the narrow reading. If the owner meant the coder too, that is a second trigger source and a new slice.

## Out of scope

- Making `.bee/result-inbox/` runtime-neutral (D1 puts it outside).
- A harness-health / credit guard before dispatch — real, but it belongs to `dispatch`.
- Artifact-liveness measurement (commits since dispatch, report mtime).
- Raising the four-slot cap, or moving it from prose into code.
- Any change to `bee-reviewing`, Gate 3, or the user-invoked review path outside the cockpit.
- Letting `route` merge anything. Merge stays a human gesture.
