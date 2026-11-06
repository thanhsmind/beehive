# thin-worker-boot — plan

Lane: high-risk (changes how every worker in every project boots).
Decisions: see CONTEXT.md D1-D7.

## Slices

1. **Fix-first: herding reads merged config (D7).** One cell, `twb-1`.
   A known-red base: cheap herding workers cannot spawn, so nothing else
   can be dispatched until this lands.
2. **Measure (D5).** Feature usage counts pane sessions; caps record
   per-cell tokens. Gives the before numbers for slice 4.
3. **Thin boot (D1-D4, D6).** Behind a config switch:
   - peer role text and a role-aware session-init (peer gets role text
     only, leader gets the flow);
   - thin shared AGENTS/CLAUDE block plus a leader-only doctrine file,
     in this repo and in the onboarding template;
   - trimmed pane launch (memory off, no skills, no extra MCP, dynamic
     sections out of the system prompt).
4. **Pilot and default (D6).** Run 3-5 cells each way, compare, flip
   the default only on a saving.
5. **Doctrine split for native subagents and Codex (D2, D9).** Moved
   after the pilot: with D10 (`458619d9`) the pane peers skip CLAUDE.md
   entirely, so the split only serves Agent-tool subagents and Codex
   workers. Cells are written after slice 4 shows the saving.

Plan rev 1: twb-3a (peer preamble) and twb-3b (pane env) hold disjoint
files and run in parallel; twb-2a runs beside them on its own files;
twb-2b waits on twb-2a.

Slices 2 and 3 get their cells after a code read that slice 1 unblocks.
Slice 2 before slice 4 is a real dependency (the comparison needs the
numbers); slices 2 and 3 can run in parallel on disjoint files.

Slice 1 note: `read_main_config` has 13 call sites (wave, pane_verbs,
fork, job_verbs, broker); one helper change fixes all. `read_config_raw`
warns on stderr for corrupt JSON, so hook paths stay quiet only while
the config parses.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | herding run reads only the tracked config | read | `packages/bee-rs/crates/bee/src/herding/run.rs:5435` | `let cfg_path = main_root.join(".bee").join("config.json");` |
| 2 | the merged reader overlays config.local.json | read | `packages/bee-rs/crates/bee/src/state.rs:177` | `let overlay = read_obj(root.join(".bee").join("config.local.json"));` |
| 3 | a local-only agent fails at spawn | ran | `bee herding run --agent paseo-claude-haiku` (3 jobs, 2026-11-06) | `unknown herding agent "paseo-claude-haiku" (herding.agents declares: agy-flash, claude-sonnet, ...)` |
| 4 | guard hooks are project-level | read | `.claude/settings.json:13` | `"SessionStart": [` with `bee hook session-init` under project settings |
| 5 | pane boots rewrite most of their prompt cache | ran | first-turn usage of 7 pane transcripts in `~/.claude/projects/*beehive--wt--*` | `cache_read 13013 cache_write 35823` (Sonnet pane) |

## Advisor consult (fable, 2026-11-06)

Approve slice 1 now; hold slices 3-4 until the open questions close.
Carried into the plan:

- paseo passes only `--env` (`paseo.rs:217-271`): the trimmed launch
  needs a provider `command` wrapper or env-only settings. Close this
  as a decision before slice 3 gets cells.
- The leader-only file must never be `@`-imported; role-aware
  session-init injects it, and the role signal is an `--env` variable.
- Codex leaders have no SessionStart hook: slice 3 names how they load
  the leader file.
- The switch off must give a byte-identical boot (snapshot test); the
  onboarding template gets a parity test.
- Slice 2 numbers are taken before slice 3 merges.

## Cells — current slice (preview)

| id | title | issue |
|---|---|---|
| twb-1a | Make herding run read the merged tracked and local config (issue 166) | D7 60186f26 |
| twb-2a | Count herding pane sessions in the feature usage record | 16eb7a31 |
| twb-2b | Record each capped cell's own token totals | 16eb7a31 |
| twb-3a | Give herding workers a short peer preamble instead of the leader preamble | 16cccbb2, 458619d9, 2d911143 |
| twb-3b | Launch herding Claude panes with the trimming env when the switch is on | 458619d9, 3173e22f, 2d911143 |

