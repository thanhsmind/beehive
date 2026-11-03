---
type: bee.pattern
title: Pin a settled contract with a test, never a provisional one
description: A test asserting a rule is present turns that rule's own agreed revert condition into a build failure. Before pinning a rule, check whether the team already agreed on what would retire it.
tags: [tests, decisions, triggers, doctrine, reversibility]
timestamp: 2026-09-02
bee:
  id: pattern-20260902-pin-a-settled-contract-with-a-test-never-a-provisional-one
  lifecycle: active
  areas: [decision-memory, workflow-state]
  sources: ["verification-contract-parity, 2026-09-02 — decision 29b853d8", "CONTRACT_UNSETTLED refusal at packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs"]
  polarity: practice
  critical: false
  evidence: exercised
  evidence_ref: "The claim door refused cell vcp-1 with CONTRACT_UNSETTLED because it cited a decision carrying an open revisit trigger; the cell was re-scoped to cite only untriggered decisions and the assertion was dropped. 2026-09-02."
---

# Pin a settled contract with a test, never a provisional one

A test that asserts a rule is present makes that rule expensive to
remove. That is the point, when the rule is settled. It is a trap when
the rule was adopted provisionally.

A provisional rule usually arrives with its own exit written down: *we
will do X, and if evidence Y shows up, we drop it.* Pin its presence
with a test and you have quietly made Y unactionable — the day the
evidence arrives, removing the rule turns the build red, and the person
holding the evidence now has to argue with a test instead of with the
decision. The safeguard defends the thing it was supposed to let go of.

**What happened.** A feature added a doctrine rule and, in the same
breath, recorded a falsifier for it: after two features had a chance to
use it, check whether anything actually did, and revert if not. That
falsifier was registered as a trigger on the decision. A later cell
proposed a parity test asserting the rule's line was present. The claim
door refused it — `CONTRACT_UNSETTLED`, because the cited decision
carried an open revisit trigger.

The refusal read like paperwork and was not. The test would have made
the revert a failing build.

**The rule.** Before writing a test that asserts a rule exists, ask what
would retire that rule and whether anyone has already written it down.

- **Settled** — no open revisit condition. Pin it. A test is the right
  owner for a rule that is meant to stay.
- **Provisional** — a recorded trigger, a falsifier, an explicit trial.
  Do not pin its presence. Its consequences can still be tested; its
  existence must stay cheap to undo.

Where a system tracks deferred conditions, this is mechanical: a
decision carrying an open trigger is provisional by definition, and a
guard can refuse the citation before the test is written. Where it does
not, the question has to be asked out loud.

And say it in the test. A header comment naming what is deliberately
*not* pinned, and why, is what stops the next person from helpfully
adding the missing assertion.
