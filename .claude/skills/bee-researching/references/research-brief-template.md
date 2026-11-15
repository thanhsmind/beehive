# Research Brief Template (standalone mode)

Written to `docs/history/research/<topic-slug>.md`. Concise, with explicit evidence boundaries. In-chain runs do NOT use this file — their findings merge into the feature's approach.

```markdown
---
artifact_contract: bee-research/v1
topic: <topic-slug>
depth: quick | standard | deep
date: <yyyy-mm-dd>
---

## Bottom Line

- Recommendation (ladder rung): reuse | built-in | adapt-upstream | build
- Why this is the lightest credible path:
- Why the next-best rung lost:
- Confidence (0–100%):
- Suggested next step: bee-shaping | bee-planning | none

## Repo Snapshot

- Repo type / primary languages / runtimes:
- Frameworks and detectable versions:
- Relevant packages, services, tools:
- Constraints or workflows that shape the answer:

## Question & Assumptions

- What was asked:
- What success appears to mean:
- Assumptions still needing confirmation:

## Findings

### Local
- Existing functionality, extension points, conventions worth preserving:
- What can be reused / what is genuinely missing:

### Upstream
- Repositories inspected; patterns worth modeling; fit with this repo:

### Docs
- Sources checked (version-matched vs latest-stable):
- Built-in capabilities that already cover the need:
- Caveats, deprecations, migration notes:

### Inference
- Conclusions drawn from the above, not directly observed:

## Risks, Unknowns, Follow-Ups

- Technical risks / evidence gaps / version uncertainties:
- Open questions (for the user, or as proof obligations for bee-planning's shape gate):

## Source Pack

- Local files read:
- Upstream repos/pages checked:
- Docs pages checked:
```

Rules: every non-trivial claim sits under its evidence label — never blurred. A finding that contradicts a locked decision is noted here with evidence; it never silently overrides the decision.
