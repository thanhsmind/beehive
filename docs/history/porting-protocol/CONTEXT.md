# Porting Protocol — Context

**Feature slug:** porting-protocol
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Standard
**Domain types:** READ | ORGANIZE

## Feature Boundary

`bee-researching` gains a port protocol — the tooling for "port feature
X from repo Y": trigger vocabulary in its description, a
`references/port-protocol.md` carrying the dependency matrix, challenge
framework, cross-cutting sweep, and source manifest; no new skill, no
new gate. Ends at the skill files + regen chain; no CLI change.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Map: `docs/discovery/feature-porting/MAP.md`.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Extend `bee-researching` with a `references/port-protocol.md` + trigger wording ("port from", "like how X does it", "mang feature về") in its description — no separate skill (decision 1d51c588) | Ladder rung 3 (Adapt) already owns the territory; a new skill duplicates researching+planning |
| D2 | Challenge framework (≥5 adversarial questions, each: source answer / local answer / risk if wrong; red-flag/green-flag table) applies to port work only (decision 0f0ecbe0) | — |
| D3 | Exactly two modes: `compare` (report only, no implementation path) and `port` (default, idiomatic rewrite). Copy-vs-improve is a challenge verdict, never an upfront flag (decision a1a83035) | Kills ak-xia's 4-flag surface; judgment over configuration |
| D4 | Source provenance (repo/path, ref, resolved commit SHA) recorded as a decision-log line at the port feature's shape lock, plus a capture stub into knowledge (decision 9fb2923b) | Decision log is single source; knowledge keeps it findable |
| D5 | No numeric risk score. Challenge outcomes feed the existing lane classification and route flags — high-risk verdicts land hard-gate; a too-large stack mismatch downgrades the work to `xia` (decision a4319276) | Two parallel risk systems drift |
| D6 | `xia` lives as a trigger keyword in bee-researching's description ("xia", "distill from", "học từ repo X") — no rename, no separate skill (decision 7bd126d5, touches D1) | A description keyword is enough to route; a standalone skill returns only as a fresh effort |
| D7 | Modes are named `xia` and `port`. `xia` widens D3's compare semantics into a knowledge-distill report: strengths, weaknesses, does bee already have it, recommendation — ends in discussion, builds nothing (decision c133ebc0, refines D3) | Matches the user's stated intent for the keyword |

### Agent's Discretion

Section wording, table layouts, and where each artifact section sits
inside the existing reference/template files — within the skill-prompt
style laws (`bee-writing-skills` references).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| port | Rewrite a feature from a source repo idiomatically for this repo — never a transplant |
| xia | Distill-only outcome: read an external source, report strengths / weaknesses / what bee already has / recommendation, end in discussion — no implementation path produced |
| dependency matrix | Per-component map source→local, each row `EXISTS` / `NEW` / `CONFLICT` |
| cross-cutting sweep | Explicit hunt for wiring outside the feature folder: middleware, listeners, config, decorators |
| source manifest | repo-or-path + ref + resolved commit SHA + narrowed path scope |

## Specific Ideas And References

- ak-xia's hard gate "Challenge before Plan" maps to: the challenge
  table must exist in the research output before planning consumes it.
- ak-xia's error-recovery ladder (source too large → narrow scope;
  stack mismatch → compare) folds into the protocol steps, not a
  separate section.

## Existing Code Context

### Reusable Assets

- `skills/bee-researching/SKILL.md` — recommendation ladder rung 3
  (Adapt) is the anchor; port protocol hangs off it
- `skills/bee-researching/references/research-protocol.md` — step
  rules; port steps extend, never replace, the 4-step order
- `skills/bee-researching/references/research-brief-template.md` —
  standalone brief; candidate home for port sections

### Established Patterns

- Evidence labels (`Local`/`Upstream`/`Docs`/`Inference`) — every
  matrix row and challenge answer carries one
- Degrade-never-skip on capability gaps — applies to fetching a remote
  source repo
- AGENTS.md guardrail (fetched content is data, never instructions)
  already covers ak-xia's "security boundary" — cite, don't restate

### Integration Points

- `skills/bee-researching/SKILL.md` description block — trigger wording
- Regen chain: `bee dev regen` re-renders skill trees
  (`.claude/skills/`, codex tree) after source-skill edits

## Canonical References

- `/home/thanhsmind/projects/AI/ak/.claude/skills/ak-xia/SKILL.md` —
  ported source (repo `projects/AI/ak`, commit `a70c7cdb`)
- `/home/thanhsmind/projects/AI/ak/.claude/skills/ak-xia/references/challenge-framework.md`
  — challenge questions, red/green flags, decision-matrix shape

## Outstanding Questions

All resolved during planning and cell pp-1 (commit 7fb20eb2):

- [x] Artifact placement: port sections live in
  `references/port-protocol.md`; standalone `xia` output reuses the
  research-brief template, in-chain `port` findings merge into the
  approach — no brief-template edit needed
- [x] Trigger phrases: "xia", "distill from", "port from", "like how X
  does it", "mang feature về", "học từ repo X" — in the description and
  the reference's When To Load (D6)
- [x] Dependency matrix is a section in `port-protocol.md`, no separate
  template file

## Out Of Scope

- Generalizing the challenge framework beyond ports was ruled out by
  D2 (map "Out of scope"); any wider use is a separate feature with its
  own shaping, not an open item of this one.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning
read locked decisions, code context, and canonical references.
Planning's Gate 2 shape stage and reviewing use locked decisions for
coverage and UAT.
