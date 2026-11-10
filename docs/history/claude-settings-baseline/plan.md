# Plan: Claude settings baseline at install

## Summary

Today onboarding adds only EnterWorktree and ExitWorktree to a host's
`.claude/settings.json`. After this, `bee onboard` (and so `install.sh` and
`install.ps1`) adds the owner's whole Claude settings baseline: bypass mode,
the two allowed worktree tools, thirteen denied tools and six top-level
switches. It only adds what is missing, never changes a present value, and
`--no-claude-settings` turns the baseline off while keeping the worktree allow.

Mode: `high-risk` — 3 risk flags: audit-security, public-contracts, cross-platform.
Why this is the least workflow that protects the work: it changes the
permission mode of every host project that installs bee.

Class playbook: feature — `skills/bee-planning/references/planning-reference.md` ("Playbooks").

## Requirements (from CONTEXT.md)

- D1, D2, D4: one add-only projection of the baseline → csb-1.
- D3, D5: `--no-claude-settings` and the durable `claude_settings: false` preference → csb-1; installer pass-through → csb-2.
- D6, D7: the notice, and the written-even-without-hooks rule → csb-1; Safety text → csb-2.
- Consolidation → csb-3.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | An add-only settings projection already exists to widen | read | packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs:557 | `pub const WORKTREE_TOOL_ALLOW: [&str; 2] = ["EnterWorktree", "ExitWorktree"];` |
| 2 | Its merge leaves odd shapes unchanged and appends only missing names | read | packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs:586 | `pub fn merge_worktree_tool_allow(settings_path: &Path) -> Merged {` |
| 3 | The plan stage plans it only when something is missing, so re-runs converge | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:1357 | `if hw::worktree_tool_allow_missing(repo_root) {` |
| 4 | The apply arm re-checks before it writes | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:778 | `if hw::worktree_tool_allow_missing(repo_root) {` |
| 5 | Onboard opt-out flags are parsed one by one in mod.rs | read | packages/bee-rs/crates/bee/src/onboard/mod.rs:210 | `} else if arg == "--no-statusline" {` |
| 6 | install.sh passes opt-out flags through to onboard | read | scripts/install.sh:481 | `  ONBOARD_FLAGS+=("--no-statusline")` |
| 7 | install.ps1 passes opt-out flags through to onboard | read | scripts/install.ps1:546 | `  if ($NoStatusline) { $onboardFlags += '--no-statusline' }` |
| 8 | A pinned flag count must move with a new flag | read | packages/bee-rs/crates/bee/src/catalog.rs:823 | `const PINNED_FLAG_COUNT: usize = 243;` |

## Discovery

Read the host-project-artifacts concept (the worktree-tool projection,
5fd24d1b) and the code it anchors. This repo's own `.claude/settings.json`
already holds the owner's baseline, so onboarding this repo plans nothing new.

## Approach

- Widen the one projection: `WORKTREE_TOOL_ALLOW` becomes part of a baseline
  set; `worktree_tool_allow_missing` and `merge_worktree_tool_allow` become
  `claude_settings_missing(repo_root, full)` and `merge_claude_settings(path, full)`
  with the add-only rules of contract 282a650f. One plan item, one apply arm.
- `--no-claude-settings` sets `full=false`, which keeps exactly today's behavior.
- Installers add the flag to their help, their parser and the onboard pass-through.

Hat wave: `docs/history/claude-settings-baseline/reports/hat-wave.md` — findings folded into csb-1 and csb-2 (D5, D6, D7).

Rejected: overwriting present values (breaks the never-overwrite promise and a
host's own defaultMode); a shell-only edit of `install.sh` (install.ps1 and a
plain `bee onboard` would drift).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Host permission mode | HIGH | csb-1 | add-only tests: a present defaultMode is never changed; `--no-claude-settings` writes no defaultMode |
| Existing worktree-allow tests | MEDIUM | csb-1 | renamed tests keep their old assertions under full=false |
| Upgrade of already-onboarded hosts | HIGH | csb-1 | existing-host-shape test; the notice prints on the next onboard |
| --no-hooks install gets bypass mode with no guard | MEDIUM | csb-1, csb-2 | named in the notice and the Safety text (D7) |
| Installer parity | MEDIUM | csb-2 | both scripts parse and pass the flag; `bash -n` and the PowerShell parse check |

## Shape

