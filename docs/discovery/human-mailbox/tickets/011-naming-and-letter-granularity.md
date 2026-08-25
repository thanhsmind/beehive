---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Two things D3 left open. First: does one letter map to one *run* or to
one *night*? Two herding runs between midnight and morning could file two
letters or be folded into one. Second: what is the file named — a
timestamp, a timestamp plus a slug of the subject, or an opaque id with
the subject only in frontmatter?

The two are linked: per-night folding needs a name that is not
run-derived.

➡️ Recommendation: one letter per run, named
`<UTC-timestamp>-<short-run-slug>.md`. A run is the unit the human
actually reasons about ("what did last night's herding run do"), folding
two runs hides that one of them died, and a timestamp-led name sorts
correctly in any file listing. The subject stays in frontmatter, where
D2 put it, so a renamed file never contradicts its own letter.

## Answer

One letter per run, file named `<UTC-timestamp>-<short-run-slug>.md` — logged as **D11 (349f25d8)**. The subject stays in frontmatter.
