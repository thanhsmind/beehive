---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Close the fix-at loop

## Summary

A mistake that a check could stop is now filed once, not once per
feature. The feedback digest stops counting old copies as repeats, and it
says how many fix-at rows are still open and how many are closed by a
landed guard. It also counts the session-start text, the largest block of
always-loaded prose. bee gets its first `clippy.toml`: it bans
`Path::strip_prefix`, the call behind a Windows mistake filed 8 times, and
all 21 call sites in `src/` move to two named helpers in the same change.
The doctrine says how a landed guard closes its row.

Mode: `standard` — 3 risk flags: public contracts (new digest fields, a
changed close count), cross-platform (path handling on Windows),
multi-domain (close, feedback, lint config, doctrine).
Why this is the least workflow that protects the work: each part is
small, but the strip_prefix move touches 19 files on a path that has gone
red on Windows CI before, so it needs its own cell, its own tester and a
plan check.

## Requirements (from CONTEXT.md)

- D1: fix-at finding identity is (layer, title) across the backlog at close.
- D2: the digest folds copies of one (layer, title) into one entry.
- D3: superseded by D8 (no `resolve` verb).
- D4: narrowed by D8 (witness only).
- D5: `clippy.toml` bans `std::path::Path::strip_prefix`; `src/` sites move; `tests/` stay.
- D6: `prose_load` gains `preamble_bytes` and `preamble_lines`.
- D7: doctrine — a landed guard files the `backlog-closed` row.
- D8: resolution is the existing `backlog-closed` tombstone; `prose_load` adds open and closed counts.
- D10: a close epoch splits each fix-at key; dedupe and fold act within the current epoch by `normalize_title`; the close pointer names the owning feature; preamble renders with runtime `claude`.
- D9: two helpers, `strip_root_lexical` and `relative_to`; a site takes `relative_to` only with absolute paths from different sources.
- C1 to C4: the contracts and their test-name markers.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line. Anchors are at base commit 011e0da29; the cells of this feature then changed several of these lines.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Close reads the run's reflections with no feature filter | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:376 | `for entry in mailbox::read_entries(&control, &run) {` |
| 2 | The close dedupe keys on the feature | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:417 | `if backlog::backlog_finding_exists(root, feature, &layer, &title) {` |
| 3 | The dedupe helper compares feature, layer and title | read | packages/bee-rs/crates/bee/src/verbs/backlog.rs:441 | `field("feature") == feature && field("layer") == layer && field("title") == title` |
| 4 | The digest pushes one candidate per backlog row | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1011 | `raw.push(RawCandidate {` |
| 5 | A closed-kind row with the same title retires its group in rank | read | docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md:46 | `appending a closed-kind row bearing the SAME title as` |
| 6 | The closed kind is the alias of backlog-closed | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:684 | `("backlog-closed", "closed"),` |
| 7 | prose_load is built in one function | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1687 | `fn prose_load_value(root: &Path) -> Value {` |
| 8 | Fix-at rows are counted raw today | read | packages/bee-rs/crates/bee/src/verbs/feedback.rs:1679 | `Some("fix-at:check") => check += 1,` |
| 9 | The preamble has one budgeted builder the hook uses | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:589 | `pub fn build_session_preamble_within(` |
| 10 | The preamble budget is 9500 bytes | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:66 | `pub(crate) const PREAMBLE_BUDGET_BYTES: usize = 9500;` |
| 11 | No clippy config exists beyond two allows | read | packages/bee-rs/Cargo.toml:9 | `[workspace.lints.clippy]` |
| 12 | A site already does plain strip then canonical strip by hand | read | packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:243 | `let stripped = path.strip_prefix(&history_root).ok().map(Path::to_path_buf)` |
| 13 | The helper module holds the canonical comparison | read | packages/bee-rs/crates/bee/src/path_identity.rs:200 | `pub fn canonical_paths_equal(a: &Path, b: &Path) -> bool {` |
| 14 | 25 `Path::strip_prefix` sites exist, 21 under `src/` | ran | probe: `CLIPPY_CONF_DIR=.bee/tmp/fal-clippy cargo clippy --all-targets` with `disallowed-methods = [std::path::Path::strip_prefix]` | 25 `disallowed` warnings: 21 in `crates/bee/src`, 4 in `crates/bee/tests` |
| 15 | The declared suite runs clippy without `--all-targets` | read | .bee/config.json | `cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings` |
| 16 | The craft page has a red-then-green section to extend | read | expertise/static-rules.md:60 | `## Show it red, then green` |
| 17 | The promotion tree has a mechanizable step | read | skills/bee-capturing/references/promotion.md:123 | `2. Mechanizable? A grep/lint line in a verify command, a guard, a hook` |
| 18 | The mailbox concept has the seen-twice section | read | docs/knowledge/areas/human-mailbox/overview.md:228 | `## A mistake seen twice` |
| 19 | The same strip_prefix mistake sits in 8 features | ran | python over `.bee/backlog.jsonl` | `lcwr compared absolute paths 8 ... 8 features, 2026-10-06/07` |

