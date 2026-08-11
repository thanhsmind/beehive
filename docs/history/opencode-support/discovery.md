# Discovery: OpenCode capability baseline (S1 install step)

**Date:** 2026-08-11
**Scope:** oc-1 — install the pinned OpenCode CLI and record the capability
baseline that later slices (S1's guard plugin, S2 render pipelines, S4
worker parity) build on. No source edits in this cell.

## Bottom line

OpenCode installed clean via `npm i -g opencode-ai@latest`, pinned at
`1.18.16` — matches the version the plan already cited as "current stable"
on 2026-08-10. No third-party model provider is configured (0 credentials),
but a live session works anyway: OpenCode ships a zero-config `opencode/*`
free-model provider, and `opencode run "say hi"` returned a real reply. No
blocker prevents a live session on this machine. Plugin, skill, and agent
discovery were each verified on disk against a scratch probe project (never
inside the repo) — all three matched the plan's Discovery section exactly,
and the installed binary's own built-in `customize-opencode` skill supplied
an authoritative path table that confirms it independently of the earlier
web digest.

## Install

- **Local** — command: `npm i -g opencode-ai@latest` (installed cleanly,
  "added 3 packages in 11s", no curl-installer fallback needed).
- **Local** — `opencode --version` → `1.18.16`.
- **Local** — resolves to
  `~/.nvm/versions/node/v24.14.1/bin/opencode`, a symlink into
  `~/.nvm/versions/node/v24.14.1/lib/node_modules/opencode-ai/bin/opencode.exe`
  (a platform-specific compiled binary shipped via `optionalDependencies` —
  `opencode-linux-x64` etc. — not inspectable as plain JS source).
- **Pin for this feature:** `opencode-ai@1.18.16`. Record this pin wherever
  later slices need a version-drift check (S5's doctor warning).

## Provider / auth state

- **Local** — `opencode auth list` → `0 credentials` at
  `~/.local/share/opencode/auth.json` (empty). No `ANTHROPIC_API_KEY`,
  `OPENAI_API_KEY`, or other provider env var is set in this shell; no
  `~/.anthropic`, `~/.config/anthropic` directory exists.
- **Local** — `opencode models` with 0 credentials still lists 7 models, all
  under the `opencode/` provider (`big-pickle`, `deepseek-v4-flash-free`,
  `laguna-s-2.1-free`, `ling-3.0-tiny-free`, `longcat-2.0-free`,
  `mimo-v2.5-free`, `nemotron-3-ultra-free`) — a zero-config, zero-auth
  default provider OpenCode ships out of the box. No `anthropic/*` or
  `openai/*` model is listed, confirming those providers are genuinely
  unconfigured, not just unlisted.
- **Local** — a real live session succeeded with no setup:
  `opencode run "say hi"` (from the scratch probe project) picked
  `build · big-pickle` and returned `Hi!`.
- **Feasibility verdict: a live `opencode run` session IS possible on this
  machine today**, on the free `opencode/*` models. Testing against a paid
  third-party provider (anthropic/openai) would additionally require
  `opencode auth login` with real credentials — not attempted here since no
  credential material was supplied and none is needed to prove the
  capability floor. This is a named gap for whichever later slice needs a
  specific paid-provider model, not a blocker for this cell.

## Verified on-disk layout

Verified by creating files under a scratch probe project outside the repo
(`/tmp/.../scratchpad/oc-probe/`, never under this checkout) and either (a)
running an OpenCode introspection command against it, or (b) reading the
path table embedded in OpenCode's own built-in `customize-opencode` skill
(source: the installed binary itself, via `opencode debug skill`) — not
copied from external docs.

| Artifact | Verified path(s) | How verified |
|---|---|---|
| Plugins | `.opencode/plugins/<file>.ts` (also accepts `.opencode/plugin/`) | Wrote `.opencode/plugins/checkplugin.ts` exporting a plugin that logs a marker on load; `opencode debug config --print-logs` and `opencode debug info` both showed the marker and listed the plugin by its `file://` spec — auto-discovered, no config entry needed. |
| Skills (project) | `.opencode/skills/<name>/SKILL.md` (also accepts singular `.opencode/skill/`) | Wrote `.opencode/skills/probe-skill/SKILL.md` with valid frontmatter (`name`, `description`); `opencode debug skill` listed it by name with `location` pointing at the exact file. |
| Skills (external, auto-loaded) | `~/.claude/skills/<name>/SKILL.md`, `~/.agents/skills/<name>/SKILL.md` | Confirmed as a side effect: this machine's real `~/.claude/skills/*` and `~/.agents/skills/*` entries (e.g. `bee-xia`, `lark-okr`) appeared in the same `opencode debug skill` listing alongside the probe skill — OpenCode reads both trees automatically. Relevant for S2: bee's existing rendered skill trees are already visible to OpenCode, separate from the `.opencode/skills/` projection S2 will add. |
| Agents (project) | `.opencode/agent/<name>.md` (also accepts plural `.opencode/agents/`) | Wrote `.opencode/agent/checkagent.md` with `description` + `mode: subagent` frontmatter; `opencode agent list` listed `checkagent (subagent)`. |
| Config | project `./opencode.json` / `./opencode.jsonc` / `.opencode/opencode.json`; global `~/.config/opencode/opencode.json` | From the built-in skill's path table; `opencode debug paths` independently confirmed `config: ~/.config/opencode`. |
| Global data/cache/state roots | `data ~/.local/share/opencode`, `cache ~/.cache/opencode`, `state ~/.local/state/opencode`, `bin ~/.cache/opencode/bin`, `tmp /tmp/opencode` | `opencode debug paths` (direct CLI output). |

Naming note for S2: OpenCode accepts both the singular and plural form for
`skill(s)`, `agent(s)`, and `command(s)` directories — the plan's render
work should pick one form deliberately (plural matches the plugin/skill
digest already in CONTEXT.md/plan.md) rather than treating the choice as
free.

## Hook/plugin mechanics (spot-checked, not exhaustively re-verified)

The installed binary's built-in `customize-opencode` skill text — read live
via `opencode debug skill`, not fetched from a web page — confirms the
plan's Discovery section point for point: `tool.execute.before` /
`tool.execute.after` are real hook names, a plugin module exports a
`default` async function of type `Plugin = (input, options?) =>
Promise<Hooks>`, and hook bodies "mutate `output` in place; return `void`"
— no documented abort/return-value block path is shown, consistent with the
plan's "a thrown `Error` is the only documented block path" finding. Proving
the throw-blocks-tool-call behavior live (a real deny + a real allow) is S1's
next step, not this cell's scope.

## Blockers

None block this cell's completion. Named gaps for later slices:

- No paid-provider (anthropic/openai/etc.) credentials are configured on
  this machine — a slice that needs to test against a specific paid model
  will need `opencode auth login` run by a human with real credentials
  first. The free `opencode/*` provider is sufficient for S1's write-guard
  proof (any model can trigger a tool call).
- The OpenCode binary is a compiled executable per-platform
  (`opencode.exe` via `optionalDependencies`), not plain JS — mechanics
  beyond what a CLI probe or the built-in skill documents cannot be
  confirmed by reading source directly.
