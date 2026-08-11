# Grooming Reference

Counting rules, hunt checklists, and the outcome record. The cycle itself lives in SKILL.md.

## Entropy Computation

```
ENTROPY SCORE = orphaned cells ×10 + unverified cells ×5 + stale decisions ×5
              + stale specs ×5 + backlog-without-outcome ×2 + stale work ×3
              + broken tools ×8, cap 100
```

Counting rules per term (all from `.bee/` records — never guess):

| Term | Count | Source |
|---|---|---|
| orphaned cells | open/claimed cells whose feature is no longer the active feature and has no handoff pointing at them | `bee cells list` vs the state and handoff records |
| unverified cells | claimed cells with no recorded verify result | cell files |
| stale decisions | active decisions citing files/paths that no longer exist, or contradicted by current code | `bee decisions active` + spot-check the citations |
| stale specs | areas whose behavior changed after their spec/concept was last updated, or that changed with no spec at all — map cells to areas by files touched, and count git commits or uncommitted changes in an area's own paths too (vibe edits outside the chain count); each area once | capped cell files, `git log --since=<updated> -- <paths>`, `git status --porcelain`, vs the area's state-layer doc |
| backlog-without-outcome | machine-backlog entries older than 30 days with no matching outcome entry — product-intent PBI rows are never entropy | `.bee/backlog.jsonl` |
| stale work | reservations past TTL and never released; a handoff older than 7 days | `.bee/reservations.json`, `.bee/HANDOFF.json` |
| broken tools | vendored helpers that error on invocation; hook crash entries since the last audit | run helpers with `--json`, read the log |

Bands: 0 = perfect · 1–25 healthy · 26–50 attention · 51–100 action required.

`broken_tools` and any bee-lib / vendored-helper bug are **harness health** — surface them in one plain line and route upstream to bee; they NEVER become project kill proposals. `stale_specs` is the one term about the user's own docs — carry it into the project hunt.

**Coverage read-out** (informational, never scored): report `specs: <N areas specced> / <M behavior-bearing locations>` — a location is behavior-bearing when its one-liner describes observable behavior, not assets or config. Low coverage is a backfill program, not week-to-week debt.

**Trend:** append an `entropy-audit` entry to the machine backlog after each audit — score, per-term breakdown, and the direction vs. the previous entry — so the next run has something to compare against.

## Hunt Checklists

Every check below hunts the project's own files only, written up in plain project language — the scope and language rules live in SKILL.md. Lockfiles join the excluded set here.

**Dead code / unused exports** — for each suspect symbol: grep every reference (imports, dynamic `import()`, `require`, string-built paths, config and registry files, reflection); check the public API surface (package entry points, exported types); check test-only usage (test-only = candidate to *move*, not to keep). No reference anywhere = candidate. Any doubt = not a candidate.

**Stale docs vs code** — compare README/docs claims (commands, file paths, flags, versions) against reality by running or resolving them; each mismatch is a candidate. Fix the doc, not the code — unless the code is the bug. Judging which side is wrong: `.bee/expertise/documentation.md`.

**Stale, missing, or duplicated area truth** — for each stale-specs hit, propose a documentation sync cell (tiny) that merges the missed behavior deltas into the area's spec or concept; a git-drift hit (files changed, no cell) gets the same cell, and "no behavioral delta — spec confirmed current" is a valid cheap outcome. An area with shipped behavior and no spec at all gets a harvest cell (small — it may need user interview time).

Then hunt **duplicates**: two documents whose pointers overlap on the same surface (including `-v2`/`-new`/date-suffixed names) — propose a merge cell that consolidates into the older stable name and deletes the fork. Two documents describing one area is worse than one stale document, because a reader cannot tell which is true; and a spec staler than the behavior it describes is worse than no spec, because an agent trusts it and acts on the old behavior. Also spot-check the reading map against reality (paths exist, one-liners still true, exactly one spec per surface), and flag **misfiled artifacts** — scripts, exports, CSVs, or survey folders parked in the spec tree pollute coverage counting; propose a tiny move-cell that relocates them and fixes the references.

Where the repo keeps a knowledge bundle instead of a spec tree, run its own consistency check (`bee knowledge check --json`) and translate each finding into one plain-language line — never paste the raw output — then flag areas whose files changed with no concept claiming authority for that subject. Same cells either way: sync, harvest, merge, all routed through bee-capturing rather than a raw doc edit, because the never-invent rules live there.

**Fresh Session Test** — five minutes per audit, answering five questions from repo artifacts alone. This catches system-of-record decay the entropy formula cannot see: a spec can be fresh and the repo still unanswerable to a cold start.

