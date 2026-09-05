# Gates, Delegation, and Judgment

Load when a gate is about to be presented, a bypass level set, work fanned
out to workers, or a judgment call made about bending a rule. Day-to-day
routing and the skill catalog live in `routing-and-contracts.md`; nothing
here is needed to pick the next skill.

## The three gates

| Gate | Earned by | Question |
|---|---|---|
| **Gate 1** | exploring | "Decisions locked. Approve CONTEXT.md before planning?" |
| **Gate 2** | planning | "Work shape is ready. Approve before current-work preparation?" — approves `shape` AND `execution` together (`bee gate --merge`) |
| **Gate 3** | reviewing (user-invoked only) | P1 > 0 → "P1 findings block merge. Fix before proceeding?" ; P1 = 0 → "Review complete. Approve merge?" |

Gates 1-2 are the default chain every lane closes through. Gate 3 is
additive: it is asked once, inside a review session the user actually
invoked (rule: agents-review-user-invoked).

**A fourth stop, `uat`, sits later still** — not part of the chain above,
so "the three gates" stays accurate for Gates 1-3. Config `uat_stop` names
its POSITION: under `"close"` (the default — absent means this,
defaults-and-agent-env D1) the merge is a publish-for-testing step — the
agent merges on green without asking, the user tests on main, and the
stop moves to `bee close`; under `"merge"` it sits at
`bee worktree merge`, after execution is done — the user's acceptance of
the finished work, required before merge for `standard`/`high-risk`
features (`tiny`/`small`/`docs`/`spike` exempt), refused with
`WORKTREE_MERGE_UAT_PENDING` until approved. No `gate_bypass` level auto-approves it at either position — not
even `total` — because unlike Gates 1-2, `uat` has no `--actor auto` path
at all: `bee gate --name uat --actor auto` is refused outright
(uat-gate-before-merge D1). Its escape hatches are per-merge
(`--skip-uat`), a logged `uat-deferral` decision at close, or repo-wide
(`uat_stop: "off"`), never bypass.

**Reading older records.** bee had four gates until validation-diet D2
merged shape and execution into one call. The review gate was numbered 4
then and is numbered 3 now; the standalone execution gate that used to
hold the number 3 no longer exists. History written before the merge is
left as written — where this doctrine refers back to it, it says "the old
standalone **execution** gate" rather than a number, so no sentence ever
means two different gates at once.

## Gate Presentation Contract

A gate message has two layers, and **only the human layer goes into chat**:

1. **Human layer (the chat message)** — written in the language the user is conversing in, jargon-free, answering four questions in order:
   - **What I'm about to do** — one sentence in the user's terms: what changes *for them*, not the mechanism.
   - **Why it's trustworthy** — the single strongest piece of evidence in plain words ("a dry run rebuilt all 3 pages byte-for-byte identical"), never a checklist.
   - **If it goes wrong** — what breaks for the user and how it would be noticed (loud failure, rollback path).
   - **What you are deciding** — the exact commitment being approved and its boundary ("current slice only").

   Then the fixed gate question verbatim, with the standard options, and a link to the full report.

