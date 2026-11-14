---
type: bee.area
title: Onboarding — Skillstation hub skills and the /<id> alias
description: "Why a hub skill served over MCP cannot be started with /<id>, what bee skillstation install and bee skillstation alias write to make it startable, the ownership rule that protects a project's own skill folders, and why content approval stays with the hub owner."
timestamp: 2026-11-14
bee:
  id: onboarding-skillstation-hub-skills-and-aliases
  lifecycle: active
  areas: [onboarding]
  required_context: [areas/onboarding/overview.md]
  decisions: ["skillstation-aliases D1-D9 (a bee skillstation verb group checks the hub skills bee relies on and writes a small alias skill so /<id> works; CONTEXT.md)", "skillstation-onboard-aliases D1-D7 (install and onboard write the aliases by default; the hub changes only behind --skillstation; CONTEXT.md)", "skillstation-principles D1-D5, D8-D10, D12-D14 (skillstation.principles hub serves the bee-principle skills to Claude Code; CONTEXT.md)", "skillstation-human-skills D1-D5 (five human-started bee skills join the hub-served set and get /<id> aliases; CONTEXT.md)"]
  sources: ["skillstation-aliases cell ssa-1 (2026-11-14; the verb group)", "skillstation-aliases cell ssa-2 (2026-11-14; this concept and the README section)", "skillstation-onboard-aliases cells sooa-1 and sooa-2 (2026-11-14; onboard and install write the aliases)", "skillstation-principles cells ssp-1 to ssp-3, ssp-6, ssp-7, ssp-9 (2026-11-14; hub-served principles)"]
  authoritative_for: "onboarding: the bee skillstation verb group"
  anchors: [packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs, packages/bee-rs/crates/bee/src/principles.rs, packages/bee-rs/crates/bee/src/onboard/agents.rs, packages/bee-rs/crates/bee/src/onboard/apply.rs, packages/bee-rs/crates/bee/src/onboard/plan.rs, packages/bee-rs/crates/bee/src/onboard/skills.rs, scripts/install.sh]
  verified_at: 2026-11-14
  verified_commit: efc1408fed3c7be37d646ce975dedf8b0e84390e
---

# Onboarding — Skillstation Hub Skills and the /<id> Alias

Read when: a hub skill cannot be started with `/<id>`, a skill marked
`disable-model-invocation` is unreachable, or you are changing what
`bee skillstation install` or `bee skillstation alias` writes or refuses, or
the `skillstation.principles` key (hub-served principles) behaves unexpectedly.

## Why the verb group exists

Skillstation serves hub skills to agents over MCP. A hub skill is not in the
host's own skill list, so the user cannot start it with `/<id>`. A skill that
agents may not trigger on their own is then unreachable (skillstation-aliases
D4). `skillstation skill checkout` was rejected: it copies the whole skill for
editing, so it is not an alias and leaves a second copy of the skill in the
project (skillstation-aliases D5).

## Behaviors & Operations

**Check and add the related skills.** `bee skillstation install` reports, for
each skill bee relies on, whether the hub has it. It previews by default, like
`bee onboard`; only `--apply` adds a missing skill (skillstation-aliases D1,
D3). The list is one constant in the verb module; `bee-deploying` is not on it
because `bee onboard --skillstation` owns that one (skillstation-aliases D2).

**Write an alias.** An alias is one small `.claude/skills/<id>/SKILL.md` that
tells the agent to fetch the real skill from the hub through the MCP
`skill_get` tool. The hub stays the only copy (skillstation-aliases D5).
`install --apply` aliases the listed skills that agents may not trigger
themselves; `alias <id>` aliases any active hub skill (skillstation-aliases
D4).

**Onboard and install write the aliases themselves.** `bee onboard` plans one
`skillstation_alias` item per related skill that is active on the hub, is not
model-triggerable, and whose local alias is missing or differs from a fresh
render; `scripts/install.sh` gets it by running onboard. It runs with no flag
when the Claude runtime is in scope. Planning only a missing or changed alias
keeps the recheck after apply at `up_to_date` (skillstation-onboard-aliases D1).
`--skillstation`, on `install.sh` and on onboard, additionally adds each missing
related skill to the hub as a draft; without it onboard never changes the hub
(skillstation-onboard-aliases D4, D5). Onboard reuses the alias render, the
ownership check and the hub read of the verb module rather than copying them,
and its tests inject the hub runner so a machine with skillstation on PATH
cannot change their result (skillstation-onboard-aliases D6).

