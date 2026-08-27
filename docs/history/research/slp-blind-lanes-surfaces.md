# Research digest — blind-lane surfaces in bee (SLP ticket 006)

- Date: 2026-08-26 · Tier: advisor (fable) — supersedes the same-day cheap-tier draft
- Context: `docs/discovery/slp-supervisor-lead-peer/tickets/006-blind-lanes.md`

## 1. What isolation exists today, and what leaks

**The dispatch door.** `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:29-51` — two runtimes, four kinds (`cell`, `gather`, `reviewer`, `advisor`), each kind mapping to a model slot (`slot_for_kind`, :44-51). Every non-cell kind is read-only by construction (`purpose_is_gather`, :115-117) and `--claim` is refused for non-cell kinds. So 2–3 parallel `--kind advisor` dispatches on the same question are ALREADY possible today: each is a fresh subagent context, its output returns only in its final message, nothing persisted — lane-to-lane leakage through the store is zero for this kind by default.

**`bee dispatch wave`** (prepare.rs:1942-2202, dispatched from close.rs:2885) is NOT this: it claims the first schedule wave of *distinct* cells for ONE feature, each with its own claim and file reservations. It cannot run the same brief N times. Wave = parallel different-work, not parallel same-brief.

**What leaks:**
- The prompt itself. The advisor/gather templates are skeletons — `packages/bee/prompts/advisor.md`: "Paths: <caller fills in>". The orchestrator composes the payload freely; nothing lints it for leaning. The single biggest hole vs spec §4.5.
- Shared disk, no read guard. A dispatched subagent inherits the OS cwd, CLAUDE.md/AGENTS.md, and can Read `.bee/state.json`, `.bee/decisions.jsonl`, `docs/discovery/*` — anywhere the orchestrator may have written its leaning. bee's hooks guard writes and secrets, never reads.
- Cell-kind extras: the worker-cell template injects machine-assembled `learned_context` and prior-round trace (prepare.rs:198-201) — shared memory across workers. Blind lanes must NOT run as `--kind cell`.
- One model per slot: the advisor slot resolves ONE name, no fall-through, by decision 4faf1de9 (prepare.rs:96-97). All lanes get the same model; heterogeneous lanes (spec §10.b) have no door.

**Strongest existing blind-ish precedent:** bee-reviewing's wave — `skills/bee-reviewing/SKILL.md:38-46`: core reviewers spawned in parallel, "Each reviewer gets the cumulative diff, the in-scope features' CONTEXT.md and plan.md, and nothing else — never session history", synthesis only after all return, independent-corroboration promotion (:58-63). An information-diet contract plus convergence synthesis — but it critiques one artifact, generates no alternatives, has no cross-critique round, and is user-invoked only.

## 2. Mapping SLP concepts to bee mechanisms

<!-- bee:not-a-deferral: Research mapping table. Its ConvergenceDossier row describes bee's EXISTING deferral guard (a "revisit when" decision is refused without --trigger) as a surface SLP could reuse. The table documents machinery, it promises nothing. -->
| SLP concept | Nearest bee mechanism | Verdict |
|---|---|---|
| Lane (isolated design session, no product write) | `dispatch prepare --kind advisor` subagent — read-only, ephemeral, output = final message | EXISTS (unenforced blindness) |
| LaneBrief + neutrality scrub (§4.5) | Prompt caller-composed, unlinted. Nearest lint pattern: decisions prose guards — supersession-stem scan (decisions/verbs_read.rs:306-355), deferral scan forcing `--trigger` (:359-432) — bounded lexical scans with typed refusals | MISSING (pattern exists, target doesn't) |
| LaneProposal (verbatim) | Advisor digest contract ("verbatim quotes only where asked"); nothing persisted | EXISTS as message, no record type |
| CrossCritique | Nothing between siblings; bee-reviewing synthesis is orchestrator-side only | MISSING (cheap: round-2 advisor dispatches handed the rival's proposal) |
| ConvergenceDossier → decision log | `bee decisions log` carries decision, rationale, alternatives, tags, relation, trigger (verbs_read.rs:202-203); `bee triggers add --decision <id> --condition <text>` is a real revisit registry (verbs/triggers/mod.rs:11,22-33); the deferral guard REFUSES a "revisit when" decision with no `--trigger` | MOSTLY EXISTS — `alternatives` is one flat string, not structured rejected[]; dossier body needs a doc, the decision line links it |
| "When do lanes open" (gate-risk rung) | Gate 3 already REQUIRES a recorded advisor consult for high-risk (`bee state advisor-ref record`, `high_risk_advisor_refusal` in set_gate.rs) — single mandatory consult, not multi-lane | PARTIAL — the hook point exists |
| Deadlock → human with dossier | `bee state waiting-on set --kind question --subject`; unattended runs file a human-mailbox letter | EXISTS as channel; dossier is the subject's linked doc |
<!-- /bee:not-a-deferral -->

## 3. Advisor opinion — cheapest honest design

<!-- bee:not-a-deferral: Advisor research finding. "Deferred out of this effort" restates the map Out-of-scope entry for heterogeneous lane models (decision 4faf1de9), which already carries its own record. A scope statement in a research note, not a new promise. -->
Build blind lanes as a *procedure over the existing door*, not new machinery: the orchestrator writes ONE LaneBrief file, runs a small forbidden-phrase lint over it (the decisions prose-guard pattern, ideally as a `dispatch prepare --kind lane` refusal so neutrality is enforced at the chokepoint, not by discipline), then fires 2–3 `--kind advisor` dispatches in one parallel batch with the byte-identical brief and an explicit "read only these paths" diet — blindness holds because advisor outputs live only in final messages and the orchestrator persists NO leaning anywhere readable before convergence. Cross-critique is round two of the same door: fresh advisor dispatches, each handed the rival's proposal verbatim. Convergence lands as `docs/discovery/<slug>/dossier.md` (proposals verbatim, critiques, reasons, rejected) plus `bee triggers add --condition "<revisit_conditions>"` and one `bee decisions log --alternatives "<rejected: reasons>" --trigger <id>` linking the dossier — every §4.6 field lands in existing records. Deadlock is the same dossier doc plus `bee state waiting-on set --kind question` (mailbox letter when unattended) — the human gets the dossier, never a coin flip. The two genuinely missing pieces worth building: the brief-lint refusal at the dispatch door, and a structured `rejected[]` (or a dossier-source convention) on the decision record. Heterogeneous lane models would need a deliberate break of the one-name advisor slot (decision 4faf1de9) — deferred out of this effort.
<!-- /bee:not-a-deferral -->
