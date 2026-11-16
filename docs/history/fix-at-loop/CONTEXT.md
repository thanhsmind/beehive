# Close the fix-at loop — Context

**Feature slug:** fix-at-loop
**Date:** 2026-11-16
**Shaping session:** complete (owner asked to review bee's five-layer lint flow against Lauren Tan's talk "All Code Is Slop (Except Mine)", then picked option A: close the loop)
**Scope:** Standard
**Domain types:** RUN | READ

## Feature Boundary

The talk's rule is: every correction becomes a deterministic check, and a
new rule comes with a cleanup of the old occurrences. bee records the layer
(`--fix-at`) and files a backlog row, then stops. Four measured gaps:

1. One mistake is filed once per feature. `mechanizable_mistakes` reads the
   closing run's reflections with no feature filter (`close.rs:376`), and
   `backlog_finding_exists` keys on (feature, layer, title)
   (`backlog.rs:435-443`). A run that closes 8 features files the same
   title 8 times; 84 of 129 fix-at keys in `bee feedback rank` have 2+
   entries, most of them copies.
2. A finding row has no status. 336 fix-at rows exist and none can say that
   a rule now guards it.
3. bee does not run its own advice. `expertise/static-rules.md` names
   `disallowed-methods` in `clippy.toml`; `packages/bee-rs` has no
   `clippy.toml`. The "plain strip_prefix on absolute paths" mistake (Windows
   8.3 short path) was filed 8 times; 25 `Path::strip_prefix` calls remain
   (21 under `src/`, 4 under `tests/`), measured with a probe clippy.toml.
4. `prose_load` counts instruction files but not the SessionStart preamble,
   the largest always-injected text.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | A fix-at finding's identity is (layer, title) across the whole backlog. `file_fix_at_rows` files a row only when no finding row with the same layer and title exists in any feature; the existing "(M already there)" count covers the skip. `bee backlog add` keeps its per-feature dedupe. | The title is the mistake's own words; the same words under another feature are the same mistake, re-read from the same run. |
