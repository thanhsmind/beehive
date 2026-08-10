---
type: bee.pattern
title: A scan set built from the git index crashes the very gate it feeds
description: "A coverage gate listed its inputs with git ls-files — the index, not the working tree — so a deferred deletion left it reading a file that no longer existed; the ENOENT killed the process and took every assertion behind it into silence."
tags: [verification, coverage, git, scan-set, fail-loud, validation-diet]
timestamp: 2026-07-28
bee:
  id: pattern-20260728-a-scan-set-from-the-git-index-crashes-the-gate-that-guards-it
  lifecycle: active
  sources: ["validation-diet cell vd-13 (deriveScanSet existence filter, scripts/tests/test_doctrine_parity.mjs:136, trace .bee/cells/vd-13.json, commit 656407c9, 2026-07-28)", "scripts/tests/test_portable_paths.mjs:23-34 (same defect, still live at close)", "scripts/tests/test_installers_e2e.mjs:191-199 (same source, correctly guarded)", docs/history/learnings/20260728-validation-diet.md L2]
  polarity: pitfall
  critical: false
---

# A scan set built from the git index crashes the very gate it feeds

`test_doctrine_parity.mjs` derived its scan set from `git ls-files`, then
`readFileSync`'d each path. A `wave-barrier` had deferred a mirror-deletion
regeneration, so the index still named files already gone from disk. The
first read threw `ENOENT`, the process died, and **every assertion behind that
point never ran** — the suite reported a crash where it should have reported a
verdict.

That is the part worth internalizing. A failing coverage gate tells you one
thing is wrong. A crashing coverage gate tells you nothing at all, and looks
identical to "the suite is broken, deal with it later." The blast radius of
the missing existence check was not one assertion; it was the whole file.

`git ls-files` is a listing of the **index**, which drifts from the working
tree in both directions: a deleted-but-unstaged file is still listed, and a
real untracked file is not. The first direction crashes a reader; the second
opens a silent hole. `test_portable_paths.mjs:23-34` has the identical source
and no existence filter — it does not crash only because its loop analyses
path strings and never touches the filesystem, so its version of the bug is
the quiet one: a staged-but-unindexed file with an illegal Windows character
passes green. `test_installers_e2e.mjs:191-199` reads from the same command
and is correct, because it guards with `fs.existsSync` before use.

**Rule.** A gate's scan set must come from what exists, not from what a
record claims exists — and when a listing source can disagree with reality,
the disagreement is filtered at the boundary, once, rather than trusted at
every use site. Prefer walking the tree; where a git listing is genuinely the
right source (tracked-ness is the property under test), union it with
`git status --porcelain` so untracked reality is included, and filter through
`existsSync` before any read. The generalisation past git: any gate whose
input list comes from a cache, an index, a manifest, or a previous run owes
the same reconciliation, because a gate that dies on its own stale input is
strictly worse than no gate — it consumes the budget of a check while
delivering the signal of a skip.

See also [[pattern-20260722-a-scan-scope-set-from-assumption-passes-green-while-hiding-the-very-bug]]
and [[pattern-20260723-a-coverage-gate-derives-its-ground-truth]]: those cover
a scan scope that is *wrong*; this one covers a scan scope that is *fatal*.
