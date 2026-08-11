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

## Discovery: throw-blocking write guard proved live (oc-2)

**Date:** 2026-08-11
**Scope:** oc-2 — hand-write `.opencode/plugins/bee-guard.ts` and prove it
blocks/allows in a real `opencode run` session on this checkout.

### Bottom line

The plugin holds zero guard rules. Every tool call it maps
(`write`/`edit`/`bash`) is forwarded verbatim as a `bee hook write-guard`
stdin payload; the verdict is read off the child process's exit code only
(`2` → deny, anything else non-zero or a spawn failure → deny, `0` →
allow). A live `opencode run` session in this checkout proved both the deny
and the allow path in a single transcript, proved the fail-closed path when
no bee binary is reachable, proved bee's rendered skills are discovered,
and proved the AGENTS.md preamble loads into the session with zero tool
calls.

### Exact plugin hook signature (Local — verified against the installed
`@opencode-ai/plugin@1.18.16` type declarations, `npm pack` +
`dist/index.d.ts`, not the web digest)

```ts
"tool.execute.before"?: (
  input: { tool: string; sessionID: string; callID: string },
  output: { args: any },
) => Promise<void>;
```

`input.tool` and `output.args` field shapes, captured live via a logging
probe plugin run against a scratch project (`/tmp/.../scratchpad/oc-probe2/`,
never this repo):

| OpenCode tool | `input.tool` | `output.args` shape |
|---|---|---|
| Write | `"write"` | `{ filePath: <absolute path>, content: <string> }` |
| Edit | `"edit"` | `{ filePath: <absolute path>, oldString: <string>, newString: <string> }` |
| Bash | `"bash"` | `{ command: <string> }` |
| Read (not gated by this cell) | `"read"` | `{ filePath: <absolute path> }` |

Naming note for S3: these are camelCase and singular (`filePath`, not
`file_path`), unlike bee's `PreToolUse` stdin shape (`file_path`,
`old_string`, `new_string`, `command`) — the plugin's `mapToolCall`
translates field names, it does not pass `output.args` through untouched.
`filePath`/`oldString`/`newString` values arrive absolute; bee's
`write-guard` (`canonical_rel_path`) accepts absolute paths directly —
confirmed by direct `bee hook write-guard` stdin probes with an absolute
`file_path` before wiring the plugin (both the deny and allow cases
resolved correctly).

### Binary resolution (no vendored `.bee/bin/bee` in this worktree)

This linked worktree has no `.bee/bin/bee` of its own (per the
orchestrator's environment note). The plugin does not hardcode any
machine-specific path — it mirrors the exact fallback chain already in
`packages/bee/hooks/claude-hooks.json`: `<project>/.bee/bin/bee(.exe)`
first, then (for a linked worktree) `git -C <project> rev-parse
--path-format=absolute --git-common-dir` → strip `/.git` → `<main
worktree>/.bee/bin/bee(.exe)`. Verified directly:

```
$ git -C /home/.../beehive--wt--opencode-support rev-parse --path-format=absolute --git-common-dir
/home/thanhsmind/projects/goglbe/beehive/.git
```

— which resolves to exactly the binary the orchestrator's note names
(`/home/thanhsmind/projects/goglbe/beehive/.bee/bin/bee`). Every live-session
proof below ran with zero extra configuration, confirming this resolution
path works with no env var or hardcoded override.

### (a) Deny — thrown Error, live transcript

Command: `opencode run "Use the write tool to create a new file at
scratch-probe.log (repo root) with the content: probe"` (run from this
checkout; `scratch-probe.log` at repo root trips bee's
`scratch-shape` guard — a rule the model does not already know from
AGENTS.md, so it genuinely attempted the tool call rather than
self-censoring):

```
✗ Write scratch-probe.log failed
Error: bee scratch-shape guard: "scratch-probe.log" looks like a .log
scratch file landing in a tracked directory. Every ephemeral file bee
writes for its own working purposes belongs in .bee/tmp/<feature-or-session>/
(feasibility code in .bee/spikes/<feature>/), never a tracked path
(docs/specs/doctrine-layer.md). FIX: write it to .bee/tmp/ instead (or
.bee/spikes/ for a feasibility proof), and let `bee tmp sweep` clear it
later.
```

The exact stderr text bee's `write-guard` returned (confirmed byte-for-byte
against a direct `bee hook write-guard` stdin probe run beforehand) is the
`Error` message the model saw and reported back — proof the plugin threw
bee's real denial reason, not a generic message. Exit code path: our
plugin's `execFileSync` catch branch fired on `err.status === 2`.

### (b) Allow — same live session, model self-corrected and succeeded

