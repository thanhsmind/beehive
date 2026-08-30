# Existence Is Not Evidence — Context

**Feature slug:** existence-is-not-evidence
**Date:** 2026-08-30
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Make a bee plan impossible to author without touching the real tree: every
load-bearing plan claim carries an evidence label with a real anchor, the
gate refuses a guessed load-bearing claim mechanically, and a second reader
audits the anchors. Ends at the plan gate — worker/cell discipline
(prove-the-whole-path territory) is out of scope.

## Origin

Field report (waggledance, 2026-08-30): 7 assertion errors across 2
features, one shape — the agent checked that a thing EXISTS but not what
it CONTAINS or whether the path RUNS. All 7 had green tests. A prose
pattern injected into every worker prompt did not prevent any of them.

Root cause (verified in this repo): evidence labels exist only in
bee-researching (SKILL.md:41-53), which planning invokes only for
"unfamiliar territory" — while the failures happen in familiar-feeling
territory where the prior is strong. plan.md's Discovery section asks for
prose evidence, not per-claim binding (planning-reference.md:43-45). The
hat-facts-gaps seat critiques the plan draft, not the tree
(gates-and-delegation.md:216-218). tiny/small lanes have no verifier at all.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | plan.md gains a mandatory **load-bearing claims table**: each row = claim, label (`read` / `ran` / `guessed`), anchor (`file:line` or command), and a verbatim quote or output line. tiny/small (no plan.md): one evidence line per load-bearing claim inside the merged gate message. | A plan that cannot be written without opening the file converts "be diligent" (judgment) into "fill this field" (mechanical). |
| D2 | The shape/merged gate **refuses mechanically** while the table is missing/malformed or a load-bearing row is labeled `guessed`. Remedy: upgrade the label with a real read/run, or move the claim to Open Questions. No waiver flag. | Prose gates on the model's own uncertainty estimate, miscalibrated exactly where the prior is strong. Hard-refusal precedent: plan-conflicts R138, advisor-ref staleness, scribing-debt walls (`set_gate.rs`). |
| D3 | Before the gate, **every lane** (tiny/small included) makes one cheap **reality touch** per novel surface the plan builds on: open the real data or run the real path once; output recorded in Discovery (or the gate message for tiny/small). | Kills the three cheap proxies (schema-for-data, docs-for-behavior, structure-for-path) in seconds. |
| D4 | `hat-facts-gaps`' plan-step mandate extends to **auditing the claims table**: open each anchor, confirm the quote matches the tree; a mismatch is a BLOCKER. | Labels can be authored falsely; the author never catches their own plausibility (pattern-20260825). Verification is a different task from generation. |
| D5 | The carrier is **skill text + templates + bee-rs enforcement**; a docs/knowledge pattern is additive, never the sole carrier. | The prose-only remedy already failed in the field (prove-the-whole-path). |

### Agent's Discretion

Exact table syntax, which plan sections count as load-bearing, the parse
rules for the bee-rs check, and where the reality-touch record lives inside
Discovery — planning decides, within D1-D5.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| load-bearing claim | A factual assertion the plan's shape depends on — an enum's variants, a field's real contents, a runtime's actual behavior. If it were false, the plan changes. |
| evidence label | `read` (opened that file, that line), `ran` (executed it, output in hand), `guessed` (inferred, not observed). |
| reality touch | One cheap direct contact with the real thing before the gate: open the real data, or run the real path once, and record the output. |

## Existing Code Context

### Integration Points

- `skills/bee-planning/SKILL.md` — Shape + Gate sections (D1, D3 land here)
- `skills/bee-planning/references/planning-reference.md` — "Artifact: plan.md" template (D1), "Tiny/small merged gate" (D1, D3)
- `skills/bee-hive/references/gates-and-delegation.md` — "Hat wave" seat table, `hat-facts-gaps` row (D4)
- `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs` — gate verb; existing hard-refusal precedents to model D2 on
- `skills/bee-researching/SKILL.md:41-53` — the existing evidence-label vocabulary; D1's labels stay deliberately smaller (3, not 4) because plan altitude needs read/ran/guessed, not source taxonomy

### Established Patterns

- Hard refusal with named remedy in the error text — `set_gate.rs` scribing-debt walls
- `docs/knowledge/patterns/20260825-plausibility-is-not-evidence...md` — the second-reader law D4 implements

## Outstanding Questions

### Deferred To Planning

- [ ] Machine-checkable table format for D2 — what exact markdown shape does the bee-rs parser require, and how does it identify "load-bearing" rows (proposal: every row in the table IS load-bearing by definition; non-load-bearing claims simply don't go in it)
- [ ] Where the D2 check runs for tiny/small lanes, which have no plan.md — gate-message text is not machine-parseable; does the merged gate call take a `--claims` input, or does the check apply only where plan.md exists (standard/high-risk) with tiny/small covered by skill text + D4-style spot checks only?

## Deferred Ideas

- Extending evidence labels to cell/worker claims at cap time — prove-the-whole-path territory, separate feature.
- A one-line reinforcement in the AGENTS.md BEE block — deferred; carrier decision D5 covers skill/template/binary first.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads
locked decisions, code context, and deferred-to-planning questions.
