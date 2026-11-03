---
artifact_contract: bee-plan/v2
mode: small
---

# Plan: Herding Pre-flight Test Seam

## Summary

Three tests spawn a real Claude agent every time the suite runs, and the pane
they leave behind never closes. This splits the pre-flight's *decision* from
*acting on it*, so those tests can check the decision without dispatching
anything.

Nothing changes for a real user: same five outcomes, same refusal text, same
envelope, same marker write.

Mode: `small` — 1 risk flag (covered-contract-change), 1 product file.

## Requirements (from CONTEXT.md)

- The pre-flight keeps D7's placement in `fn run`, above the transport choice
  and above the detached re-launch.
- The five outcomes, the refusal text and D5's marker write are unchanged.
- No test in this crate may reach `transport_for_run`.

## Load-bearing claims

Labels: `read` = opened at the named line; `ran` = the command was executed and
its output kept; `guessed` = neither, and no such row may survive the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | `fn run` builds the real transport, so any test calling it can spawn | read | `packages/bee-rs/crates/bee/src/herding/run.rs:3966-3978` | `match transport_for_run(&opts.main_root) {` inside `fn run` |
| 2 | Three tests call `run(&[...])` directly | read | `packages/bee-rs/crates/bee/src/herding/run.rs:9680,9702,9736` | `let exit = run(&[` in each of `replay_with_unreadable_digest_proceeds_to_spawn`, `job_with_brief_but_no_result_proceeds_to_spawn`, `continue_at_round_2_with_same_task_proceeds_and_is_not_refused` |
| 3 | The file already forbids exactly this | read | `packages/bee-rs/crates/bee/src/herding/run.rs:478-481` | `tests inject a fake instead of a real herdr on PATH (D7's seam, no` · `process anywhere in this crate's test suite)` |
| 4 | Real agent sessions were created, one per suite run, each pointed at a deleted tempdir | ran | `ls ~/.claude/projects/*crates-bee/` and the first user message of each transcript | `Read the file /tmp/.tmpBs3KFO/.bee/mailbox/job-crashed-1/brief-1.txt and follow its instructions exactly.` — 12 such sessions, 3.4 MB |
| 5 | The default agent the leak starts is `claude-sonnet` | read | `.bee/config.json` → `herding.agent_command` | `"claude-sonnet"` |
| 6 | A leaked pane was observed live and idle | ran | `herdr pane list` | `w1:pCJ claude idle .../packages/bee-rs/crates/bee` |

## Discovery

The suite is green in the main checkout with no visible pane, which is why this
hid: the leak leaves a *session* every run but only sometimes wins a pane split
that stays open. Counting `~/.claude/projects/*crates-bee/` rather than watching
panes is what made it reproducible.

## Approach

Extract `preflight_receipt(bee_dir, &opts) -> PreflightAction` — pure over the
filesystem, no spawn, no transport. `PreflightAction` moves to module scope and
carries what the caller needs to print: `ReturnReceipt { round }`,
`Refuse { round, result_path, stored_digest, incoming_digest }`, `Proceed`.
`fn run` matches on it and is the only actor.

The three tests call `preflight_receipt` and assert the variant.

**Rejected:** injecting a fake transport into `fn run` — bigger change, and it
would still let a future test spawn by forgetting the fake. Removing the tests —
loses the coverage that found two real bugs.

