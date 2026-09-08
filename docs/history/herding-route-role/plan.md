---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Herding Route Role

Mode: `high-risk` — 5 risk flags: audit-security, public-contracts, multi-domain, covered-contract-change, proof-weakening
Why this is the least workflow that protects the work: D2 widens what an unattended agent may do without the human, which is the one axis bee's own scars say to widen only under a named, owner-armed exception — that earns the full ceremony even though the code is one new enum variant, one optional ledger field, and one role document.

## Requirements (from CONTEXT.md)

- **D1** — The control loop takes a fourth role, `route`, beside `dispatch`, `merge` and `supervisor`. Its finish signal is a **finished worktree**: phase `compounding-complete`, zero cells open or claimed, a clean tree, `HEAD` on `wt/<slug>`. One cold iteration acts on exactly one finished worktree, then exits. `.bee/result-inbox/` stays out of scope. (`a685d557`)
- **D2** — A named, scoped carve-out to `agents-review-user-invoked`. Inside the herding cockpit only, and only while **both** the owner enable marker exists **and** `gate_bypass` is `full` or `total`, `route` may start a reviewer with no human ask. The reviewer runs on a **different** `herding.agents` entry than the agent that produced the work. Everywhere else the rule is unchanged. (`8388df3e`, touches `565e68d0`, `b34fdea9`)
- **D3** — CHANGES becomes a cell in that feature's lane, served ahead of anything else in the lane. BLOCKED stops the routing cold and reports. An unclassifiable outcome is reported and left standing, never dropped. (`4a395ea7`)

## Load-bearing claims

