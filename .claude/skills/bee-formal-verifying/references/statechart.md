---
name: statechart
description: Draw a Mermaid stateDiagram-v2 by hand from a TLA+ model and TLC output in the reader's words, mark rule-breaking states, and replay each bug path on the real code.
---

# Statechart — reading TLA+ models and TLC traces

Use this reference when building a TLA+ model or reading TLC output to explain system behavior and counterexamples to a reviewer. You draw the Mermaid `stateDiagram-v2` by hand from the model and TLC trace; no script or external renderer is required.

## 1. Write the TLA+ model for clear traces

- **Rules as `INVARIANT`s:** express every rule the reader agrees to as a TLA+ `INVARIANT`. If the model lacks a variable needed to state a rule, add that variable. Never weaken an invariant to make a model pass.
- **Named actions in `Next`:** write `Next` as a disjunction of named actions (for example, `ClaimFile \/ ClaimCellUnderLock \/ CloseCellUnderLock`). If `Next` is a single anonymous expression, TLC labels every step `Next` in traces.
- **Explicit end state:** a state with no outgoing transition triggers TLC's deadlock check (`NoDeadlock` / `CHECK_DEADLOCK`). For valid terminal states, add a stuttering `Done` action that is enabled only in those end states and keeps `UNCHANGED vars`.
- **Small bounds:** keep constants small (or add a `CONSTRAINT` / `ACTION_CONSTRAINT`) because TLC continues exploring past invariant-violating states and an unbounded model can run indefinitely.

## 2. Draw the Mermaid `stateDiagram-v2` in the reader's words

- **Reader-visible labels:** label each state with what an operator or user sees or cares about (such as `Claimed, claim file present`), not raw TLA+ variable names. Label each arrow with the observable event or outcome (such as `session closes cell`), not the internal TLA+ action name. When a condition decides where an arrow goes, put it in brackets (for example, `claim [when open]`).
- **Mark rule-breaking states:** give any state that violates an invariant a marked class (such as `classDef breaks fill:#fdd,stroke:#c00` and `class <state> breaks`). If a merged chart state represents some model states that pass and some that break, mark it `mayBreak` and explain in the legend or handoff which rule breaks and under which condition.
- **Keep it around 20 arrows:** a denser chart is hard to read. Merge model states that behave identically for the reader, and omit noisy internal steps whose return changes nothing. Hold three rules when simplifying:
  1. Never merge two states that break different rules, or the reader cannot tell which rule fails where.
  2. Keep every chart state reachable from a start state and keep at least one outgoing arrow from every non-terminal state.
  3. Name any omitted steps or merged states in the handoff.

## 3. Replay every bug path on the real code

TLC checks the model, not the implementation directly. Before reporting a counterexample as a product bug:

1. Walk each step of TLC's shortest counterexample trace against the real source files, one step per action.
2. Prove the bug path on the real code with a test or a live reproduction when practical.
3. If any step in the trace cannot happen in the real code, treat the failure as a **model error** until shown otherwise, fix the model to match the code, and re-run TLC.
4. If every step can happen in the real code, report the bug with the violated rule, the plain-word step sequence, and the harm in one sentence.
