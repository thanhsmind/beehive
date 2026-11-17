# gis-2t check: date, echo, stat and pwd in a read-only herded worker (issue 170)

Checked: cell gis-2, commit 0607c94d2, decision D3 (6d88458e). The tester did not use the gis-2 worker's context.

## Reproduction

The parent commit is 0607c94d2^. On it, `evaluate_read_only("Bash", {"command": "date -u"}, ...)` returns the refusal from issue 170:
"bee worker-guard denied this call: `date -u` is not allowed — this herded run is read-only."
On the fixed code the same call returns `None`, which means it is allowed. The gis-2 fix adds `Bash(date:*)`, `Bash(echo:*)`, `Bash(stat:*)` and `Bash(pwd:*)` to `READ_ONLY_ALLOWED` at `packages/bee-rs/crates/bee/src/hooks/worker_guard.rs:10`.

## Truths

| Truth | Result | Anchor |
|---|---|---|
| `date -u`, `echo ok`, `stat x` and `pwd` are allowed in a read-only herded run | PASS | `worker_guard.rs:609` (gis-2 test), `worker_guard.rs:641` (tester edge test: `date -u +%FT%TZ >> mailbox/progress.jsonl`, `stat -c %s ... 2>/dev/null`, `pwd; echo ok`) |
| `echo` with a redirect outside the mailbox is still refused | PASS | `worker_guard.rs:629`, `worker_guard.rs:659-666` (`>>`, `>|`, `/tmp`, `mailbox/../escape`, `2>&1 > mailbox/../../x`, each refused as "outside the job mailbox") and the guard at `worker_guard.rs:346` |
| command substitution is still refused | PASS | `worker_guard.rs:630`, `worker_guard.rs:668-670` (`"$(date -u)"` inside double quotes, `<(stat a)`, `stat $(pwd)/a`, each refused as "command substitution") and the guard at `worker_guard.rs:342` |

## Red on the parent commit

The tester built a detached worktree at 0607c94d2^, kept that commit's `READ_ONLY_ALLOWED` and added both test functions. Result: `8 passed; 2 failed`.

- `read_only_allows_date_echo_stat_and_pwd` FAILED: expected allow: `date -u` -> Some("... `date -u` is not allowed ...")
- `read_only_echo_and_date_redirect_edges` FAILED: expected allow: `date -u +%FT%TZ >> '.../progress.jsonl'` -> Some("... `date -u +%FT%TZ` is not allowed ...")

Truths 2 and 3 are refusals that were already in place. The parent commit refused these commands too, through the substitution check and the redirect check. The fix did not change either check, and both run before the allowlist. Each truth is therefore red on the parent only as part of its test function, where the allow cases fail first. On their own, the deny assertions guard against regressions and cannot be red on the parent.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- hooks::worker_guard`

Result: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 5392 filtered out`. The run includes `hooks::worker_guard::tests::read_only_allows_date_echo_stat_and_pwd ... ok` and `hooks::worker_guard::tests::read_only_echo_and_date_redirect_edges ... ok`.

## Found, out of scope

`Bash(date:*)` also allows `date -s <time>`, which sets the system clock. This needs root, so it fails for a normal worker. D3 names `date` without qualification, so this check records the behavior and does not count it as a failure.
