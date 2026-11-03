# Plan: paseo-worker-subagent

## Summary
A Paseo herded worker becomes a subagent of the lead that started it, and
still runs in its worktree workspace (decision c09228a8, which narrows
paseo-worktree-workspace D5). Only the one `paseo run` that starts the
worker keeps `PASEO_AGENT_ID` and `PASEO_AGENT_CWD`; every other paseo
process bee starts stays scrubbed. `--workspace` stays on the run, so the
worker never lands in the lead's workspace or cwd.

Mode: `small` — 1 risk flag: external-systems. One cell, one source file plus
its knowledge concept.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | every paseo process is scrubbed today | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:864 | let mut child = scrubbed_command(command) |
| 2 | the worker is started through cli.call(run_argv) | read | packages/bee-rs/crates/bee/src/herding/run.rs:3623 | let run_res = cli.call(&argv); |
| 3 | the run already gets --workspace inside Paseo | read | packages/bee-rs/crates/bee/src/herding/run.rs:3600 | let workspace = match resolve_paseo_workspace(cli, &opts.main_root, &opts.cwd) { |
| 4 | Paseo takes the parent from the caller even with an explicit workspace | read | @getpaseo/server create-agent/intent.js | const parentAgentId = input.legacyDetached ? null : (input.caller?.id ?? null); |
| 5 | an explicit workspace with a caller gives worktree cwd and lead parent | ran | paseo run --workspace wks_44cf… from agent ed6469dc | {"Cwd":"/mnt/data/Projects/goglbe/beehive--wt--paseo-cli-autodetect","ParentAgentId":"ed6469dc-…"} |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pwsa-1 | Start Paseo workers as subagents of the calling lead | paseo.rs, run.rs, the-paseo-channel.md | — | a worker shows under its lead in Paseo and still runs in the worktree | herding::paseo and herding::run tests |

```json
[
  {
    "id": "pwsa-1",
    "feature": "paseo-worker-subagent",
    "lane": "small",
    "role": "code",
    "change_class": "bugfix",
    "title": "Start Paseo workers as subagents of the calling lead",
    "deps": [],
    "decisions": ["c09228a8-2bef-4777-a33e-76a3a1698705"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Per decision c09228a8. In packages/bee-rs/crates/bee/src/herding/paseo.rs add a second method to trait PaseoCli (about line 802): fn call_as_caller(&self, args: &[String]) -> Result<String, String>, with a default body that returns self.call(args) so every fake keeps working. RealPaseoCli implements it like call but spawns WITHOUT removing AGENT_SCOPE_VARS (give run_with_deadline a keep_scope: bool, or an unscrubbed twin of scrubbed_command); call itself stays scrubbed. In packages/bee-rs/crates/bee/src/herding/run.rs execute_paseo (about line 3623) start the worker with cli.call_as_caller(&argv) instead of cli.call(&argv); keep resolve_paseo_workspace and --workspace exactly as they are, and keep every other paseo call (ls_label_argv fallback, inspect, wait, archive, send, workspace calls, prepare probe) on the scrubbed path. The --env pairs given to the worker still exclude AGENT_SCOPE_VARS. Tests red first: beside paseo_children_do_not_see_agent_scope_vars (paseo.rs about line 1786, same env lock and restore) add a test that RealPaseoCli call_as_caller, plain and with_timeout, DOES see PASEO_AGENT_ID and PASEO_AGENT_CWD, while call still does not; in run.rs, with the existing fake PaseoCli used by the execute_paseo tests, assert the run argv goes through call_as_caller and the other calls do not (record which method each argv used). In docs/knowledge/areas/bee-herding/the-paseo-channel.md rewrite the lines that say a worker has no ParentAgentId and that every paseo process runs without the two variables (about lines 96, 217, 441) to state: the worker-starting paseo run keeps them so the worker is the lead's subagent, other paseo processes run without them, citing c09228a8 and that it narrows 57793b86 D5; add c09228a8 to the frontmatter decisions list. Write no code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::",
    "must_haves": {
      "truths": [
        "the paseo run that starts a worker keeps PASEO_AGENT_ID and PASEO_AGENT_CWD",
        "every other paseo process bee starts runs without them",
        "the worker run still carries --workspace inside Paseo",
        "the worker --env pairs still exclude PASEO_AGENT_ID and PASEO_AGENT_CWD"
      ],
      "prohibitions": [
        "no change to resolve_paseo_workspace or the refuse-before-spawn rule",
        "no new config key"
      ]
    }
  }
]
```