## Discovery

The leader read `mechanizable_mistakes` and `file_fix_at_rows` in full,
the backlog row builder and dedupe, the digest's backlog ingestion and
`prose_load_value`, the preamble builders, and the rank's closed-group
handling. The read of rank found the tombstone convention (claims 5, 6),
so D8 replaced the planned `resolve` verb: no new verb, no registry
change. A probe clippy config listed every `Path::strip_prefix` site
(claim 14) without touching the tree. A read-only gather (haiku, report in
`.bee/mailbox/job-1794847341486-2161960-1/report-1.md`) confirmed the
anchors.

## Approach

Recommended path: three code cells, each followed by a tester that shares
no context with it, one docs cell with a checker, and the writer cell.

- fal-1 (D1, C1): `file_fix_at_rows` asks a new backlog helper whether a
  finding with this layer and title exists in any feature. `bee backlog
  add` keeps `backlog_finding_exists` unchanged.
- fal-2 (D2, D6, D8, C2, C4): the digest folds copies of one finding
  (type, layer, title) into one candidate with the earliest `ts`;
  `prose_load` adds open and closed fix-at counts and the preamble bytes
  and lines.
- fal-3 (D5, D9, C3): `packages/bee-rs/clippy.toml`, two helpers in
  `path_identity.rs`, 21 sites moved.
- fal-4 (D7, D8): the doctrine lines, then `bee dev regen`.
- fal-5: knowledge consolidation and `delivery.md`.

fal-4 files the `backlog-closed` row for the strip_prefix finding with the
command it documents, as the live proof of the loop. After the merge the
leader rebuilds and reinstalls `.bee/bin/bee` and checks a new string in
the installed binary before calling the feature live.

