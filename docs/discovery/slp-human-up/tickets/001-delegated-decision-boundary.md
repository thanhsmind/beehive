---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Decision 83baf03f fixes the delegated-decision tier's four criteria but not
its bee-term boundary. Three parts, all user calls: (a) which queued-ask
classes may the supervisor auto-decide — only asks the away-queue already
holds, or also live waiting-on questions of kind `question`? (b) how does the
tier compose with c706053e's silence-is-consent — is a delegated decision the
ACTIVE arm of the same queue (one mechanism, two arms) or a separate record?
(c) what happens to 322695d6's observer-only wording — partial supersession
now, or does the tier land as a new role beside the observer?

## Answer

Resolved 2026-08-29 by the user, in the opposite direction of the draft: the
delegated-decision tier is DROPPED. The supervisor is assumed to run on a
cheap model, so it observes, connects, and packages questions for the human —
it decides nothing. (a)/(b)/(c) all dissolve: no auto-decidable ask class
exists; c706053e's silence-is-consent stays as the deterministic-layer
exception it already is (its timeout never belonged to the model); 322695d6's
observer-only contract stays intact, no supersession needed. Decision
704b691c supersedes 83baf03f. Companion decision 58796a73: the human working
directly with one project's lead is a first-class channel — the supervisor is
never a mandatory middleman.