| D2 | The feedback digest folds backlog finding rows with the same (layer, title) into one entry, keeping the earliest `ts` as `first_seen`. `bee feedback rank` and the digest therefore stop counting historic copies as repeats. `.bee/backlog.jsonl` is never rewritten. | The store is append-only and edited only through the CLI; the fold happens at read time. |
| D3 | New verb `bee backlog resolve --layer <fix-at:check\|fix-at:architecture> --title <exact title> --guard <what now stops it> [--commit <sha>]` appends one row `{ts, type:"resolution", layer, title, guard, commit?}`. It refuses when no finding with that layer and title exists, and refuses an empty `--guard`. A second resolve of the same key appends nothing and says so. | A finding row has no id; (layer, title) is its identity (D1). An append-only resolution row keeps the store shape. |
| D4 | Readers honor resolutions: the digest excludes resolved fix-at findings from its entries, and `prose_load` reports `fix_at_check_open`, `fix_at_architecture_open` and `fix_at_resolved` beside the existing raw row counts. The printed line names open and resolved counts. | The loop is visible only when "converted" is a number. |
| D5 | `packages/bee-rs/clippy.toml` bans `std::path::Path::strip_prefix` through `disallowed-methods`, with a reason that names the replacement helper. One helper in `path_identity.rs` relativizes a path against a root: plain strip first, then both sides canonicalized (dunce), returning `None` when neither holds. It is the only allowed call site. All 21 `src/` call sites move to it in the same change, behavior-preserving. The 4 `tests/` sites stay: `commands.test` runs clippy without `--all-targets`, so tests are outside the rule. | The talk: a new rule ships with the cleanup of old occurrences. The reason text is the error message that tells the agent the fix. |
| D6 | `prose_load` gains `preamble_bytes` and `preamble_lines`: the SessionStart preamble rendered for the repo by the same builder the hook uses, within the same budget. A render failure reads as null, never fails the digest. | The preamble is always-loaded prose; a witness that skips it under-reports the layer. |
| D7 | Doctrine: the fix-first cell that lands a guard for a fix-at row files the `backlog-closed` row of D8 naming the guard it landed. The line goes into the "A mistake seen twice" concept, `expertise/static-rules.md` ("Show it red, then green"), and `skills/bee-capturing/references/promotion.md`. The strip_prefix row is resolved by this feature with the D5 guard. | A rule nothing records as landed reads the same as a rule never written. |
| D8 | Supersedes D3 and narrows D4 after a read of the feedback ranking. The resolution home already exists: a `backlog-closed` row with the same title retires the group from `bee feedback rank`, and a newer non-closed entry reopens it as `recurred` (docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md). No `resolve` verb is built. Resolving a fix-at row is `bee backlog add --type backlog-closed --title "<exact title>" --layer <same fix-at layer> --severity P3 --detail "guard: <what now stops it> (<commit>)"`. D4 keeps only the witness: `prose_load` adds `fix_at_check_open`, `fix_at_architecture_open` and `fix_at_closed`, where a fix-at title counts as closed when a `backlog-closed` row with the same normalized title is newer than its newest finding row; the digest entries are not filtered, because rank already retires closed groups. | One fact, one home: a second resolution row type would split what rank already reads. |
| D9 | Refines D5. A canonical fallback changes what a call site returns when the plain strip fails, so D5 ships two helpers in `path_identity.rs`: `strip_root_lexical(path, root)` (exactly today's `Path::strip_prefix`, for paths that come from one source or are relative) and `relative_to(path, root)` (plain first, then both sides through `dunce::canonicalize`). Each `src/` site moves to `strip_root_lexical` unless both sides are absolute paths read from different sources, which takes `relative_to`; the cap lists each site that took `relative_to`. The clippy reason names both and when each applies. | A behavior change hidden inside a rename is the kind of change a reviewer cannot see. |
| D10 | Refines D1, D2 and D8 after the plan check (both hats, BLOCKER). A close epoch splits a fix-at key in time: the epoch of a row is the count of `backlog-closed` rows with the same `normalize_title` and an older `ts`. Title matching everywhere in this feature uses `normalize_title`. D1: close skips filing only when a finding row with the same layer and normalized title exists in the CURRENT epoch (newer than the latest close); after a close, the same mistake at close files a new row, which rank reports as `recurred`. D2: the fold applies to `finding` and `friction` rows only, keyed by (type, layer, normalized title, epoch), so a post-close copy is never folded into a pre-close entry. When the skip comes from a row under another feature, the close line names that feature in its pointer. `preamble_bytes` renders with runtime `claude` and is a snapshot of this session's state. | A dedupe that ignores closes makes a closed mistake unable to recur, which is the signal the loop exists to give. |
| C1 | Contract fix-at-identity: closing two features in one run with one mechanizable reflection files exactly one row. Proving tests carry `fix_at_identity` in their names. | |
| C2 | Contract fix-at-open (per D8): a `backlog-closed` row with the same title as a fix-at finding moves it from the open count to `fix_at_closed`; a finding row newer than the close moves it back to open; copies of one (layer, title) count once in open and in the digest entries (D2). Proving tests carry `fix_at_open` in their names. | |
| C3 | Contract strip-prefix-rule: `cargo clippy -- -D warnings` stays green with the clippy.toml in place; the helper's tests cover plain match, canonical-only match (a symlinked root), and no match. Proving tests carry `relative_to` in their names. The red proof is the probe run that listed 21 `src/` sites before the move. | |
| C4 | Contract preamble-load: `prose_load` carries `preamble_bytes` and `preamble_lines`, both at most the preamble budget on a fixture repo, null on a render failure. Proving tests carry `prose_load` in their names. | |

### Agent's Discretion

- The helper's name and exact signature, given D5's two-step order.
- The wording of refusals and of the printed digest clause.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/human-mailbox/overview.md | suspect | Item 17 (close files fix-at rows) and "A mistake seen twice" — D1, D3, D7 change both. |
| docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md | suspect | The preamble budget D6 measures. |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs` (347-434) | `mechanizable_mistakes`, `file_fix_at_rows` |
| `packages/bee-rs/crates/bee/src/verbs/backlog.rs` (407-443) | finding row builder and dedupe |
| `packages/bee-rs/crates/bee/src/verbs/feedback.rs` (986-1018, 1650-1717) | backlog ingestion, `prose_load_value` |
| `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs` (496-589) | preamble builders |
| `packages/bee-rs/crates/bee/src/path_identity.rs` | home of the new helper |

## Not in this feature

- Rewriting `.bee/backlog.jsonl` to delete historic copies.
- Bulk-resolving the other rows; each needs its own landed guard.
- Linting `tests/` (adding `--all-targets` to `commands.test` brings 98 unrelated warnings).
- A revert-rate or red-CI-rate witness.
