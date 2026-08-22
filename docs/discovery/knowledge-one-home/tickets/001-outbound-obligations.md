---
type: grilling
status: closed
claimed-by:
blocked-by: none
---

## Question

When a rule settles (capture, decision log), what is the exact outbound
list bee demands before the cell can cap — skill text, area spec, code,
test, hook manifest — and who computes it: the agent by hand, or bee
from the area's declared ownership? Leaning (user, round 1): bee
enforces at write time; cap refuses when the list is incomplete without
a recorded reason.

## Answer

(user, 2026-08-22) Decision 27e55095:
- bee computes the update list from the home's `applied_at` list plus
  the area ownership map (D3); agent may add, never remove.
- Two kinds of home: discipline rules in AGENTS.md, mechanism rules in
  the area spec. Skills and help text hold a one-line restatement plus a
  pointer.
- `applied_at` names every restating or enforcing file — skill text,
  help strings in Rust source, generated payloads, tests.
- Rules carry an id; copies cite it; `knowledge check` flags id-less
  rule blocks, one id in two bodies, dangling `applied_at` targets.
- Capture stub for an area that owns a skill must answer "skill changed"
  or "not, because ...".
- Why the status quo fails: copies use different words (text search
  misses them), hide in help strings and generated JSON, and "done" had
  no N to count against.

