# Plan: release-clean-tree

Lane: small · class: bugfix · flags: authorization, proof-weakening · product files: 4

## Summary

A pi release still cannot finish. The deploy worker passed authorization for
the first time (permit `1ed10fb1`), then two things stopped it. First, the
worker ran `bee dispatch authorize` by hand, which used up the one-time permit
before `scripts/release.sh` ran. Second, bee itself dirties main before
release.sh's clean-tree check: `bee herding run` appends a wave-ledger row,
the session wait mark rewrites the lane record, and `authorize` writes an
un-ignored permit marker. One cell makes release.sh tolerate exactly those two
tracked bee store paths (never committing or resetting them), ignores
`.bee/authorizations/` like other bee runtime state, and tells the deploy
worker never to call authorize itself. The user chose this path (option A).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | release.sh authorizes before its clean-tree check. | read | `scripts/release.sh:106` | `"$BEE_BIN" dispatch authorize --id "$BEE_DISPATCH_ID" --release-version "$AUTH_VERSION" \` |
| 2 | Any dirt at all refuses the release. | read | `scripts/release.sh:155-158` | `DIRT="$(git status --porcelain)"` then `fail "working tree is not clean (paths above)` |
| 3 | The release commit snapshot takes every dirty path. | read | `scripts/release.sh:224-230` | `CHANGED=()` then `done < <(git status --porcelain -z)` |
| 4 | The abort path resets every tracked change. | read | `scripts/release.sh:174` | `git checkout -- . 2>/dev/null \|\| true` |
| 5 | herding run appends a wave-ledger row on main at spawn. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2141` | `if let Err(e) = wave_ledger::append_wave(main_root, &row) {` |
| 6 | The permit marker directory is not git-ignored. | ran | `git check-ignore -v .bee/authorizations/1ed10fb1-8c5c-41f4-93aa-69680e26dbf4.json` | `exit=1` |
| 7 | The lane record changed on main during the release from the session wait mark. | ran | `git diff -- .bee/lanes/deploy-stage-execution.json` | `"kind": "turn-end",` |
| 8 | The worker's hand-run authorize consumed the permit, and the next call was refused. | ran | `.bee/logs/timings.jsonl` (main) | `{"ts":"2026-09-15T05:37:19.420Z","cmd":"dispatch authorize","ms":10,"ok":true}` then a `05:37:44.348Z ... ok=false` row |
| 9 | The deploy brief does not forbid calling authorize. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1310-1318` | `Execute the authorized release command:\n\` then `Wait for release CI to pass and verify the published release assets.\n` |

## Cells — current slice preview

```json
[
  {
    "id": "rct-1",
    "feature": "release-clean-tree",
    "title": "Let a dispatched release pass the clean-tree check without widening it",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "scripts/release.sh",
      "scripts/release-dirt.sh",
      ".gitignore",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"
    ],
    "read_first": [
      "scripts/release.sh",
      ".gitignore",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "docs/history/release-clean-tree/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "BUG: a pi deploy dispatch cannot finish scripts/release.sh. (1) bee dirties main before release.sh checks it: `bee herding run` appends to `.bee/wave-ledger.jsonl` (herding/run.rs `2141:    if let Err(e) = wave_ledger::append_wave(main_root, &row) {`), the session wait mark rewrites `.bee/lanes/<feature>.json`, and `release.sh:106` `dispatch authorize` writes `.bee/authorizations/<id>.json`, which .gitignore does not list. release.sh then refuses at `155:  DIRT=\"$(git status --porcelain)\"`. (2) The deploy worker ran `bee dispatch authorize` by hand and used up the one-time permit. Anchors in scripts/release.sh: `155:  DIRT=\"$(git status --porcelain)\"`; restore_tree `174:    git checkout -- . 2>/dev/null || true`; commit snapshot `224:  CHANGED=()` ... `230:  done < <(git status --porcelain -z)`; commit `258:  git add -- \"${CHANGED[@]}\"`. Brief: prepare.rs `1310:pub(crate) fn deployment_prompt(release_version: &str) -> String {`. Existing tests to mirror: tests.rs `11019:    fn test_direct_release_script_call_refuses_before_mutation() {` (runs scripts/release.sh from repo_root via Command) and the deploy stdin assert `11127:        assert!(stdin_deploy_wt.contains(\"scripts/release.sh 2.38.0\"), ...`. STEP 1 (red first), in tests.rs: (a) a test that makes a temp git repo, commits `a.txt`, `.bee/wave-ledger.jsonl` and `.bee/lanes/x.json`, dirties all three plus an untracked `b.txt`, runs `bash <repo_root>/scripts/release-dirt.sh` with cwd = that temp repo, and asserts the output names `a.txt` and `b.txt` but not the two bee paths; and with only the two bee paths dirty the output is empty; (b) a test asserting `git check-ignore -q .bee/authorizations/x.json` succeeds at repo_root (skip like 11019 if the checkout is absent); (c) extend the deploy stdin assert so the brief says the worker must not run `bee dispatch authorize` itself. Run and watch all three fail. STEP 2 (fix): add `scripts/release-dirt.sh` — reads `git status --porcelain -z` and prints the NUL-separated porcelain entries (rename/copy pairs kept together) whose path is NOT exactly `.bee/wave-ledger.jsonl` and NOT matching `.bee/lanes/*.json`; that exact two-pattern allowlist is the whole exemption. In release.sh use it at all three sites: the DIRT check (refuse when it prints anything, same message), the CHANGED snapshot (build from its output so the bee paths are never staged or committed), and restore_tree (reset only the tracked non-allowlisted paths instead of `git checkout -- .`, so an abort never discards bee's ledger row or lane record). Add `.bee/authorizations/` to .gitignore beside `.bee/claims/`. In deployment_prompt add one plain sentence: run only that command; never run `bee dispatch authorize` yourself, because release.sh runs it and the permit works one time. Keep every other release.sh precondition, message and exit unchanged. STEP 3: run the verify command.",
    "verify": "bash -n scripts/release.sh && bash -n scripts/release-dirt.sh && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee release_ && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee deploy_",
    "must_haves": {
      "truths": [
        "release.sh refuses a dirty tree exactly as before for every path except .bee/wave-ledger.jsonl and .bee/lanes/*.json",
        "the release commit never stages .bee/wave-ledger.jsonl or .bee/lanes/*.json",
        "an aborted release never resets .bee/wave-ledger.jsonl or .bee/lanes/*.json",
        ".bee/authorizations/ is git-ignored",
        "the deploy brief tells the worker never to run bee dispatch authorize itself"
      ],
      "artifacts": [
        {"path": "scripts/release-dirt.sh", "substantive": "prints non-allowlisted porcelain entries; exact two-pattern allowlist"},
        {"path": "scripts/release.sh", "substantive": "DIRT check, CHANGED snapshot and restore_tree all use release-dirt.sh"},
        {"path": ".gitignore", "substantive": ".bee/authorizations/ listed"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "deployment_prompt forbids calling dispatch authorize"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs", "substantive": "red-first tests for the dirt filter, the ignore rule and the brief"}
      ],
      "key_links": ["release.sh calls scripts/release-dirt.sh at the dirt check, the commit snapshot and the restore path"],
      "prohibitions": [
        "No change to dispatch authorize or its refusal checks",
        "No path other than .bee/wave-ledger.jsonl and .bee/lanes/*.json is exempt from the dirt check",
        "No other release.sh precondition, message, or exit changes"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Proof and close

The cell's `verify` syntax-checks both scripts and runs the release and deploy
tests in release mode. The release then runs `scripts/release.sh 2.37.3`
through a fresh pi deploy dispatch, which runs the full declared suite before
it tags.