2. **Machine layer (the linked report)** — the full mechanical material (reality-gate tables, feasibility matrices, the hat wave's synthesized plan findings — the plan-checker the wave absorbed, "Hat wave" below — cell lists) is written to `docs/history/<feature>/reports/` and **linked** from the gate message. It is never pasted into the gate message. It exists for the agent, the audit trail, and grooming — not for the human's eyes at decision time.

Litmus test: **the user must be able to restate what they are approving in their own words.** A gate the user cannot restate is a dead gate. A technical term (BLOCKER count, spike id) may appear in the human layer only with an immediate plain-language gloss.

This contract applies to all three gates, in every mode, including go mode.

### AskUserQuestion — honor the tool's schema (a valid call, every time)

Gates, decisions, and confirm-before-doing prompts are presented with the `AskUserQuestion` tool. If the call violates the tool's schema the harness rejects the **whole** call with **"Invalid tool parameters"**. Build the call inside these limits:

- **`header` ≤ 12 characters** — it is a short chip label, NOT the question. Vietnamese/English descriptive headers ("Xử lý external", "Cách hiển thị") overflow instantly — use "Approach", "Scope", "External". **This is the #1 cause of the error.**
- **2–4 options per question** — never 1, never 5+. An "Other" free-text choice is added automatically, so fold overflow there or into a follow-up question.
- **1–4 questions per call** — batch independent questions (up to 4), serialize dependent ones.
- Every option needs both a **`label`** and a **`description`**; put the recommended option first with "(Recommended)" in its label.

A question that "needs" a long header or >4 options is a signal to reshape it — split it, or push detail into the option descriptions — never to exceed the schema.

### Gate bypass mode (opt-in autopilot)

Off by default. Set from `bee-hive`'s Gates section — on the user's instruction the agent writes `.bee/config.json` `gate_bypass` (persistent per-repo), logs the change as a decision, and states the chosen level's row in the same turn. When on at any level, the agent does **not** stop at a bypassed gate — it takes the RECOMMENDATION option itself and continues. This is the one deliberate exception to "gates are never self-approved" (rule: agents-gates-never-self-approved); **headless mode is not** — headless still stops at every gate.

**Scope of the switch (D2, traceable-runs).** `gate_bypass` decides only whether the run STOPS for the human at a gate; it never decides whether that gate's brief or approval record EXISTS. Every code-touching lane's brief (Lane ceremony table, `routing-and-contracts.md`) is written before the gate is reached regardless of bypass, and the approval record is always written at the gate — bypassed or not. What differs is the `actor` on that record: `"user"` when a human answered, `"auto"` plus the bypass level and reason when the agent auto-approved (below).

**`gate_bypass` is a level.** The config value normalizes to a level, and the level decides how far bypass reaches. Above `normal`, the human has said, in advance and explicitly, "when you have a recommended option I will always approve it — do not stop me; the result is what I care about." Honor that literally: at the chosen level, the recommended option IS the approval.

| Level | Config value | Auto-approves | Still stops for the human |
|---|---|---|---|
| `off` | `false` / absent | nothing — every gate stops | every gate (default) |
| `normal` | `true` / `"on"` / `"normal"` | Gates 1-2 for `tiny`/`small`/`standard` non-hard-gate work | high-risk/hard-gate Gates 1-2 · secret reads · Gate 3 UAT/P1 |
| `full` | `"full"` | **all** Gates 1-2 at every lane, high-risk/hard-gate included | secret-file reads · a review P1 finding |
| `total` | `"total"` | **everything** — all Gates 1-2 any lane, secret-file reads, Gate 3 UAT, review P1 findings | **nothing — zero stops** |

Legacy `true` maps to `normal`. The table's "Gate 3 UAT" cell names Gate 3's
own user-acceptance-testing checklist item inside a review session — a
different thing from the `uat` gate above, which sits at merge time and no
row of this table ever auto-approves, at any level.

At **Gate 1 or Gate 2** when the level bypasses that gate:

1. **Safety floor is level-scoped, not absolute.** Under `normal` the floor holds: a `high-risk` lane or any hard-gate flag (auth · authorization · data loss · audit/security · external provider · validation removal · database migration/schema change) is **NOT** bypassed — present it to the human normally. Under `full` and `total` the high-risk/hard-gate floor is **lifted** — the human lifted it by choosing the level — so those gates auto-approve too.
2. Do not ask. Instead: the brief (Lock's `CONTEXT.md`, full or short per lane — D1) is already written by this point; bypass never skips its authoring. Select the option the RECOMMENDATION favors and record it with `bee gate --merge --approved true --actor auto --bypass-level <level> --reason "<why>"` (or `--name <gate>` in place of `--merge` for a single gate) — the same write the human's "yes" would trigger, now stamping the record `actor: "auto"` plus `bypass_level` and `reason` instead of `actor: "user"`; still write the machine-layer report to `docs/history/<feature>/reports/`; log a one-line audit entry — `.bee/bin/bee decisions log --decision "auto-approved Gate N (bypass): <choice>" --rationale "<the recommendation's why>" --relation none` — so the approval is never silent; then post a **short chat line** (not a question) — `⚡ auto-approved Gate N (bypass): <what/why in one plain sentence>` — and continue. The human sees what happened and can still interrupt.

**Bypass suppresses approvals, never genuine information-gathering.** The point of the levels is to stop the agent asking merely to be *approved* — not to gag a real question. So distinguish two kinds of "question": an **approval** (the agent already has a confident best answer; the human would only rubber-stamp it) is suppressed under `full`/`total` — the agent takes its own answer and continues. An **information** question (the answer turns on a preference or knowledge only the human holds, and the agent cannot resolve it from evidence with a confident default) is still asked, even under `total`. This is where `bee-shaping`'s Socratic Explore step still stops when it must (its materiality test + the information-vs-approval refinement): the human asked to keep being consulted for real information, only never for a rubber stamp. Litmus: *"do I already have a confident best answer?"* — yes → proceed; no, and only the human can supply it → ask.

**Gate 3 and secret reads follow the level.** Under `normal` and `full`, Gate 3 is never fully bypassed and bypass never creates a review session: a review only exists once the user invoked `bee-reviewing`, its UAT items are always presented, and any P1 always stops. Under `total`, a review the user started runs to completion without stopping — UAT items and P1 findings auto-proceed on the recommended resolution. **Secret-file reads** stop for the human under `off`/`normal`/`full`; only `total` auto-proceeds on them (the human accepted that credential contents may enter context/logs unprompted). Bypass still never *creates* a review session on its own at any level.

The mechanical guards do not change: cell claiming and the write-guard still require `approved_gates.execution: true` — bypass simply means the agent records that approval itself for eligible work instead of waiting for the human. Bypass state is surfaced every session (the preamble and `bee_status` both print a loud level-specific `GATE BYPASS` banner — `NORMAL` / `FULL AUTOPILOT` / `TOTAL AUTOPILOT — ZERO STOPS`) so the active level is never silently in effect.

**The bypass is mechanized at runtime, not prose-only.** The rule above is still the assistant's to follow, and the runtime honors it too: the session-stop checkpoint hook emits a turn-control block that forces continuation when the assistant tries to stop mid-planning at a gate the active level covers and is still pending. What it prescribes is the **merged** approval — `bee gate --merge --approved true --actor auto --bypass-level <level> --reason "<why>"` — because Gate 2 is one gate over two fields: a net that set `execution` alone would leave the gate it just approved half open. For the same reason it treats Gate 2 as pending unless **both** `shape` and `execution` are already true, so a record granted through the standalone `--name` path is not a hole through the net. It is loop-guarded (blocks once per `sessionId:phase:gate:level`, then degrades to advisory) and excludes exploring/Gate 1 (genuine information questions still stop even under `total`).

### Headless mode (never ask; defer into Outstanding Questions)

With `mode:headless`: never ask blocking questions. Perform onboarding checks and routing only when
unambiguous; defer every ambiguity (stale onboarding needing `--apply`, HANDOFF present, unclear
route) into an `Outstanding Questions` section of a structured terminal report. The three gates are
NEVER self-approved in headless mode (rule: agents-gates-never-self-approved) — the only mechanism
that self-approves gates is the explicit
opt-in gate-bypass switch above, and how far it reaches is its level (`normal` = normal-lane only;
`full` = also high-risk/hard-gate; `total` = everything incl. UAT/secrets). Headless and bypass are
independent: headless without bypass still stops at every gate. Go mode's own headless behaviour is
in `references/go-mode.md` ("Headless Go Mode").

### Green base check (before the first claim)

**Before your first `cells claim`, never on arrival.** Not one of the three gates: the trigger is the *claim*, so a session that claims no cell owes no check. This is base-awareness, not a mandatory full-suite run — the agent owns test scope here too: know the state of the tree before building on it. Read the last recorded proof first — a recent cap's proof line, or `.bee/logs/test-results.json` if one exists — and when the area you're about to touch has no recent record, a scoped run of the tests that cover it answers the question; a full-suite run is never owed just to claim. A known-red base is surfaced to the user and becomes its own fix-first tiny cell (rule: agents-never-build-on-red).

Reading the record yourself is the normal case: a fresh proof line or `.bee/logs/test-results.json` answers about *your* tree already, scoped to what someone actually ran, and CI runs the project's full declared `commands.test` on push and pull_request regardless. Read CI instead (`gh run list`/`gh api` for the base branch, plus any open `verify-red` issue) when no recent local record exists — CI's answer is about the base branch as of its last run, so it is evidence about your tree only while nothing has changed under you. When no command is recorded, `bee status` warns and the capture belongs to exploring or onboarding, never to guesswork.

### Delegation contract (fan-out: decide-altitude vs gather-altitude)

The one orchestration pattern bee runs: the session model (the owner's best model) stays the orchestrator in every phase, and mechanical gather/render/mine steps dispatch down-tier as I/O workers that return digests.

- **Decide-altitude stays on the session model**: gates, Socratic questions, the mode gate, synthesis of findings, accept/reject of worker results, state writes, human conversation.
- **Delegation rubric** — a mechanical step delegates down-tier when its content is needed as a digest, not verbatim (rule: doctrine-layer-delegation-threshold); the orchestrator may override either way at dispatch. Prose-ruled — no hook enforces the threshold.
- **Lane rule** — the rubric applies in every lane and every phase, tiny/small included. The "0 subagents" rule for tiny/small caps *ceremony* subagents only (reviewers/checkers/panels) — it never speaks about execution workers (rule: agents-never-zero-execution-workers), and I/O workers are exempt from both counts. A 1-file tiny fix never crosses the rubric, so it stays inline naturally.
- **Digest contract** — an I/O worker returns paths read, the facts extracted (with file:line anchors), and verbatim quotes only where asked; the orchestrator never re-reads what a digest already answers.
- **Transport** — the door is `.bee/bin/bee dispatch prepare --runtime <rt> --kind <cell|gather|reviewer|advisor> [--role <name>] --json`: `prepare` reads the ROLE slot out of `.bee/config.json` and returns the tool plus the payload. `--role <name>` names the JOB this dispatch is and overrides the slot the kind would have resolved — `--kind gather --role extraction` is how a read-only gather reaches the cheap reader and gets `bee-extract`. Any name `models.<runtime>` carries is legal (bee holds no fixed list); a `--role` naming a name nothing configures is refused by name with a FIX, never resolved onto some other consumer's model — a model-shaped slot returns an Agent/Task payload naming the rendered bee agent, a `{kind:"herding"}` slot returns a Bash `bee herding run` payload, a `{kind:"cli"}` slot returns a Bash external-executor payload. `subagent_type: bee-build|bee-gather|bee-extract|bee-review` survives in the bullet only as what `prepare` RETURNS for a model-shaped slot, and as a spelling the guard still accepts (D2) — each rendered agent file declares the ordered ROLE list it serves and pins whatever model that list resolves to, so naming the agent declares the role and needs nothing else. Or an anchored `[bee-tier: <role>]` marker (the marker keeps its historical spelling and carries a role name), or a `model` param. A marker naming any role this runtime configures — plus `ceiling`, the escalation word for a session-model run — is a declaration; a marker naming a name nothing configures is DENIED by name (`role-not-configured`) with the configured roles in its FIX, never read as plain text. Where two of them disagree — a marker plus `subagent_type: general-purpose`, or a marker plus a mismatched `model` — the guard rewrites the request to config rather than refusing it, and says so in one line; you do not re-issue the dispatch. Plus one work-language intent sentence of what the worker will find/build/check plus the model name in the Agent description (a description that is only a model name or a codename is a red flag), background dispatch where the runtime supports it, the dispatch log as the audit trail. I/O workers do **not** register in `bee state worker add` — the registry stays swarm-cell-scoped (reservations/status are execution concerns); the dispatch log is the audit surface for gathers.
- **Execution worker (second named class)** — the Delegation contract's other dispatch shape, distinguished from the I/O-offload worker by **authority and state effects**, not by task size. Unlike an I/O worker, an execution worker **does** register in the swarm registry (`bee state worker add`) and **does** take reservations under its own nickname; it implements exactly one assigned cell (claim → read `read_first` → implement within `files` → commit → finish, which caps the cell, releases the reservations, and records the required proof line, checked — never re-run — by `bee close`/`bee worktree merge`) and returns exactly one status token (`[DONE]`/`[BLOCKED]`/`[HANDOFF]`/`[NOOP]`) — it is authority-bearing, never a digest-only gather. Every `bee-swarming` worker dispatch belongs to this class: full waves in `standard`/`high-risk`, and the single dispatched worker that carries out `small` cell implementation (`bee-swarming/references/swarming-reference.md` ("Single execution worker in full")) (rule: agents-never-zero-execution-workers); `tiny` may execute inline in the orchestrator session instead, and when a tiny cell IS dispatched it belongs to this class too. **Parallel by default:** a `small` lane's 1-3 cells fan out to concurrent execution workers whenever every cell's product file set is disjoint — reservations are the proof and the police, 3-4 live workers is the cap; serial requires a named conflict recorded in the dispatch note (worker returns and its done-report lands before the conflicting next cell is claimed/dispatched) — never assumed as the default. **Parallel criterion:** cells run in parallel whenever every cell's *product* file set is provably disjoint; a cell's regen targets (release manifest, onboarding ledger, plugin mirrors) drop out of that comparison when it carries `regen_obligation_ack: "wave-barrier"` (the orchestrator then owes the full regen chain once, at wave close); any *actually shared* product file still forces serial — in doubt, serial. An independent reviewer or checker (a hat seat of the plan-step wave that absorbed the plan-checker, a cell reviewer, a panel member) is **neither** class: it is a review-class dispatch — read-only, no registry entry, no reservations, no cell of its own — and is never called an "execution worker."
- **cli gather branch** — when the resolved gather role is a `cli` type, a gather dispatch runs the configured command **verbatim** via the shell — nothing appended, ever; the prompt goes in on **stdin**; every path handed to the worker is **absolute**; the run is **read-only** by contract. **Stdout IS the digest**, framed by a delimiter contract: the worker prompt instructs the CLI to emit its digest between `<<<BEE_DIGEST` and `BEE_DIGEST>>>` lines, and the orchestrator extracts only what sits between them — missing delimiters or an empty digest is a **failed run**, surfaced loudly, never accepted as a silent green. No `result.json`, no cell, no reservation, no `bee state worker add` registration for a gather, same as any other I/O worker. **Known measurement gap, named not solved here:** a Bash-launched gather emits zero `dispatch.jsonl` rows — closing that gap is Slice 3's job, not this branch's.
- **herding execution branch** — `bee herding run` is the cell-execution mirror of the cli gather
  branch above: same shape (a foreign, bee-ignorant CLI agent is the worker), opposite purpose (write
  work through a cell, never a gather). It starts one herdr-supported agent in a fresh pane instead of
  a one-shot stdin→stdout process, hands it a fully self-contained brief over a file mailbox
  (`.bee/mailbox/<job-id>/`) instead of piping the prompt on stdin, and reads back a written
  `result-N.json` instead of framed stdout — the mailbox's result file is this branch's delimiter
  contract, the completion signal in place of `<<<BEE_DIGEST`/`BEE_DIGEST>>>` (herding-executor D1,
  D3). `herding-tier D1-D6` (widened by `herding-review-slots D1`) routes every purpose against a
  `{kind:"herding"}` slot — cell, gather, reviewer, advisor, extraction alike — through `bee herding run`;
  `bee dispatch prepare --kind gather` against such a slot returns this branch's herding-exec Bash payload
  today, superseding the `cli`-mirrored cell-execution-only boundary this branch used to draw
  (`herding-executor D7`, now superseded by `herding-tier D1-D6`). The worker itself stays bee-ignorant; ALL bee
  bookkeeping this branch owes — `cells finish`, the proof line, reservations, the dispatch-log row —
  is done by the orchestrator after it reads the result back, exactly the D4 split gather workers
  never needed in the first place (`bee-herding/references/operational-invariants.md`, "`bee
  herding run` — one foreign agent as a cell-execution worker"). prepare's transport_ready is the reachability fact; the fallback model applies only when it is false — never guess from channel.

### Blind lanes and convergence

Two or three isolated advisor consults design an answer to ONE hard question, critique each other, and converge into one document plus one decision entry. **This section is the single home for the blind-lane PROCEDURE** — when lanes open, the four moves, the rule that binds the checker, and the three named limits. Every other surface (AGENTS.md, `bee blind check`'s own help, the `advisor-protocol` knowledge concept) carries a one-line pointer back here, never a second copy.

**When lanes open.** The agent opens 2-3 lanes on its OWN judgment when a decision is both high-stakes AND ambiguous, and logs the reason at open time with `bee decisions log` — there is no approve-each-lane wait. The user may order lanes directly at any point. A convergence that produces no chosen answer hands the human the dossier unchanged — `bee state waiting-on set --kind question` when attended, `bee cells block --id <id> --reason <why>` when unattended, which is the one producer of a letter's "Needs your call" item — and never resolves itself by coin flip (slp-blind-lanes D1, D2(e)).

**Lanes are not hats.** Lanes GENERATE designs from one byte-identical brief; hats CRITIQUE one request from fixed disjoint perspectives. Different purpose, so neither replaces the other, and a hat wave is never reported as a lane run (slp-blind-lanes D7).

**The four moves** — the shape `bee-reviewing`'s wave already uses, applied to generation instead of critique:

1. **Fan out.** One `bee dispatch prepare --runtime <rt> --kind advisor --role lane-N --brief-file <path>` per lane, in parallel — lane one takes `--role lane-1`, lane two `--role lane-2`, lane three `--role lane-3`, so each lane can be pointed at a DIFFERENT model (lane-model-diversity D1). Each lane gets the SAME brief bytes and the read diet that brief declares; it is denied every sibling proposal, the orchestrator's own leaning, session history, and `--expertise` beside a brief (a second, unlinted reading channel is refused at the door). A lane never runs as `--kind cell` — refused by type, before the file is read (D3).
2. **Cross-critique.** Round two: fresh advisor dispatches, each handed the rival proposal VERBATIM inside a fence whose info string is the one tag `lane-proposal`. A brief over the 8192-byte cap does not paste the proposal: the round-2 brief names its PATH in the read diet and the lane reads it there.
3. **Converge.** One dossier at `docs/history/<feature>/blind/<run-id>.md`, holding every proposal verbatim, the critiques with their round-2 dispatch ids, the chosen answer, the rejected set with reasons, and the citations.
4. **Record.** `bee decisions log --rejected "<what>: <why>" --trigger <id>` — the rejected set is a list on the record, and the revisit condition is a registered `bee triggers` id, never a memory.

**The lane seat roles are ordinary `models.<runtime>` roles.** `lane-1`, `lane-2` and `lane-3` sit in the one model table beside `code`, `review` and `advisor` — no separate config file (lane-model-diversity D1). The constant of record is `SEAT_ROLES` in `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs`: eight names, three lanes and five hats, closed on purpose. A seat whose slot resolves NOTHING — key absent, `null`, or a shape the resolver reads as nothing — falls through to `advisor` on an advisor-kind dispatch instead of refusing, so lanes run unconfigured exactly as they always did; the `[bee-tier: …]` marker names the RESOLVED role (`advisor` after a fall-through), and the dispatch log keeps the asked-for seat as `requested_role` (D2, D4). A name outside those eight keeps its ordinary refusal, so a typo never quietly borrows the advisor's model. No dispatch-time model flag exists — the model comes only from the table.

**Convergence RUNS `bee blind check --dossier <path>` green BEFORE it logs the decision.** No door forces that — the verb is one a caller must choose to run, so this sentence is the whole enforcement. Its digest check resolves each lane's `dispatch_id` against `.bee/logs/dispatch.jsonl`, so the check runs AT THE ROOT whose log holds the run: a run dispatched from a worktree checks in that worktree. The shipped example (`docs/history/slp-blind-lanes/blind/example-run.md`) refuses in the main checkout for exactly that reason — its dispatch ids never ran there — and that refusal is the rule working, not a broken example.

**Three limits, said plainly rather than buried:**

- **Provenance, never faithfulness.** A resolved citation proves the quoted span is a whole sentence of the named lane's own bytes, and nothing more. A quote whose meaning is governed by the sentence BEFORE it still resolves and passes (decision `79b5437b`).
- **Round two sits outside the evidence chain.** Round-2 briefs differ per lane by construction, so the brief-digest check, the recorded-brief re-lint and the citation check all cover round ONE only. The dossier's `## Cross-critiques` section therefore CARRIES the round-2 dispatch ids in its text, and nothing mechanically checks them — round two is audited by a reader against that log, or not at all.
- **The tagged fence is a claim, not a proof.** `lane-proposal` matches as one exact token, trimmed and ASCII-case-folded; anyone can type it, and nothing checks that the fenced bytes came from another lane. A forged tag is a named lie inside a recorded brief, settled by evidence at convergence — the same trust posture the citation check already takes.

**Pushback counts only when it names the specific missing context.** An objection with no named gap does not stand — in a lane, in a cross-critique, or at convergence (slp-blind-lanes D5).

### Hat wave — fixed perspectives critique one draft

Lanes GENERATE designs; hats CRITIQUE one existing draft from fixed disjoint
perspectives (slp-blind-lanes D7). **This section is the single home for the
hat-wave PROCEDURE** (decision `07328333`); bee-shaping carries the one-line
trigger pointers, never a second copy.

**Two windows, two jobs — one procedure.** The **plan-step wave** is the
default firing point: the leader opens it itself once shaping has clarified
the spec, and the wave BUILDS the implementation plan. The **pre-Lock
spec-critique window** stays discretionary and CRITIQUES a drafted big spec
before Lock. Neither absorbs the other, and neither is retired
(proactive-leader-intake D5, decision `98ac20a1`).

#### The plan-step wave (the default firing point)

**When hats open.** At the PLAN step, never at raw intake. The leader
clarifies the spec first (interview/scout as today); once the spec is clear
enough, the leader proactively opens the wave to build the implementation
plan, and the clarified spec is the draft the hats anchor on
(proactive-leader-intake D1, decision `a52c854d`, superseding `8fb1e0da`).
Synthesized answers feed plan.md THROUGH the leader — synthesis is
decide-altitude and never delegates, and a hat finding never lands in Lock
directly (Lock renders, it never originates).

**Threshold.** Big, vague, or high-risk work gets the wave; a clear or tiny
ask keeps today's fast path with NO wave. The unit is once per FEATURE, never
per message — five dispatches on a typo fix is the named ceremony-capture
failure (D2, `a52c854d`). The agent logs the open reason with
`bee decisions log`, the same D1 posture lanes hold.

**Seats.** Three by default — `hat-facts-gaps`, `hat-alternatives`,
`hat-user-impact`. All five seats on high-risk work (D3, `423e1664`).

**The seats at plan altitude, one instrument each, by POINTER to its home —
never copied:**

| Hat | Role | Asks at the plan step | Instrument |
|---|---|---|---|
| facts-gaps | `hat-facts-gaps` | what the drafted plan cannot answer | 5-Layer rubric + Truth Table Test over the plan, plus the claims-table audit — open each anchor, confirm the quoted bytes, sweep the prose for a load-bearing claim that never became a row; mismatch = BLOCKER (`.bee/expertise/review.md` ("Claims-table audit") is the home — CITED, never copied) |
| alternatives | `hat-alternatives` | is there a cheaper shape for this plan | bee-planning's inline SMALLER PATH mandate, read at plan altitude — CITED, never copied (`bee-planning/SKILL.md` is the one home); several viable designs on a high-stakes ambiguous choice hands off to blind lanes |
| user-impact | `hat-user-impact` | what the user sees and feels in the planned behavior | gray-area probes over the planned behavior + the SEE mock (bee-shaping references) |
| risks *(5-seat only)* | `hat-risks` | what breaks, and can it be undone | CRUD Lifecycle check — the delete half is the reversibility interrogation (`.bee/expertise/review.md`) |
| value *(5-seat only)* | `hat-value` | is this worth its cost | materiality test (bee-shaping's shaping-reference) |

**Absorption — the wave IS the internal consult.** The planning review wave
(the plan-checker) and the high-risk advisor gate consult RUN AS this hat wave
from now on (D4, decision `b34fdea9`). Two consult surfaces collapse into one.
The wave's synthesis is recorded with `bee state advisor-ref record --advisor
<identity> --digest-file <path>`, which satisfies the existing high-risk Gate 2
precondition unchanged — no code edit. **Timing law:** record the ref AFTER
plan.md has reached its gate-ready bytes and AFTER the last pre-gate
`bee decisions log` write. The verb stamps its own staleness anchors — the
active feature, the newest active decision id, and the sha256 of that feature's
plan.md (`verbs/state_group/advisor_ref.rs:166-180`) — so a ref recorded at
wave time goes stale by construction and refuses the very gate it was meant to
open. **bee-reviewing and Gate 3 are untouched by this absorption**: the
independent review stays user-invoked (rule: agents-review-user-invoked).

**Mandate ownership (the absorbed plan-checker's two vocabularies).** The two
vocabularies never merge. MANDATE 1 **Structure** — BLOCKER/WARNING over the
five structure dimensions — rides `hat-facts-gaps`' plan-step question.
MANDATE 2 **cold-pickup** — CRITICAL/MINOR — stays with the LEADER at cell
drafting, the same self-check the `tiny`/`small` lanes already run.

**Budget.** One wave, wall-clock ceiling 10 minutes. A seat that misses the
ceiling is DROPPED and named in the synthesis record; a partial return
synthesizes what came back and never blocks the gate on a missing seat.

**Quorum.** No hard quorum — the wave runs with whatever seats resolve.
All-fall-through (zero diversity) and every dropped seat are named in the
record; `bee doctor`'s hat advisory stays the config nag.

**Idempotence.** The recorded advisor-ref IS the once-per-feature mark. A live
(non-stale) ref means the wave already ran, so a resumed or compacted session
never re-runs on it; a ref gone stale after a material plan change permits
exactly one re-run.

**Gate bypass `full`/`total`.** The wave still runs — it is an internal
consult, not a gate. Its questions are RECORDED as the plan's Open Questions
exactly as headless records them, the recommended option proceeds, and nothing
new stops; the always-stop information-question law at exploring/Gate 1 is
untouched.

**Headless.** An unattended wave never blocks and never self-answers. Its
questions land as the plan's open questions (approach.md "Questions still
open"), and the interview is never simulated (D6, decision `f73d6c49`).

**Communication.** While the wave runs the user sees ONE plain state line, with
no hat vocabulary. The output reaches the user as ONE leader voice, and every
finding is filtered against the request text before anything surfaces (D7).

#### The pre-Lock spec-critique window (discretionary, kept)

**When it opens.** Inside bee-shaping, AFTER the spec content is drafted
and BEFORE Lock — the only window where a product-altitude finding still has
a channel back into the draft (post-Lock it can only become a supersession).
Discretionary, reserved for big, hard-to-reverse specs; the agent logs the
open reason with `bee decisions log`, the same D1 posture lanes hold. A
small spec gets no wave — ceremony capture is the named failure.

**Same seats, SPEC altitude.** The five hats run with their instruments read
against the spec draft: facts-gaps asks what the spec cannot answer,
alternatives runs the SMALLER PATH question at spec altitude, user-impact runs
gray-area probes plus the SEE mock, risks runs the CRUD Lifecycle check, value
runs the materiality test. Accepted findings route back through the interview
as questions or Open Questions in the draft. Plan-altitude hats cannot critique
a spec draft — that is why this window survives the absorption above.

#### The moves and the seat roles (both windows)

**The moves.** Parallel `bee dispatch prepare --kind advisor --role <hat-role>`
dispatches, one hat each, each naming its own role from the table above; the
perspective rides the PROMPT body — never
`--brief-file`, whose neutrality lint exists for lane briefs and would fight
a brief whose leaning IS its job. Hats never see each other. The
orchestrator is the synthesizing BLUE hat — synthesis is decide-altitude and
never delegates.

**Prompt framing — open questions, not validation.** A hat prompt presents
the PROBLEM and CONSTRAINTS, never the leader's draft solution. Frame each
dispatch as an open question that invites the hat's own exploration:

| Validation-seeking (avoid) | Open-ended (use) |
|---|---|
| "Does this plan look right?" | "Given [context], what approaches would you explore?" |
| "Any concerns with X?" | "What risks do you see in this space?" |
| "I'm thinking Y because Z — agree?" | "Here's the constraint. What would you check before picking a direction?" |

The hat sees the spec, the constraints, and enough context to form its own
view — never the leader's preferred answer. Advisors who react to a proposal
circle the leader's framing; advisors who explore from the problem bring back
what the leader could not see.

**The hat seat roles carry a description.** The five `hat-*` names are
`models.<runtime>` roles like any other, and the constant of record is the same
`SEAT_ROLES` in `packages/bee-rs/crates/bee/src/verbs/drivers/models.rs` the
lanes read from. Two rules differ from a lane seat. First, a CONFIGURED hat slot
states its purpose in a `description` field, so the config reads back
self-documenting (lane-model-diversity D3) — `bee doctor` reports a configured
hat that carries none as an advisory, never as a verdict, because the field is
planner documentation and never selects a model. A `null` hat is a seat switched
off, not an undescribed one, and is passed over. Second, everything else matches
the lanes: an unconfigured hat falls through to `advisor` on an advisor-kind
dispatch, the marker names the resolved role, and a name outside the eight keeps
its ordinary refusal (D2, D4).

**No checker verb.** Unlike blind convergence, nothing here autonomously
logs a decision — the HUMAN is the check, and which human moment differs by
window: the plan-step wave's check is **Gate 2** (the merged shape+execution
gate the plan is built for), and the pre-Lock wave's check stays **Lock**. Same
checker-less posture bee-reviewing's critique wave already holds.

### Judgment contract — rails for workers, boundaries for the orchestrator

Rules bind differently by rule kind and by role.

**Three rule kinds:**

1. **Boundary rules** hold as written, for every role, at every bypass level
   that does not explicitly lift them: gate-before-source, declared tests
   green at the boundary (`bee close`/`bee worktree merge`), CLI-only state
   mutation, reservations/holds, secret handling. These constrain
   OUTCOMES. They are never "form".
2. **Form rules** constrain the PATH between boundaries: step order, line
   shapes, templates, tick phrasing, report structure. For a cold dispatched
   worker they are rails — followed as written, deviation only through the
   worker's own Deviation Rules. For the orchestrator they are DEFAULTS:
   when a form rule's letter stops serving its purpose in the situation at
   hand, the orchestrator says so in one line and deviates with a recorded
   reason (a decision-log line or a deviation note in the relevant trace).
   Silent deviation is the defect; named deviation is the system working.
3. **Environment-conditioned rules** presuppose a fact about the world — a
   CI, a git history, a runnable regen chain, a GitHub remote. Such a rule
   CHECKS its precondition first; absent, it names the gap and takes its
   recorded fallback (the ack field, the sentinel value, the documented
   downgrade) — it never demands its ritual in an environment that cannot
   satisfy it, and needing to "work around" such a rule is a signal the rule
   is missing its precondition check, worth a friction entry.

**What this never licenses:** skipping a gate, capping without the proof
path, hand-editing state, writing through a reservation, reading secrets
unprompted, or silencing a red. Judgment widens the path, never the
boundary.

### Goal-check judge tier — verification, not review

The swarming goal-check has a **semantic** judge tier by lane, layered on the frozen judge (`bee cells judge`, undeclared-file check) — this is verification of a capped cell, never the user-invoked review session (Gate 3 and the candidates ledger are untouched by every row below).

| Lane | Judge | Model | Verdict handling |
|---|---|---|---|
| `tiny` / `small` | mechanical only (frozen judge) | — | — |
| `standard` | SELECTIVE: the per-slice checklist judge — a pinned `bee-review` dispatch, the `review` role, read-only, covering every capped `behavior_change` cell of the slice in one dispatch — dispatches when ANY of: the goal-check smells, the slice contains a worker's (or model's) first cells of the feature, or the ~1-in-3 sample falls on it (state the sample choice in the slice-close tick; never silently skip). ESCALATION: any `NEEDS_REVISION` puts that worker's remaining slices on judge-every-slice for the rest of the feature. Unjudged slices still pass the frozen judge per cell — that stays universal and free | `review` role config | per judged cell, each verdict recorded via `cells judge-record`: `PASS` → counts; `NEEDS_REVISION` + `automatic` → cell NOT done, re-dispatch with the exact failing checks + a ledger entry; `NEEDS_REVISION` + `authority` → escalate to the user |
| `high-risk` | same checklist judge as `standard` | independence preferred — model differs from the builder's resolved model; if equal, record `model_independence: "same-model"` honestly and the judge still runs | same verdict handling as `standard` |

The judge returns the `judge-verdict/1` schema, recorded via `bee cells judge-record`; free-prose output is a failed judge run, re-dispatched once, then recorded `unverified`. This table is the single home for the judge-tier rule — every other surface (bee-swarming SKILL + reference, bee-hive SKILL, go-mode, AGENTS.md + its template, bee-capturing SKILL) carries only a one-line pointer back here, never a repeated table.

### Test scope (agent-owned, proof-per-change-type)

The agent owns test scope end to end (rule: agents-proof-at-cap) — mechanism in `bee-swarming/references/swarming-reference.md` ("Proof at finish and close, in full").

**Suite rent.** A suite is not immortal: every guard suite pays rent by catching real defects. A suite that has not caught one in ~6 months is a demotion candidate — moved out of the local/impacted hot path to the CI/nightly tier by a RECORDED decision (never a silent delete; the suite still runs, just not on every developer loop). `bee-grooming` owns the audit: read the verify logs for which suites have gone red for a real defect (environment reds don't count as rent paid), list the never-fired tenants, and propose demotions. Institutional/meta guards (fences, parity checks, doctrine gates) are the usual tenants — product-behavior suites earn rent more often and mostly stay.


