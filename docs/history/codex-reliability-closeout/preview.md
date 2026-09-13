# Documented Gate Preview Flag Alias Repair

## Context and Defect

During work on Codex reliability closeout (`docs/history/codex-reliability-closeout/plan.md`), decision `be57302a-9683-4f89-bc29-44111764389d` recorded:
> Use the supported `state gate preview` subcommand during this repair: `gate --preview` and `state gate --preview` currently refuse despite documented help. Record this CLI alias defect alongside the repairs; do not skip the preview check.

Both `bee gate --help` and `bee state gate --help` document `--preview` as an accepted boolean flag:
```
--preview (boo) — Preview current-slice cell packets and compute the plan hash before approving the shape gate.
```

However, executing either documented flag spelling failed with an `unsupported argument shape` error.

## Root Cause Trace

1. `bee gate` is registered in `router::FLOW_VERBS` as an alias expanding to `["state", "gate"]`.
2. Both spellings dispatch to `verbs::state_group::set_gate::run_gate`.
3. `run_gate` already contained handling for `--preview`:
   ```rust
   if flags.get("preview").is_some() {
       return run_gate_preview(flags, use_json, t0);
   }
   ```
4. However, before reaching `run_gate`, arguments are parsed by `parse_flags` (`packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs`).
5. `parse_flags` consults `FLAG_ALONE_BOOLEANS` to determine which flags are boolean when appearing without an `=` value:
   ```rust
   if FLAG_ALONE_BOOLEANS.contains(&name) {
       (name, FlagV::Present)
   } else {
       match tokens.get(i + 1) {
           None => return None,
           Some(v) => {
               i += 1;
               (name, FlagV::S((*v).to_string()))
           }
       }
   }
   ```
6. `FLAG_ALONE_BOOLEANS` lacked `"preview"`. Consequently, when a command such as `bee gate --preview --lane <feature>` was parsed:
   - `--preview` was treated as a value flag.
   - The subsequent token `--lane` was consumed as the string value of `--preview`.
   - The following token `<feature>` did not begin with `--`, triggering `if !tok.starts_with("--") { return None; }`.
   - `parse_flags` returned `None`, causing `try_native` to decline and emitting `unsupported argument shape`.
   - Even when `--preview` was the final token, `parse_flags` returned `None` due to missing value.

## Pre-Fix Reproduction (RED)

### Direct CLI Invocations

```bash
$ bee gate --preview --lane codex-reliability-closeout
bee: unsupported argument shape for `bee gate`: `bee gate --preview --lane codex-reliability-closeout`. Its required arguments are all present, so what it refused is an optional flag, a flag value, or a target that does not exist. FIX: `bee gate --help` for every accepted flag and its type.
[exit: 1]

$ bee state gate --preview --lane codex-reliability-closeout
bee: unsupported argument shape for `bee state gate`: `bee state gate --preview --lane codex-reliability-closeout`. Its required arguments are all present, so what it refused is an optional flag, a flag value, or a target that does not exist. FIX: `bee state gate --help` for every accepted flag and its type.
[exit: 1]
```

### Automated Integration Test Failures (`tests/gate_preview.rs`)

Before updating `FLAG_ALONE_BOOLEANS`, running `cargo test --test gate_preview` failed 4 of 5 tests:

```
running 5 tests
test cli_state_gate_subcommand_continues_to_work ... ok
test cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged ... FAILED
test cli_state_gate_preview_flag_with_following_lane_accepts_and_renders ... FAILED
test cli_gate_preview_flag_with_following_lane_accepts_and_renders ... FAILED
test cli_gate_preview_reversed_order_and_default_record ... FAILED

failures:
---- cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged stdout ----
thread 'cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged' panicked at crates/bee/tests/gate_preview.rs:153:10:
Unexpected failure: code=1, stdout="{\"error\":\"bee: unsupported argument shape for `bee gate`: `bee gate --preview --lane preview-feat --json`...\"}"

---- cli_state_gate_preview_flag_with_following_lane_accepts_and_renders stdout ----
thread 'cli_state_gate_preview_flag_with_following_lane_accepts_and_renders' panicked at crates/bee/tests/gate_preview.rs:105:10:
Unexpected failure: code=1, stderr="bee: unsupported argument shape for `bee state gate`: `bee state gate --preview --lane preview-feat`\n"

---- cli_gate_preview_flag_with_following_lane_accepts_and_renders stdout ----
thread 'cli_gate_preview_flag_with_following_lane_accepts_and_renders' panicked at crates/bee/tests/gate_preview.rs:77:10:
Unexpected failure: code=1, stderr="bee: unsupported argument shape for `bee gate`: `bee gate --preview --lane preview-feat`\n"

---- cli_gate_preview_reversed_order_and_default_record stdout ----
thread 'cli_gate_preview_reversed_order_and_default_record' panicked at crates/bee/tests/gate_preview.rs:194:10:
Unexpected failure: code=1, stderr="bee: unsupported argument shape for `bee gate`: `bee gate --lane preview-feat --preview`\n"

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## The Repair

Added `"preview"` to `FLAG_ALONE_BOOLEANS` in `packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs`:

```rust
pub(crate) const FLAG_ALONE_BOOLEANS: &[&str] = &[
    "json", "stdin", "active-only", "dry-run", "write", "as-lane", "no-lane",
    "waive-scribing-debt", "waive-compounding", "html", "string", "cleanup",
    "no-cleanup", "force-ownership", "local", "all", "untagged", "check",
    "with-companion", "lanes-full", "strict", "queue-submit", "show",
    "isolate", "set", "brief", "all-but-active", "merge", "claim", "skip-uat",
    "preview",
];
```

This ensures `parse_flags` parses `--preview` as `FlagV::Present` without consuming the subsequent `--lane` flag or other trailing tokens.

## Post-Fix Verification (GREEN)

1. **Integration Test Suite**:
   ```bash
   $ PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml --test gate_preview
   running 5 tests
   test cli_gate_preview_flag_with_following_lane_accepts_and_renders ... ok
   test cli_state_gate_subcommand_continues_to_work ... ok
   test cli_state_gate_preview_flag_with_following_lane_accepts_and_renders ... ok
   test cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged ... ok
   test cli_gate_preview_reversed_order_and_default_record ... ok

   test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
   ```

2. **Unit Test Suite**:
   ```bash
   $ PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee plan_packets
   test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 3558 filtered out; finished in 0.00s
   ```

3. **Release Manifest Verification**:
   ```bash
   $ cargo run --manifest-path packages/bee-rs/Cargo.toml -p bee -- dev release-manifest --check
   release_manifest --check: 376 file(s) match stored manifest, 1 unhashed artifact(s) present
   ```

## Invariants Upheld

- **No Gate Bypass**: `run_gate_preview` mutates only `gate_preview`; approval gates (`approved_gates.shape` and `approved_gates.execution`) remain completely unchanged and unapproved.
- **Subcommand Parity**: `state gate preview` continues to work identically.
- **Both Spellings Accept Following Lane**: Both `bee gate --preview --lane <feature>` and `bee state gate --preview --lane <feature>` successfully parse and execute.
- **No New Flags or Workarounds**: Resolved at the root parser table (`FLAG_ALONE_BOOLEANS`) without inventing new flags or relying on documentation changes.

## Test Contract Correction (`approved_gates`)

Following review in `docs/history/codex-reliability-closeout/crc-4-revision.md`:
- Prior tests seeded and asserted synthetic `gates.shape.approved` and `gates.execution.approved` fields rather than real state and lane fields.
- Real store records carry `approved_gates` (`approved_gates.shape` and `approved_gates.execution`), which are plain booleans.
- Integration tests in `packages/bee-rs/crates/bee/tests/gate_preview.rs` were updated to assert real `approved_gates` across both:
  1. Default record paths (`bee gate --preview --no-lane`, `bee state gate --preview --no-lane`)
  2. Lane record paths (`bee gate --preview --lane <feature>`, `bee state gate --preview --lane <feature>`, `bee state gate preview --lane <feature>`)
- The suite tests both:
  1. Unapproved gates (`shape: false, execution: false`), confirming preview does not approve gates.
  2. Already-true approvals (`shape: true, execution: true`), confirming preview preserves existing approval state rather than clearing or resetting it.
- State assertions verify both on-disk projections (`.bee/lanes/<feature>.json`, `.bee/state.json`) and the real CLI state reader (`bee status --brief --json`).

### Corrected Integration Test Suite Execution Output

```bash
$ PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml --test gate_preview
running 7 tests
test cli_state_gate_subcommand_continues_to_work ... ok
test cli_gate_preview_flag_with_following_lane_accepts_and_renders ... ok
test cli_lane_preview_preserves_existing_true_approvals ... ok
test cli_state_gate_preview_flag_with_following_lane_accepts_and_renders ... ok
test cli_gate_preview_json_outputs_packet_and_leaves_approval_unchanged ... ok
test cli_default_record_preview_preserves_existing_true_approvals ... ok
test cli_gate_preview_reversed_order_and_default_record_unapproved ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