The same session, having been blocked, retried at the guard's own suggested
path and succeeded:

```
← Write .bee/tmp/scratch-probe.log
Wrote file successfully.
```

A second, independent allow proof — a plain markdown file with no guard
rule against it — also succeeded cleanly with no denial in the transcript:

```
$ opencode run "Use the write tool to create a file at
  docs/history/opencode-support/scratch-probe.md with the content: guard
  allow proof"
← Write docs/history/opencode-support/scratch-probe.md
Wrote file successfully.
```

(Both scratch outputs — `.bee/tmp/scratch-probe.log`, gitignored, and
`docs/history/opencode-support/scratch-probe.md` — were deleted after the
proof; neither is part of this commit.)

### Fail-closed path — live proof, missing binary

Copied the exact committed `.opencode/plugins/bee-guard.ts` into a fresh
scratch directory with no `.git` and no `.bee` anywhere in its ancestry
(`/tmp/.../scratchpad/oc-probe-nobinary/`, never this repo) and ran a live
session there:

```
$ opencode run "Use the write tool to create a file named plain.txt with the content: hi"
✗ Write plain.txt failed
Error: bee write-guard could not find the bee binary (.bee/bin/bee) in
this project or its main worktree — denying rather than letting a write
through unchecked. FIX: run `bee onboard --apply` (or vendor .bee/bin/bee)
and retry.
```

Every subsequent write/bash tool call in that same session was denied
identically (the model tried several `bash` commands to investigate, e.g.
`ls -la`, `pwd && ls` — each threw the same fail-closed error). This is the
`resolveBeeBinary` throw path exercised live, not just read from source:
undecidable (`git rev-parse` also failed there — "not a git repository")
never falls through to an allow.

### (c) Skills discovery — confirmed from inside this checkout

`opencode debug skill` run from this checkout lists bee's rendered skills
by name and resolves each to a real on-disk path:

```
bee-hive      -> <checkout>/.agents/skills/bee-hive/SKILL.md
bee-swarming  -> <checkout>/.agents/skills/bee-swarming/SKILL.md
bee-planning  -> <checkout>/.agents/skills/bee-planning/SKILL.md
bee-executing -> ~/.claude/skills/bee-executing/SKILL.md   (global, not project)
```

Full list confirmed 24 `bee-*` skill names discovered (mix of this
project's `.agents/skills/` tree — the onboarding sync path's output — and
the operator's global `~/.claude/skills/`). Important fact for S2: none
came from `.claude-plugin/skills/` or `.codex-plugin/skills/` (the
`render-skill-trees` marketplace pipeline) — those trees are invisible to
OpenCode today, exactly as the plan's Discovery section predicted
(`.opencode/skills/` doesn't exist yet; only `.claude/skills/`/`.agents/skills/`
projections and the two global equivalents are in OpenCode's scan order).
Bee is fully usable in OpenCode today only because the onboarding sync path
already populated `.agents/skills/` in this checkout — a repo relying
solely on the marketplace pipeline would see zero bee skills until S2 adds
the `.opencode/skills/` projection.

`opencode debug config` also independently confirms plugin auto-discovery,
with no config entry needed:

```json
"plugin": ["file:///.../.opencode/plugins/bee-guard.ts"],
"plugin_origins": [{"scope": "local", ...}]
```

### (d) AGENTS.md preamble — confirmed loaded, zero tool calls

Prompt (deliberately worded to forbid any tool call, so a correct answer
can only come from content already in the session's system context):

```
$ opencode run "Without using any tools (no read, no bash, no write),
  answer from what is already in your context only: according to this
  project's AGENTS.md, what single command should be run when starting or
  resuming work in this repo, and what does the doc call the two authors
  of gates, decisions, and privacy approvals?"

`bee orient`.

The two authors are **the user** (owns gates, decision answers, and
privacy approvals) and **the agent** (owns every bee command).
```

The transcript shows zero tool-call markers (no `←`/`→`/`$` lines) before
this answer — the model answered from AGENTS.md's actual text
("`bee orient`"; "Gates, decision answers, and privacy approvals belong to
the user … every bee command belongs to the agent") without reading the
file, confirming OpenCode auto-injects AGENTS.md into the session preamble
by default (no `instructions:` config entry exists in this checkout or in
the global `~/.config/opencode/`).

### Verified layout names feeding S2

- Plugin: `.opencode/plugins/<file>.ts`, auto-discovered, no config entry —
  reconfirmed with the real guard plugin (not just oc-1's marker probe).
- Skills actually reaching OpenCode today: `.agents/skills/<name>/SKILL.md`
  (project, via onboarding sync) and the two global trees — `.opencode/skills/`
  does not exist yet in this checkout (S2's job).