| Question | Answered by | Fix when unanswerable |
|---|---|---|
| What is this system? | the system overview (or the bundle's generated index) | bee-capturing **bootstrap** — a provable-facts skeleton for the missing map; regenerate the index in bundle mode |
| How is it organized? | the reading map (or the bundle's area index) | same as above |
| How do I run it? | the recorded setup/start commands | run the command detector, confirm the candidates into config |
| How do I verify it? | the recorded test/verify commands — run them, don't just read them | same as above |
| Where are we now? | `bee status --json` | self-answering — the command is the artifact |

A probe finding is filed with its one-command fix named, never as an open-ended "document the project".

**Suite rent audit** — from the verify logs, list every suite by when it last went red for a REAL defect (environment and pre-existing reds don't count). Suites past ~6 months of unpaid rent are demotion candidates: propose moving them off the local hot path to the CI/nightly tier, one recorded decision per demotion, never a silent delete. Report one line: total suites, tenants in arrears, slowest five by wall time.

**Friction clusters** — group friction strings from capped cells and friction backlog entries by module or topic; 2+ hits on the same thing is a cluster worth a proposal. Tally by layer (spec | context | environment | verification | state) and report one line: the largest layer is this cycle's bottleneck, and fix proposals aim there first.

**Backlog drift** — reconcile the product backlog against reality: an in-flight item with no matching feature directory, a shipped feature with no item, two items telling one story. Each is a tiny correction cell that ends by re-rendering the generated backlog view. Reconcile against the recorded counts; never recount by hand.

**TODO/stub debris** — grep `TODO|FIXME|HACK|XXX|not implemented|placeholder`; each hit is either a real backlog item (file it with predicted impact) or debris (kill candidate). Never leave it as a comment-shaped promise.

**Unverified verify-commands** — run each distinct verify command from open cells in a dry form; one that cannot run (missing script, renamed target) makes its cell unexecutable — propose a fix cell.

**Superseded-but-still-cited decisions** — for each superseded decision, grep code comments, docs, and plans for its wording; stale citations are candidates.

**Architecture lens** — hunt structural debt only where the code is
alive: take the hot spots from recent `git log --oneline` (files
changing often, or the area the user named) — a shallow module nobody
touches costs nothing and is not debt. In those spots look for:

- a concept smeared across several modules — every change to it touches all of them
- a shallow module — its interface restates its implementation (a pass-through, a wrapper adding no decision)
- pure functions extracted "for testability" that stranded their logic far from the one place using it
- a seam that leaks — callers reach around the interface to internals, or two modules share knowledge the interface pretends to hide

Judge every suspect with the deletion test and the adapter-count rule
(`.bee/expertise/architecture.md`): delete it mentally — complexity
reappearing at call sites means it earns its keep; vanishing means
pass-through. Each finding is a deepening proposal (merge, inline,
re-cut the interface), rides the normal kill-proposal shape at lane
small, and carries a confidence badge: **strong** (deletion test
positive, hot spot, tests reachable through the interface) /
**worth exploring** (signal present, evidence partial) /
**speculative** (pattern only — name it, never lead with it). Propose
the re-cut, never design the new interface in the report — interface
design happens after the user picks, in planning.

## Proposal report

When a proposal round has three or more candidates, or any
architecture-lens finding, render it as one Markdown file the user can
read visually: `docs/history/grooming/<date>-proposals.md`. Per
candidate, one card — what dies or deepens, `file:line` evidence, pain
/ predicted impact, risk lane, confidence badge, and for structural
candidates a small Mermaid before/after of the module shape. End the
file with exactly ONE top recommendation and its single reason. Present
it as a viewer URL when the project has `doc_viewer` configured or the
harness carries a markdown viewer; the bare path otherwise. The report
is presentation only: approvals stay per-candidate in the
conversation, and the file records the round — it is never edited into
a decision log.

**Slop patterns in recent diffs** — scan the last ~20 commits for:

- empty or log-only `catch` blocks that swallow errors
- redundant `return await` inside async functions
- dead flags: config or env switches with only one live branch
- copy-paste drift: near-duplicate blocks that diverged in one detail
- commented-out code kept "just in case"
- defensive re-checks of conditions the caller already guarantees
- stub handlers returning fixed values in non-test code

## Proposal Record

Each candidate is recorded in the machine backlog as a `kill-proposal` carrying what dies, its pain, its predicted impact, the risk lane, and the evidence of non-use or staleness (with `file:line`). Present them one question at a time — candidate and evidence, your recommendation with one reason, and the approve / keep / defer choice. Approval covers exactly the candidate asked about.

A promote-to-check proposal (a repeated finding becoming a grep, lint, or guard) rides the same shape; changing what a *gate* blocks is never tiny.

## Outcome Template

After the kill cell caps — or fails — record a `kill-outcome` against its proposal: what died, what you predicted, what actually happened including the surprises, and the cell that did it.

```
bee backlog add --type kill-outcome --title "<what died>" \
  --detail "<predicted vs actual, incl. surprises>" --feature <feature>
```

A prediction that missed is the most valuable thing the pass produced — hand it to bee-capturing as a learning candidate.