One slice: the Rust projection, then the installer pass-through, then the writer.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| csb-1 | Project the Claude settings baseline add-only in onboarding | onboard/hooks_wiring.rs, onboard/plan.rs, onboard/apply.rs, onboard/mod.rs, onboard/tests.rs, registry_payload.json, catalog.rs, hooks/cli_shape.rs, reservations/flags.rs | — | onboarding a fresh repo writes the baseline; a second run plans nothing; a present defaultMode stays | `cargo test … -p bee -- claude_settings onboard` |
| csb-2 | Pass --no-claude-settings through both installers | scripts/install.sh, scripts/install.ps1 | csb-1 | both installers name the flag in their help and hand it to onboarding | `bash -n scripts/install.sh && bee dev release-manifest --check` |
| csb-3 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md | csb-1, csb-2 | the onboarding concept states the baseline and its add-only rules | `bee knowledge check` |

```json
[
  {
    "id": "csb-1",
    "feature": "claude-settings-baseline",
    "title": "Project the Claude settings baseline add-only in onboarding",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "bd6ebab1",
      "282a650f",
      "5fd24d1b",
      "118c4686",
      "a9daf9d2",
      "9f22ec8d"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs",
      "packages/bee-rs/crates/bee/src/hooks/cli_shape.rs",
      "packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs"
    ],
    "read_first": [
      "docs/history/claude-settings-baseline/CONTEXT.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract 282a650f. In onboard/hooks_wiring.rs widen the worktree-tool projection into the Claude settings baseline of decision bd6ebab1: one constant set (allow EnterWorktree, ExitWorktree; deny EnterPlanMode, ExitPlanMode, DesignSync, NotebookEdit, SendMessage, PushNotification, RemoteTrigger, ReportFindings, ScheduleWakeup, AskUserQuestion, CronCreate, CronDelete, CronList; permissions.defaultMode bypassPermissions; top-level disableBundledSkills, disableWorkflows, disableRemoteControl, disableClaudeAiConnectors, disableArtifact true and syncClaudeAiSkills false). Replace worktree_tool_allow_missing and merge_worktree_tool_allow with claude_settings_missing(repo_root, full) and merge_claude_settings(path, full); full=false owns only the two allow names, exactly today's behavior. Add-only: append missing allow/deny names after existing entries in baseline order; write defaultMode only when permissions has none; write a top-level key only when absent; keep every other key, entry and order; a parse failure, non-object top level, non-object permissions or non-array allow/deny returns the text unchanged. Rename the plan item to merge_claude_settings in plan.rs and its apply arm in apply.rs (apply re-checks before writing). Add the --no-claude-settings onboard flag (bool, default off) in mod.rs and the registry, and a project preference read exactly like statusline_config_preference (.bee/config.json \"claude_settings\": false, never written by onboarding); full is false when the flag is given OR the preference is false (D5). claude_settings_missing returns false for every shape the merge refuses, and deny, defaultMode and the switches are inspected only when full is true. In notices.rs add claude_settings_notices, emitted in plan and apply output whenever merge_claude_settings with full=true is listed: one 'notice: ' line saying Claude permission prompts are turned off for this project and the baseline tools are denied, naming each baseline key kept at the project's own value and each baseline deny name the project's allow list already carried, saying the baseline is written even when bee's guard hooks are not wired, and naming --no-claude-settings and \"claude_settings\": false as the opt-outs (D6, D7); bump any pinned flag count (catalog.rs PINNED_FLAG_COUNT, reservations/flags.rs, cli_shape.rs) with the new flag. Red first: keep the assertions of the four hooks_wiring.rs worktree tests (hooks_wiring.rs:1422, :1444, :1474, :1492) under full=false, renamed, and update the six onboard/tests.rs end-to-end tests at :2827-:2929 that pin the item name merge_worktree_tool_allow; and add tests: a fresh file gets the whole baseline; a file with defaultMode default and an allow list keeps both and gains the rest; deny entries are appended once; a second merge is unchanged; --no-claude-settings on a fresh repo writes only the two allow names; an onboard apply then a plan reports up_to_date; an already-onboarded host shape (the two allow names, hooks and statusLine present, no deny, no defaultMode) gains the baseline, keeps every hooks and statusLine key in order, and takes one .bak; a non-array deny plans nothing and reports up_to_date; \"claude_settings\": false with a bare apply writes no defaultMode, deny or switch; the notice names a kept defaultMode and an allowed-then-denied tool. Knowledge delta in the same commit: in docs/knowledge/areas/onboarding/host-project-artifacts.md widen the worktree-tool paragraph to the baseline and its opt-out, citing this feature's CONTEXT.md D1-D3 (not raw decision ids), with a Read when: line if the concept lacks one. No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee -- claude_settings onboard",
    "must_haves": {
      "truths": [
        "Onboarding a repo with no .claude/settings.json writes the whole baseline",
        "A present permissions.defaultMode, a present top-level key and existing allow/deny entries are never changed or reordered",
        "Missing allow and deny names are appended once; a second apply plans nothing",
        "The --no-claude-settings flag writes only EnterWorktree and ExitWorktree, as before",
        "An unparsable or odd-shaped settings file is left unchanged",
        "An opted-out host (flag or config preference) gets no baseline on a later bare apply",
        "Onboarding prints a notice whenever it plans or writes the full baseline, naming kept keys and the opt-outs",
        "An already-onboarded host gains the baseline with its hooks and statusLine untouched"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/hooks_wiring.rs",
          "substantive": "the baseline constant set, claude_settings_missing and merge_claude_settings with tests"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/mod.rs",
          "substantive": "the --no-claude-settings flag"
        }
      ],
      "key_links": [
        "plan.rs plans merge_claude_settings only when claude_settings_missing; apply.rs re-checks before writing"
      ],
      "prohibitions": [
        "Never overwrite a present value",
        "Never touch ~/.claude/settings.json",
        "No code comments"
      ],
      "open_choices": [
        "Whether full=false is a separate constant or a filter over the baseline"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "csb-2",
    "feature": "claude-settings-baseline",
    "title": "Pass --no-claude-settings through both installers",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "csb-1"
    ],
    "decisions": [
      "bd6ebab1",
      "118c4686",
      "9f22ec8d"
    ],
    "files": [
      "scripts/install.sh",
      "scripts/install.ps1",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/claude-settings-baseline/CONTEXT.md",
      "scripts/install.sh",
      "scripts/install.ps1"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Mirror --no-statusline exactly. scripts/install.sh: add --no-claude-settings to the usage text (one entry saying it skips the Claude settings baseline — bypass mode, the denied tools and the six switches — and keeps the worktree-tool allow), to the argument parser, and to the ONBOARD_FLAGS pass-through; add one Safety bullet saying the baseline is add-only and never overwrites a present value, that an already-onboarded project gains it on its next install or onboard apply, that it is written even with --no-hooks, and that \"claude_settings\": false in .bee/config.json opts out for good. scripts/install.ps1: add the matching -NoClaudeSettings switch, its help line, and the '--no-claude-settings' pass-through into $onboardFlags. Prove with bash -n scripts/install.sh, a pwsh parse check of install.ps1 when pwsh exists (name it skipped otherwise), and rg showing the flag in both scripts. Then run .bee/bin/bee dev regen so the release manifest records the new installer bytes, and commit the manifest with the scripts. One commit.",
    "verify": "bash -n scripts/install.sh && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "install.sh names --no-claude-settings in its help and passes it to onboarding",
        "install.ps1 accepts -NoClaudeSettings and passes --no-claude-settings to onboarding",
        "install.sh's Safety section says the baseline is add-only"
      ],
      "artifacts": [
        {
          "path": "scripts/install.sh",
          "substantive": "flag in usage, parser and ONBOARD_FLAGS"
        },
        {
          "path": "scripts/install.ps1",
          "substantive": "switch, help and pass-through"
        }
      ],
      "key_links": [
        "both installers reach the same onboarding flag"
      ],
      "prohibitions": [
        "No settings edit in shell or PowerShell code",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "csb-3",
    "feature": "claude-settings-baseline",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "high-risk",
    "role": "writer",
    "status": "open",
    "deps": [
      "csb-1",
      "csb-2"
    ],
    "decisions": [
      "bd6ebab1",
      "118c4686",
      "a9daf9d2",
      "9f22ec8d"
    ],
    "files": [
      "docs/knowledge/**",
      "docs/history/claude-settings-baseline/delivery.md"
    ],
    "read_first": [
      "docs/history/claude-settings-baseline/CONTEXT.md",
      "docs/history/claude-settings-baseline/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature claude-settings-baseline`. Dedupe and trim the deltas the cells wrote, anchor each concept to the code it describes, never restate code, and write docs/history/claude-settings-baseline/delivery.md listing the concepts changed and why, one line each. Cite this feature's CONTEXT.md D-numbers, never raw decision ids, and state in one sentence that denying AskUserQuestion makes gate questions plain chat text in hosts, and route every D-ID (D1-D7) into a docs/knowledge concept. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "the onboarding concept states the baseline, add-only rules and opt-out, anchored to code",
        "delivery.md lists each concept changed with one line on why"
      ],
      "artifacts": [
        {
          "path": "docs/history/claude-settings-baseline/delivery.md",
          "substantive": "one line per concept changed"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No raw decision ids in docs outside docs/history/claude-settings-baseline/"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Behavior | Test |
|---|---|
| fresh file, add-only merge, odd shapes, convergence | onboard/hooks_wiring.rs tests |
| the opt-out keeps today's behavior | onboard/tests.rs |
| installers parse and pass the flag | bash -n, pwsh parse |

## Open Questions

- (none)

## Out of scope

- `~/.claude/settings.json`; Codex and Pi settings.
