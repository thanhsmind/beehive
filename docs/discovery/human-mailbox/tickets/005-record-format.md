---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

What exactly is the file that lands in `.bee/human-mailbox`, given D1
(bee owns the data, waggledance renders it) and D2 (a required
one-sentence subject)? Candidates:

(a) one markdown file per letter, typed YAML frontmatter carrying the
    structured contract (subject, timestamps, run/project/feature,
    status, and per-item arrays: what changed, files, commit, proof,
    departure + reason), body carrying the human prose;
(b) one JSON file per letter, with the frontend responsible for all
    presentation;
(c) a JSON record plus a rendered markdown twin;
(d) an append-only `inbox.jsonl` stream plus a rendered file per letter.

Also open inside this ticket: the file naming scheme, and whether one
letter maps to one run or one night.

➡️ Recommendation: (a) — one file that is simultaneously the data and
the letter. The human can read it raw in an editor with no tooling, the
waggledance inbox parses the frontmatter for its list rows and structured
panes, and there is exactly one source of truth. (c) and (d) buy
structure at the cost of two artifacts that can drift apart; (b) breaks
the human's ability to just open the file.

## Answer

(a) One markdown file per letter with typed YAML frontmatter — logged as
**D3 (1b079912)**. Frontmatter is the machine contract (subject, run,
project, filed_at, items[] with what/files/commit/proof/departure,
needs_you[]); body carries the human prose. One file is both the data and
the letter: no JSON twin, no separate index stream.

Still open inside this area and carried forward as fog: the file naming
scheme, and whether one letter maps to one run or one night.
