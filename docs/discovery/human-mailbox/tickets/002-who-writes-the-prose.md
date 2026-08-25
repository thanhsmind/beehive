---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Who authors the human-readable sentences: (a) the worker/orchestrator
writes one plain sentence at the moment of each event, while the work is
still in context; (b) a composer reads the stored records cold at the
end of the run and writes the whole letter; (c) both — sentences written
at the moment, an end-of-run pass only orders and joins them.

➡️ Recommendation: (c), with the rule that the end-of-run pass may
reorder, group and drop, but never invent a fact no stored entry
carries. Cold composition from traces is where a summary starts
hallucinating; writing at the moment costs nothing because the context
is already loaded.

## Answer

(c) Both — logged as **D8 (1c7a9d87)**. Sentences are written at the
moment of each event, while the work is still in context; the end-of-run
pass may reorder, group and drop, and may never state a fact no stored
entry carries.