## Business Rules

- Content approval stays with the hub owner. `skill add --yes` never approves
  third-party content, so an added skill arrives as a draft and the report names
  `skillstation skill review <id>` as the owner's next step. bee never approves,
  activates or writes a hub file itself (skillstation-aliases D3).
- bee overwrites a `.claude/skills/<id>/` folder only when its SKILL.md carries
  the alias marker for that id. Any other folder is a refusal naming it; `--force`
  overrides. This protects a user's own skill and a bee-rendered skill
  (skillstation-aliases D6).
- An alias is written only for a skill that is active on the hub. A draft is not
  aliased, because the hub would refuse to serve it (skillstation-aliases D4,
  D7).
- A re-run of the switch rewrites the alias after the removal, even when the file
  already equals the alias render, because the `after_publish` removal before it
  deletes the folder; only a plan without `--skillstation` skips a current alias
  (skillstation-human-skills D3-D4).
- `--skillstation` plans a publish for a skill only when the hub has no such
  skill, the hub copy is not active, or its files differ from the engine skill
  rendered for claude. Files compare as (relative path, raw sha256) sets against
  `skill show --json` `manifest.resources`. A hub read that fails keeps the
  publish, and the `after_publish` removals and `set_principles_hub` are planned
  even when no publish is (skillstation-publish-skip D1-D3).
- `skillstation.principles` set to the string `hub` in the merged config
  (config.local.json over config.json) is read by one helper,
  `principles_on_hub`. It makes the principle header line name the MCP tool
  `skill_get`, and it appends `mcp__skillstation__skill_get` to the `tools:` line
  of every rendered Claude agent file, so a dispatched subagent can fetch a
  principle. Any other value or absence leaves both byte-identical. Plan and
  apply render agent files through the same function, so a hub-mode re-plan
  carries no `sync_agent_file` item (skillstation-principles D1, D4, D5).
- In hub mode with the Claude runtime in scope, onboard reads every
  `bee-principle-*` directory of the engine skills source on the hub. Only when
  all are active does the repo-claude target (`.claude/skills`) drop them from
  its source list, so the ordinary deletion domain removes the local copies;
  the codex, opencode, plugin and legacy global targets keep theirs. A principle
  that is missing or not active, or a hub read that skips, keeps every local
  copy and adds one `skipped_skills` row with target `skillstation-principles`.
  Every repo has an index: `principles::principle_lines` reads the repo's own
  `principles::PRINCIPLE_INDEX` file when it exists and otherwise the copy
  compiled into bee from that same file at build time, so dropping the local
  copies never leaves the agent without principle lines (skillstation-principles
  D8, which replaced the D7 skip row for a missing index).
  With `--skillstation` and the Claude runtime in scope, onboard treats the
  project as hub mode even while the config still says local, and the one run
  switches it: each principle is published from the engine source rendered for
  claude (never from `.claude/skills`, which the run deletes), then the
  `.claude/skills` principle folders are removed and `skillstation.principles`
  is written as `hub` into the tracked `.bee/config.json`. The removals and the
  config write carry `after_publish` and sit after every publish item in the
  plan; apply runs none of them once any principle publish failed and adds one
  `skipped_skills` row (target `skillstation-principles`) naming the failed id.
  Holding the config write back too is a choice: hub mode with unpublished
  principles would point every session at a hub that cannot serve them.
  `scripts/install.sh` rechecks without `--skillstation`, so the recheck reads
  local state and plans no publish (skillstation-principles D9, D10).

- The hub-served set is the `bee-principle-*` skills plus five human-started
  skills, `bee-how`, `bee-teach`, `bee-technical-writing`, `bee-why` and
  `bee-writing-skills`. One constant and one predicate in `onboard/skills.rs`
  name it; every rule above (engine-rendered publish with routing flags,
  publish skip, drop only when every member is active, the `after_publish` gate)
  applies to all members, under the same hub mode and `--skillstation` switch
  (skillstation-human-skills D1, D2).
