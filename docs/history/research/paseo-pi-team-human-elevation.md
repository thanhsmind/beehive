---
artifact_contract: bee-research/v1
topic: paseo-pi-team-human-elevation
depth: standard
date: 2026-08-29
---

## Bottom Line

- Recommendation (ladder rung): adapt-upstream, three mechanisms, in this order: (1) a delegated-decision tier for bee's supervisor (their `SUPERVISOR_DECISION`), (2) a mid-flight typed message channel over the paseo transport (their `peer_ask_lead` / `PEER_MESSAGE_V1`), (3) a `HUMAN_DECISION_REQUIRED` bit on every queued ask and report item. Successor-lead recovery is a fourth, later.
- Why this is the lightest credible path: all three graft onto machinery bee already shipped — the supervisor role, the away/back queue, silence-is-consent (c706053e), WakeReport, and the herding mailbox. None of them builds a second control plane; herdr stays the base and paseo is only the transport and lifecycle layer, which is exactly how the source repo splits it.
- Why the next-best rung lost: building a lead-to-lead hierarchy is not what the source does — it has NO lead↔lead channel at all. Copying their V3 authority-brief parser would duplicate bee's stronger filesystem hooks.
- Confidence: 80% (their code read directly; the bee-side grafting points are `Local`-verified; the paseo-transport claim for herding panes is `Inference` until spiked).
- Suggested next step: bee-shaping for item 1 (it moves a locked supervisor boundary — user gate); items 2–3 can ride the same shaping interview.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | /home/thanhsmind/Projects/refs/slp/paseo-pi-team |
| Ref | HEAD (working tree) |
| Resolved commit SHA | 94ead115960df493409d281cecbbbf02b6ce8bf0 |
| Narrowed scope | prompts/{supervisor,lead}.md, skills/paseo-team-lead/SKILL.md, scripts/team-communication.mjs, docs/multi-host.md, README structure |

## Question & Assumptions

- What was asked: the user wants the human moved UP — leads finish work among themselves, the human limited to a few genuinely necessary points. Distill how paseo-pi-team does this. Base stays herdr; only the methods for talking to paseo are learned.
- Success: a mechanism list bee can graft onto herding, not a re-architecture.
- Assumption corrected during recon: "leads work with each other" is NOT how the source works — see Findings.

## Findings

### How the source actually elevates the human (Upstream)

The human is not elevated by making leads talk to each other. It is elevated by three walls, each fail-closed:

1. **Delegated decisions** (prompts/supervisor.md §Delegated decisions). The Supervisor may decide ON THE HUMAN'S BEHALF — issue a `SUPERVISOR_DECISION` — only when ALL four criteria hold: small scope (one file/step, no contract/API/schema/security/payment/data), reversible by `git revert` or ordinary fix, PROVEN observation (never a suspected mechanism), and inside the existing protocol. One decision per message (no bundling past the threshold), prefer the most reversible option, mandatory rollback path, mandatory follow-up for at least one observation round, and a wrong delegated decision reclassifies that whole matter as escalate-only. Repeat offenders (failed once, fixed once, failing again) ALWAYS escalate. Expanding the delegation boundary is itself always a human decision.
2. **The Lead honors it** (prompts/lead.md §Authority): a low-risk reversible `SUPERVISOR_DECISION` counts as a valid decision with no human round-trip. The human keeps exactly: merge, push, deploy, external systems, anything irreversible, and anything the supervisor marked `HUMAN_DECISION_REQUIRED: yes`.
3. **Every observation carries the escalation bit**: the output contract forces `HUMAN_DECISION_REQUIRED: yes | no` on each message, so "what needs the human" is a typed field, never prose the human must dig for.

Result: the human's surface shrinks to (a) irreversible actions, (b) boundary changes, (c) repeat offenders, (d) whatever the supervisor is unsure about — unclear is defined as escalate.

### What "leads working together" means there (Upstream)

docs/multi-host.md:108: one Lead per project/daemon, the Supervisor observes ACROSS them, the Human coordinates the portfolio. There is no lead↔lead channel, and the deep-dive (§9) explicitly forbids the supervisor becoming a joint lead or using project A's evidence to accept project B. Cross-project traffic is: Peer→own parent Lead only (parent-scoped), Supervisor→any Lead (open questions + relayed decisions), Supervisor→Human (reports). The one inter-lead mechanism is succession, not conversation: the Supervisor's single orchestration power is creating a successor Lead when the incumbent provably cannot recover — shape-guarded by the extension (provider must be pi-lead, purpose=recovery, project pinned), and archiving the old lead stays human.

### How they talk to paseo (Upstream) — the transport lessons for herdr