```json
[
  {
    "id": "twb-1a",
    "feature": "thin-worker-boot",
    "lane": "tiny",
    "role": "code",
    "change_class": "bugfix",
    "affects_skills": [],
    "affects_specs": [],
    "behavior_change": true,
    "deps": [],
    "title": "Make herding run read the merged tracked and local config",
    "action": "read_main_config in packages/bee-rs/crates/bee/src/herding/run.rs reads only .bee/config.json, while dispatch prepare reads the merged view (state::read_config_raw). Return Value::Object(crate::state::read_config_raw(main_root)) instead, so an agent declared only in .bee/config.local.json herding.agents resolves. Add a red-first unit test: tracked config without the agent, local config declaring a paseo agent, PaseoSpec::from_config on read_main_config's result returns Some.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::",
    "must_haves": {
      "truths": [
        "an agent declared only in config.local.json herding.agents is found by herding run",
        "the new test fails before the fix and passes after",
        "the herding test module is green"
      ]
    },
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/state.rs"
    ],
    "decisions": [
      "60186f26"
    ]
  },
  {
    "feature": "thin-worker-boot",
    "affects_skills": [],
    "affects_specs": [],
    "deps": [],
    "id": "twb-2a",
    "lane": "standard",
    "role": "code",
    "change_class": "behavior",
    "behavior_change": true,
    "title": "Count herding pane sessions in the feature usage record",
    "action": "Today `bee close` writes .bee/usage/<feature>.json (bee-usage/v1, verbs/drivers/close.rs) from the leader session's bee-perf rows; Claude sessions started by `bee herding run` in panes (paseo, herdr, claude-session) are missing, so a feature that used ~9.2M Sonnet tokens in panes records 39k. Make the record include every pane session that ran a herding job for the feature. Find the link yourself: panes carry BEE_HERDING_JOB_ID and BEE_HERDING_WORKER=1 in their env, the job file in the mailbox names the cell and feature, and hooks/session_close/perf.rs writes bee-perf rows at session end. Check whether pane sessions write rows at all and with which project path; if they do not, add what is needed so they do (or read the pane transcript usage, deduplicated by message id). Contract: bee-usage/v1 gains a `herding` block shaped like `subagents` ({new, cached, total} plus per-model rows), counted inside `totals.total`; a session counted as main or subagent is never counted again as herding. Records without herding sessions keep their current bytes. Pane sessions run with the worktree as cwd (perf.rs derives the project from the transcript cwd, ~763-834), so join pane sessions by cwd plus BEE_HERDING_JOB_ID, not by the main root. Do NOT edit herding/run.rs (another cell owns it); if the job record needs a new field, stop and report it.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml usage",
    "must_haves": {
      "truths": [
        "a feature whose cells ran in herding panes shows those panes' tokens in its usage record, per model",
        "a pane session is counted once",
        "a usage record with no pane sessions is byte-identical to today's",
        "tests cover one pane session and one leader-only feature"
      ]
    },
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs"
    ],
    "decisions": [
      "16eb7a31"
    ]
  },
  {
    "feature": "thin-worker-boot",
    "affects_skills": [],
    "affects_specs": [],
    "deps": [
      "twb-2a"
    ],
    "id": "twb-2b",
    "lane": "standard",
    "role": "code",
    "change_class": "behavior",
    "behavior_change": true,
    "title": "Record each capped cell's own token totals",
    "action": "When a cell is capped, record the tokens its worker used: per model, new / cached / total, from the same source twb-2a uses for pane sessions (the herding job of that cell), or from the subagent rows when the cell ran as a native subagent. Store it on the cell record (field `usage`), show it in `bee cells show`. A cell with no attributable session gets `usage: null` and a reason, never a guessed number.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml cells",
    "must_haves": {
      "truths": [
        "a capped cell that ran through herding carries its per-model token totals",
        "a cell with no attributable session records null plus a reason",
        "no number is estimated"
      ]
    },
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/close.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/mod.rs"
    ],
    "decisions": [
      "16eb7a31"
    ]
  },
  {
    "feature": "thin-worker-boot",
    "affects_skills": [],
    "affects_specs": [],
    "deps": [],
    "id": "twb-3a",
    "lane": "standard",
    "role": "code",
    "change_class": "behavior",
    "behavior_change": true,
    "title": "Give herding workers a short peer preamble instead of the leader preamble",
    "action": "Add config switch `dispatch.thin_peer_boot` (bool, default false; document it in docs/config-reference.md). When it is true AND the session env has BEE_HERDING_WORKER=1, `bee hook session-init` (hooks/session_init.rs) and `bee hook prompt-context` (hooks/prompt_context.rs) emit ONLY the text of a new embedded file packages/bee/prompts/peer-boot.md (include_str!, like verbs/drivers/prompt.rs:33), and nothing else: no handoff, no phase, gates, counts, principles or project map. Side effects (session registration, heartbeats, activity) stay as they are. Switch false, or env without the marker: output byte-identical to today (snapshot test). peer-boot.md content, under 1500 chars, stable text only (no dates, ids or counts): you are a bee peer doing the one job in your brief; work only in the files and scope the brief grants, a guard refusal means stop and report; never read or print secret-shaped files; never edit .bee state by hand, run only the bee commands the brief names; no code comments except a #! line, a SAFETY: line, a license header; never start agents or workflows; do not read AGENTS.md, CLAUDE.md or bee process docs, the brief is complete, report what is missing; run the brief's verify command and report its real output, never write done or green without it; end with the brief's finish command exactly as written. Reuse the existing BEE_HERDING_WORKER predicate in hooks/mod.rs (~95) instead of a new env read.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml hooks::",
    "must_haves": {
      "truths": [
        "switch on plus BEE_HERDING_WORKER=1: session-init and prompt-context print only peer-boot.md",
        "switch off: both outputs byte-identical to today",
        "peer-boot.md is under 1500 chars and has no dynamic value",
        "session registration still happens for the worker"
      ]
    },
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee/prompts/peer-boot.md",
      "docs/config-reference.md"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      "packages/bee-rs/crates/bee/src/hooks/prompt_context.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prompt.rs"
    ],
    "decisions": [
      "16cccbb2",
      "458619d9",
      "2d911143"
    ]
  },
  {
    "feature": "thin-worker-boot",
    "affects_skills": [],
    "affects_specs": [],
    "deps": [],
    "id": "twb-3b",
    "lane": "standard",
    "role": "code",
    "change_class": "behavior",
    "behavior_change": true,
    "title": "Launch herding Claude panes with the trimming env when the switch is on",
    "action": "When config `dispatch.thin_peer_boot` is true (the key twb-3a documents; read it through state::read_config_raw), every Claude pane that `bee herding run` starts (paseo transport via paseo_env_pairs / run_argv --env, herdr and tmux pane export at herding/run.rs ~2716-2723, claude-session spawn env at ~3946) also gets CLAUDE_CODE_DISABLE_CLAUDE_MDS=1, CLAUDE_CODE_DISABLE_AUTO_MEMORY=1, ENABLE_CLAUDEAI_MCP_SERVERS=false, CLAUDE_CODE_DISABLE_BUNDLED_SKILLS=1. Only for Claude-provider agents (paseo provider claude, or a claude command); pi, codex and other agents get nothing new. Switch off: argv and env byte-identical to today. A per-agent env value of the same name wins over the default. Insert the four defaults BEFORE the per-agent env is merged (run.rs ~2716), so a per-agent value wins and only the BEE_HERDING_* markers are forced after.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding::",
    "must_haves": {
      "truths": [
        "switch on: a paseo claude agent's run argv carries the four --env pairs",
        "switch on: a pi agent's argv is unchanged",
        "switch off: all transports unchanged",
        "a per-agent env value overrides the default"
      ]
    },
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs"
    ],
    "decisions": [
      "458619d9",
      "3173e22f",
      "2d911143"
    ]
  }
]
```