- A removed local copy of one of the five is replaced by a `/<id>` alias at
  `.claude/skills/<id>/SKILL.md`, rendered with the engine skill's own
  description and without `disable-model-invocation`, so the agent still lists
  it and a human can type `/<id>`. Principles get no alias. In the switch the
  alias items are `after_publish` and follow the removals, so a failed publish
  writes none (skillstation-human-skills D3).
- The repo-claude deletion domain leaves a hub-served `bee-*` directory alone
  when its SKILL.md carries the alias marker for its own name and hub mode drops
  it from the sources. Without that exception each onboard deleted the alias and
  the next wrote it again, and the installer recheck never reached `up_to_date`
  (skillstation-human-skills D4). Codex, opencode, the plugin trees and the
  legacy global targets keep the five (skillstation-human-skills D5).

## Edge Cases Settled

- During onboard, a `.claude/skills/<id>/` without the alias marker gets no item
  and one `skipped_skills` entry naming the folder; onboard never forces
  (skillstation-onboard-aliases D2). A missing binary, non-zero exit or unreadable
  JSON gives one `skipped_skills` entry with target `skillstation-alias` for the
  whole check, not one per skill, and the onboard result is otherwise normal
  (skillstation-onboard-aliases D3).

- No `skillstation` binary, a non-zero exit or unreadable JSON: `install` prints
  one skip row per skill with the reason and exits 0, the same contract as the
  onboard publish skip. `alias <id>` exits non-zero instead, because an explicit
  request that cannot work must say so (skillstation-aliases D7).
- Claude Code only. Codex, opencode and the plugin trees keep their local
  copies (skillstation-aliases D8, skillstation-principles D1).
- The alias body relies on the host having the skillstation MCP server; when the
  tool is missing the alias tells the user to run `skillstation connect -g`.
- The `--skillstation` switch renders the Claude agent files in hub mode at plan
  time, because the plan reads config before `set_principles_hub` has written it.
  Those `sync_agent_file` items are `after_publish` and sort behind
  `set_principles_hub`, so apply's own re-render reads hub. Rejected: planning
  from the old config; the installer recheck then found `sync_agent_file` on all
  six files and exited 1. A failed principle publish skips them with the config.

## Pointers (implementation)

- `packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs` — the verb group;
  find the related-skill constant, the alias render and the ownership check by
  searching for `RELATED_SKILLS`, `render_alias` and `carries_alias_marker`.
- `packages/bee-rs/crates/bee/src/onboard/apply.rs` — `run_skillstation`, the
  CLI runner shared with the onboard `--skillstation` publish, and the
  `skillstation_alias` arm.
- `packages/bee-rs/crates/bee/src/onboard/plan.rs` — find the planning of the
  alias items by searching for `skillstation_items`.
- `packages/bee-rs/crates/bee/src/onboard/skills.rs` — `without_principles` on
  `SkillSources` is the one filter; find where the repo-claude target picks it by
  searching for `claude_sources` in `compute_skill_sync`.
- `scripts/install.sh` — the `--skillstation` flag is forwarded to onboard
  through `ONBOARD_FLAGS`.
- The hub CLI shape is not frozen by bee; list the current flags with
  `skillstation help skill add` before changing a call.

## Activating a principle on a real hub

Read when: a principle publish commits but the hub says "Not ready for activation: missing trigger", or a failed switch leaves stale local principle copies.

- The real hub refuses to activate a skill with no trigger, no `--not-for` entry and no `--min-scope`. Only `bee-principle-*` publishes add those flags to `skill commit`; the trigger is the principle's own SKILL.md description, read from the staged folder, so no second list exists. `bee-deploying` keeps its plain commit. A principle with no description commits without `--trigger` and the hub refuses it loudly.
- During the `--skillstation` switch the repo-claude target syncs principles exactly like local mode, and one `after_publish` removal is planned for every principle the source has. A failed publish therefore leaves current local copies; a full success removes them. Rejected: skipping the principle sync during the switch, which left stale copies after a failed publish.
- `install.sh` logs one line before the apply under `--skillstation`, because the apply prints nothing for about a minute per published skill.
