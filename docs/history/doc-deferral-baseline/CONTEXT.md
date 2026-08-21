# Doc Deferral Baseline — Context

**Feature slug:** doc-deferral-baseline
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Quick
**Domain types:** RUN

## Feature Boundary

`bee close`'s doc-deferral door gains a tracked baseline of the deferral-shaped
lines that already exist, so it blocks only on lines that are new. The door's
file selection, its word list, its fenced-code exemption, and both of its
escapes stay exactly as they are; nothing else in `bee close` moves.

## Why

The door has fired five times — staging-optional, staging-lane,
uat-gate-before-merge, test-doctrine-text-sweep, auto-wait-mark — and every
flagged line on every occasion was prose *describing* deferral machinery, not
prose deferring work. Zero true positives.

The cause is a vocabulary collision no word list can fix: this repo's own domain
is deferral queues and triggers, so `defer`, `later` and `for now` are its nouns.
Combined with a file-scoped scan, a feature that edits one line of a long-lived
doc inherits every pre-existing match in it.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Keep the whole-file scan and the word list. Add a tracked baseline of the deferral-shaped lines that already exist; the door then blocks only a line absent from it. Identity is the line's **normalized content**, per file — never its line number. | Line numbers break the moment anything is inserted above; content identity survives edits elsewhere in the file. Two alternatives were rejected: scoping the scan to the feature's changed lines has no data to work from (`trace.files_changed` is a hand-typed path list, not a diff, and by close time the branch is already merged), and deleting the door was offered on the five-for-five record and declined. |
| D2 | The baseline is seeded **once per repo, by the door itself**, on the first run that finds no baseline file: it records what it flagged, passes, and writes the file. Every run after that enforces. Nothing is ever adopted automatically again. | Seeding on introduction makes this a migration, not a permanent hole. Doing it inside the door needs no new verb, flag, catalog entry, or `registry_payload.json` edit — which matters, because that file is generated in name only and must be hand-merged on every conflict. |
| D3 | The baseline file is **git-tracked**, beside `.bee/backlog.jsonl`. | An untracked baseline re-seeds in every clone and every fresh worktree, which would silently re-open the hole. `.bee/state.json` and `.bee/runtime/` are gitignored; `.bee/backlog.jsonl` and `.bee/cells/` are tracked — this belongs on the tracked side. |
| D6 | **Supersedes D2's scope.** The seed is REPO-WIDE: on the first run finding no baseline file, the door walks every markdown file under `docs/` and records every deferral-shaped line in the whole tree, then passes. Enforcement afterwards stays per-feature over the existing scan set. The seed **always** writes the file, even when it flags nothing. | The door's scan set is per-*feature*. A scan-set-wide seed freezes only the docs that one feature touched, so the next feature touching a different long-lived doc enters enforcement against an empty entry and eats every pre-existing line in it — the false positives return on a delay. And an absent file *is* the seed state, so skipping the write on an empty run means the first genuine deferral line ever added gets adopted instead of blocked. Freeze all existing debt once; then police only what each feature touches. |
| D7 | Every pre-existing door test runs in **ENFORCE** mode, with a baseline fixture covering nothing relevant, so each still discriminates. | A door test that runs in seed mode is vacuous by construction — the seed arm returns non-blocking regardless of what the scan found. Mutation-disabling the fenced-code exemption and the citation escape left two of the six tests green. |
| D5 | The seed writes only on a **real** `bee close` run, never on `--dry-run`. A dry-run that finds no baseline file reports the door as non-blocking and names what it would baseline and how many lines. | A dry-run that writes breaks the one property it has. A dry-run that stays silent is worse the other way: it would report a blocking door a real close sails straight through — a dry-run lying about the thing it exists to predict. |
| D4 | The door's two existing escapes are unchanged and are the only way out for a line the baseline does not cover: cite a registered trigger inline, or log a `doc-deferral`-tagged decision naming the feature. No third escape. The baseline is never hand-edited to silence a new line. | A false positive after the seed is exactly what those two escapes were built for, and both already have tests. A hand-edit path would turn a frozen migration record into an ordinary suppression file — which is how lint baselines rot. |

### Agent's Discretion

- The baseline file's exact name and serialization. Constraints: under `.bee/`,
  git-tracked, and stable enough that two runs over an unchanged tree produce a
  byte-identical file (no timestamps, sorted keys).
- What "normalized content" means precisely — at minimum trim surrounding
  whitespace. Constraint: whatever normalization is chosen must be applied
  identically when seeding and when matching, and a test must prove a baselined
  line still matches after unrelated lines are inserted above it.
- Whether the baseline stores raw line text or a hash of it. Constraint: a human
  reading the file must be able to tell which line it refers to.

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1100-1157` — `build_doc_deferral_door`, the whole door. Calls `doc_deferral_scan_files` (`:1048`) for files, `matches_deferral_prose` for lines, `line_trigger_ids` (`:1066`) + `trigger_registered` for the citation escape, `has_doc_deferral_decision` (`:848`) for the decision escape.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1104-1113` — the read + per-line loop, including the `in_fence` toggle that is the only exemption today. The baseline check belongs beside it.
- `packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs:342-407` — `matches_deferral_prose`, the word list (`defer`/`defers`/`deferred`/`deferring`, `later`, `for now`, `revisit when`, `revisit if`). **Unchanged by this feature.**
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1388-1403` — `feature_touched_files`, reading each capped cell's `trace.files_changed`. **Unchanged.**

### Established Patterns

- Atomic JSON writes go through `crate::fsutil::write_json_atomic`; reads through `read_json`. Follow those rather than `std::fs` directly.
- `close.rs:1041-1042` carries a stale comment claiming "close spawns no git today" — untrue since `commit_close_bookkeeping` (`:2326`). Not this feature's to fix, but do not trust it.

## Canonical References

- Decisions `41e796f3` (D1), `7311c427` (D2), `5d880f8b` (D3/D4).
- Backlog rows: "bee close's doc-deferral door must scan the lines a feature changed, not every line of a touched doc" and "Make the doc-deferral door scan changed lines, not whole files" — both superseded in approach by D1, and both should be answered by this feature.

## Outstanding Questions

### Resolve Before Planning

None.

## Deferred Ideas

- Scoping the scan to a real changed-line set. Genuinely more correct, but it
  needs a line-level diff `bee close` cannot cheaply obtain today. If the door
  ever gains a reliable baseline sha, revisit.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
