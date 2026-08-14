# Checklist examples — writing bee skills (handbook guide)

Worked examples for the SKILL.md checklist items in the body. Full
persuasion-principle catalog lives in `pressure-test-template.md`'s
neighbor table below.

## Description trap

A workflow summary in the description makes Claude follow the description
and skip the skill body. Every time.

```yaml
# ❌ BAD
description: Use when creating skills — run baseline test, write minimal skill, run tests

# ✅ GOOD
description: Use when creating a new bee skill or editing an existing one
```

## Dependency metadata style

Write `metadata.dependencies` as a mapping keyed by dependency id — never a
YAML array of objects (generic evaluators reject that shape). A bee skill's
one real dependency is the vendored binary; no Node runtime is involved:

```yaml
metadata:
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Every bee record is read and written through the vendored binary.
```

`missing_effect: unavailable` is right here, not `degraded` — without the binary
the skill cannot run at all. Reserve `degraded` for a dependency whose absence
costs a capability but still leaves the skill usable.

## Persuasion principles

Apply deliberately, matched to the rule being enforced:

| Principle | Implementation | Use For |
|---|---|---|
| **Authority** | "YOU MUST", "Never", "No exceptions" | Discipline-enforcing rules |
| **Commitment** | Ordered checklists, announce skill usage | Multi-step processes |
| **Scarcity** | "Before proceeding", "IMMEDIATELY after X" | Verification requirements |
| **Social Proof** | "Teams report...", "X without Y = failure. Every time." | Common failure patterns |
| **Unity** | "our skills", collaborative framing | Techniques, guidance |
