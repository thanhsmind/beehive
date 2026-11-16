# cwc-3: contract check for clippy-warnings-clean

This checks contract clippy-warnings-clean (C1) on cwc-1 `9858e78c6` and cwc-2 `0e8bdee36`, measured against the feature base `d3ee458f1`. No source file was edited.

## Verdict: PASS. All six truths hold.

| # | Check | Command | Result |
|---|---|---|---|
| 1 | clippy `-D warnings` exits 0 on the default targets | `cargo clippy --manifest-path packages/bee-rs/Cargo.toml -- -D warnings` | PASS: `CLIPPY_EXIT=0` |
| 2 | The bin test count is not below the base of 5229 | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` | PASS: bee bin `5229 passed; 0 failed; 26 ignored`. Across all 49 result lines: 5827 passed, 0 failed, 30 ignored. `TEST_EXIT=0` |
| 3 | The workspace allows are exactly `too_many_arguments` and `result_large_err`, and no item-level allow was added | `git diff d3ee458f1 0e8bdee36 \| rg '^\+.*(#!?\[(allow\|expect)\(\|clippy::)'`; `rg '\[workspace.lints\|^\[lints\|= "allow"'` over the three Cargo.toml files | PASS: no allow or expect attribute was added. `[workspace.lints.clippy]` has only those two, and both crates set `[lints] workspace = true`. The item-level clippy allow count is 21 at base and 21 at head |
| 4 | cwc-1 and cwc-2 added no code comment | `/tmp/cwc3_cmt.py`: for each file, counts the words on added comment lines minus the words on removed comment lines | PASS: every added `//`/`///` line restates removed doc text, either re-indented by doc_lazy_continuation / doc_overindented_list_items or reflowed. Only two files show extra tokens: `workflows.rs` `refusal);///` and `orient.rs` `/// >`. Both are clippy rewrites of existing doc text, not new comments (see findings). No `/*` lines were added |
| 5 | Every deleted dead_code item has no caller in any target | A regex over the removed fn/struct/enum/const/static/type/trait/mod definitions, then `rg -w <name>` over `packages/bee-rs` including tests | PASS: see the table below. The full suite compiles every target |
| 6 | The non-mechanical hunks of cwc-2 do not change behavior | Read the whole `git show 0e8bdee36` diff | PASS: see below |

### Deleted items

| Item | Deleted from | References left at head |
|---|---|---|
| `fn env_nonempty` | `verbs/reservations/leases.rs` | Only the separate copies in `store.rs`, `claims.rs` and `perf.rs` and their own callers. No `leases::env_nonempty` path existed at base |
| `fn knowledge_holdout_percent` | `verbs/drivers/prepare.rs` | The name now appears only as a config key string. `holdout_percent`/`in_holdout` still read that key |
| `fn progress_path(job_dir)` | `herding/notify.rs` | Every caller uses `mailbox::progress_path(bee_dir, job_id)` |
| `fn render_agent_template` | `onboard/agents.rs` | None in code. One doc line at `onboard/agents.rs:184` still names it (see findings) |
| `fn session_body_with_runtime` | `hooks/session_init.rs` | None |
| variant `PromptSkew::NoEmbedded` | `verbs/drivers/prompt.rs` | Never constructed at base: its only references were the definition and its match arm, so dropping its message text changes no output |
| field `ProgressNote.ts` | `herding/status_digest.rs` | Parsed but never read |
| field `HandoffDismiss::Ok.record` | `verbs/state_group/store.rs` | Kept under `#[cfg(test)]`. The only non-test reader used `{ .. }` |

Other dead_code fixes put items behind `#[cfg(test)]` instead of deleting them: `ack_present`, `mark_orphans`, `dispatch_pane`, `logs_argv`, `execute_paseo`, `ROLE_DESCRIPTION_MAX`, `role_slot_description`, the `cross_project_denial_with_registry` and `emit_for` re-exports, `record_dissent`, `posix_shell`, `TEST_CWD_LOCK`, `protocol::parse`, `derive_candidates` and `interventions_store_path`.

### Behavior read of cwc-2

Each of these was checked as equivalent:
- `sort_by(|a,b| b.k.cmp(&a.k))` became `sort_by_key(|a| Reverse(a.k))` in five places, including the supervisor's `(needs_human, rank)` tuple. Both forms are stable sorts, so ties keep their order.
- `feedback.rs` `normalize_title`: the old if/else-if pushed `' '` in both branches. It is now a single `if in_ws`, which is identical.
- `guard.rs` `requested_model`: `cond && declared.is_some()` followed by `unwrap` became `declared.filter(|_| cond)`. Same result.
- `adapter.rs`: `raw.replace('\\', "\\")` became `raw.to_string()`. The old call was a no-op replace.
- `herding/run.rs` `RealWaitSource::poll` gained `let _ = child.wait();`. It runs only after `try_wait()` returned `Some`, when the child has already exited, so it returns at once and reaps the child. Nothing observable changes.
- Several `match`/`_ => {}` blocks became `if let`, and nested `if`s became let-chains in `prompt_context.rs`. Conditions and order are unchanged.
- Boxed variants (`RunOutcome::Result`, `Pipeline::Ok`, `ClaimDecision::Claimable`, `PreWt::Go`, `TriggerEntry::Ok`), type aliases, `strip_prefix`, `?` instead of `let-else return None`, `extend`/`fill`/`repeat_n` loops, and `Diff { missing, ..Default }` are type or shape changes only.
- `run.rs` `Options`/`ExecResult` went from private to `pub(super)` for private_interfaces. That is visibility only.
- `.bee/config.json` `commands.test` and the `ci.yml` clippy step now end in `-- -D warnings`, and the concept line cites clippy-warnings-clean D1/D2. The proof_gate tests pass (8 passed).

## Findings (none violate the contract)

1. `packages/bee-rs/crates/bee/src/verbs/state_group/workflows.rs:493`: the clippy fix in cwc-2 joined the blank line into the doc text, which now reads `...refusal);///`. This is cosmetic doc damage.
2. `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs` (cwc-1, machine fix): the line `>= CAPTURE_QUEUE_...` rendered as a markdown blockquote, and clippy continued it with `/// >` on the next two lines. Cosmetic.
3. `packages/bee-rs/crates/bee/src/onboard/agents.rs:184`: a doc line still names the deleted `render_agent_template`. It is a stale reference only.
4. The 21 item-level `#[allow(clippy::too_many_arguments)]` existed before this feature and are now redundant with the workspace allow. Removing them is a possible follow-up. D2 forbids adding allows, not keeping these.
5. cwc-2 appended one runtime line to the tracked log `packages/bee-rs/crates/bee/.bee/logs/timings.jsonl`. It is outside the cell's declared files and is an incidental artifact.
6. Several doc paragraphs were re-indented under list items to satisfy doc_lazy_continuation (`handlers_close.rs`, `handlers_select.rs`, `status_full/store.rs`, `registry.rs`, `phases.rs`). This is rendering only.
