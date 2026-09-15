# Plan: release-red-fixes

Lane: small · class: bugfix · flags: proof-weakening · product files: 4

## Summary

The 2.37.3 release reached its test gate and stopped: the declared suite went
red on three tests, and all three came from the last three fixes. One cell
turns them green without touching release behavior. A prose mention of a
dispatch spelling inside the closed `deploy-issuer-session` plan is pinned as a
historical exception, because `docs/history/**` is not rewritten. The
`.bee/authorizations/` ignore line moves into the onboarding template, because
`bee dev regen` rewrites the managed gitignore block from that template. A
`work_verbs` test strips `BEE_SESSION_ID`, which deploy worker panes now carry.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The spelling test fails on main on one docs/history line. | ran | `cargo test --release -p bee --bin bee no_shipped_command_spelling` | `docs/history/deploy-issuer-session/plan.md` then `test result: FAILED. 0 passed; 1 failed` |
| 2 | The flagged text is in the fenced cells JSON of that plan. | read | `docs/history/deploy-issuer-session/plan.md:48` | `"action": "BUG: \`bee dispatch prepare --stage deployment\` writes` |
| 3 | docs/history lines are pinned as exceptions, not rewritten. | read | `packages/bee-rs/crates/bee/src/hooks/cli_shape.rs:1145-1152` | `/// A handful of pinned exceptions: fenced lines inside \`docs/history/**\`` then `const KNOWN_HISTORICAL_EXCEPTIONS: [&str; 2] = [` |
| 4 | bee dev regen strips a hand-added line from the managed gitignore block. | ran | `.bee/bin/bee dev regen` then `git check-ignore -q .bee/authorizations/x.json` in the worktree | `regen exit=0` then `check-ignore exit=1` |
| 5 | The managed block renders from the template list, and new patterns join at its tail. | read | `packages/bee-rs/crates/bee/src/onboard/templates.rs:24` and `:58-61` | `pub const GITIGNORE_BLOCK_PATTERNS: &[&str] = &[` then `so a new pattern joins at the tail` |
| 6 | The work_verbs test fails when BEE_SESSION_ID is set. | ran | `BEE_SESSION_ID=repro-session cargo test --release -p bee --test work_verbs the_session_env_var_resolves` | `left: Null` then `right: "the env ask"` |
| 7 | That test strips the herding markers but not BEE_SESSION_ID. | read | `packages/bee-rs/crates/bee/tests/work_verbs.rs:191-193` | `.env_remove("BEE_HERDING_WORKER")` then `.env_remove("BEE_HERDING_JOB_ID")` then `.env("CLAUDE_CODE_SESSION_ID", "s-env")` |
| 8 | With the deploy pane env set, the full suite fails in exactly these two targets. | ran | `BEE_SESSION_ID=… BEE_DISPATCH_ID=… BEE_RELEASE_VERSION=2.37.3 cargo test --release --no-fail-fast` on main | `test result: FAILED. 3601 passed; 1 failed` (`-p bee --bin bee`) and `test result: FAILED. 13 passed; 1 failed` (`--test work_verbs`) |

## Cells — current slice preview

