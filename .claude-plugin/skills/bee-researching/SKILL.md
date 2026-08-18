---
name: bee-researching
description: >-
  Evidence-labeled research into unfamiliar, ambiguous, or version-sensitive territory. Use when the user asks to research a topic, library, or approach; when planning discovery lands on L2 or deeper; before high-risk work in a repo with no precedent for it; or when the request names an external repo or source to distill or bring in (triggers: "xia", "distill from", "port from", "like how X does it", "mang feature về", "học từ repo X"). Not for locking product decisions, proving feasibility, or writing code.
metadata:
  version: '0.3'
  ecosystem: bee
  dependencies:
    web-docs-search:
      kind: capability
      missing_effect: degraded
      reason: Checks current official documentation version-aware; absent, docs claims degrade to Inference and travel as open questions into planning's shape gate.
    upstream-pattern-research:
      kind: capability
      missing_effect: degraded
      reason: Inspects public repositories for proven patterns; absent, the upstream step degrades to direct public-repo reading, never a silent skip.
---

# Researching — return with a map, not honey

## Order is the protocol

Local evidence comes first; web research before it is a red flag, not a
shortcut. Full step rules: `references/research-protocol.md`.

1. **Stack ledger** — classify the repo and map languages, frameworks,
   and installed versions from real artifacts (manifests, lockfiles,
   configs, tests) — never from folder names, branding, or memory.
2. **Local reuse** — search feature-adjacent code, tests, config, docs:
   what exists, what is reusable, which extension points are open, what is
   genuinely missing — "missing" needs code, config, docs, tests checked.
3. **Upstream patterns** — the framework repo, the library repo,
   official starters: reusable proof, not inspiration.
4. **Current official docs** — version-matched to the repo. Precedent
   beats research: when local behavior and docs disagree, local
   behavior is current truth — record the mismatch.

Depth mirrors the need: quick (one API or behavior confirmed), standard
(all four steps — the default), deep (cross-cutting or version-sensitive).

## Evidence labels

Every non-trivial claim carries one, never blurred:

| Label | Meaning |
|---|---|
| `Local` | proven from this repository's files or command output |
| `Upstream` | observed in a public repository or official starter |
| `Docs` | stated by official, version-matched documentation |
| `Inference` | concluded from the above; not directly observed |

A capability gap (no web search, no repo browsing) degrades a step, never
skips it silently: affected claims become `Inference`, open questions for planning.

## Recommendation ladder

Lightest credible path wins; each skipped rung needs a stated reason:

1. **Reuse** existing local functionality.
2. **Built-in** framework/library capability at the installed version.
3. **Adapt** a proven upstream pattern that fits this repo.
4. **Build** from scratch — only with rungs 1–3 rejected for cause.

Rung 3 is the port protocol's territory: a request that names an
external repo or source to distill or bring in runs
`references/port-protocol.md` — the dependency matrix, cross-cutting
sweep, and challenge framework that turn Adapt into evidence, not
inspiration.

State why the chosen rung beats the next-best, and what evidence would
change the answer. Finish before recommending; ask one targeted question
only when paths differ materially in behavior, risk, or migration cost.

## Output

- **In-chain** (from planning discovery): findings merge into the
  feature's approach — no separate file. Version caveats and
  `Inference`-only claims become open questions for the shape gate.
- **Standalone**: write `docs/history/research/<topic-slug>.md` from
  `references/research-brief-template.md`, lead with the Bottom Line,
  and suggest the next step — bee-shaping if the topic is becoming a
  feature, bee-planning if scope is already clear.
- Flag a genuinely new first-principles finding for bee-capturing.

## Hard rules

- Research only: no source edits, no cells, no code "just to try it" —
  that is a spike, a different lane.
- Locked decisions win: a finding that contradicts one is noted with
  its evidence; superseding it is the user's move.
- Headless: `bee-hive` ("Headless") governs.

## References

| File | When to load |
|---|---|
| `references/research-protocol.md` | step rules in full, tool roles, ask-when-it-matters criteria |
| `references/research-brief-template.md` | standalone brief structure |
| `references/port-protocol.md` | the request names an external repo or source to distill (`xia`) or port — source manifest, dependency matrix, cross-cutting sweep, challenge framework |
