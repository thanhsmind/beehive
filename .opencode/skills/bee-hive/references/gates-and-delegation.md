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
invoked, and never automatically at the end of any lane.

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

2. **Machine layer (the linked report)** — the full mechanical material (reality-gate tables, feasibility matrices, plan-checker findings, cell lists) is written to `docs/history/<feature>/reports/` and **linked** from the gate message. It is never pasted into the gate message. It exists for the agent, the audit trail, and grooming — not for the human's eyes at decision time.

Litmus test: **the user must be able to restate what they are approving in their own words.** A gate the user cannot restate is a dead gate — worse than no gate, because it manufactures false confidence. A technical term (BLOCKER count, spike id) may appear in the human layer only with an immediate plain-language gloss.

This contract applies to all three gates, in every mode, including go mode.

### AskUserQuestion — honor the tool's schema (a valid call, every time)

Gates, decisions, and confirm-before-doing prompts are presented with the `AskUserQuestion` tool. If the call violates the tool's schema the harness rejects the **whole** call with **"Invalid tool parameters"** — a recurring, silent waste (the model then retries a valid one). Build the call inside these limits:

- **`header` ≤ 12 characters** — it is a short chip label, NOT the question. Vietnamese/English descriptive headers ("Xử lý external", "Cách hiển thị") overflow instantly — use "Approach", "Scope", "External". **This is the #1 cause of the error.**
- **2–4 options per question** — never 1, never 5+. An "Other" free-text choice is added automatically, so fold overflow there or into a follow-up question.
- **1–4 questions per call** — batch independent questions (up to 4), serialize dependent ones.
- Every option needs both a **`label`** and a **`description`**; put the recommended option first with "(Recommended)" in its label.

A question that "needs" a long header or >4 options is a signal to reshape it — split it, or push detail into the option descriptions — never to exceed the schema.

### Gate bypass mode (opt-in autopilot)

Off by default. Set from `bee-hive`'s Gates section — on the user's instruction the agent writes `.bee/config.json` `gate_bypass` (persistent per-repo), logs the change as a decision, and states the chosen level's row in the same turn. When on at any level, the agent does **not** stop at a bypassed gate — it takes the RECOMMENDATION option itself and continues. This is the one deliberate exception to "gates are never self-approved"; **headless mode is not** — headless still stops at every gate.

**`gate_bypass` is a level.** The config value normalizes to a level, and the level decides how far bypass reaches. The whole point of the levels above `normal` is that the human said, in advance and explicitly, "when you have a recommended option I will always approve it — do not stop me; the result is what I care about." Honor that literally: at the chosen level, the recommended option IS the approval.

| Level | Config value | Auto-approves | Still stops for the human |
|---|---|---|---|
| `off` | `false` / absent | nothing — every gate stops | every gate (default) |
| `normal` | `true` / `"on"` / `"normal"` | Gates 1-2 for `tiny`/`small`/`standard` non-hard-gate work | high-risk/hard-gate Gates 1-2 · secret reads · Gate 3 UAT/P1 |
| `full` | `"full"` | **all** Gates 1-2 at every lane, high-risk/hard-gate included | secret-file reads · a review P1 finding |
| `total` | `"total"` | **everything** — all Gates 1-2 any lane, secret-file reads, Gate 3 UAT, review P1 findings | **nothing — zero stops** |

Legacy `true` maps to `normal`. At **Gate 1 or Gate 2** when the level bypasses that gate:

