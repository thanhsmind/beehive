---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset>
---

# Plan: Proof strength and expiry

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the vocabulary is ~10
lines in ONE function that already does exactly this job for `red`. Everything
expensive is teaching cost, and the plan's shape is chosen to pay that bill
precisely rather than broadly.

**Revision 2.** A two-seat hat wave found five blockers in revision 1, three of
which inverted the design. Named in `## Discovery`; revision 1's central
mechanism was wrong.

## Requirements (from CONTEXT.md)

- **D1** — The result segment closes over `green:live`, `green:unit`,
  `green:static`. A bare `green` is refused at the write path, naming the three.
  `red` keeps refusing.
- **D2** — Write path closes; READ path stays tolerant of a historical bare
  `green` on an already-capped cell.
- **D3** — Meanings pinned once: `live` drove the real product, `unit` ran
  automated tests, `static` compiled/type-checked/linted with nothing executed.
- **D4** — `bee worktree merge` emits a named `proof-stale` advisory and never
  refuses on it.
- **D5** — Every doc, skill, prompt and test example moves to a qualified value;
  the worker brief states the three and their meanings.
- **D6** — No change to the three-segment shape, to `commands.test`, to what any
  door runs, or to the `red` rule.

## Load-bearing claims

Labels are `read`, `ran`, or `guessed`. Evidence is a verbatim byte substring of
the anchored line(s); multi-line evidence joins lines with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The result segment today accepts ANY non-empty string — emptiness is the only structural rule | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:90-91` | `if command.is_empty() \|\| result.is_empty() \|\| reason.is_empty() { / return None;` |
| 2 | The READ path calls the same parser, so closing `parse_tests_proof` would close BOTH paths at once and retroactively refuse every historical cap | read | `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs:61` | `let valid = matches!(report.get("tests"), Some(Value::String(s)) if parse_tests_proof(s).is_some());` |
| 3 | ...and the correct landing site already exists: `red` is refused in `parse_report_flag`, on the TUPLE, write-path only. D1 goes exactly here and D2 then needs no read-path code at all | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:195-196` | `Some(Value::String(s)) => match parse_tests_proof(s) { / Some((_, result, _)) if result == "red" => {` |
| 4 | The refusal MESSAGE itself teaches the soon-to-be-refused form, inside the very function being changed | read | `packages/bee-rs/crates/bee/src/verbs/cells/finish_support.rs:204` | `(e.g. \"cargo test -p bee — green — touched close.rs\").` |
| 5 | ~200 tracked cell records carry a bare `— green —`, not 25 — the read path reads the archive too, which the 25-count glob could not reach | ran | `rg -l '— green —' .bee/cells/ \| wc -l` | `200` |
| 6 | The recorded commit exists and is already read, so D4 needs no new field | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:942` | `commit: report_line("commit").filter(\|c\| c != "none"),` |
| 7 | BUT `ProofCheck` carries no commits, so D4 needs a commit-carrying read — it is a `proof.rs` change, not only a door change | read | `packages/bee-rs/crates/bee/src/verbs/cells/proof.rs:34-38` | `pub(crate) struct ProofCheck { / pub(crate) blocking: bool, / pub(crate) bad_ids: Vec<String>, / pub(crate) proven_count: usize, / pub(crate) legacy_count: usize,` |
| 8 | The merge door reads proof in `phases.rs`, not `merge.rs` — that is where D4's advisory lands | read | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:156` | `crate::verbs::cells::feature_proof_check(main_root, feature)` |
| 9 | The merge base is NOT in hand there: it is a local inside `branch_changed_files`, whose only caller sits ~150 lines later. D4 needs its own `merge-base` call | read | `packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs:214` | `let base = run_git(main_root, &["merge-base", "HEAD", branch]);` |
| 10 | ...and that sole caller is in the untracked-dirt check, far below the proof read | read | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:300` | `match current_branch(&worktree_root).and_then(\|b\| branch_changed_files(main_root, &b)) {` |
| 11 | `proof.rs` itself contains two bare-`green` fixtures that must SURVIVE the sweep — they are D2's own in-tree evidence | ran | `rg -c '— green —' packages/bee-rs/crates/bee/src/verbs/cells/proof.rs` | `2` |
| 12 | A domain-correct fence home already exists, so no new test file is earned | ran | `ls -la packages/bee-rs/crates/bee/tests/proof_gate.rs` | `packages/bee-rs/crates/bee/tests/proof_gate.rs` |
| 13 | A closed vocabulary with a typed refusal naming its legal set has a working model here | read | `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:287-288` | `pub(crate) const ROUTE_CLASS_VALUES: [&str; 8] = / ["feature", "bugfix", "docs", "refactor", "research", "release", "spike", "perf"];` |
| 14 | ...whose refusal message is itself pinned by a test — the second half of that model, which D1 copies | read | `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs:1725` | `"route --set: invalid flag(s): --class \"nope\" (must be one of feature, bugfix, docs` |

## Discovery

Revision 1 was wrong about its own central mechanism, and the wave caught it.

**The landing site inverted (rows 2-3).** Revision 1 put the vocabulary in
`parse_tests_proof` and then described D2's write/read split as something the
plan would engineer. But `feature_proof_check` calls that same parser
(`proof.rs:61`), so closing it closes both paths in one edit — the exact outcome
D2 exists to prevent. The repo already holds the right model and revision 1
never named it: `red` is refused in `parse_report_flag`, on the tuple
`parse_tests_proof` returns. Put D1 there and **D2 needs no code at all** — the
read path keeps working because the parser it calls is untouched. The split is
free; revision 1 proposed building what already exists.

**The historical population is 8× larger (row 5).** 25 was a live-store count;
the read path walks the archive too, and the recursive count is ~200. The
direction was right, the number was wrong, and D2 matters more than stated.

**D4's landing site and inputs were both wrong (rows 7-10).** The advisory goes
in `phases.rs`, not `merge.rs`; `ProofCheck` carries no commits so `proof.rs`
must gain a commit-carrying read; and the merge base is a local inside a helper
whose only caller runs ~150 lines after the proof read, so D4 needs its own
`merge-base` invocation. Revision 1 claimed the left-hand side was already in
hand. It is not.

**The fence was unbuildable as worded.** "No tracked source carries a bare
`— green —`" forbids the very fixtures that prove D2 (row 11), plus ~200 cell
records and a regen output. Six exception classes for a fence whose selling
point was exhaustiveness. It becomes an ALLOWLIST of the sites that TEACH the
form, checked against the vocabulary constant — `route_class_parity.rs`'s own
design — homed in the existing `proof_gate.rs` (row 12) rather than a new file.

**The Open Question was already answerable.** `site/guide/vi/cell-lane.html` is
hand-authored source: `site/guide/build.mjs` reads it as a fragment and writes
to `dist/`. Edit it directly. Revision 1 carried it as an open risk for no
reason.

## Approach

**Recommended path.** Add a three-value constant and check it in
`parse_report_flag`, immediately beside the existing `red` arm (row 3), and pin
the refusal message with a test the way the route-class model does (row 14).
Touch neither `parse_tests_proof` nor `proof.rs` for the vocabulary — that
inaction IS D2, and it gets a recorded comment so a later reader does not
"fix" it. Separately, give `ProofCheck` a commit-carrying field and emit D4's
advisory in `phases.rs` with its own `merge-base` call. Finally sweep only the
sites that TEACH the form, and pin that set with an allowlist fence in
`proof_gate.rs`.

**Rejected alternatives.**
- The vocabulary in `parse_tests_proof` — rejected by row 2: it closes the read
  path too and retroactively invalidates ~200 historical caps.
- A whole-tree denylist fence — rejected by row 11 and the ~200 cell records: it
  forbids D2's own evidence and needs six carve-outs.
- A new fence test file — rejected by row 12: `proof_gate.rs` is the proof
  domain's existing contract file, and a new file would copy its helpers.
- A separate `strength` key in the Result form — rejected: D1 fixes the
  LOCATION as the result segment; a fourth key is a fourth vocabulary in a
  fourth place.
- Deriving strength from the command segment — rejected by D3: deriving meaning
  from free text is the ambiguity this feature closes.
- Refusing a merge on a stale proof — rejected by the doors' check-never-run
  contract; D4 is an advisory, whole stop.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| The vocabulary in `parse_report_flag` | LOW | unit tests both ways, plus the pinned refusal message (row 14's model) |
| Read-path tolerance surviving | HIGH | a test that a capped bare-`green` cell still passes `feature_proof_check`, AND a comment at `proof.rs` recording that the untouched parser is deliberate |
| The two `proof.rs` fixtures being swept by mistake | HIGH | row 11 — each gains a one-line comment marking it as D2 evidence, so a later sweep cannot silently kill it |
| Refusal messages teaching the refused form | MEDIUM | row 4 — they change in the same cell as the vocabulary, never later |
| `proof-stale` correctness | MEDIUM | a test with a capped commit that is not an ancestor of the merge base: the advisory fires AND the merge still succeeds |
| The allowlist fence rotting | MEDIUM | the fence asserts each listed site still CONTAINS its anchor, so a site cannot silently drop out — `route_class_parity.rs`'s own guard against a rotted allowlist |

## Shape

Three slices. Slices 1 and 2 touch disjoint files and may run concurrently;
slice 3 needs slice 1's vocabulary to exist before it can teach it.

| Slice | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | The three-value constant + the check in `parse_report_flag` beside the `red` arm; the refusal-message example at :204 (:211 carries no example and needs no change); the pinned-message test; a recorded comment at `proof.rs` saying the untouched parser is what makes D2 work | The whole behaviour change, in one function that already does this job for `red` | `— green —` refuses naming the three; `— green:unit —` caps; a capped bare-`green` cell still passes the close door | slice 3 has a vocabulary to teach |
| 2 | `ProofCheck` gains a commit-carrying field; `phases.rs` emits the named `proof-stale` advisory with its own `merge-base` call | Independent of the vocabulary — different files, no shared symbol. The cheapest, highest-signal user win, so it does not wait behind the sweep | a merge whose capped commit is not an ancestor of the base prints the advisory and STILL merges | — |
| 3 | The ~9 teaching sites: `packages/bee/prompts/worker-cell.md` (with D3's meanings), `skills/bee-swarming/references/worker-details.md`, three `docs/product-description/` files, `site/guide/vi/cell-lane.html`, and the display strings at `handlers_close.rs` / `session_preamble/budget.rs`; plus the allowlist fence in `proof_gate.rs` | An example showing the refused form teaches it; the fence stops a site dropping out later | the fence fails when a listed site loses its qualified value | — |

Write-path test fixtures are NOT their own work: slice 1 turns them red
automatically, so the test run is the sweep. Read-path fixtures are explicitly
NOT swept (row 11).

## Test matrix

The triad, at its smallest demonstrating size. Each cell's writer judges existing
coverage first (`.bee/expertise/tests.md`) and authors only the gap.

- **Happy path** — `— green:unit —` parses and caps; the advisory stays silent
  when every capped commit is an ancestor of the merge base.
- **Edge** — the free split: a capped cell carrying a bare `green` still passes
  `feature_proof_check` with NO read-path code changed (rows 2-3, 5); and the
  allowlist fence over the teaching sites.
- **Error** — a bare `green` refuses at the write path with a typed message
  naming all three values, and that message is pinned by a test (row 14's
  model); `red` still refuses. Existing assertions are updated deliberately,
  never deleted.

## Open Questions

(none — revision 1's only Open Question is answered in `## Discovery`:
`site/guide/vi/cell-lane.html` is hand-authored source, read as a fragment by
`site/guide/build.mjs`.)

## Out of scope

- The proof line's three-segment shape, `commands.test`, what any door RUNS, and
  the `red` rule (D6).
- Any refusal on a stale proof. The advisory is the whole of D4.
- The ~200 historical bare-`green` caps. They stay exactly as recorded; D2 is
  the decision not to rewrite history.
- The remaining two gaps `docs/history/research/pstack-xia.md` ranks — no rule
  for a quietly-dead worker, and questions that reach the human when running
  something would have answered them. Separate work, not started here.
