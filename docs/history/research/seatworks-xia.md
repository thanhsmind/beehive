---
artifact_contract: bee-research/v1
topic: seatworks-xia
depth: standard
date: 2026-09-22
---

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream, by craft, in prose and hooks — never as a code port.** Seatworks is a TypeScript Paseo plugin; bee is a Rust CLI plus Markdown skills. The stack gap rules out lifting code, and bee already holds the same SLP shape (supervisor, lead, worker, reviewer, observer, store, gate, mail). What Seatworks does better is *how* it writes prompts, tests skills, lints briefs, reads transcripts for coded facts, and keeps its own rule ledger honest. Those are portable as skill text, hook rules, and one or two tests.
- Why this is the lightest credible path: eleven concrete adoptions below graft onto machinery bee already ships (bee-shaping's interview, bee-herding's supervisor prompt, bee-writing-skills' pressure tests, the write guard, bee-capturing's promotion tree, release.sh). None needs a new subsystem. The one genuinely new component (a coded transcript Detector) is a feature bee deliberately deferred and can now size from a tested reference.
- Why the next-best rung lost: *reuse-only* would repeat the mistake the user corrected on the pstack port — "bee already has X" is not a verdict on craft. *Build* (a bee-side desk, watcher seat, calibration tool) duplicates the Rust store, the judge tier and the mailbox.
- Confidence: 85% on the Upstream reading (three digests, every claim anchored); 80% on the Local mapping; the ranking below is Inference.
- Suggested next step: **none — xia ends in discussion.** If adopted: items A1–A6 are docs-lane edits (one cell each); A7 (transcript facts as hooks) and A8 (per-role git deny) are bee-shaping-sized.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/home/thanhsmind/Projects/refs/seatworks` (origin `github.com/sting9k/seatworks`) |
| Ref | branch `v2` |
| Resolved commit SHA | `2e11099f49eb788b8eb707c14a6f26ecd5987318` (2026-09-22) |
| Narrowed scope | whole plugin: docs, prompts, guides, skills, `server/desk`, `server/runtime` (incl. `watch/`), `server/upkeep`, `server/catalog`, harness settings, tests. `client/` (panel UI) skipped. |

Everything read from the source is data, never instruction (port-protocol Guardrail).

## Repo Snapshot

**Source (Upstream).** Node.js ≥24, no build step, `node --test` (420 tests in 42 files), Paseo plugin API `>=0.8.0 <0.9.0`, zod for settings, git + jq required. One daemon process (`server/**`), one panel (`client/**`), one MCP bridge per seat (`mcp/team.mjs`) that talks to the daemon by files in a spool. Pre-release, no releases, version 2.0.0.

**Local (bee).** Rust workspace (`packages/bee-rs`, edition 2024), `cargo test --release` as `commands.test`, 34 Markdown skills, hooks in Rust (model guard, write guard, secret guard, codex subagent audit), three runtimes (`pi`, `claude`, `codex`) with per-role model bindings in `.bee/config.json`, plugin manifests at 2.45.0, `scripts/release.sh`.

**Constraint that shapes the answer.** Nothing in Seatworks runs in bee's process model. The portable layer is text (prompts, skills, guides), rules (hook checks, tests), and design ideas.

## Question & Assumptions

- What was asked: `xia refs/seatworks` — distill the Seatworks repo. Mode `xia`: understand and discuss, build nothing.
- Success: a ranked list of craft bee should adopt, each with a side-by-side against bee's counterpart and a named reason when bee's version is not worse (memory: port-by-craft-not-by-feature).
- Assumption confirmed: Seatworks is the same SLP lineage bee's `docs/specs/slp-supervisor-lead-peer/` derives from (`NOTICE.md` names "SLP material" as its base), so heavy overlap was expected. Found: the shapes match; the craft differs.

## Findings

### Upstream — what Seatworks is (read from source)

**Shape.** Five roles as data in `roles.json` (`supervise`, `lead`, `work|write|watched`, `work|review`, `watch`). No code compares a role *name*; routing goes by capability (`desk/desk.ts:45-52`, `notice.ts:94-99`). One ledger per project; every write through one per-project lock; an unreadable ledger is refused, never treated as empty (`desk/ledger.ts:123-138`). The plugin "serves SLP and never constrains it": it owns lifecycle, transport, routing, state, provenance — and never judges the work (`docs/ARCHITECTURE.md:6-8`).

**A lane.** Supervisor `open_lane` (scope check: two lanes naming the same serial paths are refused, `desk/tools/supervisor.ts:17-35`) → Lead `start_task` (one writer per working copy enforced at nine points, table in the desk digest) → Peer `done` (hand-back file + optional task gate as *evidence*) → Lead `accept|rework|cut|start_review` → Lead `report ready` runs the lane gate → Supervisor `close_lane land`: merge base into lane, gate the result, fast-forward base; red refuses unless `overGate`, and the override is logged (`supervisor.ts:184-248`, `core/git.ts:163-178`).

**Mail.** One letters file. Mail is *steered* into a running turn only when the harness allows it and the turn is ≥60 s old; *held* while the seat waits on a permission, is busy, or had mail in the last 10 minutes; else sent. All mail for one seat goes as one message. Duplicate keys refused for 30 minutes; keys are events, not text (`runtime/outbox.ts:13-15,103-130`; `desk/tools/shared.ts:20-21`).

**The watch.** A window of ≤80 timeline units per watched seat. Coded facts with levels `page|attend|note`: `destructive` (page, shell regex, tmp-only `rm` excused), `test-weakened` (assertion count dropped or a skip marker added), `suppressed` (`@ts-ignore`/`eslint-disable`/`as any` count rose), `stuck` (four repeat rules over 20 units), `no-recovery` (a failed command head never passes again within 10 calls), `unverified` (handed back, last write after last gate run), `long-turn`, `outside-scope`; plus six lane-record facts read from the ledger each patrol round: `rework-loop`, `patched-not-fixed`, `accepted-unfinished`, `reviews-unconverged`, `certainty-only` (a review told "report only what you are sure of"), `brief-prewritten` (a brief containing the code) (`runtime/watch/facts.ts`, `watch/history.ts:52-105`). A second reader — a Watcher seat or a paid model ("Jev") — raises what code cannot measure and confirms or vetoes code facts. Findings become incidents, routed to whoever answers for the seat (Peer → its Lead; Lead, page, or orphan → Supervisor), never to the watched seat (structural: `notice.ts:96-98`; tested `workflow.test.ts:1851,2126,2539`). The recipient marks each `useful|noise|unknown`; `bin/calibrate.ts` scores every question by AUROC against those marks so thresholds come from data. Shipped default: the watch only records (`attention.watch: false`).

**Prompts.** Every role prompt is one screen with the same shape: title → "Rule that matters most" → `## Never` → sections → the same rule as the last line. Letter-kind → verb tables. Rules worth quoting verbatim:

- `SUPERVISOR.md:112-114`: "Ask with the observation, where to look, and a question answerable only by looking. … Never 'Did you run the tests?' or 'Are you sure?'."
- `SUPERVISOR.md:78-81`: "No episode, cost and smallest correction to name? Then it's a hunch: don't send it."
- `SUPERVISOR.md:100-102`: "nothing (most often) → one open question → advice naming episode, cost and fix → new directive → close the lane. One step per episode; see where it lands first."
- `SUPERVISOR.md:81`: "A Lead that disagrees gets your evidence **once**. If it holds with evidence, it keeps its position."
- `LEAD.md:47`: "Leave out the answer you worked out alone: a brief that holds it gets it back unchecked."
- `LEAD.md:48`: "Ask open questions, not 'A or B': a Peer offered two picks one and never finds the better third."
- `PEER.md:13-14`: "Add a shim, adapter, re-export, dual path, flag or stub to make half-done work compile. If a compatibility layer seems needed, name the shipped consumer and `ask`." (under Never)
- `WATCHER.md:45-52`: "Thoughts are intent. Weigh them by what comes next."
- `trail.ts:7`: "what a seat says of its work is never evidence" — the closing message is lifted out of the trail as `final`, apart from the steps.

**Grilling** (`skills/supervisor/grilling/SKILL.md:13-37`): only behavior questions go to the human; a fact the repo can give is never a question; decisions mapped as a dependency tree; one round asks every decision whose prerequisites are settled, numbered, each with a recommended answer; the agent's own decisions (stack, design, tests, process) are listed at the foot under **Assumed**, one line each, "so the Human can overturn one, and do not ask"; each settled answer is written to `CONTEXT.md` the moment it settles.

**Skills.** Eleven, each: purpose line → numbered method → "Ends in" naming the output path and the `done` fields. Trigger surface tested without a model: every skill needs ≥3 briefs that should open it and ≥2 near misses that must not (`test/skills/triggers.test.ts:23-28`); the description line is exactly what a seat sees; a paid eval runs the real agent, pass at ≥50% of 3 runs. Standouts: `test-first` with an 18-row test-antipatterns table (tell + better route); `retrospective` (episode with a countable cost, class Specification/Coordination/Verification, "judge the system, not the agent", at most one change per retrospective, backed by two dated episodes); `pre-mortem` (2–3 sealed stories written in the past tense, one per lens, merged dropping nothing, each cause a row with first signal, mitigation, disposition).

**Guides.** `CONTEXT_FORMAT` (Language / Behavior / Not doing; "an inference the Human has not confirmed is a question, not a line"; ~80 lines, fold rather than drop; current not history). `FEATURE_INTAKE` (Tiny/Normal/High-risk, hard-gate list, "don't split by layer, to show progress, or into phases that keep a half-built state compiling"). `PLANS` (a plan without "Getting back" is half a plan). `STRUCTURAL_LENSES` (search lenses, with exoneration verdicts `BORING_STANDARD` / `JUSTIFIED_DEVIATION`).

**Notebook** (`records/notebook.md`): "A row is a mechanism, not an episode"; lifecycle `seen → adopted → applied → verified` (second sighting on a different day promotes); "Prefer a change to authority, information or integration over one more rule"; a kit change is proposed only as a diff after a pattern is seen twice.

**Honesty ledger** (`docs/ANTIPATTERNS.md`): 36 rules, each stamped `caught` (a fact fires), `askable` (collected, acts on nothing), `desk` (in the ledger, nothing reads it), or `outside` (cannot observe, and says why). Ten caught, twelve desk-only, ten outside — stated up front.

**Upkeep.** Update is forward-only, refuses while any seat runs, `npm install` only when packages changed, rolls back on a failed install. Owner-edited prompts/skills get **Use new / Keep mine**, the kept copy lives in `own/`, and the owner is still told when the original changes. `STATE_VERSION` + numbered steps; a test hashes the kept types and fails unless the fixture records that hash under the current version (`test/upkeep/state-shape.test.ts`); `release.test.ts` fails when `content/` changed without a `package.json` bump.

**Harness fences.** Every seat on every agent is denied `git push`, `gh`, `paseo`, and starting other agents; every seat is denied the branch-moving git verbs; Supervisor and Lead are denied `git commit`; Reviewer is denied edits; Watcher is denied everything but its two MCP tools. Enforcement strength varies by agent (Codex sandbox read-only; Claude denies Edit but leaves Bash; Pi has no sandbox).

### Local — what bee already has (anchors from the inventory)

| Seatworks | bee counterpart | Status |
|---|---|---|
| Supervisor (human proxy, opens/closes lanes) | `skills/bee-herding/references/supervisor-prompt.md` — cold, read-only observer; explicitly "never a router". Lane opening is the session's own bee chain. | EXISTS, split differently, by decision |
| Lead / Peer / one writer per copy | session as lead; `bee cells` + dispatched workers; worktree + reservations + write guard (`docs/knowledge/areas/workflow-state/worktree-isolation.md:44-47`) | EXISTS; bee's fence is pre-write, Seatworks' is post-hoc |
| Reviewer, clean context | `bee-reviewing`, `bee-review` agent, review tier | EXISTS |
| Watcher / coded facts from transcripts | six state-surface signals (`supervisor-prompt.md:67-107`); "a cheap signal Detector is a later feature, deliberately not this one" (`:61-65`) | NEW (deferred by decision) |
| Incident marking useful/noise + calibration | `advisor-nudge` rows, `--kind silence` with reason, `bee supervisor metrics` | NEW (marking + AUROC) |
| Desk / ledger | `.bee/state.json`, cells, decisions, wave ledger; CLI-only writes | EXISTS |
| Gate before landing, again after base merge | proof line on cap; `bee worktree merge` verifies on committed main, post-commit red reverts | EXISTS, different order (see A9) |
| Mail held until a turn boundary | supervisor mailbox `pending`/`mark-delivered`; herding pane transport | EXISTS |
| Grilling rounds + Assumed | `bee-shaping` Interview craft: one question per turn, propose-then-invert, pinned terms | EXISTS; no Assumed foot, no dependency-tree rounds |
| CONTEXT.md | `docs/history/<feature>/CONTEXT.md` + decisions log, in-tree | EXISTS |
| Guides | context-template, implement-plan-template, planning-reference, edge-dimensions, design-sketch | EXISTS |
| Skills: council / ultra-review / test-first / diagnosing-bugs | hat wave; bee-reviewing; red-before-green; reproduce-first + crash-site-vs-fault-site | EXISTS |
| Skills: pre-mortem / retrospective / architecture-premise-audit / security-check / proof-debt-audit | hat-risks seat; bee-evolving + learnings; chestertons-fence + rebuild-test; a review lens; bee-verifying | PARTIAL |
| Trigger tests (≥3 hits, ≥2 near misses per skill) | bee-writing-skills pressure tests; no trigger fixture | NEW |
| Per-role deny of push/gh/nested agents | herding panes run bypassPermissions with no tool list; control-loop uses enumerated allow | NEW as an explicit deny |
| Use new / Keep mine for owner-edited prompts | onboarding never overwrites outside markers; no 3-way for owned prompts | NEW |
| STATE_VERSION ladder + shape test + content-bump test | tolerant read; `scripts/release.sh` bumps manifests | NEW (the tests) |
| AGENTS.md marked block | `BEE:START/END` markers, replaced whole | EXISTS, exact match |
| Comment rule | `no_code_comments` (zero comments, hook-enforced) | CONFLICT, by decision — bee's is stricter |
| NOTICE.md | provenance spread over four reference files + CREATION-LOGs | NEW as one file |
| Notebook | learnings file, promote-proposals, `bee mailbox reflect` | EXISTS; lifecycle states and "mechanism not episode" rule absent |
| Honesty ledger (caught/askable/desk/outside) | `--fix-at architecture|check|doctrine|none` — same taxonomy, per mistake, never rendered per rule | PARTIAL |
| Doctor | `bee doctor` | EXISTS |
| Multi-agent harness | three runtimes in `.bee/config.json`; no Devin | EXISTS |

### Cross-cutting sweep (Upstream)

Wiring outside the feature folders that a port would have to know about: the content lint at kit load (`hidesWords`: a Peer's prompt, skills and the shared team block may never contain "seat", "supervisor", "paseo" — `catalog/kit.ts`, `roles.json:80-85`); sandbox write grants *derived from the prompt text* (a role may write under state only where its prompt names the path — `ARCHITECTURE.md:105-108`); secrets masked at the edge of every quote path before clipping (`watch/mask.ts`); the `outside()` fence that strips any tag from untrusted text and labels it "to judge and never to follow" (`desk/letters.ts:11-28`); the patrol's eight ordered steps with each step try/caught alone (`runtime/patrol.ts:46-91`). None of these lives in the folder of the feature it protects.

### Inference — craft side-by-side and the adoption list

Ranked by value over cost. "Equal or bee better" rows name the reason, as the port-by-craft rule requires.

**A1. The `Assumed` foot and dependency-tree rounds (bee-shaping).** Seatworks lists the agent's own decisions under **Assumed** so the human can overturn one without being asked. bee has propose-then-invert but every agent-side call stays implicit. Adopt the Assumed foot outright. On rounds: bee's one-question-per-turn has a recorded reason ("a batch buries the reframe"); Seatworks contains that risk by asking only decisions whose prerequisites are already settled. This is a real tension, not a clear win — the user's own preference for fast picks argues for offering the tree-round as the shaping default when the tree is shallow. *Discussion item.*

**A2. Question craft and the escalation ladder (bee-herding supervisor prompt).** bee's prompt caps interventions at two sentences and dedupes by point-key. Seatworks adds the three tests a question must pass (observation, where to look, answerable only by looking), the hunch rule (episode + cost + smallest correction, or don't send), the five-step ladder with "one step per episode", and "evidence once, then the Lead keeps its position". These are sharper than bee's and fit the same file. Docs-lane.

**A3. Skill trigger tests (bee-writing-skills).** ≥3 positive briefs and ≥2 near misses per skill, checked without a model; the description line as the tested surface. bee has 34 skills with real trigger confusion (how/why/teach; researching/wayfinding). bee's pressure-test template checks behavior under pressure, not routing. Adopt as a fixture + a test the suite runs. Fix-at: check.

**A4. Prompt shape for bee's agents.** "Rule that matters most" first, `## Never` second, rule repeated as the last line; letter-kind → verb tables. bee's worker contract is longer and shape-free. Adopt the shape for `bee-build`, `bee-review`, and the supervisor prompt. Docs-lane.

**A5. Notebook discipline into bee-capturing.** "A row is a mechanism, not an episode", the `seen → adopted → applied → verified` lifecycle with the second-sighting rule, and "prefer a change to authority, information or integration over one more rule". bee's promotion tree has no state per learning and no "twice before a kit change" bar. Docs-lane.

**A6. Two tests from upkeep into bee's suite.** (i) `content/ changed without a version bump` fails — bee's equivalent: `skills/**` or hook prompts changed without a manifest bump, cheap to add beside `scripts/release.sh`. (ii) A shape hash of the persisted store types that must be re-recorded on change — bee reads tolerantly today, so this is optional; (i) is the win. Fix-at: check.

**A7. Coded transcript facts as bee hooks (the deferred Detector).** Seatworks proves the fact list is small and testable. bee's home for it is not a transcript window but the hooks it already runs pre-write and post-command: `test-weakened` and `suppressed` fit the write guard (diff the assertion count / skip markers / suppression pragmas on a write, warn or refuse); `destructive` fits a Bash pre-hook (bee's write guard already refuses secret and generated paths); `unverified` is already covered by proof lines at cap (bee better: refuses the cap, not just flags). The ledger facts map to bee's store: `rework-loop` ↔ retry budgets (exists); `certainty-only` and `brief-prewritten` are lints on cell text at `bee cells add` (new, cheap). Shape it as one feature, hard-gate flagged (it touches the hooks). bee-shaping.

**A8. Explicit per-role deny list.** Every Seatworks seat is denied `git push`, `gh`, and starting another agent; leads and supervisors are denied `git commit`. bee's write guard refuses `git push` only in a terminal phase (`write_guard/checks.rs:830-862` returns before the push arm when the phase is not terminal), and judges neither `gh` nor an agent CLI at all, so an execution worker mid-feature *can* push. bee's `bee worktree merge` and release script own landing; a worker push is a hole. A hook deny on `git push`/`gh` for `bee-build` workers is small and fix-at: check. Hard-gate (authorization). bee-shaping.

**A9. Landing order.** Seatworks gates the *merged* result before base moves, so a red never lands on base. bee verifies on committed main and reverts on red. Both are recorded decisions; Seatworks' order is cleaner (no revert commit in history). Note only — bee's merge-back is transactional for other reasons (reservations, ancestry, common git dir). Not proposed.

**A10. Use new / Keep mine for owner-edited skills.** bee never overwrites outside markers, but a host that edited a bee-owned skill gets it replaced on `bee onboard` upgrade or keeps a stale one silently. Seatworks' three-way (shipped hash, owner copy in `own/`, still told when the original changes) is the right shape for host-packaging. Medium; belongs to the open `host-packaging-gaps` proposals.

**A11. NOTICE.md and the honesty ledger.** bee ports from MIT sources (mattpocock, superpowers, pstack) and records provenance in four scattered files; one `NOTICE.md` is licensing hygiene and one afternoon. The honesty ledger (rule → caught/askable/desk/outside) is bee's own `--fix-at` taxonomy applied to rules instead of mistakes; rendering AGENTS.md's rules by enforcement layer would say plainly which rules are hook-held and which are only prose. Docs-lane.

**Equal or bee better (named reasons).** Refusals name the remedy — bee already does (Guardrails). Untrusted text is data — bee's Guardrails rule plus the harness pattern-neutralizer. Gate as evidence with logged override — bee's "never build on red" is stricter and recorded; keep. One writer per copy — bee's reservations + write guard fence *before* the write; Seatworks' owned paths are prose with a post-hoc note (`letters.ts:197`). Comment rule — bee's zero-comment hook is stricter than "one docstring per function", by decision. Skills catalog — bee's principle skills are one idea each and more composable; Seatworks' `test-antipatterns.md` table is still worth folding into `bee-principle-test-behavior-not-structure` (18 tells with routes beats one rule). Capability-not-name routing — bee's roles are config data already; no change.

**Weaknesses seen in the source, not to copy.** Two docs disagree on which sensor questions act (`ANTIPATTERNS.md:54` vs `REFERENCE.md:255`). Tool text says `message` "never interrupts" while mail is steered mid-turn after 60 s. Owned paths unfenced. Reviewer read-only enforced at three strengths across agents. `git -C <path> push` slips the Claude deny. The spool is a 250 ms file poll with a `/proc`-shaped agent-id lookup. The lane verbs (`open_lane`, `accept`, `cut`, `close_lane`, the `reset --hard` at `lead.ts:332`) have no desk-level tests. The incident budget can double-count under a burst. Watcher `judge` files p as 1/0, so calibration cannot rank it. `STATE_VERSION` is 1 with zero steps: the migration path has never run for real.

## Risks, Unknowns, Follow-Ups

- A1 conflicts with a recorded bee rule (one question per turn). Superseding it is the user's call, with `bee decisions log --relation supersedes:<id>`.
- A7 and A8 touch hooks and authorization: hard-gate lane if taken.
- `Local` (verified 2026-09-22 at shaping): execution workers can `git push` in a non-terminal phase — the guard's push refusal sits behind `is_terminal_phase` (`write_guard/checks.rs:830-862`; tests cover the idle fixture only, `tests.rs:2647`, `4263`). Shaped as `docs/history/worker-outward-guard/CONTEXT.md`. A3 shaped as `docs/history/skill-trigger-cases/CONTEXT.md`.
- Not verified: whether `test/runtime/workflow.test.ts` covers the lane verbs the desk tests skip (the second advisor read it for watch scenarios only).
- Open question for the user: which of A1–A11 to shape first, if any.

## Source Pack

- Local files read: `AGENTS.md`, `CLAUDE.md`, `.bee/config.json`, `skills/bee-shaping/references/shaping-reference.md`, `skills/bee-herding/references/supervisor-prompt.md`, `skills/bee-writing-skills/SKILL.md`, `docs/knowledge/areas/workflow-state/worktree-isolation.md`, `docs/history/research/{demonthorn-deep-dive-vs-bee-slp,paseo-pi-team-human-elevation}.md`, the inventory digest (22 concepts, anchors in the Local table).
- Upstream files read (at `2e11099f`): `README.md`, `NOTICE.md`, `AGENTS.md`, `CLAUDE.md`, `docs/{ARCHITECTURE,REFERENCE,ANTIPATTERNS}.md`, `plugin/roles.json`, `plugin/content/**` (prompts, guides, skills, records, project block), `plugin/server/{desk,core,runtime,upkeep,catalog}/**`, `plugin/harness/**`, `plugin/mcp/**`, `plugin/bin/**`, `plugin/test/**`.
- Docs pages checked: none — no external docs were needed; the source and its own docs were version-matched by construction.