- **Peer→Lead mid-flight**: scripts/team-communication.mjs — resolve own agent id (`PASEO_AGENT_ID`), `paseo inspect <self>` → `paseo.parent-agent-id`, then `paseo send <parent> --prompt <PEER_MESSAGE_V1> --no-wait`. Typed envelope: `KIND: question | blocked | dependency | progress`, `CORRELATION_ID` (lead-side dedup, deliberately NOT transport retry — send is a mutation with delivery ambiguity), `TASK_ID`, `FROM_AGENT_ID`. Parent-scoped by construction: it cannot target an arbitrary agent.
- **Lead→Peer**: `send_agent_prompt`, restricted to five uses (new constraints, correction findings, dependency resolution, scope clarification, answering a peer message); any follow-up that re-grants authority must repeat the full brief — authority never carries over a turn.
- **Watchdog**: bounded-concurrency inspect sweep; only a SUCCESSFUL inspect with stale `UpdatedAt` is `stale` (a suspicion), a failed inspect is `unknown`; neither is ever an automatic recovery signal, and no writer is replaced while old workspace/Git state is unclear.
- **Routing evidence**: every create verifies provider/model/thinking on the exact target daemon and then bounded-polls OBSERVED runtime identity; typed `BLOCKED:` codes; no silent fallback of model or host. Local daemon = MCP; remote daemon = CLI wrapper (`--host` is CLI-only) — mixing them is their named failure mode.

### What bee already has (Local)

- Observer supervisor with cold ticks, mailbox interventions, presence away/back, queued asks, WakeReport, UrgentAlert (9f5cd250, c80debd7).
- The delegation seed already locked: c706053e — narrow opt-in silence-is-consent for non-gate queued asks, with earned-autonomy metrics (66c4c251). This is a TIMEOUT tier; the source's `SUPERVISOR_DECISION` is an ACTIVE tier on top of the same queue.
- Dissent/StopAndAsk as the worker's typed voice at turn end (4b7aa303, a2affcba) — but a2affcba explicitly declined a live mid-flight channel "for lack of a transport". Under paseo, herding panes and sibling sessions ARE durable, addressable agents; the missing transport now exists (`Inference` until a spike proves `paseo send` reaches a busy herdr pane usefully).
- Multi-session etiquette: lanes, claims, holds, `claim-next` — bee's cross-session coordination is store-mediated, not message-mediated.
- Session liveness/heartbeat + pane classifier ≈ their watchdog; the stale-vs-unknown distinction is already bee's spirit (a dead session's mark expires).

### The graft list (Inference — the recommendation)

1. **Supervisor delegated-decision tier** — extend the queued-ask machinery: an ask the supervisor can prove small+reversible+in-protocol gets an active `SUPERVISOR_DECISION` record (decision log entry, rollback path, one follow-up tick obligation, repeat-offender and unsure → escalate), rendered prominently in the WakeReport exactly like c706053e's auto-proceeds. Gates/UAT/merge stay human — same carve-out c706053e already draws. This is the single biggest "human up" lever and it moves a locked boundary (322695d6's observer-only reading), so it is a shaping interview with the user, not a patch.
2. **Typed mid-flight messages over paseo** — a `PEER_MESSAGE_V1`-shaped envelope (question/blocked/dependency/progress + correlation id) from herdr workers/sessions to their parent session via `paseo send`/SendMessage, answered at the parent's next turn. Supersedes a2affcba's "no transport" premise rather than contradicting its rule — that decision's own rationale names the missing transport as the reason. Parent-scoped only; lead↔lead stays store-mediated (claims/holds) plus supervisor relay, per the source's own design.
3. **`HUMAN_DECISION_REQUIRED: yes|no` on every queued ask, letter item, and WakeReport line** — a typed bit, sorted first. Cheap; pure docs/templates + report renderer.
4. **Successor-session recovery (later)** — supervisor proposes (never silently replaces) a successor session with a handoff bundle when liveness + evidence prove non-recovery; old session's teardown stays the human's. bee's handoff records already carry the bundle shape.

### Not recommended (Inference)

- Lead↔lead conversation channel — the source deliberately has none; bee's store-mediated coordination is the same choice already made.
- V3 authority-brief parser — bee's hooks/reservations enforce authority at the filesystem, strictly stronger than prompt-block parsing.
- Copying their routing cycle wholesale — `dispatch prepare` + model-guard already owns this; only the "observed identity, typed BLOCKED, no silent fallback" phrasing is worth folding into herding docs if gaps appear.

## Risks, Unknowns, Follow-Ups

- `paseo send` into a busy herdr pane mid-turn: delivery timing unproven (`Inference`) — spike before shaping item 2.
- Item 1 touches locked decisions 322695d6/c80debd7/c706053e territory — any adoption is a supersession the user must approve.
- Delegation drift is the named failure mode; the source's countermeasures (one-decision-per-message, rollback path, reclassify-on-error, boundary changes human-only) must come along or the tier is unsafe.

## Source Pack

- Upstream (read in full): prompts/supervisor.md, prompts/lead.md, skills/paseo-team-lead/SKILL.md, scripts/team-communication.mjs, docs/multi-host.md (head), README.md (structure). Skimmed only: peer.md, watchdog.mjs, templates (labeled accordingly).
- Local: docs/history/slp-supervisor-heartbeat/CONTEXT.md (9f5cd250, c706053e, 66c4c251, 322695d6), docs/history/slp-dissent-stop-and-ask/CONTEXT.md (a2affcba), skills/bee-herding/references/supervisor-prompt.md, AGENTS.md multi-session etiquette.
- Related brief: docs/history/research/demonthorn-deep-dive-vs-bee-slp.md (same lineage, doctrine layer).
