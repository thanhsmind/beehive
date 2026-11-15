---
artifact_contract: bee-plan/v2
mode: docs
---

# Plan: cite-knowledge-first

## Summary

When an agent cites a source for how something works, it now cites the
knowledge concept first, then code. History files are cited only for why
a decision was made. The leader doctrine says this in the citation rule
and in the knowledge-first rule. The full form is in the claim-labels
contract. The line that said "cite" history now says what to cite it for.

Mode: `docs`. Class docs, 0 flags. Product files: `packages/bee/LEADER.md`
and `skills/bee-hive/references/routing-and-contracts.md`.
Why this is the least workflow that protects the work: one prose cell in
two owned homes, proved by the parity and pointer tests that read them.

Playbook: `skills/bee-planning/playbooks/docs/docs.md` (class `docs`, situation `docs`).

## Requirements (from CONTEXT.md)

- D1: source order in `agents-verify-before-agreeing` and in "Claim labels".
- D2: the history line scoped to the locked decision and its reason; `agents-knowledge-before-code` covers a how-it-works answer.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The leader line tells the agent to cite history with no scope | read | `packages/bee/LEADER.md:53` | `the locked ones — cite them, never reinterpret them.` |
| 2 | The knowledge-first rule fires only before a code read | read | `packages/bee/LEADER.md:161` | `your first read of source under` |
| 3 | The citation rule ranks no source | read | `packages/bee/LEADER.md:202` | `path:line`, or `[n]` the command run with its result. One source` |
| 4 | Claim labels is the full form of the citation rule | read | `skills/bee-hive/references/routing-and-contracts.md:299` | `Full form of rule: agents-verify-before-agreeing.` |
| 5 | The rule index carries a spoken line for the citation rule | read | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:226` | `cite each checked code claim with a numbered source listed under the reply` |

## Discovery

`.bee/LEADER.md` is byte-equal to `packages/bee/LEADER.md` today; `bee dev
regen` renders it. `routing-and-contracts.md` has five rendered copies
under `.agents/`, `.claude/`, `.claude-plugin/`, `.codex-plugin/` and
`.opencode/`, all written by the regen chain.

## Approach

Recommended path. ckf-1 edits the two homes and the rule index spoken
lines, then runs `bee dev regen` once.

Rejected: a hook that refuses history citations (a guard cannot tell a
rationale claim from a behavior claim); a new rule id (the two existing
rules own the behavior).

Risk map:
- a parity test reds on the changed spoken line / LOW / Lands in ckf-1 / `rule_index_parity`.
- a new path in the prose does not resolve / LOW / Lands in ckf-1 / `pointer_integrity`.

Waves: one cell.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"documentation","classification":"required","role":"docs","reason":"ckf-1 edits doctrine prose and runs the regen chain."},
    {"stage":"implementation","classification":"not-applicable","role":"code","reason":"No code changes."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Docs cell; parity and pointer tests are its proof."},
    {"stage":"knowledge-consolidation","classification":"not-applicable","role":"writer","reason":"ckf-1 owns the rule-index edit and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"No release unless the owner asks."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Plan check runs inline at this size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Inline check at this size."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Inline check."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Inline check."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline check."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline check."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI changes."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | the citation and knowledge-first rules name the source order | Owner correction 2026-11-15 | `.bee/LEADER.md` after regen carries the source-order sentence | every host gets it on its next refresh after a release |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ckf-1 | Cite the knowledge concept before history | `packages/bee/LEADER.md`, `routing-and-contracts.md`, rule index concept, regen outputs, release manifest, delivery | — | `.bee/LEADER.md` says a how-it-works claim cites the knowledge concept and history is cited for why | `cargo test … --test rule_index_parity --test pointer_integrity --test instruction_laws` green, `bee dev release-manifest --check` |

```json
[
  {
    "id": "ckf-1",
    "feature": "cite-knowledge-first",
    "title": "Cite the knowledge concept before history",
    "lane": "small",
    "role": "docs",
    "deps": [],
    "decisions": ["D1", "D2", "eaddc44a-abeb-4ee0-99c4-3f8f77850b49", "0124ee8b-f135-431b-bff8-121ad5cae3ff"],
    "affects_skills": ["skills/bee-hive/references/routing-and-contracts.md"],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
    "files": [
      "packages/bee/LEADER.md",
      ".bee/LEADER.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      ".agents/skills/bee-hive/references/routing-and-contracts.md",
      ".claude/skills/bee-hive/references/routing-and-contracts.md",
      ".claude-plugin/skills/bee-hive/references/routing-and-contracts.md",
      ".codex-plugin/skills/bee-hive/references/routing-and-contracts.md",
      ".opencode/skills/bee-hive/references/routing-and-contracts.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/history/cite-knowledge-first/delivery.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/cite-knowledge-first/CONTEXT.md",
      "packages/bee/LEADER.md",
      "skills/bee-hive/references/routing-and-contracts.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"
    ],
    "action": "Edit three homes, short sentences, no new rule id. (1) packages/bee/LEADER.md: in the CLI-only state bullet, change the end 'docs/history/<feature>/CONTEXT.md holds the locked ones — cite them, never reinterpret them.' so it says history holds the locked ones, cite it for a locked decision and its reason, never reinterpret it, and current behavior is cited from its docs/knowledge concept (D2). In rule block agents-knowledge-before-code add one sentence: the same search comes first when a turn answers how something works, before docs/history is opened for that answer (D2). In rule block agents-verify-before-agreeing, after the Sources sentence, add the source order (D1): a claim about how something works now cites the docs/knowledge concept that holds it, then code file:line; docs/history is cited only for why a decision was made; a decision hit leads to the concept that cites its D-number first. Keep every rule marker line unchanged. (2) skills/bee-hive/references/routing-and-contracts.md, section 'Claim labels and pushback': add a 'Source order' paragraph after 'Claim labels' with the D1 order, the decision-hit step, and a short example that cites a docs/knowledge path for a behavior claim and a docs/history CONTEXT.md D-number for a reason. (3) docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md: extend the spoken line of agents-verify-before-agreeing with the source order, and the spoken line of agents-knowledge-before-code with the how-it-works answer; cite cite-knowledge-first D1/D2, never raw decision ids. Then run .bee/bin/bee dev regen once; it re-renders .bee/LEADER.md and the five rendered copies of routing-and-contracts.md. If regen touches .claude/skills paths that are hub-served aliases, restore them with git restore before commit. Write docs/history/cite-knowledge-first/delivery.md (what changed, proof, known limit: bee decisions search still names no concept, p-eb824507). One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test instruction_laws && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json",
    "must_haves": {
      "truths": [
        "the citation rule says a how-it-works claim cites the docs/knowledge concept first and docs/history only for why",
        "the knowledge-first rule covers a turn that answers how something works before docs/history is opened",
        "the LEADER.md history line scopes its cite instruction to the locked decision and its reason",
        ".bee/LEADER.md is byte-equal to packages/bee/LEADER.md after regen"
      ],
      "artifacts": [
        {"path": "packages/bee/LEADER.md", "substantive": "source order in two rule blocks and the scoped history line"},
        {"path": "skills/bee-hive/references/routing-and-contracts.md", "substantive": "Source order paragraph in Claim labels"}
      ],
      "key_links": [
        "Claim labels stays the full form of agents-verify-before-agreeing",
        "the rule index spoken lines match the rule blocks"
      ],
      "prohibitions": [
        "No new rule id",
        "No hook or guard change",
        "No hand edit of rendered copies"
      ],
      "open_choices": [
        "Exact wording",
        "Example paths in the Source order paragraph"
      ]
    }
  }
]
```
