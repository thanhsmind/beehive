# Plan: device-aware-cap

## Summary

Two disjoint cells, one wave. dac-1 makes the herding limit machine-aware
(D1, D2). dac-2 runs the worktree prune sweep after a green close (D3).
Playbook: `skills/bee-planning/playbooks/feature.md`. Lane `small`, recorded
deviation: two product files, one flag (public contracts: two new config
behaviors); the memory read is Linux-only but skips cleanly elsewhere, so
cross-platform is not counted.

SMALLER PATH check: PASS. Each decision lands in one existing function
(`check_limits`, `run_close`); no new module is needed.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The concurrency limit is read only from config | read | packages/bee-rs/crates/bee/src/herding/wave.rs:1101 | HerdingLimits { concurrency: limits["concurrency"].as_u64(), depth: limits["depth"].as_u64() } |
| 2 | A set limit with no readable pane list refuses | read | packages/bee-rs/crates/bee/src/herding/wave.rs:1131 | wave_ledger::Occupancy::Fallback(live) => Some(serde_json::json!({ |
| 3 | Prune has a callable core taking the main root | read | packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs:610 | pub(crate) fn run_prune_core( |
| 4 | Close resolves its root and runs close_handler in run_close | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3899 | let out: R2<Out> = close_handler(&ctx.root, &feature, dry_run, declared, None, &pattern_verdicts) |

## Cells, current slice preview

```json
[
  {
    "id": "dac-1",
    "feature": "device-aware-cap",
    "title": "Make the herding worker limit aware of CPU count and free memory",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["a9e06a0f-2545-4875-8f21-ea4555a77843", "9bbef1a9-e281-444a-8bfc-32f268da8c32", "4bb9da28-16aa-4ec0-b296-dc03f7db0dcc"],
    "files": ["packages/bee-rs/crates/bee/src/herding/wave.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/wave.rs", "docs/history/device-aware-cap/CONTEXT.md"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "In herding/wave.rs, extend herding_limits/limits_refusal/check_limits (search `pub(crate) fn herding_limits`, `pub(crate) fn limits_refusal`, `pub(crate) fn check_limits`). Per D1: when herding.limits.concurrency is unset, use CPU count / 4 (std::thread::available_parallelism), never below 1; an explicit value wins unchanged. Per the herding-limits contract: a CPU-derived default never yields the limits.unverifiable refusal; with a Fallback occupancy it lets the run start. Per D2: read MemAvailable from /proc/meminfo; when it is below herding.limits.min_free_mem_gb (default 4; 0 turns the check off) refuse with reason limits.memory, retryable true, available_gb, min_free_gb and a fix line naming the key. Unreadable memory skips the check. Keep the CPU count and memory reading injectable so tests do not depend on the host. Add unit tests beside the existing limits tests (search `a_run_at_the_concurrency_limit_is_refused_and_retryable`). No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml wave::",
    "must_haves": {
      "truths": [
        "Unset concurrency on a 16-CPU host gives a limit of 4; on a 2-CPU host it gives 1",
        "An explicit herding.limits.concurrency keeps today's behavior, including limits.unverifiable on a Fallback count",
        "A default limit with a Fallback occupancy does not refuse",
        "Available memory below min_free_mem_gb refuses with reason limits.memory and retryable true",
        "Unreadable memory or min_free_mem_gb 0 never refuses for memory"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/herding/wave.rs", "substantive": "CPU default, memory floor, and unit tests for each truth"}],
      "key_links": ["check_limits applies the CPU default and the memory floor"],
      "prohibitions": ["No change to the existing refusal JSON keys", "No new crate dependency"]
    }
  },
  {
    "id": "dac-2",
    "feature": "device-aware-cap",
    "title": "Run the worktree prune sweep after a green close",
    "lane": "small",
    "role": "code",
    "deps": [],
    "decisions": ["14b76800-5dfc-4e29-bab8-2addfa145de5", "ea534256-a4ce-4d67-8344-2d61112e1ff4"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs", "docs/history/device-aware-cap/CONTEXT.md"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3 and the close-prune contract: after a green, non-dry-run bee close (search `pub(crate) fn run_close` and `close_handler(&ctx.root`), call run_prune_core (verbs/worktree/prune.rs, search `pub(crate) fn run_prune_core`) once on the MAIN checkout root (close may serve from a granted worktree; resolve main the way run_prune does) with dry_run false and the default 7-day threshold. Add the prune lines to the human output and a `prune` key to the JSON output. A prune Err adds one line naming the error and never changes the exit code. A dry-run or refused/red close never runs prune. Add tests beside the close tests (search `fn dry_run_and_red_close_never_commit`). No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml close::",
    "must_haves": {
      "truths": [
        "A green close runs prune once on the main checkout and reports its lines",
        "A dry-run close and a refused close never run prune",
        "A prune error leaves the close exit code at success and adds one line"
      ],
      "artifacts": [{"path": "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs", "substantive": "post-close prune call and tests"}],
      "key_links": ["run_close calls run_prune_core after a green close"],
      "prohibitions": ["No change to prune classification rules in prune.rs"]
    }
  }
]
```

## Open Questions

(none)