**SMALLER PATH check.** Could the tests just assert on a smaller helper without
moving the enum? No — the outcome they need to distinguish (`Proceed` vs
`ReturnReceipt`) *is* the enum. PASS.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "cell-execution", "classification": "required", "role": "code", "reason": "One Rust file and its tests."},
    {"stage": "test-authoring", "classification": "required", "role": "test", "reason": "The three leaking tests are rewritten against the new seam."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every claim was read or run directly."},
    {"stage": "gather", "classification": "not-applicable", "role": "read", "reason": "One file."},
    {"stage": "extraction", "classification": "not-applicable", "role": "extraction", "reason": "One file."},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "Single slice."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "No documented contract changes; the seam is internal."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The user invokes an independent review.", "reason": "User-invoked only."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No release."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage has a named role."},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "Small lane; no high-risk gate."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "Small lane takes no hat wave."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Small lane takes no hat wave."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Small lane takes no hat wave."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "Small lane takes no hat wave."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "Small lane takes no hat wave."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No open design shape."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No open design shape."}
  ]
}
```

## Shape

One cell.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| hpts-1 | Make the receipt pre-flight a pure decision so no test spawns a real agent | `packages/bee-rs/crates/bee/src/herding/run.rs` | — | `cargo test` stops creating a Claude session and stops leaving an idle pane behind | the suite green, plus a before/after count of `~/.claude/projects/*crates-bee/` sessions across a full run |

```json
[
  {
    "id": "hpts-1",
    "feature": "herding-preflight-test-seam",
    "title": "Make the receipt pre-flight a pure decision so no test spawns a real agent",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["10e20c92", "2eb61b7b", "42a59eea", "5d0692b5"],
    "files": ["packages/bee-rs/crates/bee/src/herding/run.rs"],
    "read_first": [
      "docs/history/herding-preflight-test-seam/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Lift the receipt pre-flight's DECISION out of `fn run` (currently inline at run.rs:3948-4030) into a new module-scope function, leaving `fn run` as the only place that ACTS on it.\n\n(1) Move `enum PreflightAction` to module scope beside the function. Give its variants what the caller needs to print, so the caller reads nothing from disk itself: `ReturnReceipt { round: u32 }`, `Refuse { round: u32, result_path: PathBuf, stored_digest: String, incoming_digest: String }`, `Proceed`.\n\n(2) Add `fn preflight_receipt(bee_dir: &Path, opts: &Options) -> PreflightAction`, pure over the filesystem — it reads, it decides, it spawns nothing and touches no transport. Move the round resolution into it unchanged (1 for a fresh run; `mailbox::latest_result_round(entries) + 1` when `opts.is_continue`, and the `None => 0` sentinel meaning 'no prior result, nothing to replay'). Keep all five outcomes byte-identical in behavior, including the stderr note on an unreadable digest.\n\n(3) In `fn run`, replace the inline block with a single `match preflight_receipt(&bee_dir, &opts)`. The Refuse arm prints the SAME message it prints today, character for character, and returns FAILURE. The ReturnReceipt arm still calls `write_inbox_marker`, still reads the stored result, still prints the same envelope, and returns SUCCESS. Proceed falls through. The call site stays exactly where it is — above the transport choice and above the `spawn_detached_runner` branch (D7); do NOT move it into `execute` or `execute_no_pane`.\n\n(4) Rewrite the three leaking tests to call `preflight_receipt` and assert the returned variant, never `run`: `replay_with_unreadable_digest_proceeds_to_spawn` (:9670) and `job_with_brief_but_no_result_proceeds_to_spawn` (:9692) assert `PreflightAction::Proceed`; `continue_at_round_2_with_same_task_proceeds_and_is_not_refused` (:9714) builds its Options with `is_continue` true and asserts `Proceed`. Each test needs an `Options` value — build one with a small local helper rather than through `parse_options`, so the test names its own inputs. Rename the two `_proceeds_to_spawn` tests to say what they now assert (they no longer spawn).\n\nLeave `replay_with_matching_digest_returns_stored_receipt_and_spawns_nothing`, `replay_with_differing_digest_refuses_loudly_and_spawns_nothing` and `replay_with_missing_digest_returns_stored_receipt_legacy` calling `run` if they already pass without spawning — they short-circuit before the transport — but if any of them reaches transport, move it to `preflight_receipt` too.\n\nDo NOT change the five outcomes, the refusal wording, the envelope shape, or D5's marker write. Do NOT touch mailbox.rs or the Pi belt.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "a full cargo test run creates ZERO new session files under ~/.claude/projects/*crates-bee/ — counted before and after",
        "a full cargo test run leaves the agent-pane count unchanged, checked with herdr pane list before and after",
        "no test in packages/bee-rs/crates/bee/src/herding/run.rs calls run() on a path that reaches transport_for_run",
        "the five pre-flight outcomes behave exactly as before: no result proceeds, absent digest returns the receipt, matching digest returns the receipt, differing digest refuses, unreadable digest proceeds with a stderr note",
        "the refusal message is unchanged character for character",
        "the receipt path still calls write_inbox_marker before printing the envelope",
        "the pre-flight call site is still in fn run above the transport choice and above spawn_detached_runner",
        "the whole suite is green"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "module-scope PreflightAction, the pure preflight_receipt function, fn run reduced to a match on it, and the three tests rewritten against the new seam"}
      ],
      "key_links": [
        "preflight_receipt performs no spawn and references no transport",
        "fn run is the only caller that acts on a PreflightAction",
        "the three rewritten tests never call run()"
      ],
      "prohibitions": [
        "Do not move the pre-flight into execute or execute_no_pane",
        "Do not change the five outcomes, the refusal wording, or the envelope",
        "Do not drop write_inbox_marker from the receipt path",
        "Do not touch mailbox.rs or .pi/extensions/bee-guard.ts"
      ]
    },
    "behavior_change": true
  }
]
```

## Verify

`cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
— the declared `commands.test`. Scope reason: the change is inside the herding
run verb, whose suites live in that manifest. The leak-specific proof is not a
test but a before/after count of session files and agent panes across a full
run, because the defect is a side effect the suite itself cannot assert.

## Test matrix

| # | Case | Pass when |
|---|---|---|
| 1 | Happy: no stored result | `preflight_receipt` returns `Proceed` |
| 2 | Happy: result + matching digest | returns `ReturnReceipt { round: 1 }` |
| 3 | Legacy: result, no digest file | returns `ReturnReceipt` |
| 4 | Error: result + differing digest | returns `Refuse` carrying both digests and the result path |
| 5 | Error: digest unreadable | returns `Proceed` |
| 6 | Edge: `--continue` at round 2, same task | returns `Proceed`, never `Refuse` |
| 7 | The leak itself | a full suite run adds 0 session files and 0 agent panes |

## Open Questions

(none)

## Out of scope

- The `.bee/logs` test pollution and the dropped prune verb — separate backlog
  rows.
- Deleting the 12 already-leaked transcripts — the user's data, not bee's.
