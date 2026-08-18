# feature-porting — discovery map

## Destination

bee answers "port feature X from repo Y" first-class: bee-researching
carries a port protocol, and this effort ends with the feature shaped
(CONTEXT.md locked) and ready for planning.

Spawned: porting-protocol — docs/history/porting-protocol/CONTEXT.md

## Notes

- Source studied: `ak-xia` (`/home/thanhsmind/projects/AI/ak/.claude/skills/ak-xia/`) —
  6-phase port skill (Recon → Map → Analyze → Challenge → Plan → Deliver).
- Lineage: beegog's `bee-xia` is the ancestor of beehive's
  `bee-researching`; evidence labels, local-first order, and the
  recommendation ladder already cover ak-xia's "understand before copy".
- Already covered by bee, no work needed: untrusted-content guardrail
  (AGENTS.md), local-first research order, handoff-not-implement shape.
- Recon tooling: git + gh suffice for fetching/pinning a source repo
  (`Inference` — no repomix equivalent needed; verify at planning).
- Artifact placement (where the dependency matrix and challenge output
  live — brief template vs approach merge) is a shape-time drafting
  choice; it travels to bee-shaping/planning, not a map ticket.

## Decisions so far

- 1d51c588: extend bee-researching with a port-protocol reference — no
  new skill (round 1 Q2)
- 0f0ecbe0: challenge framework applies to port work only (round 1 Q3)
- a1a83035: two modes only — compare and port; copy/improve are
  challenge verdicts, not flags (round 1 Q4)
- 9fb2923b: source provenance (repo, ref, commit SHA) recorded at shape
  lock + capture stub (round 1 Q5)
- a4319276: no separate numeric risk score — challenge outcomes map
  into existing lane classification (round 1 Q6)

Interview-settled in the charting session; no ticket files exist for
these — the decision log is the single source (named deviation:
closed-ticket stubs would add files with no open question).

## Not yet specified

(none — frontier empty)

## Out of scope

- A separate `bee-porting` skill (ruled out by 1d51c588).
- Generalizing the challenge framework beyond ports (ruled out by
  0f0ecbe0; returns only as a fresh effort).
- Porting ak-xia's speed flags (`--fast`/`--auto`) — bee's gate-bypass
  levels already own that axis.
