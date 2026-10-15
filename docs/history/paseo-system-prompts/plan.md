# Plan: paseo-system-prompts

## Summary
Two defects seen live on 2026-10-15 with a Claude leader inside Paseo.
First, every `<paseo-system> Schedule "bee-leader" fired` prompt reached the
prompt-context hook as if the human had spoken, so it cleared the leader's
`gate` wait mark and the user never saw the uat ask. Second,
`tests/pi_plugin_contracts.rs` kept the leader's `PASEO_AGENT_ID` when it ran
the real bee-guard extension, so each session-start test created a real
bee-leader heartbeat on the leader (63 of them). Decision f247ae6c.

Mode: `small` — 0 risk flags. One cell, two files.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | every prompt clears the wait mark | read | packages/bee-rs/crates/bee/src/hooks/prompt_context.rs:371 | clear_and_reap_waiting_on_best_effort(root, ctx, plan.feature.as_deref()); |
| 2 | the hook payload carries the prompt text | read | packages/bee-rs/crates/bee/src/hooks/adapter.rs:266 | pub payload: Map<String, Value>, |
| 3 | a Paseo schedule prompt starts with the paseo-system tag | ran | leader transcript 044d1be3 | <paseo-system> Schedule "bee-leader" fired (id=73e689a4, …) bee heartbeat </paseo-system> |
| 4 | the harness strips only the herding vars | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1164 | cmd.env_remove("BEE_HERDING_WORKER"); |
| 5 | 63 active bee-leader schedules targeted the leader | ran | ~/.paseo/schedules/*.json | 63 fe017d43-… bee-leader, created 03:54–04:16 |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| psp-1 | Keep wait marks on Paseo system prompts and keep tests off the real daemon | prompt_context.rs, pi_plugin_contracts.rs | — | a heartbeat no longer hides a pending question; the test suite creates no Paseo heartbeats | prompt_context tests + pi_plugin_contracts |

```json
[
  {
    "id": "psp-1",
    "feature": "paseo-system-prompts",
    "lane": "small",
    "role": "code",
    "change_class": "bugfix",
    "title": "Keep wait marks on Paseo system prompts and keep tests off the real daemon",
    "deps": [],
    "decisions": ["f247ae6c"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "judge_obligation_ack": "the leader, a different model from the worker, reads the prompt_context diff and runs the tests before cap",
    "action": "Per decision f247ae6c. (1) In packages/bee-rs/crates/bee/src/hooks/prompt_context.rs, in execute() (about line 371), call the clear part of clear_and_reap_waiting_on_best_effort only when the prompt is a human prompt: a prompt is NOT human when ctx.payload's \"prompt\" string, after trim_start, starts with \"<paseo-system>\". On a non-human prompt still run crate::verbs::workflow_store::reap_stale_waiting_on (split the function so the reap always runs and the two clears are skipped). A missing or non-string prompt counts as human, so today's behavior holds. Test red first beside run_clears_a_live_default_waiting_on_mark_through_the_real_hook_entry_point (about line 2755): the same fixture with payload {cwd, prompt: \"<paseo-system>\\nSchedule \\\"bee-leader\\\" fired (id=x, run=y).\\nbee heartbeat\\n</paseo-system>\"} keeps the live mark; and a plain prompt {cwd, prompt: \"ok\"} still clears it. (2) In packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs, in run_harness_spec_with_env (about line 1162), also env_remove PASEO_AGENT_ID, PASEO_AGENT_CWD and PASEO_CLI before the env_vars loop, so a test that needs one passes it through env_vars. Check every test that relies on an inherited PASEO_* value still passes; any Command::new of the bee binary in that file that loads the extension gets the same three env_remove calls. Write no code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prompt_context && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "a <paseo-system> prompt keeps a live waiting_on mark",
        "a human prompt still clears a live waiting_on mark",
        "the stale-mark reap still runs on every prompt",
        "pi_plugin_contracts never passes the caller's PASEO_AGENT_ID, PASEO_AGENT_CWD or PASEO_CLI to the extension"
      ],
      "prohibitions": [
        "no change to the Stop hook turn-end logic",
        "no text heuristic beyond the <paseo-system> prefix"
      ]
    }
  }
]
```
