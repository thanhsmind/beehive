---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: skillstation-principles

## Summary

A project can set `skillstation.principles` to `hub`. Then Claude Code gets
the 25 `bee-principle-*` skills from the Skillstation hub by id, not from
`.claude/skills`, and their descriptions leave every session's context.
Workers get the `skill_get` tool, and orient tells the agent where the
principle bodies live. With the key absent, nothing changes.

Mode: `standard` — 2 risk flags: external systems (the `skillstation` CLI and hub), public contracts (onboard plan output, rendered agent files, orient text)
Why this is the least workflow that protects the work: two serial code cells and one writer cell, plan check inline (6 product files, no hard-gate flag).

Playbook: `skills/bee-planning/playbooks/feature.md` (class `feature`).

## Requirements (from CONTEXT.md)

- D1: config key `skillstation.principles` = `local` (default) | `hub`; Claude runtime only.
- D2: hub mode drops principles from the Claude target sync only when every one is active on the hub; else one `skipped_skills` row.
- D3: `--skillstation` in hub mode publishes the principles from the engine source, commit then activate.
- D4: hub mode adds `mcp__skillstation__skill_get` to the rendered Claude agent `tools:` lines.
- D5: hub mode changes the principle header line to name `skill_get`; one mode helper in `skillstation_group.rs`.
- D6: no hat wave; dev skill trees and release out of scope.
- D7: drop local principles only where the principle index file exists; else one skip row. (superseded by D8)
- D8: the binary carries the principle index; orient names principles in every project.
- D9: onboard --skillstation sets hub mode, publishes, and removes local principles in one apply.
- D10: install.sh --skillstation passes the flag; the recheck runs without it. (superseded by D8)
- D8: the binary carries the principle index; orient names principles in every project.
- D9: onboard --skillstation sets hub mode, publishes, and removes local principles in one apply.
- D10: install.sh --skillstation passes the flag; the recheck runs without it. (superseded by D8)
- D8: the binary carries the principle index; orient names principles in every project.
- D9: onboard --skillstation sets hub mode, publishes, and removes local principles in one apply.
- D10: install.sh --skillstation passes the flag; the recheck runs without it.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | One function renders the principle header for orient and the preamble | read | `packages/bee-rs/crates/bee/src/principles.rs:111` | `pub(crate) fn principle_lines(root: &Path, class: Option<&str>) -> Vec<String> {` |
| 2 | The session preamble calls the same function | read | `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:679` | `lines.extend(crate::principles::principle_lines(` |
| 3 | Agent templates list only built-in tools | read | `packages/bee/agents/bee-worker.md.tmpl:4` | `tools: Read, Edit, Write, Grep, Glob, Bash` |
| 4 | Apply re-renders each agent file through one function | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:870` | `if let Some(rendered) = super::agents::render_claude_agent_file(engine, agent) {` |
| 5 | Onboard skill sync removes target bee-* folders absent from the source | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:292` | `items.push(item("remove_skill", name, name, "installed"));` |
| 6 | The publish path commits then activates a staged skill | read | `packages/bee-rs/crates/bee/src/onboard/apply.rs:360` | `let committed = run(&["skill", "commit", &staged.to_string_lossy(), "--yes", "--json"])` |
| 7 | Onboard already has a flag-gated publish branch | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:1501` | `if opts.skillstation && opts.sync_skills && skill_sync.blocked.is_none() {` |

## Discovery

`compute_skill_sync` builds one `SkillSources` from the engine tree for every
target, and `compute_skill_items` removes any target `bee-*` folder that the
source list does not name. So leaving the principles out of the Claude
target's source list is enough to remove the local copies, with the same
symlink and alias guards. The hub read already has a test seam
(`set_test_hub`), which keeps onboard tests deterministic.

## Approach

Recommended path: one helper `principles_on_hub(root) -> bool` in
`skillstation_group.rs` reads `skillstation.principles` from the merged config.
ssp-1 uses it for the agent `tools:` line (plan and apply render the same
bytes) and the principle header. ssp-2 uses it in onboard: a hub check of
every principle decides the Claude target source list (D2), and
`--skillstation` adds one publish item per principle, staged from the engine
source rendered for claude (D3). ssp-3 documents the key.

Rejected: an alias stub per principle (it keeps the description in context,
which is the cost this feature removes); moving every bee skill to the hub
(workers, version coupling and relative paths break).

Risk map:
- a host left with no principles / HIGH / Lands in ssp-2 / the hub check of every principle gates the drop; a test with one draft principle keeps all local copies.
- local mode output drift / MEDIUM / Lands in ssp-1 and ssp-2 / tests assert byte-identical agent files, orient text and skill plan with the key absent.
- onboard tests seeing a real hub / MEDIUM / Lands in ssp-2 / tests use `set_test_hub`.

Waves: ssp-1, then ssp-2, then ssp-3.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"ssp-1 and ssp-2 change Rust with tests."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Tests ride the code cells; the live run is the leader's."},
    {"stage":"documentation","classification":"not-applicable","role":"docs","reason":"The writer cell owns all docs."},
    {"stage":"knowledge-consolidation","classification":"required","role":"writer","reason":"ssp-3 updates README, config reference, concepts and delivery.md."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release is out of scope (D6)."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"Plan check runs inline at this size."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"No hat wave (D6)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"No hat wave (D6)."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"No hat wave (D6)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"No hat wave (D6)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"No hat wave (D6)."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI changes."}
  ]
}
```

## Shape

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Workers can fetch, orient names the fetch | A hub principle is useless if nobody can load it | with the key set to hub, `bee orient` header names `skill_get` and `.claude/agents/bee-worker.md` lists it | ssp-2 can drop local copies safely |
| 2 | Onboard publishes and drops local principles | The context saving | `bee onboard --skillstation --apply` twice in a hub-mode project leaves no `.claude/skills/bee-principle-*` | Every project can opt in |
| 3 | Docs | The key must be findable | `docs/config-reference.md` names the key | Owner can turn it on |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| ssp-1 | Give agents the hub fetch tool and name it in the principle header | `skillstation_group.rs`, `principles.rs`, `onboard/agents.rs`, `onboard/apply.rs` | — | hub mode: worker agents list `skill_get`, orient says where principles live | `cargo test … --bin bee -- principles agents skillstation session_preamble` green |
| ssp-2 | Serve principles from the hub in onboard | `onboard/skills.rs`, `onboard/plan.rs`, `onboard/apply.rs`, `onboard/tests.rs` | ssp-1 | hub mode: onboard publishes the principles and removes the local copies | `cargo test … --bin bee -- skillstation onboard` green |
| ssp-4 | Keep local principles where orient cannot name them | `onboard/plan.rs`, `onboard/tests.rs`, `principles.rs`, docs | ssp-2 | a host repo in hub mode keeps its principles and gets one skip row | `cargo test … --bin bee -- skillstation onboard` green |
| ssp-5 | Name principles in every project from an index built into bee | `principles.rs`, `onboard/plan.rs`, `onboard/tests.rs` | ssp-4 | orient names principles in any project | `cargo test … --bin bee -- principles onboard` green |
| ssp-6 | Switch a project to hub principles in one onboard --skillstation run | `onboard/plan.rs`, `onboard/apply.rs`, `onboard/tests.rs`, `scripts/install.sh` | ssp-5 | `install.sh --skillstation` switches a project in one run | `cargo test … -- skillstation onboard install` green |
| ssp-7 | Document the one-step switch to hub principles | `README.md`, `docs/config-reference.md`, `docs/knowledge/**`, delivery.md | ssp-6 | docs match the one-step switch | `.bee/bin/bee knowledge check` green |
| ssp-8 | Render agent files in hub mode during the --skillstation switch | `onboard/plan.rs`, `onboard/agents.rs`, `onboard/apply.rs`, `onboard/tests.rs` | ssp-6 | `install.sh --skillstation` exits 0 on an installed project | `cargo test … -- skillstation onboard agents` green |
| ssp-9 | Make the principle publish activate on a real hub and keep local copies fresh | `onboard/plan.rs`, `onboard/apply.rs`, `onboard/skills.rs`, `onboard/tests.rs`, `scripts/install.sh` | ssp-8 | principles go active on a real hub; install logs the publish | `cargo test … -- skillstation onboard install` green |
| ssp-3 | Document the skillstation.principles key | `README.md`, `docs/config-reference.md`, `docs/knowledge/**`, delivery.md | ssp-2 | the key is in the config reference | `.bee/bin/bee knowledge check` green |

```json
[
  {
    "id": "ssp-1",
    "feature": "skillstation-principles",
    "title": "Give agents the hub fetch tool and name it in the principle header",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D1",
      "D4",
      "D5"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/principles.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "docs/history/skillstation-principles/plan.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
      "packages/bee-rs/crates/bee/src/principles.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs"
    ],
    "action": "Add one pub(crate) helper in skillstation_group.rs, `principles_on_hub(root: &Path) -> bool`, true only when the merged config (.bee/config.json overlaid by .bee/config.local.json, through the existing config merge helper) sets `skillstation.principles` to the string `hub`; any other value or absence is false (D1, D5). D5: in principles.rs, when the helper is true for `root`, `principle_lines` appends to the header line the sentence ` Load each one with the skillstation MCP tool skill_get, the slug as skill_id.` (exact text); rows are unchanged, and with the helper false the output is byte-identical to today. D4: the rendered Claude agent file gets `, mcp__skillstation__skill_get` appended to its frontmatter `tools:` line when the helper is true for the repo root; change `render_claude_agent_file` to take the repo root (or a bool) and update both callers, `compute_agent_file_plan` in agents.rs and the `sync_agent_file` arm in apply.rs (around line 870), so plan and apply render the same bytes. A template with no `tools:` line is left as is. With the helper false every rendered agent file is byte-identical to today. Tests: helper true/false/absent/local-overlay cases; header line with and without hub; agent file with and without hub; a re-plan after apply in hub mode carries no sync_agent_file item.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- principles agents skillstation session_preamble",
    "must_haves": {
      "truths": [
        "with skillstation.principles set to hub the principle header line names the skillstation MCP tool skill_get",
        "with skillstation.principles absent the principle lines are byte-identical to before",
        "with skillstation.principles set to hub every rendered Claude agent tools line ends with mcp__skillstation__skill_get",
        "with skillstation.principles absent every rendered Claude agent file is byte-identical to before"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs",
          "substantive": "principles_on_hub helper"
        }
      ],
      "key_links": [
        "principles.rs and agents.rs both call principles_on_hub, not a copy of the config read",
        "plan and apply render agent files through the same function"
      ],
      "prohibitions": [
        "No code comments",
        "No change to the agent templates under packages/bee/agents"
      ],
      "open_choices": [
        "Whether render_claude_agent_file takes the root or a bool"
      ]
    }
  },
  {
    "id": "ssp-2",
    "feature": "skillstation-principles",
    "title": "Serve principles from the hub in onboard",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-1"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "docs/history/skillstation-principles/plan.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs"
    ],
    "action": "D2: when `principles_on_hub(repo_root)` is true and the Claude runtime is in scope, onboard reads each `bee-principle-*` directory of the engine skills source on the hub with `hub_show` through `hub_runner(repo_root)`. When every one is `active`, the skill sync for the repo Claude target only (target kind repo-claude, `.claude/skills`) leaves every `bee-principle-*` entry out of its source list, so the existing deletion loop plans `remove_skill` for local copies. Codex, opencode, plugin and the legacy global targets are unchanged. When any principle is missing or not active, or the hub read skips, the source list is unchanged and onboard reports exactly one `skipped_skills` entry `{target: \"skillstation-principles\", reason}` naming the first failing id or the skip reason. D3: when `opts.skillstation` and hub mode, plan one `publish_skillstation` item per `bee-principle-*` source skill that stages from the engine source rendered for the claude runtime (the same render the Claude sync uses), not from `.claude/skills`; apply runs the existing commit-then-activate path. Without hub mode, plan and apply output are byte-identical to today; without --skillstation the hub is never written. Tests use `set_test_hub`, never the real binary: hub mode with all active plans removal of local principle folders only under .claude/skills and keeps .agents copies; one draft keeps all and gives one skip row; hub skip gives one skip row; --skillstation in hub mode plans one publish item per principle; key absent leaves the plan unchanged; a re-plan after apply in hub mode with all active is up to date.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard",
    "must_haves": {
      "truths": [
        "in hub mode with every principle active on the hub onboard removes the local bee-principle folders under .claude/skills only",
        "in hub mode with one principle not active onboard keeps every local principle and reports one skipped_skills entry with target skillstation-principles",
        "bee onboard --skillstation in hub mode publishes each bee-principle skill from the engine source",
        "with skillstation.principles absent the onboard plan is unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/skills.rs",
          "substantive": "Claude target source list without principles in hub mode"
        }
      ],
      "key_links": [
        "onboard calls principles_on_hub and hub_show from skillstation_group.rs",
        "the publish stages from the engine source, not .claude/skills"
      ],
      "prohibitions": [
        "No code comments",
        "Never change the hub without --skillstation",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": [
        "Where in compute_skill_sync the Claude target source list is filtered"
      ]
    }
  },
  {
    "id": "ssp-3",
    "feature": "skillstation-principles",
    "title": "Document the skillstation.principles key",
    "lane": "standard",
    "role": "writer",
    "deps": [
      "ssp-2"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3",
      "D4",
      "D5"
    ],
    "files": [
      "README.md",
      "docs/config-reference.md",
      "docs/knowledge/**",
      "docs/history/skillstation-principles/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/config-reference.md"
    ],
    "action": "Document the key: docs/config-reference.md gets `skillstation.principles` (values local and hub, default local, Claude Code only, two onboards to switch). README.md § Skillstation gets one short paragraph. Extend docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md with a section on hub-served principles (D1-D5) and add one line to the Principle homes section of router-triage-and-the-agents-md-duplication-boundary.md pointing at it (one fact, one home). Cite `skillstation-principles D<n>`, never raw decision ids. Run `bee knowledge consolidate --feature skillstation-principles`, write docs/history/skillstation-principles/delivery.md, run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "docs/config-reference.md names skillstation.principles with its two values",
        "the skillstation onboarding concept explains hub-served principles"
      ],
      "artifacts": [
        {
          "path": "docs/history/skillstation-principles/delivery.md",
          "substantive": "the feature's delivery record"
        }
      ],
      "key_links": [
        "the delivery lands in the same merge as the code"
      ],
      "prohibitions": [
        "No restating of code in concepts",
        "No code comments"
      ],
      "open_choices": []
    }
  },
  {
    "id": "ssp-4",
    "feature": "skillstation-principles",
    "title": "Keep local principles where orient cannot name them",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-2"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md"
    ],
    "decisions": [
      "D2",
      "D7",
      "608d5298-e450-47b0-ac10-6655f4999e04",
      "803d608e-c159-41fe-b9c0-e1b3750ae841"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/principles.rs",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/config-reference.md",
      "README.md"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/principles.rs",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md"
    ],
    "action": "D7: in onboard plan, hub mode drops the local principles only when the repo root holds the file principles::PRINCIPLE_INDEX (make the constant reachable from onboard, pub(crate) is enough; do not copy the path string). Check this before the hub read: when the file is absent, principles_from_hub stays false, no hub read runs for principles, and onboard adds exactly one skipped_skills entry {target: \"skillstation-principles\", reason: \"<PRINCIPLE_INDEX> is absent, so bee orient cannot name principles in this repo\"}. When the file is present the ssp-2 behavior is unchanged. The --skillstation publish is unchanged. Update the existing ssp-2 hub-mode tests so their repo carries the index file, and add a test: hub mode, every principle active, no index file -> no remove_skill for any bee-principle folder and one skip row naming the index. Docs: say in docs/config-reference.md (the skillstation.principles row), README.md (the Skillstation paragraph; also fix its count: the source has 24 bee-principle skills, not 25) and the onboarding concept that the local copies are removed only in a repo that holds the principle index, which today is the bee repo itself; cite skillstation-principles D7. One commit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard",
    "must_haves": {
      "truths": [
        "in hub mode with every principle active but no principle index file onboard keeps every local principle and reports one skipped_skills entry naming the index",
        "in hub mode with the principle index file and every principle active onboard still removes the local principle folders under .claude/skills"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/plan.rs",
          "substantive": "principle index check before the principle drop"
        }
      ],
      "key_links": [
        "onboard reads the index path from principles::PRINCIPLE_INDEX, not a copy"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": []
    }
  },
  {
    "id": "ssp-5",
    "feature": "skillstation-principles",
    "title": "Name principles in every project from an index built into bee",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-4"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D8",
      "c52e40d8-8044-456b-ab34-ccca85555bb9"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/principles.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/principles.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/version.rs"
    ],
    "action": "D8: compile the principle index into the binary with include_str! of the repo file named by PRINCIPLE_INDEX (path relative to principles.rs is ../../../../../docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md; version.rs:32 is the precedent). principle_lines reads the repo's own file when it exists, else uses the built-in copy, so a project without that file still gets principle lines for its route class (with the hub sentence in hub mode, as ssp-1 made it). Then remove the ssp-4 D7 guard in onboard plan.rs (the `repo_root.join(PRINCIPLE_INDEX).is_file()` branch and its skip row): every repo now has an index. Update the ssp-4 tests: the no-index test now expects the principle drop to follow the hub check alone. Tests: principle_lines in a temp root with no index file returns a non-empty block for class bugfix and names bee-principle-red-before-green; a temp root with its own index file still uses that file (existing test).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- principles onboard",
    "must_haves": {
      "truths": [
        "principle_lines in a repo without the index file returns the principle block from the copy built into bee",
        "principle_lines in a repo with its own index file reads that file",
        "onboard in hub mode no longer skips the principle drop for a missing index file"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/principles.rs",
          "substantive": "built-in index fallback"
        }
      ],
      "key_links": [
        "the built-in copy comes from the PRINCIPLE_INDEX file at build time, not a second hand-written list"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": []
    }
  },
  {
    "id": "ssp-6",
    "feature": "skillstation-principles",
    "title": "Switch a project to hub principles in one onboard --skillstation run",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-5"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D9",
      "D10",
      "8076f811-1199-4e31-b79c-b21c64267a97",
      "7a5e21b0-85ca-4a61-b65d-f2dfebadd6ef"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "scripts/install.sh",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "scripts/install.sh",
      "packages/bee-rs/crates/bee/src/devtools/install_support.rs"
    ],
    "action": "D9: when `opts.skillstation` and the Claude runtime is in scope, onboard treats the project as hub mode even when skillstation.principles is not hub yet. Plan: (a) when the tracked .bee/config.json does not set skillstation.principles to hub, one item `set_principles_hub` (path .bee/config.json) that apply runs by setting the key to the string hub, keeping every other key, through the existing config write helper; (b) the principle publish items as today; (c) for every local `.claude/skills/bee-principle-*` folder that the hub check does not already remove, a `remove_skill` item carrying `\"after_publish\": true`. Apply runs every after_publish item only after all principle publish items ran, and only when every principle publish succeeded; when one failed it runs none of them and adds one skipped_skills entry {target: \"skillstation-principles\", reason} naming the failed id. The local copies under .agents, .opencode and the plugin trees stay. Without --skillstation nothing here changes: the mode comes from config alone (ssp-2 behavior). D10 in scripts/install.sh: update the --skillstation help text to say it switches the project to principles served from the Skillstation hub and removes the local principle copies; the post-apply RECHECK runs with ONBOARD_FLAGS minus --skillstation (build a second array), so it checks local state and does not plan a publish. install.sh is a release-manifest root: run `bee dev regen` and commit the manifest change. Tests with set_test_hub: --skillstation on a local-mode project plans set_principles_hub, publish items and after_publish removals; apply with every publish succeeding writes hub to config and removes .claude/skills principle folders but keeps .agents ones; apply with one failing publish keeps all and reports one skip row; a plan without --skillstation after a successful switch is up_to_date with the hub reporting every principle active.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard install && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "bee onboard --skillstation --apply on a local-mode project writes skillstation.principles hub into .bee/config.json",
        "bee onboard --skillstation --apply removes the .claude/skills principle folders in the same run when every principle publish succeeded",
        "when one principle publish fails onboard keeps every local principle and reports one skipped_skills entry with target skillstation-principles",
        "scripts/install.sh runs its recheck without --skillstation",
        "without --skillstation onboard does not change skillstation.principles"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/apply.rs",
          "substantive": "after_publish removals and set_principles_hub"
        }
      ],
      "key_links": [
        "the removal reuses the remove_skill item and apply_remove_skill, not a new delete path",
        "install.sh plan and apply carry --skillstation, the recheck does not"
      ],
      "prohibitions": [
        "No code comments",
        "Never change the hub without --skillstation",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": [
        "How apply defers after_publish items"
      ]
    }
  },
  {
    "id": "ssp-7",
    "feature": "skillstation-principles",
    "title": "Document the one-step switch to hub principles",
    "lane": "standard",
    "role": "writer",
    "deps": [
      "ssp-6"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md"
    ],
    "decisions": [
      "D8",
      "D9",
      "D10"
    ],
    "files": [
      "README.md",
      "docs/config-reference.md",
      "docs/knowledge/**",
      "docs/history/skillstation-principles/delivery.md"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md",
      "docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md",
      "docs/config-reference.md",
      "README.md"
    ],
    "action": "Rewrite the hub-principles text so it matches D8-D10: one `scripts/install.sh --skillstation` (or `bee onboard --skillstation --apply`) switches a project: it sets skillstation.principles to hub, publishes the 24 principles, and removes the local .claude/skills copies in the same run; a failed publish keeps them. Without the flag install leaves the mode as it is. bee orient names principles in every project from an index built into bee. Remove the D7 text (the guard is gone, superseded by D8). Update README.md § Skillstation, the docs/config-reference.md row, the onboarding concept, and the --skillstation line of installer-entrypoints-and-source-staging.md. Cite `skillstation-principles D<n>`, never raw ids. Append the round to docs/history/skillstation-principles/delivery.md. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "README.md says install --skillstation switches a project to hub principles in one run",
        "no doc still says the local copies are kept for a missing principle index"
      ],
      "artifacts": [
        {
          "path": "docs/history/skillstation-principles/delivery.md",
          "substantive": "the second-round delivery record"
        }
      ],
      "key_links": [
        "the delivery lands in the same merge as the code"
      ],
      "prohibitions": [
        "No restating of code in concepts",
        "No code comments"
      ],
      "open_choices": []
    }
  },
  {
    "id": "ssp-8",
    "feature": "skillstation-principles",
    "title": "Render agent files in hub mode during the --skillstation switch",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-6"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D11",
      "bdd71343-d97d-49ef-9304-3e7f81a3e0c4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/agents.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs"
    ],
    "action": "Bug, reproduced live: on a project installed in local mode, `PATH=<fake skillstation>:$PATH scripts/install.sh --source <bee> -d <repo> -y --skillstation` exits 1 because the recheck finds `sync_agent_file` for all six .claude/agents files: the agent items were planned and applied from the old local config, before `set_principles_hub` ran. Red first: add a test that runs the onboard plan+apply with --skillstation on a local-mode repo (set_test_hub, every publish succeeding), then plans again without --skillstation, and asserts no sync_agent_file item and that .claude/agents/bee-worker.md's tools line ends with mcp__skillstation__skill_get; watch it fail for this reason. Then fix (D11): when the --skillstation switch is active, plan the Claude agent file items as hub-mode renders (give compute_agent_file_plan / render_claude_agent_file an explicit hub override instead of the config read only), mark them `after_publish`, and make apply run `set_principles_hub` before any other after_publish item, and the agent items after it, so apply's re-render reads hub. When a principle publish failed, the agent items do not run. Also add a test: one failing principle publish leaves the agent files local and the config unchanged.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard agents",
    "must_haves": {
      "truths": [
        "after bee onboard --skillstation --apply on a local-mode project a plain onboard plan has no sync_agent_file item",
        "after a successful switch every rendered Claude agent tools line ends with mcp__skillstation__skill_get",
        "when one principle publish fails the agent files and config stay local"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/tests.rs",
          "substantive": "switch then re-plan regression test"
        }
      ],
      "key_links": [
        "apply runs set_principles_hub before the agent file items"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation"
      ],
      "open_choices": [
        "How the hub override reaches the agent render"
      ]
    }
  },
  {
    "id": "ssp-9",
    "feature": "skillstation-principles",
    "title": "Make the principle publish activate on a real hub and keep local copies fresh",
    "lane": "standard",
    "role": "code",
    "deps": [
      "ssp-8"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": [
      "D12",
      "D13",
      "D14",
      "097ba39f-a78d-4583-8410-ba3d5cc50210"
    ],
    "regen_obligation_ack": "wave-barrier",
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "scripts/install.sh"
    ],
    "read_first": [
      "docs/history/skillstation-principles/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "scripts/install.sh"
    ],
    "action": "Three bugs found live on survey-platform (real hub, skillstation 2026 CLI). Red first for 1 and 2. (1) D12: the real hub refuses to activate a skill whose routing has no trigger: `skillstation skill review bee-principle-attack-the-premise --json` says 'Not ready for activation: missing trigger'; an active skill needs a trigger, a --not-for entry (or a rationale) and --min-scope (`skillstation help skill activate`). For bee-principle-* publish items only, apply_publish_skillstation must add to the `skill commit` argv: `--trigger <the principle's SKILL.md frontmatter description, whitespace collapsed>`, `--not-for \"a task bee orient did not route to this principle\"`, `--min-scope multi_step`. bee-deploying's argv is unchanged. Test with the injected hub runner: record the argv of the commit call for a principle and assert the three flags and values; assert bee-deploying's argv has none. (2) D13: during the --skillstation switch the repo-claude target must sync the principles exactly like local mode (sync_skill items for missing or changed ones), and the after_publish remove_skill items must cover every principle the source has, so on success apply syncs then removes, and on a failed publish the local copies are current. Test: a repo with 2 stale local principle folders, --skillstation, one publish failing: after apply the local principle folders match the source and a plain re-plan (local mode) has no sync_skill item; with every publish succeeding: no .claude/skills principle folder remains. (3) D14 in scripts/install.sh: when SKILLSTATION is 1, right before the onboard apply, log one line: `skillstation publishing bee skills to your Skillstation hub - about a minute per skill, the apply prints nothing until it ends` (use the existing log helper). install.sh is a release-manifest root, but do NOT run bee dev regen and do not touch the release manifest; the leader runs it at merge (regen_obligation_ack wave-barrier).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- skillstation onboard install",
    "must_haves": {
      "truths": [
        "the skill commit call for a bee-principle skill carries --trigger, --not-for and --min-scope multi_step",
        "the skill commit call for bee-deploying carries no routing flag",
        "after a --skillstation apply with a failed publish the local .claude/skills principle folders match the source",
        "scripts/install.sh with --skillstation logs the hub publish line before the onboard apply"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/apply.rs",
          "substantive": "principle routing flags on skill commit"
        }
      ],
      "key_links": [
        "the trigger text comes from the principle's own SKILL.md description, not a hand-written list"
      ],
      "prohibitions": [
        "No code comments",
        "Onboard tests never call the real skillstation",
        "No release manifest or regen change"
      ],
      "open_choices": [
        "Where the description is read for the trigger"
      ]
    }
  }
]
```

## Test matrix

- Happy: hub mode, every principle active. Pass when onboard plans `remove_skill` for each `.claude/skills/bee-principle-*` and none for `.agents/skills`.
- Happy: hub mode. Pass when the orient header names `skill_get` and each rendered Claude agent `tools:` line ends with `mcp__skillstation__skill_get`.
- Edge: hub mode, one principle draft. Pass when no principle is removed and one `skillstation-principles` skip row is reported.
- Edge: `--skillstation` in hub mode. Pass when one publish item per principle is planned.
- Error: hub mode, skillstation absent. Pass when no principle is removed and one skip row is reported.
- Regression: key absent. Pass when plan, agent files and orient text are byte-identical to before.

## Open Questions

(none)

## Out of scope

Codex, opencode and plugin runtimes (D1); the bee repo's own dev skill trees and the release (D6).