```json
[
  {
    "id": "rrf-1",
    "feature": "release-red-fixes",
    "title": "Turn the three release-gate reds green without changing release behavior",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/tests/work_verbs.rs",
      ".gitignore",
      ".bee/onboarding.json"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/tests/work_verbs.rs",
      "docs/history/release-red-fixes/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED BASE: `scripts/release.sh 2.37.3` stopped at its test gate on three failures. Reproduce each first (they are red on main today). (1) `cargo test --release -p bee --bin bee no_shipped_command_spelling` fails on `docs/history/deploy-issuer-session/plan.md`: its fenced cells JSON (line 48) holds the prose span `bee dispatch prepare --stage deployment` inside a cell action. docs/history is immutable, so do NOT edit that plan: add one entry to `const KNOWN_HISTORICAL_EXCEPTIONS` in `hooks/cli_shape.rs` (anchor `1152:    const KNOWN_HISTORICAL_EXCEPTIONS: [&str; 2] = [`; bump the array length) whose string is exactly the candidate the test prints, with a comment in the same style naming cell dis-1 of deploy-issuer-session and why it is a prose mention, not a transcript. (2) The `.bee/authorizations/` line was added by hand inside the onboarding-managed gitignore block, and `bee dev regen` (which release.sh runs) rewrites that block from `GITIGNORE_BLOCK_PATTERNS` in `onboard/templates.rs` and drops it (reproduced: regen then `git check-ignore -q .bee/authorizations/x.json` exits 1). Append `\".bee/authorizations/\"` at the TAIL of `GITIGNORE_BLOCK_PATTERNS` (after `\".bee/result-inbox/\",`) with a comment in the style of the tail entries (runtime data, one-use deploy permit markers written by `bee dispatch authorize`; joined at the tail because the order is hashed). Then remove the hand-added `.bee/authorizations/` line from the middle of the block in `.gitignore`, build the binary from THIS worktree (`cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee`), and run THAT freshly built binary's `dev regen` from the worktree root so `.gitignore` and `.bee/onboarding.json` carry the new rendered block and hash — never the vendored `.bee/bin/bee`, which is a stale copy (pattern 20260830 release-regen-with-a-stale-vendored-binary). Keep only `.gitignore` and `.bee/onboarding.json` from that regen; revert any other file it rewrites (for example the verify-app feature copies), because that drift is not this cell's. Confirm `git check-ignore -q .bee/authorizations/x.json` exits 0 after regen. (3) `BEE_SESSION_ID=repro cargo test --release -p bee --test work_verbs the_session_env_var_resolves_the_record_when_no_flag_names_one` fails (`left: Null`, `right: \"the env ask\"`) because the test at `tests/work_verbs.rs:188-193` strips BEE_HERDING_WORKER and BEE_HERDING_JOB_ID but not BEE_SESSION_ID or PI_SESSION_ID, and BEE_SESSION_ID outranks CLAUDE_CODE_SESSION_ID. Add `.env_remove(\"BEE_SESSION_ID\")` and `.env_remove(\"PI_SESSION_ID\")` beside the existing removes, matching the neighbour test at line 209. Do not change any product behavior, release.sh, or the session resolution order.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee no_shipped_command_spelling && BEE_SESSION_ID=pane-issuer-session PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test work_verbs && git check-ignore -q .bee/authorizations/x.json && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee onboard::",
    "must_haves": {
      "truths": [
        "the widened CLI-shape spelling test passes on the repo docs",
        "work_verbs passes with BEE_SESSION_ID set in the environment",
        "after bee dev regen with the fresh binary, .bee/authorizations/ stays git-ignored"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs", "substantive": "one commented KNOWN_HISTORICAL_EXCEPTIONS entry for the dis-1 prose span"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/templates.rs", "substantive": ".bee/authorizations/ appended at the tail of GITIGNORE_BLOCK_PATTERNS with a comment"},
        {"path": "packages/bee-rs/crates/bee/tests/work_verbs.rs", "substantive": "the env-session test strips BEE_SESSION_ID and PI_SESSION_ID"},
        {"path": ".gitignore", "substantive": "managed block as rendered by the fresh binary, authorizations at the tail"},
        {"path": ".bee/onboarding.json", "substantive": "managed gitignore hash matching the new block"}
      ],
      "key_links": [".gitignore managed block is the regen rendering of GITIGNORE_BLOCK_PATTERNS"],
      "prohibitions": [
        "No edit to docs/history/deploy-issuer-session/plan.md",
        "No change to scripts/release.sh, dispatch authorize, or session resolution order",
        "No regen drift outside .gitignore and .bee/onboarding.json in the commit"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Proof and close

The cell's `verify` runs the spelling test, `work_verbs` with the deploy pane's
`BEE_SESSION_ID` set, the ignore check, and the onboarding tests in release
mode. Then the release runs again through a fresh pi deploy dispatch; its test
gate runs the full declared suite in the same pane env that went red.