Rejected: a `bee backlog resolve` verb (a second home beside the
tombstone); rewriting `.bee/backlog.jsonl` (CLI-only, append-only store);
linting `tests/` (98 unrelated warnings); one helper with a silent
canonical fallback (hidden behavior change, D9).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"fal-1, fal-2 and fal-3 write Rust."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"fal-1t, fal-2t and fal-3t add the new tests, apart from the implementers."},
    {"stage":"docs","classification":"required","role":"docs","reason":"fal-4 writes the doctrine lines and runs the regen chain."},
    {"stage":"slice-judge","classification":"required","role":"review","reason":"fal-4c runs the parity and pointer checks on the docs cell."},
    {"stage":"writing","classification":"required","role":"writer","reason":"fal-5 consolidates knowledge and writes delivery.md."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Default plan-check seat."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Default seat; cites the SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Default seat; the close line and digest line are user-facing."},
    {"stage":"read-only-gather","classification":"required","role":"read","reason":"The planning gather that mapped close, backlog, feedback and path helpers."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Not high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Not high-risk."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release only on the owner's word, through scripts/release.sh."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Risk map

- A site moved to `relative_to` changes its result — MEDIUM — the one fact: the default target is `strip_root_lexical`, which is today's call; only sites with two absolute paths from different sources take `relative_to`, and the cap names each one — Lands in: fal-3 — proof: `cargo clippy -- -D warnings` green, `cargo test --bin bee -- path_identity` green, and the tester's `relative_to` tests over a symlinked root.
- The close stops filing a real new mistake — LOW — the one fact: identity is exact (layer, title); a new mistake has new words — Lands in: fal-1 — proof: tester's `fix_at_identity` test that two different titles still file two rows.
- The digest fold changes rank output — MEDIUM — the one fact: only exact (type, layer, title) copies fold; groups keyed by normalized title keep their other entries — Lands in: fal-2 — proof: tester's `fix_at_open` tests, plus the existing rank tests green.
- Rendering the preamble from the digest fails on a host — LOW — the one fact: a render error reads as null — Lands in: fal-2 — proof: `prose_load` test on an empty root.
- A doctrine edit breaks a parity or pointer test — LOW — Lands in: fal-4 — proof: `pointer_integrity`, `instruction_laws`, `rule_index_parity`.

Waves: serial, fal-1 → fal-1t → fal-2 → fal-2t → fal-3 → fal-3t → fal-4 → fal-4c → fal-5. Reason: every code cell builds the one `bee` crate in one worktree, so a half-written sibling breaks another cell's build (a single scarce resource). The docs cell runs the regen chain over the built binary, so it follows the code.

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature.md`, situation `skills/bee-planning/playbooks/feature/feature.md` (cited, not copied). Step 3 (statechart) does not apply: no event-order flow. Step 8 (formal model) does not apply: no lifecycle code moves. Architect skipped: the data shape is the existing backlog row and the existing tombstone; no new type.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| One mistake, one row | close dedupe across features | D1 | 1 | `fix_at_identity` |
| Honest counts | digest fold, open/closed, preamble | D2, D6, D8 | 1 | `fix_at_open`, `prose_load` |
| bee runs its own advice | clippy.toml + helpers + cleanup | D5, D9 | 1 | clippy green, `relative_to` |
| Loop closes in doctrine | guard → `backlog-closed` | D7, D8 | 1 | parity and pointer tests |

PLAN CHECK (hat wave, two of three seats). hat-facts-gaps and hat-user-impact both found one BLOCKER: a dedupe and fold that ignore `backlog-closed` rows stop a closed mistake from recurring. Closed by D10 (close epoch, `normalize_title`), new fal-1t cases (4)(5) and fal-2t case (3). WARNINGS applied: fold limited to finding and friction rows (D10); one title form, `normalize_title` (D10); the close pointer names the owning feature (D10); the strip_prefix close now has a cell, fal-4, with a rank line as proof; runtime pinned for the preamble and the printed line has an example; binary rebuild named in Approach. Dismissed: which feature a folded digest entry shows (entries carry no feature field; `RawCandidate` holds type, title, layer, ts, pain and source only). Named gap: the Windows 8.3 short-name case has no local test (Linux host); Windows CI runs the `relative_to` plain-match tests. hat-alternatives did not run: its configured transport is a herdr pane (`agy-flash`) and this session is not inside herdr; the inline SMALLER PATH check below stands for that seat.

SMALLER PATH: doctrine only (tell agents to dedupe and close by hand) is cheaper, but it keeps the fix in the prose layer, the thing the talk says to shrink. Banning `strip_prefix` without moving the 21 sites is cheaper, but `-D warnings` would turn the suite red; per-site `#[allow]` attributes would be 21 escape hatches. One helper instead of two is cheaper, but it hides a behavior change (D9). PASS.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| fal-1 | File a fix-at mistake once across features | drivers/close.rs, verbs/backlog.rs | — | closing a second feature in the same session says `(1 already there)` instead of filing a copy | `cargo test … --bin bee -- fix_at` |
| fal-1t | Test fix-at identity at close | drivers/close.rs tests | fal-1 | — | `cargo test … --bin bee -- fix_at_identity` |
| fal-2 | Fold copies and report open, closed and preamble load | verbs/feedback.rs | fal-1t | `bee feedback digest` prints open and closed fix-at counts and the preamble size; rank no longer shows copies as repeats | `cargo test … --bin bee -- feedback` |
| fal-2t | Test the fold, open counts and preamble load | verbs/feedback.rs tests | fal-2 | — | `cargo test … --bin bee -- fix_at_open prose_load` |
| fal-3 | Ban Path::strip_prefix and move src sites to named helpers | clippy.toml (new), path_identity.rs, 19 src files | fal-2t | `cargo clippy` names the helper when an agent writes `strip_prefix` on a path | `cargo clippy … -- -D warnings`; `cargo test … --bin bee -- path_identity` |
| fal-3t | Test the two path helpers | path_identity.rs tests | fal-3 | — | `cargo test … --bin bee -- relative_to` |
| fal-4 | Say how a landed guard closes its row | expertise/static-rules.md, promotion.md, regen outputs | fal-3t | the craft page and the promotion tree end the red-then-green proof with the `backlog-closed` row | regen + pointer and parity tests |
| fal-4c | Check the doctrine edit | — | fal-4 | — | `pointer_integrity`, `instruction_laws`, `rule_index_parity` |
| fal-4t | Pin the fix-at closing step in the doctrine | tests/instruction_laws.rs | fal-4 | — | `cargo test … --test instruction_laws -- fix_at_close_doctrine` |
| fal-5 | Consolidate knowledge and write delivery | docs/knowledge/**, delivery.md (new) | fal-4c | the mailbox and feedback concepts describe the new behavior | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "fal-1",
    "feature": "fix-at-loop",
    "title": "File a fix-at mistake once across features",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "45d56b6f-68ef-4d86-a1a3-4f321a25f7b3",
      "61540fb2-c57a-4159-8688-34adda64a1b0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/"
    ],
    "action": "Implement docs/history/fix-at-loop/CONTEXT.md D1. Anchors: in verbs/drivers/close.rs `fn file_fix_at_rows(root: &Path, feature: &str) -> FixAtRows {` and its call `if backlog::backlog_finding_exists(root, feature, &layer, &title) {`; in verbs/backlog.rs `pub(crate) fn backlog_finding_exists(root: &Path, feature: &str, layer: &str, title: &str) -> bool {`. Add a sibling helper in backlog.rs that, per D10, returns the feature of a finding row with this layer and the same crate::verbs::feedback::normalize_title in the CURRENT close epoch (its ts is newer than the newest backlog-closed row with the same normalized title), or None; call it from file_fix_at_rows in place of the per-feature check. After a close, the same mistake files a new row. A skip still counts in rows.skipped; when the owning feature differs from the closing one, the close line's `bee backlog findings --feature <feature>` pointer names the owning feature (anchor: the `(M already there)` line near close.rs:3217). backlog_finding_exists and bee backlog add stay unchanged. Update the doc comments the change makes false only by deleting them; write no new comment. Edit only the existing tests the change forces; the tester cell adds the new ones. Run the related tests and quote the count the filter matched.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at",
    "must_haves": {
      "truths": [
        "A run that closes two features with one mechanizable reflection files one row, and the second close counts it as already there and names the owning feature",
        "After a backlog-closed row with the same title, the same mistake at close files a new row",
        "Two different titles still file two rows",
        "bee backlog add keeps its per-feature dedupe"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/backlog.rs",
          "substantive": "the cross-feature finding check"
        }
      ],
      "key_links": [
        "file_fix_at_rows calls the cross-feature check"
      ],
      "prohibitions": [
        "Never rewrite .bee/backlog.jsonl",
        "No code comments",
        "No change to backlog_finding_exists"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "fal-1t",
    "feature": "fix-at-loop",
    "title": "Test fix-at identity at close",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "fal-1"
    ],
    "decisions": [
      "45d56b6f-68ef-4d86-a1a3-4f321a25f7b3",
      "61540fb2-c57a-4159-8688-34adda64a1b0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/backlog.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Contract C1 of docs/history/fix-at-loop/CONTEXT.md. First cite the existing tests that already cover file_fix_at_rows by name, then add only the gap, each test name containing fix_at_identity: (1) a finding with layer fix-at:check and title T under feature A exists, file_fix_at_rows for feature B with a mechanizable mistake titled T files nothing and counts one skip; (2) a different title under feature B files one row; (3) bee backlog add with the same title under a new feature still appends (its per-feature dedupe is unchanged); (4) finding T under feature A, then a backlog-closed row titled T, then file_fix_at_rows for feature B with mistake T files one new row; (5) a whitespace variant of T is treated as T. Show each new test red against the parent commit of fal-1 where the contract changed (tests 1), or state why it passes on both (tests 2, 3). Quote the count the filter matched. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_identity",
    "must_haves": {
      "truths": [
        "fix_at_identity tests exist and pass, with a matched count above zero"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
          "substantive": "fix_at_identity tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No change to product code",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "fal-2",
    "feature": "fix-at-loop",
    "title": "Fold copies and report open, closed and preamble load",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "fal-1t"
    ],
    "decisions": [
      "097acbf6-840e-4647-a782-848c39627fd6",
      "6f4070cf-faa5-4e9c-94b6-3e38e1cd4939",
      "3bc9f528-d5d9-4c23-9d07-085ef094cf45",
      "cc75554e-1d15-46d6-9bb9-9f07007a0173",
      "61540fb2-c57a-4159-8688-34adda64a1b0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md",
      "docs/knowledge/areas/hook-runtime/token-economy-of-injected-context.md",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/feedback-digest/"
    ],
    "action": "Implement D2, D6 and D8 of docs/history/fix-at-loop/CONTEXT.md (D3 is superseded, D4 narrowed by D8). Anchors in verbs/feedback.rs: the backlog ingestion `raw.push(RawCandidate {` under `match resolve_in_scope(&real_root, SRC_BACKLOG) {`, `fn fix_at_row_counts(root: &Path) -> (Option<u64>, Option<u64>) {`, `fn prose_load_value(root: &Path) -> Value {`, `fn prose_load_line(digest: &Value) -> String {`. (1) D2 with D10: during backlog ingestion, a finding or friction row whose (type, layer, normalize_title, close epoch) equals an earlier row's is folded into that row's candidate instead of pushed: the candidate keeps the earliest ts as first_seen and its pain is not summed. The close epoch of a row is the count of backlog-closed rows with the same normalize_title and an older ts. Other row types and other sources are untouched; backlog-closed rows are never folded. So a copy filed after a close stays its own entry and rank still reports the group as recurred. (2) D8: prose_load gains fix_at_check_open, fix_at_architecture_open and fix_at_closed. A distinct (layer, title) of a fix-at finding is closed when a backlog-closed row whose normalize_title equals the finding's normalize_title has a ts newer than the newest finding row of that key; otherwise it is open. Distinct keys are counted, never raw rows; the existing fix_at_check_rows and fix_at_architecture_rows stay raw and unchanged. The printed prose line names the open and closed counts and the preamble size, for example `fix-at: 129 open (110 check, 19 architecture), 1 closed; preamble 8,912 bytes / 151 lines`. (3) D6: prose_load gains preamble_bytes and preamble_lines from the preamble rendered for root by crate::hooks::session_preamble::budget::build_session_preamble_within with PREAMBLE_BUDGET_BYTES as the budget, runtime claude, and no session id or handoff (the number is a snapshot of the current state); when the render panics or root is not a bee project both are null (wrap with std::panic::catch_unwind or check the builder's own fail-open contract first and say which). The digest never fails because of these fields. Edit only the existing tests the change forces (the prose_load_reports_lines_patterns_and_rows_in_object_and_sentence fixture may need new keys); the tester adds the new tests. No code comments. Quote the matched test count.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- feedback",
    "must_haves": {
      "truths": [
        "Eight identical fix-at rows produce one digest entry with the earliest first_seen",
        "A fix-at title with a newer backlog-closed row counts as closed, not open",
        "prose_load carries preamble_bytes and preamble_lines, null on a render failure",
        "The existing raw row counts are unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
          "substantive": "fold, open/closed counts, preamble load"
        }
      ],
      "key_links": [
        "prose_load_value calls build_session_preamble_within"
      ],
      "prohibitions": [
        "Never rewrite .bee/backlog.jsonl",
        "No new verb or flag",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "fal-2t",
    "feature": "fix-at-loop",
    "title": "Test the fold, open counts and preamble load",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "fal-2"
    ],
    "decisions": [
      "3bc9f528-d5d9-4c23-9d07-085ef094cf45",
      "6f4070cf-faa5-4e9c-94b6-3e38e1cd4939",
      "097acbf6-840e-4647-a782-848c39627fd6",
      "61540fb2-c57a-4159-8688-34adda64a1b0"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/feedback.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Contracts C2 and C4 of docs/history/fix-at-loop/CONTEXT.md. Cite the existing prose_load and rank tests by name, then add the gap. Names containing fix_at_open: (1) three identical fix-at:check rows give one digest entry and fix_at_check_open 1; (2) a backlog-closed row with the same title and a newer ts moves it to fix_at_closed 1 and open 0, and rank lists it as retired; (3) a finding row newer than that close moves it back to open, stays a separate digest entry, and rank reports the group as recurred; (3b) a friction row and a pbi row are never folded. Names containing prose_load: (4) a fixture bee root gives preamble_bytes above 0 and at most 9500 and preamble_lines above 0; (5) an empty directory gives null for both and the digest still succeeds. Show each red on the parent commit of fal-2 where the behavior is new. Quote the matched counts. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- fix_at_open prose_load",
    "must_haves": {
      "truths": [
        "fix_at_open and prose_load tests exist and pass with matched counts above zero"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/feedback.rs",
          "substantive": "fix_at_open and prose_load tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No change to product code",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "fal-3",
    "feature": "fix-at-loop",
    "title": "Ban Path::strip_prefix and move src sites to named helpers",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "fal-2t"
    ],
    "decisions": [
      "9471088f-fd3f-4dc4-8ed0-0ccd8b859967",
      "54872359-dfb9-4090-808a-a67210a0fffc"
    ],
    "files": [
      "packages/bee-rs/clippy.toml",
      "packages/bee-rs/crates/bee/src/path_identity.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs",
      "packages/bee-rs/crates/bee/src/devtools/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/workspace_store.rs",
      "packages/bee-rs/crates/bee/src/verbs/workflow_store/record.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/consolidate.rs",
      "packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/render.rs",
      "packages/bee-rs/crates/bee/src/verbs/decisions/concept_links.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/obligation.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_write.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/hooks/state_sync.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs",
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/src/hooks/chain_nudge.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/devtools/plugin_distribution.rs",
      "packages/bee-rs/crates/bee/src/comments.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "expertise/static-rules.md",
      "packages/bee-rs/crates/bee/src/path_identity.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement D5 as refined by D9 in docs/history/fix-at-loop/CONTEXT.md. (1) Create packages/bee-rs/clippy.toml with disallowed-methods = [{ path = \"std::path::Path::strip_prefix\", reason = \"...\" }]; the reason says: use path_identity::strip_root_lexical when both paths come from one source or are relative, path_identity::relative_to when both are absolute paths read from different sources (it also matches through symlinks and Windows 8.3 short names). (2) In path_identity.rs add pub fn strip_root_lexical(path: &Path, root: &Path) -> Option<&Path> (exactly Path::strip_prefix(...).ok()) and pub fn relative_to(path: &Path, root: &Path) -> Option<PathBuf> (plain strip first, then both sides through dunce::canonicalize and strip again; None when neither holds). These two bodies are the only allowed calls, each under #[allow(clippy::disallowed_methods)]. (3) Move every Path::strip_prefix call under crates/bee/src (21 sites in the listed files; find them with the same probe: CLIPPY_CONF_DIR pointing at the new file, cargo clippy without --all-targets) to strip_root_lexical, keeping each site's Result or Option handling equivalent. A site takes relative_to only when both sides are absolute paths from different sources; leader_check.rs:243, which already does plain then canonical by hand, is one and collapses to relative_to. Do not touch str::strip_prefix, and do not touch crates/bee/tests. (4) The cap lists each site that took relative_to with a one-line reason, and quotes the probe count before (21) and after (0). Run cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings and the related tests. No code comments.",
    "verify": "cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- path_identity leader_check",
    "must_haves": {
      "truths": [
        "cargo clippy -- -D warnings is green with clippy.toml in place",
        "No Path::strip_prefix call remains under crates/bee/src outside the two helpers",
        "Writing Path::strip_prefix in src makes clippy print a reason that names both helpers"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/clippy.toml",
          "substantive": "the disallowed-methods rule with its reason"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/path_identity.rs",
          "substantive": "strip_root_lexical and relative_to"
        }
      ],
      "key_links": [
        "leader_check.rs calls relative_to"
      ],
      "prohibitions": [
        "No #[allow(clippy::disallowed_methods)] outside the two helpers",
        "No edit under crates/bee/tests",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    },
    "full_suite_reason": "cargo clippy here is the lint rule this cell adds, run once as its own red-to-green proof; the cargo test part is filtered"
  },
  {
    "id": "fal-3t",
    "feature": "fix-at-loop",
    "title": "Test the two path helpers",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "fal-3"
    ],
    "decisions": [
      "9471088f-fd3f-4dc4-8ed0-0ccd8b859967",
      "54872359-dfb9-4090-808a-a67210a0fffc"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/path_identity.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/path_identity.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Contract C3 of docs/history/fix-at-loop/CONTEXT.md. Add tests whose names contain relative_to: (1) plain match returns the tail for both helpers; (2) a root reached through a symlink (tempdir, std::os::unix::fs::symlink under cfg(unix)) gives None from strip_root_lexical and the tail from relative_to; (3) an unrelated path gives None from both; (4) a path equal to the root gives an empty tail. Show the rule red once: write a throwaway Path::strip_prefix call in a scratch src file, run cargo clippy -- -D warnings, quote the disallowed_methods error naming both helpers, then remove the call; do not commit it. Quote the matched test count. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- relative_to",
    "must_haves": {
      "truths": [
        "relative_to tests exist and pass",
        "the clippy error text was shown once and names both helpers"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/path_identity.rs",
          "substantive": "relative_to tests"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No change to product code",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "fal-4",
    "feature": "fix-at-loop",
    "title": "Say how a landed guard closes its row",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [
      "fal-3t"
    ],
    "decisions": [
      "fb1807c1-5a39-44e1-ad3e-f2d902ca8339",
      "3bc9f528-d5d9-4c23-9d07-085ef094cf45"
    ],
    "files": [
      "expertise/static-rules.md",
      "skills/bee-capturing/references/promotion.md",
      ".bee/backlog.jsonl",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "expertise/static-rules.md",
      "skills/bee-capturing/references/promotion.md",
      "docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md"
    ],
    "affects_skills": [
      "skills/bee-capturing/SKILL.md"
    ],
    "affects_specs": [],
    "action": "Implement D7 with D8's verb shape. In expertise/static-rules.md, section `## Show it red, then green`, add a closing step: after the rule is green, close the fix-at row it guards with bee backlog add --type backlog-closed --title \"<exact finding title>\" --layer <same fix-at layer> --severity P3 --detail \"guard: <rule> (<commit>)\"; say that a newer finding with the same title reopens it in bee feedback rank. In skills/bee-capturing/references/promotion.md step 2 (`2. Mechanizable?`), add one line pointing to that step. Short sentences, active voice, no new section. Then run bee dev regen so the vendored copies match, and include its outputs in the commit. Last, use the documented command once for real: close the strip_prefix finding (its exact title starts with `lcwr-1 compared absolute paths with a plain strip_prefix`; read it with .bee/bin/bee backlog findings --feature leader-check-windows-paths) with layer fix-at:check and detail `guard: packages/bee-rs/clippy.toml disallowed-methods std::path::Path::strip_prefix (<fal-3 commit>)`, then quote the line of .bee/bin/bee feedback rank that lists it as retired. The knowledge concepts are fal-5's.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test instruction_laws && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "static-rules.md tells the agent to file the backlog-closed row after the rule goes green",
        "promotion.md step 2 points to it",
        "vendored copies match after regen",
        "the strip_prefix finding shows as retired in bee feedback rank"
      ],
      "artifacts": [
        {
          "path": "expertise/static-rules.md",
          "substantive": "the closing step"
        }
      ],
      "key_links": [
        "promotion.md step 2 names static-rules.md"
      ],
      "prohibitions": [
        "No new verb named",
        "No edit to LEADER.md",
        "Append to .bee/backlog.jsonl only through bee backlog add, one row"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "fal-4c",
    "feature": "fix-at-loop",
    "title": "Check the doctrine edit",
    "lane": "standard",
    "role": "review",
    "status": "open",
    "deps": [
      "fal-4"
    ],
    "decisions": [
      "fb1807c1-5a39-44e1-ad3e-f2d902ca8339"
    ],
    "files": [
      "expertise/static-rules.md",
      "skills/bee-capturing/references/promotion.md"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "expertise/static-rules.md",
      "skills/bee-capturing/references/promotion.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Read-only check of fal-4: the bee backlog add line in static-rules.md uses only flags bee backlog add --help lists; the type backlog-closed is accepted (run bee backlog add --help and read the KIND_ALIASES note, never append a row); the vendored copies equal their sources; pointer_integrity, instruction_laws and rule_index_parity pass. Report PASS or the exact mismatch.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test instruction_laws --test rule_index_parity",
    "must_haves": {
      "truths": [
        "the parity and pointer tests pass",
        "the documented command uses real flags"
      ],
      "artifacts": [],
      "key_links": [],
      "prohibitions": [
        "Never edit a file",
        "Never append to .bee/backlog.jsonl"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    },
    "regen_obligation_ack": "read-only check cell; fal-4 runs the regen chain and the manifest check"
  },
  {
    "id": "fal-4t",
    "feature": "fix-at-loop",
    "title": "Pin the fix-at closing step in the doctrine",
    "lane": "standard",
    "role": "test",
    "status": "open",
    "deps": [
      "fal-4"
    ],
    "decisions": [
      "fb1807c1-5a39-44e1-ad3e-f2d902ca8339",
      "1507dff2-852b-4a09-98e3-996d5338396c"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/instruction_laws.rs"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "expertise/static-rules.md",
      "skills/bee-capturing/references/promotion.md",
      "packages/bee-rs/crates/bee/tests/instruction_laws.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Contract C5 (decision 1507dff2). In packages/bee-rs/crates/bee/tests/instruction_laws.rs add one test whose name contains fix_at_close_doctrine. It reads expertise/static-rules.md and asserts the section starting at the heading \"## Show it red, then green\" (up to the next ## heading) contains the text \"--type backlog-closed\" and \"--layer\"; and it reads skills/bee-capturing/references/promotion.md and asserts the step 2 paragraph (the line starting \"2. Mechanizable?\" up to the line starting \"3.\") mentions closing the fix-at row. Follow the file-reading helpers the file already uses. Show it red once by running it against a copy of the section with the closing step removed (a scratch string in the test run, never committed), then green. Quote the matched count. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test instruction_laws -- fix_at_close_doctrine",
    "must_haves": {
      "truths": [
        "a fix_at_close_doctrine test exists and passes with a matched count above zero"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/tests/instruction_laws.rs",
          "substantive": "fix_at_close_doctrine test"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No change to the doctrine files",
        "No code comments"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "fal-5",
    "feature": "fix-at-loop",
    "title": "Consolidate knowledge and write delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "fal-1",
      "fal-1t",
      "fal-2",
      "fal-2t",
      "fal-3",
      "fal-3t",
      "fal-4",
      "fal-4t"
    ],
    "decisions": [
      "45d56b6f-68ef-4d86-a1a3-4f321a25f7b3",
      "097acbf6-840e-4647-a782-848c39627fd6",
      "3bc9f528-d5d9-4c23-9d07-085ef094cf45",
      "6f4070cf-faa5-4e9c-94b6-3e38e1cd4939",
      "9471088f-fd3f-4dc4-8ed0-0ccd8b859967",
      "54872359-dfb9-4090-808a-a67210a0fffc",
      "fb1807c1-5a39-44e1-ad3e-f2d902ca8339"
    ],
    "files": [
      "docs/knowledge/areas/human-mailbox/overview.md",
      "docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md",
      "docs/history/fix-at-loop/delivery.md"
    ],
    "read_first": [
      "docs/history/fix-at-loop/CONTEXT.md",
      "docs/history/fix-at-loop/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/human-mailbox/",
      "docs/knowledge/areas/feedback-digest/"
    ],
    "action": "Run bee knowledge consolidate --feature fix-at-loop. Update docs/knowledge/areas/human-mailbox/overview.md item 17 (identity is layer and normalized title across features within the current close epoch, D1 and D10) and `## A mistake seen twice` (a landed guard files the backlog-closed row, D7, D8). Update docs/knowledge/areas/feedback-digest/ranking-and-self-improvement.md with the fold of copies within a close epoch (D2, D10) and the open, closed and preamble fields of prose_load (D6, D8). Cite history D-numbers as fix-at-loop D<n>, never raw decision ids. Name packages/bee-rs/clippy.toml and path_identity.rs in the Pointers of the concept that fits, or of a new pattern if none fits. Write docs/history/fix-at-loop/delivery.md.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "the two concepts describe the new behavior",
        "delivery.md exists"
      ],
      "artifacts": [
        {
          "path": "docs/history/fix-at-loop/delivery.md",
          "substantive": "what shipped and the proof"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No raw decision ids in area docs"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

Revision (execution): fal-4c ended as a drop because a standard cap refuses an empty files_changed, which leaves the docs cell fal-4 without a capped test-role dependent; fal-4t adds that tester (contract C5) and fal-5 now depends on it in place of fal-4c.

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| one reflection, two closes | happy | second close files 0, skipped 1 |
| two titles | edge | two rows filed |
| eight copies in the digest | happy | one entry, earliest first_seen |
| closed then refiled | edge | open again |
| preamble on an empty dir | error | both fields null, digest exit 0 |
| clippy with the rule | happy | `-D warnings` exit 0 |
| a new `Path::strip_prefix` in src | error | clippy error names both helpers |
| symlinked root | edge | `relative_to` returns the tail, lexical returns None |
| same digest on main and head for a backlog with no copies | regression | identical entries |

## Open Questions

(none)

## Out of scope

- Rewriting `.bee/backlog.jsonl`.
- Closing the other fix-at rows; each needs its own landed guard.
- Linting `tests/`.
- A revert-rate or red-CI-rate witness.
