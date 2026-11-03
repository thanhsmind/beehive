---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Rung 1 of the ladder (executing lead → spec owner A) must be typed, recorded,
and door-enforced (fbf06b0d) with NO shared store (6f039742). Where does each
half live: B records its dissent in B's repo (existing `bee cells dissent`
teeth), but how does A learn of it, where is A's owed verdict recorded, and
which door on WHICH side refuses progress while the verdict is missing — B's
close/merge door (already refuses unanswered dissent), A's spec record, or
both?

## Answer

Resolved 2026-08-29 by the user, by dissolving the question's premise
(decision 8fea3561): repos record separately, exactly as bee works today, and
the question wrongly presupposed A needed its own notification channel. B
records its dissent locally with the existing dissent machinery; the ONE
cross-project entity — the waggledance supervisor's read-only rollup —
surfaces it as an attention item / packaged question; the reply comes back as
data over the existing dispatch transport and lands through B's existing
dissent-verdict verb; the doors stay B-local and unchanged. Nothing new is
built for rung 1 beyond what 12deaa34's transport already carries.