1. **Safety floor is level-scoped, not absolute.** Under `normal` the floor holds: a `high-risk` lane or any hard-gate flag (auth · authorization · data loss · audit/security · external provider · validation removal · database migration/schema change) is **NOT** bypassed — present it to the human normally. Under `full` and `total` the high-risk/hard-gate floor is **lifted** — the human lifted it by choosing the level — so those gates auto-approve too.
2. Do not ask. Instead: select the option the RECOMMENDATION favors; set `approved_gates.<gate>` in `.bee/state.json` (same write the human's "yes" would trigger); still write the machine-layer report to `docs/history/<feature>/reports/`; log a one-line audit entry — `.bee/bin/bee decisions log --decision "auto-approved Gate N (bypass): <choice>" --rationale "<the recommendation's why>"` — so the approval is never silent; then post a **short chat line** (not a question) — `⚡ auto-approved Gate N (bypass): <what/why in one plain sentence>` — and continue. The human sees what happened and can still interrupt.

**Bypass suppresses approvals, never genuine information-gathering.** The point of the levels is to stop the agent asking merely to be *approved* — not to gag a real question. So distinguish two kinds of "question": an **approval** (the agent already has a confident best answer; the human would only rubber-stamp it) is suppressed under `full`/`total` — the agent takes its own answer and continues. An **information** question (the answer turns on a preference or knowledge only the human holds, and the agent cannot resolve it from evidence with a confident default) is still asked, even under `total`. This is where `bee-shaping`'s Socratic Explore step still stops when it must (its materiality test + the information-vs-approval refinement): the human asked to keep being consulted for real information, only never for a rubber stamp. Litmus: *"do I already have a confident best answer?"* — yes → proceed; no, and only the human can supply it → ask.

**Gate 3 and secret reads follow the level.** Under `normal` and `full`, Gate 3 is never fully bypassed and bypass never creates a review session: a review only exists once the user invoked `bee-reviewing`, its UAT items are always presented, and any P1 always stops. Under `total`, a review the user started runs to completion without stopping — UAT items and P1 findings auto-proceed on the recommended resolution. **Secret-file reads** stop for the human under `off`/`normal`/`full`; only `total` auto-proceeds on them (the human accepted that credential contents may enter context/logs unprompted). Bypass still never *creates* a review session on its own at any level.

The mechanical guards do not change: cell claiming and the write-guard still require `approved_gates.execution: true` — bypass simply means the agent records that approval itself for eligible work instead of waiting for the human. Bypass state is surfaced every session (the preamble and `bee_status` both print a loud level-specific `GATE BYPASS` banner — `NORMAL` / `FULL AUTOPILOT` / `TOTAL AUTOPILOT — ZERO STOPS`) so the active level is never silently in effect.

**The bypass is mechanized at runtime, not prose-only.** The rule above is still the assistant's to follow, and the runtime honors it too: the session-stop checkpoint hook emits a turn-control block that forces continuation when the assistant tries to stop mid-planning at a gate the active level covers and is still pending. What it prescribes is the **merged** approval — `bee gate --merge --approved true` — because Gate 2 is one gate over two fields: a net that set `execution` alone would leave the gate it just approved half open. For the same reason it treats Gate 2 as pending unless **both** `shape` and `execution` are already true, so a record granted through the standalone `--name` path is not a hole through the net. It is loop-guarded (blocks once per `sessionId:phase:gate:level`, then degrades to advisory) and excludes exploring/Gate 1 (genuine information questions still stop even under `total`).

### Headless mode (never ask; defer into Outstanding Questions)

With `mode:headless`: never ask blocking questions. Perform onboarding checks and routing only when
unambiguous; defer every ambiguity (stale onboarding needing `--apply`, HANDOFF present, unclear
route) into an `Outstanding Questions` section of a structured terminal report. The three gates are
NEVER self-approved in headless mode — the only mechanism that self-approves gates is the explicit
opt-in gate-bypass switch above, and how far it reaches is its level (`normal` = normal-lane only;
`full` = also high-risk/hard-gate; `total` = everything incl. UAT/secrets). Headless and bypass are
independent: headless without bypass still stops at every gate. Go mode's own headless behaviour is
in `references/go-mode.md` ("Headless Go Mode").

### Green base check (before the first claim)

**Before your first `cells claim`, never on arrival.** Not one of the three gates: the trigger is the *claim*, so a session that claims no cell owes no check. If `.bee/config.json` records `commands.test`, establish a green base — **never build on red**, and a red is surfaced to the user and becomes its own fix-first tiny cell.

Running it yourself is the normal case: `commands.test` answers about *your* tree, it is the same command every other door runs, and it is what CI runs on push and pull_request. Read CI instead (`gh run list`/`gh api` for the base branch, plus any open `verify-red` issue) only when the chain is genuinely long — CI's answer is about the base branch as of its last run, so it is evidence about your tree only while nothing has changed under you. When no command is recorded, `bee status` warns and the capture belongs to exploring or onboarding, never to guesswork.

### Delegation contract (fan-out: decide-altitude vs gather-altitude)

The one orchestration pattern bee runs: the session model (the owner's best model) stays the orchestrator in every phase, and mechanical gather/render/mine steps dispatch down-tier as I/O workers that return digests.

- **Decide-altitude stays on the session model**: gates, Socratic questions, the mode gate, synthesis of findings, accept/reject of worker results, state writes, human conversation.
- **Delegation rubric** — a mechanical step delegates down-tier when it needs reading >3 files OR content the main model only needs as a digest, not verbatim; the orchestrator may override either way at dispatch. Prose-ruled — no hook enforces the threshold.
- **Lane rule** — the rubric applies in every lane and every phase, tiny/small included. The "0 subagents" rule for tiny/small means zero *ceremony* subagents (reviewers/checkers/panels); I/O workers are exempt. A 1-file tiny fix never crosses the rubric, so it stays inline naturally.
- **Digest contract** — an I/O worker returns paths read, the facts extracted (with file:line anchors), and verbatim quotes only where asked; the orchestrator never re-reads what a digest already answers.
- **Transport** — `subagent_type: bee-build|bee-gather|bee-extract|bee-review` (the rendered agent file already IS its tier — generation/extraction/review — so naming it declares the tier and needs nothing else; prefer this shape), or an anchored `[bee-tier: <tier>]` marker, or a `model` param. Only `ceiling|generation|extraction|review` count as tier words; anything else in that marker reads as plain text, not a declaration. Where two of them disagree — a marker plus `subagent_type: general-purpose`, or a marker plus a mismatched `model` — the guard rewrites the request to config rather than refusing it, and says so in one line; you do not re-issue the dispatch. Plus one work-language intent sentence of what the worker will find/build/check plus the model name in the Agent description (a description that is only a model name or a codename is a red flag), background dispatch where the runtime supports it, the dispatch log as the audit trail. I/O workers do **not** register in `bee state worker add` — the registry stays swarm-cell-scoped (reservations/status are execution concerns); the dispatch log is the audit surface for gathers.
- **Execution worker (second named class)** — the Delegation contract's other dispatch shape, distinguished from the I/O-offload worker by **authority and state effects**, not by task size. Unlike an I/O worker, an execution worker **does** register in the swarm registry (`bee state worker add`) and **does** take reservations under its own nickname; it implements exactly one assigned cell (claim → read `read_first` → implement within `files` → commit → finish, which runs the declared tests and releases the reservations) and returns exactly one status token (`[DONE]`/`[BLOCKED]`/`[HANDOFF]`/`[NOOP]`) — it is authority-bearing, never a digest-only gather. Every `bee-swarming` worker dispatch belongs to this class: full waves in `standard`/`high-risk`, and the single dispatched worker that carries out `small` cell implementation (`bee-swarming/references/swarming-reference.md` ("Single execution worker in full")) — never zero of them from `small` up; `tiny` may execute inline in the orchestrator session instead, and when a tiny cell IS dispatched it belongs to this class too. **Parallel by default:** a `small` lane's 1-3 cells fan out to concurrent execution workers whenever every cell's product file set is disjoint — reservations are the proof and the police, 3-4 live workers is the cap; serial requires a named conflict recorded in the dispatch note (worker returns and its done-report lands before the conflicting next cell is claimed/dispatched) — never assumed as the default. **Parallel criterion:** cells run in parallel whenever every cell's *product* file set is provably disjoint; a cell's regen targets (release manifest, onboarding ledger, plugin mirrors) drop out of that comparison when it carries `regen_obligation_ack: "wave-barrier"` (the orchestrator then owes the full regen chain once, at wave close); any *actually shared* product file still forces serial — in doubt, serial. An independent reviewer or checker (plan-checker, cell reviewer, panel member) is **neither** class: it is a review-class dispatch — read-only, no registry entry, no reservations, no cell of its own — and is never called an "execution worker."
- **cli gather branch** — when the resolved gather tier is a `cli` type, a gather dispatch runs the configured command **verbatim** via the shell — nothing appended, ever; the prompt goes in on **stdin**; every path handed to the worker is **absolute**; the run is **read-only** by contract. **Stdout IS the digest**, framed by a delimiter contract: the worker prompt instructs the CLI to emit its digest between `<<<BEE_DIGEST` and `BEE_DIGEST>>>` lines, and the orchestrator extracts only what sits between them — missing delimiters or an empty digest is a **failed run**, surfaced loudly, never accepted as a silent green. No `result.json`, no cell, no reservation, no `bee state worker add` registration for a gather, same as any other I/O worker. **Known measurement gap, named not solved here:** a Bash-launched gather emits zero `dispatch.jsonl` rows — closing that gap is Slice 3's job, not this branch's.

### Judgment contract — rails for workers, boundaries for the orchestrator

Rules bind differently by rule kind and by role.

**Three rule kinds:**

1. **Boundary rules** hold as written, for every role, at every bypass level
   that does not explicitly lift them: gate-before-source, declared tests
   green at finish and close, CLI-only state mutation, reservations/holds,
   secret handling. These constrain OUTCOMES; they bind rarely and
   at the right moments. They are never "form".
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
| `standard` | SELECTIVE: the per-slice checklist judge — a pinned `bee-review` dispatch, review tier, read-only, covering every capped `behavior_change` cell of the slice in one dispatch — dispatches when ANY of: the goal-check smells, the slice contains a worker's (or model's) first cells of the feature, or the ~1-in-3 sample falls on it (state the sample choice in the slice-close tick; never silently skip). ESCALATION: any `NEEDS_REVISION` puts that worker's remaining slices on judge-every-slice for the rest of the feature. Unjudged slices still pass the frozen judge per cell — that stays universal and free | review-tier config | per judged cell, each verdict recorded via `cells judge-record`: `PASS` → counts; `NEEDS_REVISION` + `automatic` → cell NOT done, re-dispatch with the exact failing checks + a ledger entry; `NEEDS_REVISION` + `authority` → escalate to the user |
| `high-risk` | same checklist judge as `standard` | independence preferred — model differs from the builder's resolved model; if equal, record `model_independence: "same-model"` honestly and the judge still runs | same verdict handling as `standard` |

The judge returns the `judge-verdict/1` schema, recorded via `bee cells judge-record`; free-prose output is a failed judge run, re-dispatched once, then recorded `unverified`. This table is the single home for the judge-tier rule — every other surface (bee-swarming SKILL + reference, bee-hive SKILL, go-mode, AGENTS.md + its template, bee-capturing SKILL) carries only a one-line pointer back here, never a repeated table.

### Test scope (one declared command, every door)

Cells run `commands.test` — the project's ONE declared test command — at finish (`bee finish` runs it and records `.bee/logs/test-results.json`; green caps, red refuses with the failing excerpt); `bee close` re-runs the same command for the feature (`bee-swarming/references/swarming-reference.md`, "Tests at finish and close, in full"); and `bee worktree merge` re-runs it against the staged merge as the semantic-conflict gate. CI runs that same command on the project's own cadence (push, nightly, or scheduled — the host workflow decides) and auto-files a `verify-red` issue when red; the release flow dispatches the CI run (`gh workflow run CI --ref main`) right after the tag push, a red result arriving back as the same `verify-red` issue, not a local gate. A host keeps every door fast by pointing `commands.test` at a suite it is willing to run on every cap. Judges and reviewers verify against the diff and `must_haves`, never by running the suite as part of a verdict.

**Suite rent.** A suite is not immortal: every guard suite pays rent by catching real defects. A suite that has not caught one in ~6 months is a demotion candidate — moved out of the local/impacted hot path to the CI/nightly tier by a RECORDED decision (never a silent delete; the suite still runs, just not on every developer loop). `bee-grooming` owns the audit: read the verify logs for which suites have gone red for a real defect (environment reds don't count as rent paid), list the never-fired tenants, and propose demotions. Institutional/meta guards (fences, parity checks, doctrine gates) are the usual tenants — product-behavior suites earn rent more often and mostly stay.


