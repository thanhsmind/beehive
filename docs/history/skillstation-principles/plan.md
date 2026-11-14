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
    "decisions": ["D1", "D4", "D5"],
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
        {"path": "packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs", "substantive": "principles_on_hub helper"}
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
    "deps": ["ssp-1"],
    "affects_skills": [],
    "affects_specs": [],
    "decisions": ["D1", "D2", "D3"],
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
        {"path": "packages/bee-rs/crates/bee/src/onboard/skills.rs", "substantive": "Claude target source list without principles in hub mode"}
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
    "deps": ["ssp-2"],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/onboarding/skillstation-hub-skills-and-aliases.md", "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
    "decisions": ["D1", "D2", "D3", "D4", "D5"],
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
        {"path": "docs/history/skillstation-principles/delivery.md", "substantive": "the feature's delivery record"}
      ],
      "key_links": ["the delivery lands in the same merge as the code"],
      "prohibitions": ["No restating of code in concepts", "No code comments"],
      "open_choices": []
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
