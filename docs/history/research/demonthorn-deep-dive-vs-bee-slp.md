---
artifact_contract: bee-research/v1
topic: demonthorn-deep-dive-vs-bee-slp
depth: standard
date: 2026-08-29
---

## Bottom Line

- Recommendation (ladder rung): reuse for ~85% of the doc — bee already shipped it, often with stronger enforcement; adapt-upstream for three narrow items (causal supervisor records, a typed reaction set, next-signal names for the deferred Detector).
- Why this is the lightest credible path: the four SLP clusters (heartbeat, dissent/stop-and-ask, blind lanes, contract/original-request) plus followup-gaps already cover the doc's role model, escalation verbs, one-writer law, evidence acceptance, and event-driven supervision — and bee adds doors, metrics, and anti-fabrication checks the doc does not have. Only the protocol-evolution loop and a few vocabulary items are genuinely missing.
- Why the next-best rung lost: building any of the doc's structures fresh (a WORKSPACE_PROTOCOL file, a Detector, a notebook store) would duplicate machinery bee already holds in config, knowledge, and the supervisor record.
- Confidence: 85%.
- Suggested next step: none (xia — discussion). If an item is adopted, item 1 is bee-shaping-sized; items 2–3 are docs-lane edits.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | /home/thanhsmind/Projects/refs/slp/paseo-pi-team |
| Ref | HEAD (working tree) |
| Resolved commit SHA | 94ead115960df493409d281cecbbbf02b6ce8bf0 |
| Narrowed scope | docs/demonthorn-agent-orchestration-deep-dive.md (1301 lines) |

The doc is a distillation of Demonthorn's Discrawl messages (2026-06-25 → 2026-08-02). It labels its own claims [DIRECT] vs [SYNTHESIS]; its profile/template blocks are reconstructions, not canon. `Upstream` label below means "stated in this doc".

## Question & Assumptions

- What was asked: xia the deep-dive against bee's existing SLP; find what bee can add or improve.
- Success: a ranked list of genuine deltas, not a re-port of what already shipped.
- Assumption confirmed during recon: bee's SLP derives from the SAME lineage (docs/specs/slp-supervisor-lead-peer/ is the interaction spec of the same "SLP: Supervisor – Lead – Peer" family), so heavy overlap was expected and found.

## Findings

### Already covered — reuse (Local)

| Deep-dive concept | bee's shipped equivalent | Where |
|---|---|---|
| Supervisor = out-of-band observer, open questions only, no routing/acceptance (§4.2, §8.17) | Cold-tick observer role; read-only surface; one record per tick; ≤2-sentence open-question interventions; "never a router" stated verbatim | skills/bee-herding/references/supervisor-prompt.md; decisions 322695d6, da7cb49b, c80debd7 |
| Human↔Supervisor presence + reports (§4.1, §8.14) | away/back presence mark, queued asks, one WakeReport, UrgentAlert | decision 9f5cd250 |
| REOPEN_REQUEST / BLOCKED / challenge rights (§4.4, §7.1–7.2, §8.1) | Dissent with FULL TEETH — blocker dissent parks work, orchestrator OWES accept/reject/escalate, close+merge doors refuse unanswered dissent; StopAndAsk with options[]+leaning; herding workers carry dissent as data | decisions 4b7aa303, a2affcba; slp-followup-gaps D3–D5 |
| Sealed council, sealed reports, distinct mandates (§2.2, §9, §8.13) | Blind lanes: byte-identical neutrality-linted brief, advisor-kind isolation, cross-critique round, dossier + anti-fabrication citation check, deadlock → human | slp-blind-lanes D1–D7 |
| original_request immutable through layers | Verbatim intent anchor rendered into every dispatch; layers may only ADD | slp-contract-original-request D5/D6 |
| Contract status; no test against unsettled contract | Derived `contract:<name>` view over the decision log + mint-trap refusal at the dispatch/claim doors | slp-contract-original-request D1–D4 |
| One writer per moving scope, worktree isolation, stable candidate (§2.3, §7.4, §8.6) | Reservations, feature worktrees, one commit per cell, review on immutable scope | AGENTS.md; worktree/reservation machinery |
| Status ≠ acceptance; evidence chain (§2.5, §8.16) | "Prove, then say so", cap proof lines, judge verdicts, doors that read recorded proof | AGENTS.md; close/merge doors |
| Provider/model discovery, no hard-coded IDs (§2.4, §5.3) | `dispatch prepare` is the ONE door; model-guard refuses hand-picked models; roles are open config | swarming-reference; models config |
| Event-driven, no polling (§5.5, §8.10) | Harness notifications; supervisor prompt forbids polling; wakeup pacing rules | supervisor-prompt; harness |
| Anti-pattern memory (§8) | 58 critical patterns in docs/knowledge + supervisor's 3 signals + a8f4b8ab's 2 telemetry signals | knowledge bundle |
| Peers can't spawn agents (§2.1) | Herding workers never run bee commands; read-only worker types are hard contracts | mailbox pin (retargeted by followup-gaps D6) |
| Topology scaled by risk (§6.6, §9) | Lane classifier (tiny→high-risk), hard-gate flags, gate bypass levels | route/lane system |