Labels: `read` = the author opened that file at that line; `ran` = the author executed that command and holds its output; `guessed` = inferred. No `guessed` row survives the gate. Evidence is a verbatim byte substring of the anchored lines.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A role is registered by parsing its name in one closed `match`, so a fourth arm is the whole parse change. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:72-75` | `"dispatch" => Some(Role::Dispatch),` / `"merge" => Some(Role::Merge),` / `"supervisor" => Some(Role::Supervisor),` / `_ => None,` |
| 2 | Each role names itself through a second closed `match`, which the prompt-file path is built from. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:81-83` | `Role::Dispatch => "dispatch",` / `Role::Merge => "merge",` / `Role::Supervisor => "supervisor",` |
| 3 | Each role carries its own default tick interval, so `route` picks its own without touching the cockpit's 60 s. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:93-94` | `Role::Dispatch \| Role::Merge => DEFAULT_INTERVAL,` / `Role::Supervisor => SUPERVISOR_DEFAULT_INTERVAL,` |
| 4 | The role's opening prompt is found by name on disk — a new role needs a new `<role>-prompt.md` and no path code. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:439` | `let leaf = format!("{}-prompt.md", role.as_str());` |
| 5 | A role's tool surface is a per-(role, transport) closed `match`, so `route` gets an enumerated surface of its own on both transports. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:273` | `fn allowed_tools_for(role: Role, kind: TransportKind) -> &'static str {` |
| 6 | The supervisor is the precedent for a role whose surface is enumerated verb by verb rather than taking the whole-CLI wildcard — the shape `route` copies. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:299` | `const SUPERVISOR_ALLOWED_TOOLS: &str = "Bash(.bee/bin/bee status:*),\` |
| 7 | The `--role` refusal names the legal set in prose, so a fourth role must be added there too or the error lies. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:153` | `format!("unknown role '{v}' (expected dispatch, merge or supervisor)")` |
| 8 | The usage line names the same set a third time, and the missing-role message a fourth. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:195` | `let role = role.ok_or_else(\|\| "--role dispatch\|merge\|supervisor is required".to_string())?;` |
| 9 | Existing tests assert the three-role set by name, so adding a fourth is a covered-contract change, not an additive one. | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:916-918` | `fn unknown_role_names_supervisor_among_the_expected_ones() {` / `let err = Options::parse(&["--role", "observer"]).unwrap_err();` / `assert!(err.contains("supervisor"), "{err}");` |
| 10 | The four finished-worktree conditions D1 reuses already exist, written once, in the merge role's protocol. | read | `skills/bee-herding/references/role-merge.md:73-76` | `1. \`phase\` is \`compounding-complete\`;` / `2. zero cells open or claimed;` / `3. a clean tree (\`git status --porcelain\` empty);` / `4. \`HEAD\` is exactly \`wt/<slug>\`.` |
| 11 | Nothing in bee forbids a model reviewing its own output, and an unset `review` slot falls through to the coding slot — so D2's different-agent rule is new law, not a restatement. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:666-668` | `pub(crate) fn tier_role_list(slot: &str) -> Vec<&str> {` / `    if slot == "review" \|\| slot == "read" {` / `        return vec![slot, "generation"];` |
| 12 | An agent cannot push at all, so nothing in this feature can reach a remote however the routing goes wrong. | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:859` | `"git push is outward-facing and is never exempted from this gate, regardless of what it would push. ",` |
| 13 | The review store already has a create/record surface with a closed decision vocabulary, so a verdict has a durable home and needs no new store. | read | `packages/bee-rs/crates/bee/src/verbs/reviews.rs:68` | `const RECORD_KINDS: [&str; 5] = ["manifest", "preflight", "finding", "uat", "decision"];` |
| 14 | The enable interlock D2 arms on is a single durable marker with one reader, so "cockpit only" is checkable, not a vibe. | ran | `.bee/bin/bee herding interlock --help` | `Report the dispatch loop's owner-enable interlock — {enabled, marker, main_root} — read before dispatch may build any dispatchable set.` |
| 15 | **The four-slot cap is prose, not code** — so `route`'s obligation to respect it is a line in its protocol document, exactly like dispatch's, and no enforcement code is owed. | read | `skills/bee-herding/references/role-dispatch.md:89-91` | `and hold it against the cap exactly as before: the cap is 4; at \`>= 4\` no` / `  slot is free — still run the anomaly scan below, but do not build or` / `  announce a dispatch decision (§6-7).` |
| 16 | **Nothing records which agent produced a worktree.** The wave-ledger worker row has seven fields and none of them is the agent. D2's different-agent rule therefore needs a new recorded field — it cannot be derived. | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:78-93` | `pub(crate) struct WorkerRow {` / `pub(crate) name: String,` / `pub(crate) pane_id: String,` / `pub(crate) worktree: String,` / `pub(crate) task: String,` |
| 17 | The herding-run dispatch log does not carry it either — seven keys, no agent and no model. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2064-2070` | `m.insert("source".into(), Value::String("herding-run".into()));` / `m.insert("job_id".into(), Value::String(opts.job_id.clone()));` / `m.insert("task".into(), Value::String(opts.task.clone()));` |
| 18 | The worker row already carries the worktree path, so the join from a finished worktree back to its spawn row exists — only the agent name is missing from it. | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:81` | `pub(crate) worktree: String,` |
| 19 | The worker row is `serde` with an existing `skip_serializing_if` optional field, so one more optional field is additive and cannot break an older row. | read | `packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:91-92` | `#[serde(default, skip_serializing_if = "Option::is_none")]` / `pub(crate) retryable: Option<bool>,` |
| 20 | **D3's "served ahead of anything else" is satisfied by construction** — a worktree is only finished when it has zero open or claimed cells, so the CHANGES cell is the only cell in that lane when `route` writes it. Cells order by natural `id` sort and carry no priority field; none is needed. | read | `packages/bee-rs/crates/bee/src/verbs/cells/read.rs:166-171` | `cells.sort_by(\|a, b\| {` / `natural_cmp(` |
| 21 | No slug→review-session index exists, but the candidate ledger row carries the feature slug — that is the cold read-back path, with no new store. | read | `packages/bee-rs/crates/bee/src/verbs/reviews.rs:1226` | `entry.insert("feature".into(), Value::String(js_trim(&feature).to_string()));` |
| 22 | A cell is an untyped JSON object validated dynamically, so the CHANGES cell `route` writes goes through the same `cells add` door as any other — no schema change. | read | `packages/bee-rs/crates/bee/src/verbs/cells/validate.rs:230` | `for field in ["id", "feature", "title", "action", "verify"] {` |

## Discovery

Inspected `control_loop.rs` end to end for how a role registers: eight closed sites (the enum, parse, as_str, default interval, the `--role` refusal, the missing-role message, the usage line, and `allowed_tools_for`) plus one prompt file resolved by name. The supervisor role, added by `slp-supervisor-heartbeat`, is a working precedent for exactly this shape, which is why the recommended path adapts it instead of designing a router.

Four read-tier gathers ran (`job-1788875649935`, `job-1788875651588`, `job-1788879365477`, `job-1788879368107`). Two of their findings changed this plan:

1. **Nothing records which agent produced a worktree** (claims 16–17). D2's different-agent rule cannot be checked against today's records at all. The shape now carries one additive optional field on the wave-ledger worker row, written at spawn — and `route` fails closed when it is absent, rather than reviewing with an unknown producer.
2. **D3's priority requirement dissolves** (claim 20). A finished worktree has zero open cells by definition, so the CHANGES cell `route` writes is the only cell in that lane. No priority field, no ordering change, no new mechanism — the smallest honest shape was smaller than the draft assumed.

## Approach

**Recommended path.** Add `route` as a fourth control-loop role, built the way `supervisor` was built (claims 1–9): a closed arm in each registration site, a `route-prompt.md`, a `role-route.md` protocol document, and an enumerated tool surface **narrower than dispatch's and wider than supervisor's** — it needs `bee reviews`, `bee cells add` and the pane verbs, and it needs no `git` write scope at all. D1's finished test is read from its one existing home (claim 10), never restated. D2's two arming conditions are the same interlock and bypass reads `role-dispatch.md` already refuses below, so the carve-out cannot arm itself. The one new record is `WorkerRow.agent: Option<String>` (claims 16, 19) — additive, append-only, and the only thing that makes D2's different-agent law checkable.

**Rejected alternatives.**
- *A standalone `bee herding route --once` verb the dispatch loop calls* — rejected: it would need its own loop, interval, ceiling and tool surface, all of which the control loop already owns; and D1 names a role.
- *Routing off `.bee/result-inbox/`* — rejected by D1: no cockpit coder writes a marker there.
- *Deriving the producing agent from the pane, the grant, or the worktree identity file* — rejected on evidence: none of them records it (claims 16–17, and the grant row is `"<id>": true`).
- *A per-cell priority field for D3* — rejected on evidence (claim 20): the lane is empty when the cell is written.
- *A slug→review-session index* — rejected: the candidate ledger already carries the slug (claim 21).
- *Letting `route` merge on a green review* — rejected: merge stays a human gesture, and nothing in D1–D3 asks for it.

**SMALLER PATH check — is there a cheaper shape that still honors D1, D2 and D3?** No. The research already shrank this shape twice: claim 20 removed a priority mechanism D3 seemed to need, and claim 15 removed cap-enforcement code. What remains maps one-to-one onto the three locked decisions — remove the role and D1 is unimplemented; remove `WorkerRow.agent` and D2's different-agent law cannot be checked at all (claims 16–18); remove the CHANGES cell and D3 is unimplemented. PASS.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| The carve-out's arming check (D2) | **HIGH** — a carve-out that arms itself is the whole danger of this feature | A test per failure direction: no marker · marker + `off` · marker + `normal` · marker + `full`. Each refuses or permits exactly once. |
| The different-agent rule (D2) | **HIGH** — silently reviewing with the producing agent is worse than not reviewing | A same-agent pairing refuses by name; a row with **no** `agent` field refuses too — fail closed, never "probably different" |
| `WorkerRow.agent` (new field) | MEDIUM — a ledger schema change on an append-only file | A test that a pre-existing row with no `agent` key still deserializes and still counts toward occupancy |
| The finished-worktree read (D1) | MEDIUM — a false "finished" routes live work | A test per failing condition, and one that the conditions are read from their single home |
| The tool surface | MEDIUM — a wildcard here is a router with merge rights | A forbidden-token test mirroring `SUPERVISOR_FORBIDDEN_TOOL_TOKENS` |
| The CHANGES cell (D3) | LOW — claim 20 removed the ordering risk | A test that the cell is written into the finished feature's lane and that `claim-next` returns it |
| The BLOCKED stop (D3) | LOW — stopping is the safe direction | A test that no dispatch and no cell write follows a BLOCKED read |

## Shape

**Feature outcome.** With the cockpit enabled and the owner's bypass at `full`, a worktree that finishes gets reviewed by a different agent without the human relaying it; a CHANGES verdict becomes the next thing that worktree works on; a BLOCKED worker and anything unclassifiable stop and reach the human.

**Repo-reality basis.** The control loop already runs three cold roles on this exact pattern (claims 1–9); the finished test, the review store, the cell door, the interlock and the occupancy count all already exist (claims 10, 13, 14, 15, 22). The one genuinely missing fact is who produced a worktree (claims 16–18), and it costs one optional field.

| Epic | Capability / Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| E1 | The role exists, cold and bounded | Nothing else can be true until `--role route` parses, ticks and refuses correctly | S1 | role parse/name/interval/usage tests; forbidden-token test on the tool surface |
| E2 | Reading a finished worktree, and knowing who made it | D1's finish signal plus the one fact nothing records | S1 | a test per failing condition; a legacy-row test for the new optional field |
| E3 | The carve-out and the reviewer | D2 — the authority widening, and the different-agent law | S2 | four arming tests; a same-agent refusal; a fail-closed test when `agent` is absent |
| E4 | CHANGES and BLOCKED | D3's asymmetry | S3 | the cell lands in the right lane and `claim-next` returns it; no side effect after BLOCKED |
| E5 | Doctrine kept honest | A rule with an unnamed exception misleads its readers | rides S1–S3 | parity checks: the skill, the invariants, AGENTS.md and the knowledge area each name what shipped |

**Slice queue.**

- **S1 — the walking skeleton (current slice).** `--role route` parses, ticks on its own interval, carries an enumerated tool surface, reads finished worktrees through the merge role's existing conditions, resolves the producing agent from the wave ledger's new `agent` field, and **announces into the chat pane what it would route and to which different agent**. It dispatches nothing and writes no cell. Real end-to-end behavior, no stubs. Ships with `route-prompt.md`, `role-route.md`, `WorkerRow.agent`, `bee herding record-worker --agent`, and SKILL.md moving from three roles to four.
- **S2 — the carve-out and the reviewer.** Depends on S1. The two arming conditions, the different-agent refusal (including the fail-closed absent case), the reviewer dispatch, the verdict recorded through `bee reviews record --kind decision`, the candidate row that makes it findable by slug. Ships with the AGENTS.md carve-out sentence and `operational-invariants.md`'s new boundary.
- **S3 — the two outcome paths.** Depends on S2. CHANGES becomes a cell in the finished feature's lane; BLOCKED stops cold; unclassifiable is reported and left standing. Ships with the knowledge-area sync.

**Current slice to prepare: S1.**

## Test matrix

High-risk: probes per applicable dimension of `references/edge-dimensions.md`, judged against existing coverage first — `control_loop.rs`'s own test module already pins the three-role set and the supervisor's surface, and those tests are the pattern each new one follows rather than a thing to duplicate.

| Dimension | Probe (S1 unless noted) |
|---|---|
| Boundary | `--role route` parses and names itself `route`; `--role rout` refuses and the message names all four |
| Empty / absent | zero granted worktrees; every granted worktree unfinished — one tick, no announcement, exit clean |
| Malformed input | a worktree whose `bee orient` cannot be read classifies as **not** finished |
| Backward compatibility | a wave-ledger row written before `agent` existed still deserializes and still counts toward occupancy |
| Concurrency | two finished worktrees in one tick — exactly one is acted on (D1) |
| Authority | the tool surface carries no `Write`, no `Edit`, no `Bash(git`, no `Bash(.bee/bin/bee:*)` wildcard |
| Idempotence | the same finished worktree over two ticks does not announce twice (the scrollback dedup the cockpit already uses) |
| Failure direction | every unreadable input classifies as not finished; an absent `agent` refuses to name a reviewer |
| Cross-platform | the tool surface and the interval resolve identically on the tmux transport |
| Arming (S2) | four cases: no marker / marker + `off` / marker + `normal` / marker + `full` |
| Identity (S2) | a same-agent reviewer pairing refuses by name; an absent producing agent refuses, never proceeds |
| Ordering (S3) | the CHANGES cell is the one `claim-next` returns for that lane |
| Side effects (S3) | after a BLOCKED read: no dispatch, no cell, no pane write beyond the report line |

## Open Questions

- **OQ5 — the tick interval is unmeasured.** Dispatch and merge run at 60 s, supervisor at 900 s. Routing's honest rate follows from how long a review actually takes, which no run in this repo has measured. S1 ships a stated default and records in `role-route.md` that the number is a starting guess, not a measurement.

*(OQ1–OQ4 from the draft were resolved by the four gathers and are recorded as claims 15, 16–19, 21 and 20 respectively.)*

## Out of scope

- Making `.bee/result-inbox/` runtime-neutral (D1 puts it outside).
- A harness-health / credit guard before dispatch — real, but it belongs to `dispatch`.
- Artifact-liveness measurement (commits since dispatch, report mtime) — the doctrine is already written; only the numbers are missing.
- Raising the four-slot concurrency cap, or moving it from prose into code.
- Any change to `bee-reviewing`, Gate 3, or the user-invoked review path outside the cockpit.
- Letting `route` merge anything. Merge stays a human gesture.
