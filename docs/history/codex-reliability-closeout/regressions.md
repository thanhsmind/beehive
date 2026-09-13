# Retrospective Transcript Regression Evidence (F2 & F4)

## Summary

This document records retrospective regression evidence for transcript parsing and session close hooks, resolving findings **F2** and **F4** from the Codex workflow compliance review (`docs/history/codex-parity-completion/reports/workflow-review-20260913.md`).

> [!IMPORTANT]
> **Retrospective Evidence Disclaimer**:
> This validation was performed retrospectively by transplanting current regression tests onto the historical pre-fix production tree (`0690c1eb1ff72302033dfa5be4b11f753abf471f`). It does NOT assert or claim that an original red-before-green test execution occurred during the historical cell run.

## Historical Source Identities

- **Pre-fix baseline commit**: `0690c1eb1ff72302033dfa5be4b11f753abf471f` (*"Track Codex session completion from native transcripts"*).
- **Subsequent fix commit**: `62408eb152b908ebc6ebeb569866a8c44115f289` (*"Fix Codex transcript normalization and session wait tracking"*).
- **Current feature HEAD**: `wt/codex-reliability-closeout` branched from `c4122d414490e008afc840474b2ce01316d02b73`.

## Test Transplantation Methodology

1. **Isolated Disposable Tree**:
   An isolated pre-fix source tree was extracted into `.bee/tmp/pre-fix-tree` using:
   ```bash
   git archive 0690c1eb1ff72302033dfa5be4b11f753abf471f | tar -x -C .bee/tmp/pre-fix-tree
   ```
2. **Exact Test Transplantation**:
   The exact four current test functions were transplanted into `.bee/tmp/pre-fix-tree/packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs` without altering any production logic or test assertions. The exact patch is preserved in:
   `docs/history/codex-reliability-closeout/test-transplant.patch`
3. **Environment & Build Isolation**:
   To prevent target directory collisions and ensure independent binary derivation:
   - `CARGO_TARGET_DIR=".bee/tmp/target-prefix"` was used for pre-fix compilation.
   - `TMPDIR=/var/tmp` and `BEE_CODEX_PROBE_BIN=/bin/false` were enforced.
   - Direct Cargo `PATH` was used without invoking any PATH wrappers.

## Regression Test Results

### 1. Pre-Fix Baseline Execution (`0690c1eb`)

The transplanted test suite was executed against the pre-fix source tree:
```bash
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false \
CARGO_TARGET_DIR=".bee/tmp/target-prefix" cargo test --release \
  --manifest-path .bee/tmp/pre-fix-tree/packages/bee-rs/Cargo.toml -p bee --bin bee -- \
  hooks::session_close::tests::codex_repeated_token_count_dedup_and_multiple_requests_in_turn \
  hooks::session_close::tests::claude_turn_end_subject_preserves_assistant_text_across_tool_result \
  hooks::session_close::tests::codex_rollup_does_not_use_response_item_payload_id_as_session_id \
  hooks::session_close::tests::codex_rollup_preserves_full_uuid_when_session_meta_missing \
  --test-threads 1
```

Full raw output is preserved in `docs/history/codex-reliability-closeout/regression-red.log`.

**Result Summary**:
- `codex_repeated_token_count_dedup_and_multiple_requests_in_turn`: **FAILED**
  - Panic: `assertion left == right failed: left: "codex", right: "o3-mini"` at `tests.rs:1606:9`.
  - Cause: Pre-fix `aggregate_usage` did not extract the active model from nested `event_msg` payload `thread_settings_applied`, falling back to `"codex"`. In addition, repeated `token_count` records were not deduplicated from cumulative sequences.
- `codex_rollup_preserves_full_uuid_when_session_meta_missing`: **FAILED**
  - Panic: `assertion left == right failed: left: "7a48da530e71", right: "01a095e4-8306-70d0-bf1f-7a48da530e71"` at `tests.rs:1639:9`.
  - Cause: Pre-fix `rollup_from_events` used `stem.rsplit('-').next()`, truncating the 36-character session UUID to the trailing 12-character fragment.
- `codex_rollup_does_not_use_response_item_payload_id_as_session_id`: **FAILED**
  - Panic: `assertion left == right failed: left: "7a48da530e71", right: "01a095e4-8306-70d0-bf1f-7a48da530e71"` at `tests.rs:1673:9`.
  - Cause: Session UUID truncation in pre-fix `rollup_from_events`.
- `claude_turn_end_subject_preserves_assistant_text_across_tool_result`: **PASSED**
  - Context: In `0690c1eb`, `final_assistant_text_line` only broke on user messages when `in_codex_turn` was true. The regression that broke Claude's backward scan across tool results was introduced during intermediate uncommitted attempts of `cpc-3` (where `type=user` was made an unconditional break before excluding `tool_result`, as documented in `cpc-3-revision-2.md:6`). The test was authored to ensure the subsequent fix preserved Claude behavior, which `0690c1eb` already exhibited.

### 2. Current Production Source Execution

The same test suite was executed against the current feature tree:
```bash
PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" TMPDIR=/var/tmp BEE_CODEX_PROBE_BIN=/bin/false \
cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- \
  hooks::session_close::tests::codex_repeated_token_count_dedup_and_multiple_requests_in_turn \
  hooks::session_close::tests::claude_turn_end_subject_preserves_assistant_text_across_tool_result \
  hooks::session_close::tests::codex_rollup_does_not_use_response_item_payload_id_as_session_id \
  hooks::session_close::tests::codex_rollup_preserves_full_uuid_when_session_meta_missing \
  --test-threads 1
```

Full raw output is preserved in `docs/history/codex-reliability-closeout/regression-green.log`.

**Result Summary**:
- `claude_turn_end_subject_preserves_assistant_text_across_tool_result`: **ok**
- `codex_repeated_token_count_dedup_and_multiple_requests_in_turn`: **ok**
- `codex_rollup_does_not_use_response_item_payload_id_as_session_id`: **ok**
- `codex_rollup_preserves_full_uuid_when_session_meta_missing`: **ok**
- Overall: **4 passed; 0 failed; 0 ignored**.

In addition, the full module suite `hooks::session_close::tests` passed with **43 passed; 0 failed**.

## F4: Source Whitespace and Committed Range Verification

Finding F4 identified committed trailing blank lines at EOF in `packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs:1916`.

1. **Correction**: Trailing blank lines after the closing brace at line 1915 were removed.
2. **Explicit Range Verification**:
   ```bash
   git diff --check 642631a59d84639922a70848c52f723d9c9e4f53 -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs
   git diff --check c4122d41 -- packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs
   ```
   Both commands exit 0 cleanly with no whitespace errors reported.