### bee is AHEAD of the doc (Local)

- Door-enforced obligations (judge-debt, dissent-debt, proof) — the doc's checklists have no enforcement story at all.
- Earned autonomy: silence-is-consent gated on measured reversal rates (c706053e, 66c4c251) — the doc has nothing like it.
- Anti-fabrication convergence check and `bee blind check` — the doc's council trusts the synthesizer.
- Decision log with supersession, triggers (revisit conditions), and who-dissented — richer than the doc's notebook.

### Genuine deltas — adapt (Upstream → candidate)

1. **Causal context on supervisor records, feeding protocol evolution** (§4.2 notebook, §7.7). The doc: a record needs observation + cause evidence + mechanism + recovery + protocol candidate, because "mechanism helps the protocol evolve; verdict alone teaches slogans." `Local`: `supervisor record` carries kind/signal/note ≤500 chars and a frequency cap — no mechanism slot, and nothing routes repeated observations into `bee feedback`/bee-evolving. The loop "anti-pattern observed → causal record → durable pattern → protocol patch" is bee's own compounding shape, and the supervisor is currently outside it. Adopt: an optional mechanism/protocol-candidate field on the record, plus a sweep that promotes repeated same-signal records into the feedback digest. Feature-sized; touches the supervisor verb group.
2. **Typed reaction set for consults and reviews** (§8.1). The doc: demand `CONFIRM / PARTIAL / CHALLENGE / BLOCK` instead of asking "please push back" — a typed slot beats an exhortation, and both extremes (sheep, performative contrarianism) are named. `Local`: blind-lanes D5 already requires pushback to name the missing context; dissent severity is a closed set. But advisor consults and plan reviews return free prose. Adopt: add the four-value reaction slot to the advisor-consult and review report forms. Docs-lane edit on prompt templates.
3. **Named next signals for the deferred Detector** (§8.3, §8.7, §8.11, §8.12). The Detector is already bee's recorded next step (da7cb49b deferral). The doc contributes concrete, cheaply detectable signatures: third correction of the same symptom → ask "what shared mechanism produces the chain?" (parachute-vs-brakes); implementer running its own success benchmark (self-acceptance); council/report count rising while evidence doesn't (ceremony capture); both debated options living inside one unexamined framing (framing capture). Adopt: record these as the Detector feature's candidate signal list in its deferred-ideas note. Zero code now.

### Noted, not recommended (Inference)

- **WORKSPACE_PROTOCOL.md as a separate orchestrator-only file** (§6). bee's equivalents — `.bee/config` (uat_stop, staging, gate_bypass, model roles), lane classifier, AGENTS.md, knowledge bundle — already split behavior/tactics/task the way §3 wants, and workers already receive narrow prompts (the attention argument is satisfied). A new file would be a second registry, the drift shape D1 of contract-status explicitly refused.
- **"Plans are provisional maps" / neutral briefs for cells** (§7.3, §8.2). bee deliberately runs gated, pre-planned cells; dissent + StopAndAsk are the pressure valve for a wrong premise. Loosening cell briefs would trade bee's verification spine for the doc's exploration style — a product decision, not a gap.
- **DEPENDENCY_REQUEST as a distinct verb** (§4.4). Covered by blocked-with-reason + dissent's alternative; a third verb adds vocabulary, not capability.
- **Human talks mainly to Supervisor to protect Lead attention** (§8.14). bee's human-facing session IS the lead; the WakeReport path already exists for the away window. Restructuring the primary channel is out of proportion to the benefit.

## Risks, Unknowns, Follow-Ups

- The doc self-labels its profiles/templates as [SYNTHESIS] reconstructions; nothing above treats them as canon.
- Item 1's exact shape (field vs sweep vs both) needs its own shaping interview if adopted.
- Locked decisions win: nothing here supersedes 787a9eb0 (bee's rules beat SLP on conflict) or da7cb49b (Detector deferred).

## Source Pack

- Source: refs/slp/paseo-pi-team/docs/demonthorn-agent-orchestration-deep-dive.md @ 94ead115 (read in full).
- Local: docs/history/slp-{supervisor-heartbeat,dissent-stop-and-ask,contract-original-request,followup-gaps,blind-lanes}/CONTEXT.md; skills/bee-herding/references/supervisor-prompt.md; skills/bee-hive/references/gates-and-delegation.md ("Blind lanes"); skills/bee-swarming/references/swarming-reference.md (advisor/dissent surfaces); docs/specs/slp-supervisor-lead-peer/slp-supervisor-lead-peer.md (lineage check); docs/history/research/slp-*.md (index only).
