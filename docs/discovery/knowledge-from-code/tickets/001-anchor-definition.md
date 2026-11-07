---
type: grilling
status: open
claimed-by: none
blocked-by: none
---

## Question

What exactly is an "anchor" on an OKF area concept for freshness grading: the
backticked paths under its Pointers section, the `bee.sources` frontmatter
entries, an explicit new `bee.anchors` list, or a verify feature file name? One
answer, applied to every concept, so `knowledge check` can grade without
guessing.

Agent recommendation: a new explicit `bee.anchors` list of repo-relative paths
(code files, test files, verify feature files), seeded once from the existing
Pointers paths that resolve on disk; Pointers stays prose for the reader,
anchors is the machine contract.

## Answer

