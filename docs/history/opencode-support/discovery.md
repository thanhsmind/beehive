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
(`write`/`edit`/`bash`, and — as of oc-3 — `apply_patch`) is translated
(field names renamed, never passed through untouched — see the field-shape
table below) into a `bee hook write-guard` stdin payload; the verdict is
read off the child process's exit code only (`2` → deny, anything else
non-zero or a spawn failure → deny, `0` → allow). A live `opencode run`
session in this checkout proved both the deny and the allow path in a
single transcript, proved the fail-closed path when no bee binary is
reachable, proved bee's rendered skills are discovered, and proved the
AGENTS.md preamble loads into the session with zero tool calls.

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

## Discovery: apply_patch bypass closed (oc-3)

**Date:** 2026-08-11
**Scope:** oc-3 — a slice judge found the installed OpenCode binary
registers a fourth write-capable tool, `apply_patch`, that oc-2's
`mapToolCall` switch did not forward — its `default: return null` arm made
that tool a TypeScript-side allow, bypassing bee's write-guard entirely.
This closes that gap and records the write-capable tool registry the
coverage claim now rests on.

### Write-capable tool registry (installed `opencode-ai@1.18.16` binary)

Two independent sources, cross-checked, since the binary is compiled (no
plain JS source to read directly — oc-1's constraint still holds):

1. **Live probe** — a scratch project (`/tmp/.../scratchpad/oc-probe-tools/`,
   never this repo) with a logging plugin hooking `tool.definition` (fires
   once per tool the session actually registers, with its full parameter
   schema) and `tool.execute.before` (fires on every real call), run against
   `opencode run "say hi"`. Registered/exposed tool set observed:
   `invalid, question, bash, read, glob, grep, edit, write, task, webfetch,
   todowrite, websearch, skill` — 13 tools. Of these, the write-capable ones
   are exactly the three oc-2 already mapped: `write`, `edit`, `bash`.
   `apply_patch` did NOT appear in this list.
2. **Static binary read** — `strings` against the resolved compiled
   executable (`~/.nvm/.../lib/node_modules/opencode-ai/bin/opencode.exe`,
   via its `~/.nvm/.../bin/opencode` symlink), grepped for `apply_patch`.
   This surfaced the tool's real registration, independent of whether the
   live session exposes it:
   - `var Nf="apply_patch"` with its input schema
     `Z.Struct({patchText:Z.String...})` — the tool ID is exactly
     `"apply_patch"` and its ONE argument field is `patchText`, carrying the
     full `*** Begin Patch ...` envelope as a single string (same shape
     `bee`'s own `apply_patch_text` detector already expects, see
     `detectors.rs:18-27`).
   - `H.register({[Nf]:...}, "edit")` — the tool is registered into
     OpenCode's tool catalog under the `"edit"` permission group.
   - `let K=["edit","write","apply_patch"]` — OpenCode's own permission
     module groups `apply_patch` with `edit`/`write` for its internal
     allow/deny/visibility rules. This is the write-group the slice judge's
     finding cited; it confirms `apply_patch` is treated as write-capable by
     OpenCode itself, not just by inference.

**Reconciling the two sources:** `apply_patch` is a real tool OpenCode
1.18.16 registers in its binary-level catalog, grouped with `edit`/`write`
for permission purposes, but it was NOT among the tools offered to the
model in the default `build` agent / free `opencode/*`-provider session
used for every proof in this feature. A direct live check confirms this
from the model's own side:

```
$ opencode run "Use the apply_patch tool (not write or edit) to create a
  new file named patchprobe.txt with the content: hello via apply_patch. ..."

I don't have an `apply_patch` tool available in this session — my
available tools are bash, edit, glob, grep, read, skill, task, todowrite,
webfetch, websearch, and write.
```

No config key or agent-mode toggle for this was found by string search
(`experimental_apply_patch`, `applyPatchEnabled`, etc. — no hits); the
most likely explanation, unconfirmed, is that OpenCode reserves
`apply_patch` for provider/model combinations that declare native
apply-patch tool-call support (the binary separately defines
`openai.apply_patch` schemas for the OpenAI Responses API surface) rather
than exposing it by default to every model. This is a named gap, not a
blocker: the coverage rule this cell enforces is "every write-capable tool
the binary registers must reach bee's write-guard, not just the ones a
given session happens to expose" — `mapToolCall` now covers `apply_patch`
regardless of whether this installation's default agent currently offers
it to a model.

### apply_patch → bee write-guard mapping (added to `bee-guard.ts`)

```
case "apply_patch":
  return { tool_name: "apply_patch", tool_input: { patch: args?.patchText } }
```

- `tool_name: "apply_patch"` matches bee's `is_apply` match arm exactly
  (`"apply_patch" | "ApplyPatch"`, `write_guard/main.rs:65`).
- `tool_input.patch` carries the full patch text — one of the three keys
  bee's `apply_patch_text` detector checks (`["input","patch","command"]`,
  `detectors.rs:18-27`), each read for a `"*** Begin Patch"` prefix.

### Live apply_patch deny probe — attempted, could not run

Per the reconciliation above, this installation's default session never
offers the model an `apply_patch` tool call to make, so there is no live
transcript of bee denying a real `apply_patch` call in this feature (unlike
oc-2's write/edit/bash proofs). What was attempted and confirmed instead:

- The live probe transcript above, proving the tool genuinely is not
  reachable through the model in this default configuration (not merely
  untried).
- A direct `bee hook write-guard` stdin probe with a synthetic
  `{"tool_name":"apply_patch","tool_input":{"patch":"*** Begin Patch\n***
  Add File: scratch-probe.log\n+hi\n*** End Patch"}}` payload was
  considered as a substitute proof of the Rust-side deny path, but the
  live-transcript requirement in this cell's action is specifically about
  proving it "in an opencode run session" — a stdin probe bypasses the
  plugin entirely and proves nothing about `mapToolCall`'s new branch, so
  it was not substituted for the missing live transcript.
- Named gap for a later slice: if a paid provider/model with native
  apply-patch support is configured (per oc-1's Blockers section — no
  paid-provider credentials exist on this machine), or if a future
  OpenCode version exposes `apply_patch` to the default agent, re-run this
  probe to get the live deny transcript this cell could not obtain.

## Discovery: opencode plumbed through both render pipelines (oc-4)

**Date:** 2026-08-11
**Scope:** oc-4 — S2 (plan.md E2). Marker grammar gains the `opencode`
value in both render sites (`onboard/render.rs`,
`devtools/skill_trees.rs`); `devtools/skill_trees.rs`'s string-keyed
target-dir pick becomes one exhaustive runtime→target mapping that refuses
an unknown runtime instead of silently rendering into `.codex-plugin/`; the
ONBOARDING SYNC PATH's already-runtime-agnostic writer
(`apply_sync_skill`/`render_skill_bytes`) is driven once against the real
checkout to produce `.opencode/skills/`.

### Two render pipelines, one intentional asymmetry

- `devtools/skill_trees.rs` (`bee dev render-skill-trees`): `RENDER_RUNTIMES`
  stays `["claude", "codex"]` — **no** opencode root. No marketplace
  equivalent exists for it (named exclusion, not an omission). A NEW,
  separate `MARKER_RUNTIMES = ["claude", "codex", "opencode"]` constant
  governs what the `<!-- bee:only RUNTIME -->` grammar accepts as a valid
  label — a strict superset of `RENDER_RUNTIMES` — so skill-source authors
  can scope content to opencode even though this pipeline never emits an
  opencode tree; an `opencode`-only block is simply stripped from both
  trees this pipeline actually writes, proven by
  `opencode_only_content_never_lands_in_either_committed_tree`.
- `onboard/skills.rs` + `onboard/render.rs` (the onboarding sync path):
  `onboard/templates.rs`'s `RENDER_RUNTIMES` (consumed only by
  `render.rs::classify_marker_line` — confirmed by a repo-wide grep before
  editing it) gained `"opencode"` outright, because this path DOES render a
  real opencode target. `apply_sync_skill`/`compute_skill_items` already
  took `runtime: &str` with no closed validation (proven pre-existing by the
  `codex_target_renders_the_codex_arm` test) — no core logic change was
  needed there, only the marker-label acceptance list.
- `onboard/templates.rs::REPO_SKILL_TARGETS` (the table `skill_sync_targets`
  in `onboard/source.rs` builds from, and what `bee onboard --apply` wires
  through `onboard/apply.rs`) deliberately did **not** gain an `opencode`
  entry in this cell — that CLI wiring (`--opencode` flag, `bee onboard`
  target, doctor version-pin drift check) is plan.md E5 (S5), a later slice.
  This cell proves the render primitives and produces the tree by calling
  them directly (see below), not by extending the onboarding CLI surface.

### The wrong-target probe (plan.md E2 risk map: HIGH)

Two tests in `devtools/skill_trees.rs` close the risk the plan named
("string fan-out sends opencode output to a codex tree"):

- `target_root_refuses_an_unknown_runtime_instead_of_defaulting_to_a_tree` —
  the exhaustive `match` (was `if runtime == "claude" {..} else {..}`) panics
  on `target_root(root, "opencode")` instead of silently returning
  `.codex-plugin/skills`.
- `opencode_only_content_never_lands_in_either_committed_tree` — a skill
  source file with an `<!-- bee:only opencode -->` block renders identically
  (block stripped) for both `"claude"` and `"codex"`, and — as a
  complementary round-trip proof — renders the block IN when the runtime
  actually is `"opencode"` (the shape the onboarding sync path uses).

### `.opencode/skills/` — the rendered-root inventory (this checkout)

Produced by running `apply_sync_skill(skills/, .opencode/skills/, <name>,
"opencode")` once per canonical skill directory, then writing the
`bee-render/2` sidecar via `build_render_sidecar`/`source_skill_digest_entries`
— the same primitives `onboard/apply.rs` uses for the claude/codex targets,
driven directly (`onboard::skills::tests::regen_opencode_skills_tree`,
`#[ignore]`d — the interim regen entry point until E5 wires this through
`bee onboard --apply`). A permanent pinned test,
`opencode_projection_matches_the_committed_tree`, re-renders `skills/` for
`"opencode"` on every `cargo test` run and byte-compares against this
committed tree, the same drift check
`devtools::skill_trees::render_matches_the_committed_trees` runs for the
other two runtimes' committed trees.

| Skill | Rendered at |
|---|---|
| bee-capturing | `.opencode/skills/bee-capturing/` |
| bee-grooming | `.opencode/skills/bee-grooming/` |
| bee-herding | `.opencode/skills/bee-herding/` |
| bee-hive | `.opencode/skills/bee-hive/` |
| bee-planning | `.opencode/skills/bee-planning/` |
| bee-researching | `.opencode/skills/bee-researching/` |
| bee-reviewing | `.opencode/skills/bee-reviewing/` |
| bee-shaping | `.opencode/skills/bee-shaping/` |
| bee-swarming | `.opencode/skills/bee-swarming/` |

9 skills, matching the canonical `skills/` source root's own count exactly
(unlike `.agents/skills/`, which independently carries 16 top-level entries
— legacy/extra content out of this cell's scope). Sidecar at
`.opencode/skills/.bee-render.json`: `{"schema": "bee-render/2",
"target_runtime": "opencode", "skills": [...9 entries, name + sha256...]}`
— `bee-render/2` because that is the schema version already in force
everywhere else in this codebase (`onboard/templates.rs::RENDER_SCHEMA`,
`devtools/skill_trees.rs::RENDER_SCHEMA`); the plan text's `bee-render/1`
citation is stale against the code, not a locked decision to reproduce.

## Discovery: full guard-hook mapping table implemented (oc-6)

**Date:** 2026-08-11
**Scope:** oc-6 — S3 (plan.md E3). Extend `.opencode/plugins/bee-guard.ts`
from write-guard-only (oc-2/oc-3) to the full hook-mapping table plan.md's
Approach section names: model-guard on the `task` tool, the read-shaped
tools (`read`/`grep`/`glob`/`question`) through write-guard, and four
advisory surfaces (session-init/prompt-context, state-sync, session-close,
tools-logger) — each wired per the exact contract listed in oc-6's action
text, never guessed.

### Bottom line

Every row is implemented with exactly two failure policies, never a third:
BLOCKING (`tool.execute.before` only — write-guard, model-guard) throws on
deny and fails CLOSED on any OpenCode-side spawn failure; ADVISORY (every
other surface) NEVER throws — a missing binary, a crash, or a non-zero exit
is swallowed and logged to `console.error`, matching bee's own
`emit_undecidable` fail-open-and-say-so posture one level up (this plugin's
advisory wrapper fails open even for failures that never reach bee's native
code at all, e.g. a missing binary). Live `opencode run` transcripts in this
checkout proved: the new read-size deny routes through write-guard exactly
like Claude's belt, and the new model-guard deny routes through the `task`
tool exactly like Claude's `Task` tool, with zero crash-log lines in either
worktree's `.bee/logs/hooks.jsonl` across the whole session.

(Amended by oc-8, below: BLOCKING's exit-0 path is not always a plain allow
— it can also carry a repair (`updatedInput`) or a still-fail-closed `ask`
verdict; see "Discovery: exit-0 repair/ask verdicts honored on the BLOCKING
path (oc-8)".)

### Implemented hook → OpenCode surface table

| bee hook | OpenCode surface | Failure policy | Status |
|---|---|---|---|
| write-guard | `tool.execute.before` on `write`/`edit`/`bash`/`apply_patch` (oc-2/oc-3) **+ new:** `read`/`grep`/`glob`/`question` **+ new:** `lsp`/`list` (oc-10) | BLOCKING — throw on deny **or** `permissionDecision: "ask"`; apply an exit-0 `updatedInput` repair onto `output.args`; throw on unparseable exit-0 verdict JSON; fail closed on any OpenCode-side spawn failure (oc-8) | Live-proved: read-size deny (below), AskUserQuestion repair-applied/ask-throws/unparseable-throws (oc-8, below); write/edit/bash/apply_patch already proved in oc-2/oc-3 |
| model-guard | `tool.execute.before` on `task` | BLOCKING — same policy as write-guard, including the exit-0 `updatedInput`/`ask`/unparseable handling (oc-8) | Live-proved: Task deny (below); dispatch-label/`subagent_type` repair path shares runBlockingHook's proof above |
| session-init | `chat.message`, ONCE per `sessionID` (in-memory `Set`, process-lifetime only) | ADVISORY — swallow + log, never throws | Live-proved: no crash across the whole session; digest text observed reaching the model (AGENTS.md-preamble-style content, distinct from AGENTS.md's own auto-load) |
| prompt-context | `chat.message`, every message | ADVISORY — swallow + log, never throws | Live-proved: same run, no crash |
| state-sync | `event` on `file.edited` and `session.idle` | ADVISORY — swallow + log, never throws | Wired; no crash observed. Live per-call proof not separately captured (advisory, always-silent-stdout by design — see tools_logger.rs-style comment in state_sync.rs) |
| session-close | `event` on `session.idle` and `session.deleted` | ADVISORY — swallow + log, never throws | Wired; no crash observed |
| tools-logger | `tool.execute.after`, every tool call | ADVISORY — swallow + log, never throws | Live-proved: new `"tool_name":"Read"` / `"tool_name":"Task"` entries in this worktree's `.bee/logs/tools.jsonl`, `agent_id`/`agent_type`/`duration_ms` all `null`/absent (see gap below) |
| codex-subagent-audit | n/a | NAMED EXCLUSION — codex-specific; no OpenCode session ever carries Codex SubagentStart/SubagentStop evidence | Not wired (unchanged from oc-2/oc-3) |
| chain-nudge | plan.md names `event: session.idle` | **NOT WIRED in this cell** — see "Deferred: chain-nudge" below | Deferred |

### Field-shape confirmations (live probes, scratch project outside the repo)

A probe plugin (`/tmp/.../scratchpad/oc-probe3/`, never this repo) hooking
`tool.definition` + `tool.execute.before` + `chat.message` + `event`,
run against `opencode run "Use the glob tool ... then use the grep tool
..."`, confirmed the field shapes oc-2's table did not yet cover:

| OpenCode tool | `input.tool` | `output.args` shape (live, not just schema) |
|---|---|---|
| Glob | `"glob"` | `{ pattern: <string> }` (`path` omitted when unset — confirmed live, not just from the schema) |
| Grep | `"grep"` | `{ pattern: <string> }` (`path`/`include` likewise omitted when unset) |
| Question | `"question"` | schema only (not exercised live — see gap below): `{ questions: [{ question, header, options: [{ label, description }], multiple? }] }` |
| Task | `"task"` | schema only for the exact field list (live-proved for `description`/`prompt`/`subagent_type` — see the Task deny transcript below): `{ description, prompt, subagent_type, task_id?, command?, background? }` |

Two of these need ZERO field-name translation — a genuine finding, not an
oversight: OpenCode's `question` args already match
`write_guard/detectors.rs::check_ask_user_question`'s expected keys exactly
(`questions[].header`, `.options[].label`, `.options[].description`), and
OpenCode's `task` args already match the three fields
`model_guard.rs::evaluate_claude_dispatch` reads
(`description`/`prompt`/`subagent_type`, model_guard.rs:516-524) exactly.
`mapToolCall` passes both through untouched.

### (a) Read-size deny — thrown Error, live transcript (new: `read` routed through write-guard)

```
$ opencode run "Use the read tool to read
  packages/bee-rs/crates/bee/src/hooks/model_guard.rs with NO offset and NO
  limit arguments (do not set them). Report exactly what happened,
  including any error verbatim."

✗ Read packages/bee-rs/crates/bee/src/hooks/model_guard.rs failed
Error: bee read-size guard: "packages/bee-rs/crates/bee/src/hooks/model_guard.rs"
is 1565 lines (threshold: 800) and this Read has neither `offset` nor
`limit` — reading it unbounded would load the whole file into context. FIX:
pass `limit` (and optionally `offset`) to read a slice, or dispatch a
`bee-extract` worker to read the whole file.
```

Byte-for-byte bee's own `check_read_size_denial` reason
(write_guard/main.rs:121-131), proving the plugin's new `Read` branch
reaches the exact same native check the Claude belt's `PreToolUse` matcher
(`Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion`) does. A
bounded read (`opencode run "Use the read tool ... with limit=1 ..."` from
the earlier AGENTS.md-preamble proof, see below) succeeded cleanly with no
denial — the presence/absence of `offset`/`limit` on the OpenCode side
correctly reaches bee's own presence check, not just a truthiness check.

### (b) Task dispatch deny — thrown Error, live transcript (new: `task` routed through model-guard)

```
$ opencode run "Use the task tool with subagent_type set to
  general-purpose, description 'probe dispatch', and prompt
  '[bee-tier: generation] probe prompt for guard test'. Report exactly what
  happened, including any error verbatim."

✗ probe dispatch failed
Error: bee-model-guard: [bee-tier: generation] dispatched with
subagent_type "general-purpose", and the generation tier carries TWO
rendered agents — the guard will not guess which.
FIX: name the one you mean. subagent_type "bee-build" executes a cell
(reserves, writes, commits, caps); subagent_type "bee-gather" reads and
reports (never writes).
```

This is the exact `generic-type-denied` deny model-guard's Claude belt
gives an ambiguous `[bee-tier: generation]` + `subagent_type:
"general-purpose"` dispatch (model_guard.rs:526-549) — proving the
`task` → `Task` mapping reaches the real native dispatch-tier logic, not a
stub. The corresponding audit line landed in this worktree's own
`.bee/logs/hooks.jsonl` (this worktree carries its own `.bee/onboarding.json`,
so the native `bee_installed` gate resolves locally rather than to the main
worktree — see the store-root note below):

```
{"ts":"2026-08-11T15:10:09.549Z","hook":"model-guard","event":"deny","tool_name":"Task","tool_input_keys":["description","prompt","subagent_type"]}
```

### (c) tools-logger — live entries, new tool names, gap made visible

This worktree's `.bee/logs/tools.jsonl` (not the main worktree's — see
below) gained, among Claude-belt entries from this very execution session:

```
{"ts":"2026-08-11T15:08:51.697Z","tool_name":"Read","agent_id":null,"agent_type":null}
```

`agent_id`/`agent_type`/`duration_ms` all absent or `null` — OpenCode's
`tool.execute.after` payload (`input: {tool, sessionID, callID, args}`,
`output: {title, output, metadata}`) carries none of them, unlike Claude's
richer PostToolUse payload. `tools_logger.rs` already treats all four as
optional (tools_logger.rs:20-35), so this degrades gracefully rather than
crashing — named gap, not a silent loss of a required field.

### Store-root note (a fact this cell's live testing surfaced, not a bug)

This feature's own linked worktree (`beehive--wt--opencode-support`)
carries its OWN `.bee/onboarding.json` (distinct from oc-2's "no vendored
`.bee/bin/bee`" finding, which is about the BINARY specifically, not the
onboarding marker or the store). Every hook whose native activation gate
checks `crate::hooks::adapter::bee_installed(&ctx.root)` — i.e. the WORK
root, not the store root — activates and writes LOCALLY to this worktree's
own `.bee/logs/*.jsonl` and `.bee/state.json`, not the main worktree's.
write-guard and model-guard's activation gates check the STORE root instead
(`write_guard/main.rs:83`, resolved via grants), which is why oc-2's proofs
and this cell's proofs both correctly exercised the MAIN worktree's guard
rules even while their audit/log side effects (model-guard's
`dispatch.jsonl`/`hooks.jsonl` deny line, tools-logger's `tools.jsonl`) land
in the LOCAL worktree's `.bee/` tree. Not a defect in this cell's plugin —
a pre-existing asymmetry in which root each native hook's OWN activation
gate reads, equally true of the Claude belt run from inside this same
worktree.

### Named gaps (unservable or deliberately deferred — not silent omissions)

- **model-guard's model-param check is hardcoded to `models.claude`.**
  `evaluate_claude_dispatch`'s strict-equality model-param check calls
  `resolve_tier(models, t, "claude")` unconditionally (model_guard.rs:570,
  625, 650, 665) — there is no `models.opencode` key in `normalize_models`
  at all (model_guard.rs:314-316 only loops `["claude", "codex"]"`). In
  practice this does not misfire today: OpenCode's `task` tool has no
  `model` argument at all (confirmed by the live probe's `tool.definition`
  schema — see the field-shape table above), so `model_param` is always
  `None` and that branch never fires; only the runtime-agnostic
  tier-marker + `subagent_type` checks run (proved live in (b) above). A
  real `models.opencode` config key is E4/S4's job (plan.md), not this
  cell's.
- **`question`/`apply_patch` still have no live deny transcript in this
  feature.** Same class of gap oc-3 already recorded for `apply_patch`: this
  installation's default `build` agent + free `opencode/*` provider does not
  offer the model a `question` tool call either — a live probe confirmed it
  (`"The question tool is not available in this session... available tools
  are: bash, edit, glob, grep, read, skill, task, todowrite, webfetch,
  websearch, write."`). The mapping is implemented and its field shapes are
  confirmed via the schema (see the field-shape table above), but the
  live-transcript proof this cell's action wants is not obtainable without
  a different agent/model configuration than is available on this machine —
  named gap for a later slice, exactly as oc-3 named for `apply_patch`.
- **A future repair on a field-translated tool would need a reverse
  mapping.** oc-8 closed the "on-allow stdout repairs are dropped" gap this
  entry used to name (see "Discovery: exit-0 repair/ask verdicts honored on
  the BLOCKING path (oc-8)" below) — `runBlockingHook` now applies
  `hookSpecificOutput.updatedInput` onto `output.args` directly. That works
  with zero reverse translation for both repair paths that exist today
  (write-guard's `AskUserQuestion` header fix, model-guard's dispatch-label/
  `subagent_type` fix) because both target `question`/`task`, the two tools
  `mapToolCall` passes through UNCHANGED (`tool_input: args ?? {}`). A
  repair on a field-TRANSLATED tool (write/edit/bash/read/grep/glob/
  apply_patch — none of which bee emits a repair for today) would still
  need a reverse mapping (the mirror image of `mapToolCall`) before direct
  `Object.assign` would apply it correctly — unbuilt because unneeded so
  far, named rather than silently assumed away.
- **`file.edited` carries no `sessionID`.** `EventFileEdited.properties` is
  `{ file: string }` only (verified against the installed
  `@opencode-ai/sdk@1.18.16` type declarations) — state-sync still runs on
  this event, but its session-heartbeat-renewal half is natively skipped
  (`get_session_id` returns `None`, state_sync.rs:131-136); only the
  `.bee/state.json` cell-count rebuild half fires. `session.idle` DOES
  carry a `sessionID` (`EventSessionIdle.properties.sessionID`) and gets
  the full behavior.
- **session-close's Stop-continuation block has no OpenCode enforcement
  equivalent.** session-close's native Stop path can emit
  `{"decision":"block","reason":...}` (the "GitHub-#18 bypass net" —
  session_close/mod.rs's header comment) to force a Claude session to keep
  going. OpenCode's `event` hook on `session.idle` has no analogous power to
  refuse a session going idle; this plugin only logs whatever session-close
  returns via `runAdvisoryHook`'s swallow-and-log path, it never enforces
  it. Advisory in the literal sense here, not just in name.
- **Deferred: chain-nudge.** plan.md's Approach section names
  `event: session.idle` as chain-nudge's OpenCode surface, but oc-6's action
  text does not enumerate it among the advisory hooks to wire (it lists
  session-init/prompt-context, state-sync, session-close, tools-logger, and
  codex-subagent-audit as a named exclusion — eight of the nine
  `HOOK_NAMES`, deliberately). chain-nudge's own payload contract wants
  subagent-completion IDENTITY (`agent_name`/`subagent_type`/`session_id`
  scoped to the ONE subagent that just stopped — chain_nudge.rs:179-195),
  which Claude's `SubagentStop` event gives natively (fired scoped to the
  exact subagent) but a bare OpenCode `session.idle` does not (it fires on
  ANY session going idle, top-level or dispatched, with no signal
  distinguishing the two without additional plumbing this cell was not
  asked to build). Left unwired rather than wired with guessed identity —
  E4/S4 (worker dispatch parity, plan.md) is the natural home for getting
  this right, since it already owns the subagent-dispatch mechanics
  chain-nudge depends on.
- **Undocumented OpenCode part-id schema, discovered live.** Pushing a
  synthetic `chat.message` text part with a bare `randomUUID()` id crashed
  the whole call server-side: `SchemaError: Expected a string starting with
  "prt", got "<uuid>"` (inside `Session.updatePart`), which OpenCode surfaced
  to the CLI as an opaque `UnknownError` (`ref: err_...`) with no other
  detail — not documented anywhere in `@opencode-ai/plugin@1.18.16`'s type
  declarations. Fixed by prefixing generated part ids with `prt_`. Recorded
  here because nothing in the installed package's `.d.ts` files or the
  built-in `customize-opencode` skill names this constraint — a future
  OpenCode version could change or drop it silently.

## Discovery: three-belt parity test authored from zero (oc-7)

**Date:** 2026-08-11
**Scope:** oc-7 — plan.md E3's parity proof. `docs/06-runtime-integration.md:143`
names a two-belt parity test that does not exist in this tree — it died with
the Node runtime at the R6 cutover (commit 5c62cad0). There was nothing to
port for the OpenCode belt specifically; the new suite,
`packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs`, is authored
from zero, inside the cargo suite `commands.test` already runs.

### Runner design

**Amended by oc-9 (below):** this section originally described two parts,
four `#[test]` functions. oc-9 closed four false-green paths the S3 judge
found in that original design (F1, F3, F4, F5) and added a third part, a
fifth `#[test]` function, and a shared discovery.md-scoping helper reused by
two of the five — see "Discovery: parity suite's false-green paths closed
(oc-9)" below for what changed and why. What follows is updated to describe
the CURRENT shape; the "Skipped-environment behavior" and "Verified on this
machine" subsections below it are oc-7's own point-in-time run, left as a
historical record of what was true before oc-9's fixes.

Three parts, five `#[test]` functions:

1. **Fixture tests over the real plugin.** A tiny, never-checked-in
   `node` harness (a Rust string constant, written fresh into a tempdir per
   test run — the same pattern `hook_contracts.rs`'s `fixture()` already
   uses for its `.bee/onboarding.json` marker) dynamically imports the real
   `.opencode/plugins/bee-guard.ts` by path, calls its default export to get
   the `Hooks` object, and invokes exactly one named surface
   (`tool.execute.before` / `chat.message` / `event` / `tool.execute.after`)
   against a STUB `.bee/bin/bee` binary that can deny (exit 2), allow (exit
   0), crash (any other nonzero exit), or be absent. `node`'s native
   TypeScript type-stripping (Node 22.6+, unflagged by default from 23.6+;
   confirmed live on the installed `v24.14.1`) runs the `.ts` file directly —
   no build step, no `ts-node`, matching how OpenCode itself loads the file.
   `every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary`
   drives all 9 rows `mapToolCall` routes (write/edit/bash/apply_patch/
   read/grep/glob/question/task — every one maps to a BLOCKING hook, since
   `mapToolCall` only ever returns `write-guard` or `model-guard`) through
   all four stub behaviors, asserting throw-on-deny (with the stub's own
   reason text reaching the thrown `Error`), allow passes `output.args`
   through unchanged, crash throws `"did not return a verdict"`, and a
   missing binary throws `"could not find the bee binary"`.
   `advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior`
   drives every surface the plugin wires an ADVISORY hook onto through the
   same four stub behaviors, asserting none of them ever throw.

2. **The three-belt parity test**
   (`three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode`)
   derives the guard-rule inventory from the catalog of record — the two
   checked-in, GENERATED hook manifests `packages/bee/hooks/claude-hooks.json`
   and `packages/bee/hooks/hooks.json` (both `hook_manifests.rs`'s `CATALOG`
   rendered to disk, drift-checked byte-for-byte by that module's own
   `hook_manifests_match_disk`) — never a hand-authored list, per
   `docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-it-
   never-compares-two-hand-lists.md`. A rule is BLOCKING iff it is wired
   under a `PreToolUse` event in either projection (today: `write-guard`,
   `model-guard`); every other event this catalog uses is advisory-only by
   construction. For each BLOCKING rule the test asserts four independent
   signals, failing with a message naming the rule AND the belt that missed
   it if any is absent:
   - **helper level** — `bee hook <rule>` itself denies a known-denying
     payload (the shared FIRST belt every runtime's translation layer calls
     into — plan.md: "helpers stay the FIRST belt on every runtime"), run
     with `BEE_HOOK_NO_DELEGATE=1` so a Node delegation can never pass
     silently;
   - **claude belt** — the rule is wired under `PreToolUse` in
     `claude-hooks.json` AND `hook_contracts.rs`'s own source (embedded at
     compile time via `include_str!`, scanned function-by-function) contains
     a deny fixture using one of claude's own matcher-derived tool names;
   - **codex belt** — the rule is wired under `PreToolUse` in
     `packages/bee/hooks/hooks.json`. Codex has no separate translation
     layer to fixture-test beyond that wiring: its `PreToolUse` command execs
     `bee hook <rule>` directly, the exact call the helper-level check above
     already proves denies (the only difference from claude is the matcher
     token — `spawn_agent` vs `Agent|Task` — a named, approved difference in
     `hook_manifests.rs`'s own `ALLOWED_DIFFERENCES`);
   - **opencode belt** — `bee-guard.ts`'s `mapToolCall` actually routes at
     least one real tool to this hook, derived by parsing the plugin's own
     `switch (tool)` statement (never hand-listed) — the deny/allow/crash/
     missing PROOF for every such row lives in part 1 above; this is the
     routing-exists cross-check that keeps part 1 from going vacuous if a
     row's `hook:` literal ever changes.

   A companion test, `advisory_gaps_the_plugin_does_not_wire_are_named_not_
   silent`, closes the other half of the coverage-gate contract: every
   ADVISORY rule the catalog carries that `bee-guard.ts` does NOT wire
   (derived by parsing its `runAdvisoryHook(directory, "...")` call sites)
   must be documented as a named gap in THIS file (`discovery.md`), ON ITS
   OWN LINE (oc-9's F5 fix — see below) — today that is exactly
   `codex-subagent-audit` (a `NAMED EXCLUSION` — see oc-6's table above) and
   `chain-nudge` (the `Deferred: chain-nudge` section above). A future
   catalog change that adds an ADVISORY hook the plugin silently fails to
   wire, with no matching same-line write-up here, fails this test by name
   rather than shipping unnoticed.

3. **The tool-registry coverage gate** (oc-9,
   `every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_
   as_a_gap`) closes F4: part 2's belt checks only ask whether `mapToolCall`
   routes AT LEAST ONE tool to a given hook, and part 1's `pairs.len() >= 9`
   floor only checks `mapToolCall` against ITSELF — neither can catch a
   registered OpenCode tool `mapToolCall`'s `default: return null` arm lets
   through unguarded, the exact `apply_patch` defect class oc-3 closed by
   hand. This test instead derives the REGISTERED tool inventory from the
   installed `opencode` binary's own text (a Bun-bundled standalone
   executable — no plain JS source to read directly, but its minified JS
   payload is still greppable ASCII inside it, exactly as oc-1/oc-3 already
   established) via two anchors: a 14-element tool-id `Set` literal
   (values-based, so it survives the surrounding minified variable name
   changing on a rebuild) plus three further ids (`invalid`, `plan_exit`,
   `lsp`) each confirmed by their own independent registration-body anchor.
   For every derived id `mapToolCall` does not map, the same binary text is
   scanned for a `filePath` parameter within a bounded window after that
   id's own anchor — confirmed non-file-capable ids (`webfetch`,
   `websearch`, `todowrite`, `skill`, `invalid`, `plan_exit`, `execute` —
   none carry `filePath`) need no further coverage; a confirmed-or-
   unclassifiable id (`lsp` — confirmed; nothing else fell in this bucket
   when oc-9 ran) must be EITHER mapped by `mapToolCall` OR named as a gap
   in discovery.md on its own line, reusing F5's fixed
   `discovery_doc_names_as_a_gap` helper. See "Discovery: parity suite's
   false-green paths closed (oc-9)" below for what this derivation actually
   found (`lsp` unmapped, confirmed; `list` unmapped, named but NOT
   mechanically derivable by this test — a real, disclosed limit, not a
   silent one).

### Skipped-environment behavior

"Node is absent" is not the only way the fixture belt (part 1 above) is
unrunnable — a live probe proved a `node` that IS on PATH but too old to
strip TypeScript natively is functionally the same gap, and a naive
existence check (`node --version` succeeds) does NOT catch it: with the
ambient PATH deliberately narrowed to put a system `node` v18.19.1 ahead of
nvm's v24.14.1, every harness spawn died with
`TypeError [ERR_UNKNOWN_FILE_EXTENSION]` on the real `bee-guard.ts` — a hard
panic, not a clean skip. Fixed by replacing the existence check with a real
capability probe, `node_typescript_probe()`: it writes a one-line `.ts` file
to a tempdir and runs it directly, exactly how the harness loads the real
plugin. On failure (node absent OR TS-incapable) both fixture tests print a
named `SKIP (env-limited: ...)` line (matching `hook_contracts.rs`'s own
`ran_native` skip convention) and return early — reconfirmed live with the
same narrowed-PATH probe, which now reports `SKIP (env-limited: \`node\`
(v18.19.1) cannot run a minimal .ts file directly ...)` and passes, instead
of panicking. The catalog-derivation tests (both belong to part 2) do not
depend on `node` at all and always run.

Verified on this machine: `cargo test --release --manifest-path
packages/bee-rs/Cargo.toml --test opencode_plugin_contracts` — `4 passed`
with the ambient PATH (nvm's `node` v24.14.1 first, every fixture actually
exercised, ~3.2s); `4 passed` again with PATH narrowed to put system `node`
v18.19.1 first (both node-dependent tests print the named SKIP line above
and return early, ~0.1s) — the suite never goes red on an incompatible
`node`, and it never goes silently green either: the SKIP line names
exactly which test degraded and why.

## Discovery: exit-0 repair/ask verdicts honored on the BLOCKING path (oc-8)

**Date:** 2026-08-11
**Scope:** oc-8 — S3 judge fixes F2 and F6. F2: `bee-guard.ts` used to
discard every exit-0 verdict's stdout, silently dropping write-guard's
`AskUserQuestion` repair, model-guard's dispatch-label/`subagent_type`
repair, and write-guard's `ask` verdict (write_guard/main.rs:389-394's own
comment: "ask, never allow") — the last of these is bee's DOMINANT
enforcement path for a repaired `AskUserQuestion` call, and treating it as
a plain allow would have silently defeated it on OpenCode specifically. F6:
`chat.message`'s `output.message.id` dereference sat outside any
try/catch, so an advisory surface could crash the whole call.

### D6 implementation

`runBlockingHook` (`.opencode/plugins/bee-guard.ts`) now parses non-empty
exit-0 stdout as JSON and, per decision D6:

- applies `hookSpecificOutput.updatedInput` onto `output.args` directly —
  correct with zero reverse field-name translation for both repair paths
  that exist today, because both target `question`/`task`, the two tools
  `mapToolCall` passes through UNCHANGED;
- throws `bee <hook>: <reason>` when `permissionDecision === "ask"`, using
  `permissionDecisionReason` (falling back to `additionalContext`) as the
  thrown reason — checked BEFORE the repair is applied, since the throw
  already carries the full context;
- logs `additionalContext` to `console.error` when present with no `ask`
  (a repair note, or a bare reservation warning) — `tool.execute.before`'s
  `output` has no text-injection field to carry it into the session proper,
  so a stderr advisory log is the surface this cell picks;
- throws on stdout that is non-empty but fails `JSON.parse` — undecidable,
  and undecidable stays fail-closed on this path, never a silent allow.

`chat.message`'s synthetic-part push (including the `output.message.id`
dereference) is now wrapped in its own try/catch, logging to
`console.error` on failure — matching every other advisory surface in this
file, none of which can throw.

### Live proof (direct plugin invocation, stub `bee` binary)

`opencode run` cannot exercise the `AskUserQuestion` repair specifically in
this installation (oc-6 already named this: the default `build` agent +
free `opencode/*` provider does not offer the model a `question` tool
call). Instead, `runBlockingHook`'s exit-0 handling was proved by importing
the real, unmodified `.opencode/plugins/bee-guard.ts` (Node's native `.ts`
type-stripping, no build step — the same load path OpenCode itself uses)
against a scripted stub `.bee/bin/bee` that emits each of bee's real
`write_guard/main.rs` verdict shapes on stdin/stdout, and calling
`tool.execute.before` on the `question` tool directly:

```
[repair]      allow, args= {"questions":[{"header":"short…","options":[]}]}
[ask]         THROW: bee write-guard: header truncated, confirm
[garbage]     THROW: bee write-guard returned an exit-0 verdict this plugin could not parse (not json{{{) — denying rather than allowing an unchecked call.
[plain-allow] allow, args= {"questions":[{"header":"ok","options":[]}]}
[chat.message] no throw, no output.message present
```

`[repair]`'s `args` shows the mutated `header` (`"short…"`, truncated) —
the repair reached `output.args`, not just an unread stdout string.
`[ask]`'s thrown message carries `permissionDecisionReason` verbatim —
proof the ask path throws rather than silently allowing. `[garbage]`
proves an unparseable exit-0 verdict fails closed. `[chat.message]` proves
F6: calling the handler with no `output.message` at all does not throw.

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test
opencode_plugin_contracts` — oc-9's suite, unmodified by this cell — still
passes `4 passed` against the changed plugin: none of its four fixture/
parity assertions exercise a repair or `ask` shape (its stub only allows
with empty stdout, denies, crashes, or is absent), so this cell's new
parsing branches are exercised by the direct-invocation proof above, not
by that suite; the not-yet-covered repair/ask/unparseable shapes are named
here for oc-9 to pick up as fixture cases, not silently assumed covered.

## Discovery: parity suite's false-green paths closed (oc-9)

**Date:** 2026-08-11
**Scope:** oc-9 — S3 judge fixes F1, F3, F4, F5. `.opencode/plugins/
bee-guard.ts` is unchanged by this cell (out of scope — any defect the new
assertions revealed is recorded here, not fixed in the plugin). The suite
now has **five** `#[test]` functions (was four); the "Runner design" section
above is updated in place to describe the current shape.

### F1 — a node-incapable shell could no longer produce a false green

`node_or_skip!` used to `eprintln!` its reason and `return` unconditionally
on any node/TS-capability failure — a shell with a pre-v24 system `node`
ahead of nvm on PATH (no override) got 4 green tests and ZERO enforcement
coverage exercised, indistinguishable in the test summary from a fully-
proved run. Fixed: a new `BEE_OPENCODE_SUITE_ALLOW_SKIP` env var is the
suite's one opt-out surface. **Unset (the default): an absent or
TS-incapable `node` (and, for oc-9's new tool-registry test, an absent
`opencode` binary) is now a hard FAIL, not a skip.** Set: the same absence
degrades to a named SKIP, matching `hook_contracts.rs`'s own `ran_native`
convention. The reason string always reaches stderr first (cargo test never
captures stderr, only stdout), so it reaches the default, captured output on
a PASS *and* a FAIL alike — never only visible after the fact.

**Decision, recorded per the cell's own instruction to match how this
project treats environment-gated proof elsewhere:** default-FAIL-unless-
opted-out was chosen over default-SKIP-unless-required, because a coverage
suite whose only job is to prove enforcement is real must never look green
when it proved nothing — the same reasoning `.bee/config.json`'s own
`commands.test`/`commands.verify` posture and this repo's CI already apply
to every OTHER cargo test (a red is surfaced, never silently downgraded).

**Real, disclosed consequence for CI:** `.github/workflows/ci.yml`'s R6-
cutover comment confirms the Node runtime/matrix was deleted outright when
this repo went all-Rust — the `verify` job installs no Node toolchain at
all. Whether `ubuntu-latest`'s ambient `node` (if any is on PATH without an
explicit `actions/setup-node` step) is TS-capable is NOT verified here. If
it is not, this suite's two node-dependent tests (and the tool-registry test
below, which needs a real installed `opencode` binary CI also does not
provision) will go from a silent, unnoticed skip to a **loud, named FAIL on
the next CI run** — an intended consequence of closing F1, not an accident,
but one CI itself does not yet accommodate. Named gap for a later cell:
either provision a TS-capable `node` (and an installed `opencode` binary) in
`ci.yml`, or set `BEE_OPENCODE_SUITE_ALLOW_SKIP=1` there deliberately (with
its own comment explaining CI intentionally runs this belt degraded) — not
decided here, since this cell's `files` do not include `ci.yml`.

### F3 — every mapped row now asserts the exact payload bee receives

Every stub `.bee/bin/bee` used to swallow stdin (`cat >/dev/null`) and every
fixture sent a generic `{"probe": tool, "value": "x"}` body — a field-name
mistranslation in `mapToolCall` (a renamed, dropped, or mis-shaped field)
would fail OPEN and stay green, since nothing ever inspected what bee
actually received. Fixed: `write_stub_bee` now captures stdin verbatim to
`last_stdin.json` next to the stub; a new `opencode_call_fixture(tool)`
table pairs a REAL, live-verified `output.args` shape per tool (field names
matching oc-2/oc-3/oc-6's own field-shape tables) with the EXACT bee-shaped
`tool_input` (+ `tool_name`, `cwd`, `session_id`) `mapToolCall`'s translation
must produce; the ALLOW scenario of
`every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary`
now asserts the captured payload equals that expectation exactly, for every
one of the 9 mapped rows. The same test also gained three D6 scenarios per
row (oc-8's exit-0 repair/ask/unparseable handling, previously proved only
by oc-8's own one-off direct-invocation script, never by this suite): a
`Repair` stub proves `updatedInput` lands on `output.args`; an `Ask` stub
proves a `permissionDecision: "ask"` verdict throws with the reason intact;
an `UnparseableVerdict` stub proves non-JSON exit-0 stdout still throws
fail-closed.

### F4 — the tool-registry inventory is now derived, not hand-floored

`opencode_tool_hook_pairs`'s `pairs.len() >= 9` floor and the three-belt
parity test's "at least one tool routes to this hook" both only checked
`mapToolCall` against ITSELF — the exact anti-pattern
`docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-it-
never-compares-two-hand-lists.md` warns about, and the reason `apply_patch`
slipped through unmapped until oc-3 caught it by hand. A new fifth test,
`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_
as_a_gap`, derives the REGISTERED tool inventory from the installed
`opencode` binary's own text (`resolve_opencode_binary` / `opencode_binary_
text` — the same "compiled binary, still-greppable embedded JS" fact oc-1's
Blockers section and oc-3's "Static binary read" already established), via:

- a 14-element tool-id `Set` literal OpenCode's own icon-lookup helper
  builds (`bash, glob, read, grep, webfetch, websearch, write, edit, task,
  apply_patch, todowrite, question, skill, execute`) — values-based, so it
  survives the surrounding minified variable name changing on a rebuild;
- three further ids the Set omits, each confirmed by its own independent
  registration-body anchor: `invalid` (`V("invalid",s.succeed({description:
  "Do not use"` — the "unregistered tool" sentinel), `plan_exit` (the
  build-agent handoff prompt), `lsp` (**this cell's finding** — an LSP query
  tool whose registration body carries a `filePath` parameter, confirmed
  live via `rg -a` against the real installed binary, not just the `strings`
  dump).

For every derived id `mapToolCall` does not map, the same binary text is
scanned (bounded 500-byte window past that id's own anchor) for a `filePath`
parameter. Re-verified directly against the real installed
`opencode-ai@1.18.16` binary at
`~/.nvm/versions/node/v24.14.1/lib/node_modules/opencode-ai/bin/opencode.exe`
(not only the `strings`-dumped copy this investigation started from):
`webfetch`, `websearch`, `todowrite`, `skill`, `invalid`, `plan_exit`, and
`execute` all confirmed FALSE (no `filePath` in their own body — network-,
session-, or catalog-scoped, never an arbitrary caller-supplied path) and
need no further coverage; **`lsp` confirmed TRUE** and is neither mapped nor
(until this line) documented — running the new test against the unmodified
plugin fails exactly this one row, by name:

```
"lsp": registered by the installed opencode binary, not mapped by
mapToolCall, and not documented on its own line as a named gap in
docs/history/opencode-support/discovery.md (filepath_evidence=Some(true))
```

**`lsp` — RESOLVED (oc-10).** The installed `opencode-ai@1.18.16` binary
registers an `lsp` tool (`V("lsp",s.gen(function*(){...filePath:n...`,
asking permission `"lsp"`) that returns file content (via LSP operations —
`hover`/`documentSymbol`/etc.) for an arbitrary caller-supplied `filePath`,
exactly the read-capable shape `read`/`grep`/`glob` already route through
write-guard — `mapToolCall`'s `default: return null` arm used to let it
through unguarded, the SAME defect class `apply_patch` was before oc-3.
Fixed: `mapToolCall` now carries an `"lsp"` case routing to write-guard's
`Read` shape, translating `filePath` -> `file_path`, the same way oc-3
added `apply_patch`. See "Discovery: lsp and list mapped through
write-guard (oc-10)" below for the mapping and verification.

**`list` — RESOLVED (oc-10); still NOT mechanically derivable by
`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_
as_a_gap`.** The S3 judge's finding also named a second unmapped registered
tool, `list` (discovery.md's oc-3 tool-registry table already shows 13
tools from a LIVE `tool.definition` probe, which oc-9's finding calls
incomplete). oc-9's own investigation could not locate any static
`V("list", ...)` registration anchor for a `list` tool anywhere in the
installed binary's text — the judge's evidence for it was almost certainly
a live probe (oc-1/oc-3's OTHER, non-static evidence source: a scratch
project with a `tool.definition`-hooking plugin run against a real
`opencode run` session), which a `cargo test` cannot reproduce
deterministically offline (no model access, no network).

oc-10 instead confirmed `list`'s existence and field shape from two OTHER
independent static-text anchors in the same installed binary (neither is
the `V("list", ...)` registration-body pattern `lsp`/`apply_patch` use, so
this does not give `derive_opencode_tool_registry` a mechanical anchor to
add — the test's silence about `list` remains a real, disclosed limit, not
a fixed one):

- the TUI's own tool-title-rendering code, which switches on the same tool
  id space as `read`/`glob`/`grep`/`bash`/`edit`/`task`/`webfetch`/
  `websearch` and includes `list`:
  `` if(a==="list"){let V=B.path,q=typeof V==="string"?V:"";return{icon:"→",title:`List ${...` `` —
  confirming both that `list` is a real registered tool (grouped with the
  other read-shaped tools in the same rendering switch, not a coincidental
  string) and that its argument field is `path`, not `filePath`;
- a second, independent UI-string function,
  `` case"list":return{icon:"bullet-list",title:r.t("ui.tool.list"),subtitle:t.path?rn(t.path):void 0} ``,
  confirming `path` again from an unrelated code path;
- the server's HTTP API surface, which registers a matching route:
  `` C.get("list",yg.list,{query:B$,success:A(g.Array(M8),"Files and directories")}).annotateMerge(... identifier:"file.list",summary:"List files",description:"List files and directories in a specified path." ``,
  confirming `list`'s purpose (directory-entry listing, not raw file
  content) and giving a bee-shape precedent: the same directory-listing
  shape `glob` already routes through write-guard.

Fixed: `mapToolCall` now carries a `"list"` case routing to write-guard's
`Glob` shape (`path` -> `path`, no rename needed — matching `grep`/`glob`'s
own field name). See "Discovery: lsp and list mapped through write-guard
(oc-10)" below for the mapping and verification. The remaining gap —
`derive_opencode_tool_registry` still cannot mechanically confirm `list` on
its own — is now moot for THIS suite's pass/fail (the tool is mapped, so
the registry-gate test's `mapped.contains(id)` branch short-circuits before
ever reaching the named-gap check for `list`), but is recorded here in case
a future `opencode-ai` release changes `list`'s shape: this mapping rests
on UI-string/HTTP-API evidence, not a `V("list", ...)` registration-body
anchor.

### F5 — the named-gap check is now scoped to the rule's own line

`advisory_gaps_the_plugin_does_not_wire_are_named_not_silent` used to AND a
per-rule `DISCOVERY_DOC.contains(rule)` with a DOCUMENT-GLOBAL
`DISCOVERY_DOC.contains("NAMED EXCLUSION") || DISCOVERY_DOC.contains(
"Deferred")` — both marker literals are always present SOMEWHERE in this
file (on `codex-subagent-audit`'s and `chain-nudge`'s own rows), so ANY rule
name mentioned ANYWHERE in the document passed, whether or not that mention
was actually tagged as a gap. Fixed: a new shared helper,
`discovery_doc_names_as_a_gap(name, markers)`, requires the name and a
marker to co-occur on the SAME LINE — every gap this file documents is
already written that way (one markdown table row carries both), so this is
a real narrowing, not a cosmetic one. The advisory-gap test now uses this
helper; the new F4 tool-registry test (above) reuses the SAME helper (with
an additional `"NAMED GAP"` marker for `lsp`'s own row) rather than
re-implementing a second, possibly-differently-buggy scoping check.

### Verified

`PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path
packages/bee-rs/Cargo.toml --test opencode_plugin_contracts` — before
adding `lsp`'s NAMED GAP line above: `4 passed; 1 failed` (the new
tool-registry test failing exactly on `"lsp"`, by name, confirming F4 closes
a REAL silent gap rather than manufacturing one). After adding the line:
`5 passed; 0 failed`, ~5.5s, with the ambient `node` v24.14.1 and the
installed `opencode-ai@1.18.16` binary both present on this machine (neither
`BEE_OPENCODE_SUITE_ALLOW_SKIP` override needed here).

## Discovery: a bee-build worker capped a real cell from inside a nested OpenCode session (oc-12)

**Date:** 2026-08-11/12. **Scope:** oc-12 — S4 (plan.md E4), the D4/D5
proof itself: does a bee worker actually run and cap a cell from inside an
OpenCode session, not merely get defined (oc-11)? Method: a throwaway cell
(`lv-1`, feature `opencode-support-proof`, added/claimed through the real
`bee cells add`/`claim` CLI, never one of this feature's own `oc-*` cells)
whose only job is to write one file and cap. Three live `opencode run`
attempts, in order, each building on what the last one found.

### Bottom line

**Yes — proved live, twice over the failure mode the cell asked about.**
A real `bee-build` OpenCode subagent, reached through the `task` tool (not
an `--agent` flag — that path silently does NOT reach a `mode: subagent`
agent, see the named gap below), wrote the file, committed it, and ran
`bee cells finish --id lv-1` to completion inside its own nested OpenCode
session — `capped lv-1 at 2026-08-11T17:32:09.802Z (tests: green)`,
confirmed independently via `bee cells show --id lv-1` (`"status": "capped"`)
and the OpenCode session database. Two real, worth-recording defects
surfaced on the way, neither in scope to fix here (per this cell's own
instruction): a reservation session-identity mismatch on nested dispatch
(architectural, D3-relevant), and `opencode run --agent <subagent>` silently
falling back to the default primary agent instead of the named subagent.
The guard belt did not go missing anywhere in the nested path — it engaged
correctly at every guarded surface it was exercised against, twice denying
for real reasons and once allowing.

### Attempt 1 — nested task-tool dispatch, denied by a real (mismatched) cross-session hold

Outer prompt (`opencode run`, default `build` primary agent, model
`opencode/big-pickle`, no `--agent`/`--model` flag): instructed to make
exactly one `task` tool call with `subagent_type: "bee-build"`,
`description: "cap cell lv-1"`, and a prompt carrying the full cell payload
(nickname `oc-live`, cell JSON, environment paths) — mirroring the shape
this very cell's own dispatch prompt used. The file had been pre-reserved
by the orchestrator (this Claude session, `oc-live`) via
`bee reservations reserve --agent oc-live --cell lv-1 --path
docs/history/opencode-support/live-worker-proof.md` before dispatch, exactly
as this cell's own `discovery.md` reservation was pre-made for it.

Live transcript (verbatim, from the nested `bee-build` child session):

```
bee cross-session hold: "docs/history/opencode-support/live-worker-proof.md"
is held by session "fd787469-f9f4-4ca3-bf5f-6d8820b24165" (agent oc-live,
cell lv-1), expires 2026-08-11T18:16:07.274Z. Wait for the hold to expire or
coordinate with that session — a cross-session hold is a hard block (D3).

**Task result (verbatim from bee-build worker):**

Diagnosis confirmed and complete. I cannot write the cell's file through any
guarded surface — this is a genuine wiring gap, not a conflict I may resolve
or write through.

**[BLOCKED]**

**Why:** Session-id namespace mismatch in the OpenCode guard plugin wiring:
- The dispatcher reserved the file for me under session
  `fd787469-f9f4-4ca3-bf5f-6d8820b24165` — this session's own
  `CLAUDE_CODE_SESSION_ID`.
- The OpenCode plugin (`.opencode/plugins/bee-guard.ts:402`) forwards
  `session_id: input.sessionID` — OpenCode's own native `ses_…` id
  (`ses_00e2b98d2ffebr3CO2Zj8FeXra`) — to bee's write-guard.
- write-guard compares that id against the reservation holder's session ->
  mismatch -> DENY exit 2.
```

The worker's own diagnosis is correct and independently verified: direct
`bee reservations list` showed the reservation's holder session as
`fd787469…` (this Claude session's `CLAUDE_CODE_SESSION_ID`, used implicitly
by `bee reservations reserve` per `verbs/reservations/reserve.rs:103-109`
when no `--session`/`BEE_SESSION_ID` is given), while `bee-guard.ts:402`
forwards `input.sessionID` — OpenCode's own per-session id — as the
write-guard payload's `session_id`. `write_guard/store.rs`'s
`find_session_conflicts` (checks.rs:351-386) denies exactly when a
reservation's `session` differs from the acting `session_id` — proving this
is the SAME native cross-session-hold logic the Claude belt uses, reached
correctly through the nested dispatch, not a stub.

**This is the real, structural finding, not an artifact of Claude
orchestrating an external CLI:** the OpenCode session DB (`sqlite3
~/.local/share/opencode/opencode.db`, `session` table) confirms the child
`bee-build` session (`ses_00e2b98d2ffebr3CO2Zj8FeXra`) has `parent_id`
pointing at the OUTER primary session (`ses_00e2bb6ebffe7ChGPN8NDtvdap`) —
parent and child are given DIFFERENT native `ses_…` ids. Any all-OpenCode
orchestrator (a primary session reserving a file, then `task`-dispatching a
`bee-build` child to write it) would hit this exact mismatch: the
reservation carries whichever session made the CLI call, but write-guard
checks the id of whichever session performs the actual write — and those
are never the same session once a nested dispatch is involved. **Named gap,
not fixed here (out of scope for this cell):** the reservation model assumes
reservation-owner and actual-writer share one session identity, true for
Claude's Task-tool nested subagents (they inherit the same
`CLAUDE_CODE_SESSION_ID`) but false for OpenCode's `task`-tool nested
subagents (each session, parent or child, gets its own distinct native id).
A real S4/S5 follow-up needs either the plugin to forward a
workspace-scoped identity instead of the raw per-session id, or the
reservation CLI to accept the not-yet-known child id, or a documented
convention that only the actually-dispatched session reserves for itself
(see Attempt 3, which sidesteps the mismatch by never invoking it).

**Guard-engagement fact, either way, as this cell asked for explicitly:**
the write-guard belt DID engage inside the nested `bee-build` session — it
did not go missing. It fired, evaluated the real cross-session-hold rule
against real reservation-store state, and returned the same native deny
text the Claude belt would. The belt is present and correct in nested
dispatch; the defect is a session-identity plumbing mismatch between the
reservation CLI's default identity source and the plugin's forwarded id, not
an absent guard.

**Model resolution, from the session DB, not from config:**

```
sqlite3 ~/.local/share/opencode/opencode.db "select id,parent_id,agent,model,title from session where id in ('ses_00e2bb6ebffe7ChGPN8NDtvdap','ses_00e2b98d2ffebr3CO2Zj8FeXra');"

ses_00e2bb6ebffe7ChGPN8NDtvdap  (no parent)                      build      {"id":"big-pickle","providerID":"opencode"}  Live OpenCode worker proof
ses_00e2b98d2ffebr3CO2Zj8FeXra  ses_00e2bb6ebffe7ChGPN8NDtvdap    bee-build  {"id":"big-pickle","providerID":"opencode"}  cap cell lv-1 (@bee-build subagent)
```

`bee-build`'s resolved model (`opencode/big-pickle`) matches
`.opencode/agent/bee-build.md`'s pinned `model: opencode/big-pickle`
frontmatter exactly — the per-agent model pin (plan.md's structural
model-guard fallback) is real, live-verified, not assumed. The outer `build`
primary agent resolved to the SAME free model, coincidentally: that is the
workspace default, unrelated to per-tier pinning, and is recorded
separately so the two are never conflated.

**Wall-clock shape (D5 — sequential-only accepted):** dispatch started
17:17:19Z, the outer `build` agent explored the repo on its own (13 Bash/
Grep/Read calls — it was told to make exactly one `task` call and did not;
see the free-model-behavior note below) before finally calling `task` at
17:22:02Z, and the whole invocation (outer exploration + nested dispatch +
deny + the worker's own diagnosis) returned at 17:22:15Z — **4m56s
end-to-end for ONE cell, entirely serial**, consistent with D5's accepted
sequential-only dispatch (upstream anomalyco/opencode #29638): there is no
concurrent second dispatch to time against; the shape to record is simply
that one nested dispatch fully blocks the outer session until it returns.

### Attempt 2 — `opencode run --agent bee-build` does not reach the subagent; guard re-engaged on the retry anyway

After releasing the stale reservation (`bee reservations release --agent
oc-live --cell lv-1`; a reservation with no session field never conflicts —
`find_session_conflicts`, store.rs:679-682 only treats a non-empty,
non-matching session string as a conflict), a second attempt tried
`opencode run --agent bee-build < prompt`, expecting to run AS the
`bee-build` subagent directly. It did not:

```
! agent "bee-build" is a subagent, not a primary agent. Falling back to default agent
> build · big-pickle
```

**Named gap:** `opencode run --agent <name>` silently falls back to the
default primary agent when `<name>` is `mode: subagent` — a subagent is
only reachable through the `task` tool (Attempt 1, Attempt 3), never
directly from the CLI's `--agent` flag. Worth documenting for anyone
scripting a direct single-worker OpenCode dispatch: it must go through a
primary session's `task` call, there is no shortcut.

Running as the (fallback) default `build` agent anyway, the write succeeded
cleanly (no reservation now exists, so write-guard's cross-session-hold rule
correctly no-ops — confirms the Attempt 1 defect was specifically the
mismatched reservation, not a blanket nested-write deny). The
concurrent-worker git guard then engaged for real, live, against actual
concurrent swarm activity in this checkout (4 other workers registered at
the time per `bee status`):

```
$ git add docs/history/opencode-support/live-worker-proof.md && git commit -m "..."
bee concurrent-worker git guard: `git add` is refused because 4 workers are
live in this checkout. it stages content into the SHARED index... FIX: ...
GIT_INDEX_FILE=<tmp> git read-tree HEAD, then GIT_INDEX_FILE=<tmp> git
update-index --add <your paths>, GIT_INDEX_FILE=<tmp> git write-tree, git
commit-tree <tree> -p HEAD -m "<msg>", git update-ref HEAD <commit>.
```

The model read the FIX text and self-recovered without further guidance,
running the temp-index sequence verbatim and landing a real commit
(`96bb8f66152a452f9bf1bf3b08c152118285a802`, trailer `cell: lv-1`,
confirmed via `git log -1 --format="%(trailers)"`). This is a second,
independent live proof that write-guard's BLOCKING path engages correctly
for a real worker doing real git operations inside an OpenCode session, and
that its FIX text is actionable by a small free model without hand-holding.

`bee cells finish --id lv-1` then refused for the reason this cell's own
dispatch prompt named in advance: `cells finish trailer lookback cannot see
worktree-local commits`:

```
capCell: cell "lv-1" refused — one commit per cell: no commit in the last 50
commit(s) of /home/thanhsmind/projects/goglbe/beehive carries the trailer
"cell: lv-1". ... — or pass --commit-pending "<reason>" to finish anyway.
```

The worker was not told about `--commit-pending` in this attempt's prompt
(an oversight in the dispatch, not a plugin defect) and, rather than
reporting `[BLOCKED]` with that exact refusal text, spent the rest of its
turn reading bee's own Rust source in the MAIN checkout to self-diagnose —
which triggered a real `permission requested: external_directory` ask that
was auto-rejected (no interactive approver in a non-interactive `opencode
run`), and the session ended there with no final status token. **Named,
not fixed:** a free-tier worker without the exact recovery flag in its
prompt will self-direct into exploration rather than stopping at
`[BLOCKED]` — a real free-model-behavior characteristic worth carrying into
any future OpenCode dispatch-prompt design (name the exact recovery command,
never assume the worker will discover it).

### Attempt 3 — nested dispatch, told about `--commit-pending`, capped clean

Third `opencode run` (nested, `task` tool, `subagent_type: "bee-build"`),
told the commit already existed and to run exactly one command:

```
$ /home/thanhsmind/projects/goglbe/beehive/.bee/bin/bee cells finish --id lv-1 \
  --outcome "live OpenCode worker proof" \
  --files docs/history/opencode-support/live-worker-proof.md \
  --commit-pending "commit 96bb8f66 landed in the linked worktree; trailer lookback only scans the main checkout's history"
```

Live transcript (verbatim):

```
Capped lv-1 at 2026-08-11T17:32:09.802Z (tests: green).
No active reservations to release.
next: reply [DONE] with the one-line outcome, files touched, and the commit hash.
[bee] cells finish 11780ms

Status: [DONE]. Outcome: live OpenCode worker proof — file
docs/history/opencode-support/live-worker-proof.md, commit
96bb8f66152a452f9bf1bf3b08c152118285a802 (cell: lv-1).
```

Independently confirmed via `bee cells show --id lv-1`:
`"status": "capped"`, `"capped_at": "2026-08-11T17:32:09.802Z"`,
`"tests": "green"`, `"commit_pending": "commit 96bb8f66 landed in the linked
worktree; trailer lookback only scans the main checkout's history"`. Session
DB confirms the same shape as Attempt 1: child session `bee-build`, model
`{"id":"big-pickle","providerID":"opencode"}`, `parent_id` pointing at a
fresh outer `build` session — nested dispatch, real subagent, real model
tier, real cap. Wall-clock: dispatch to return, 17:31:43Z to 17:32:17Z
(34s) — the actual capping work inside `bee cells finish` itself took
11.78s per its own reported duration; the rest is OpenCode session overhead.
This is by far the fastest of the three runs, consistent with a
single-command prompt leaving nothing for the free model to explore.

### tools-logger inside nested dispatch — inconclusive, recorded honestly

`.bee/logs/tools.jsonl` entries generated strictly during Attempt 1's
`opencode run` window (17:17:19Z–17:22:15Z, isolated by timestamp since this
orchestrating Claude session was blocked the whole time and could not be
logging its own calls) all carry `agent_id: null, agent_type: null` — same
shape oc-9 already documented for a non-nested probe. This run cannot tell
whether that is because `tool.execute.after` never fires for a CHILD
session's own tool calls (as distinct from the outer session's), or fires
but the plugin still has no per-session agent metadata to forward (the
already-documented gap). Not re-asserted as a new fact beyond what oc-9
already named — recorded here only so a future S5 pass does not have to
re-discover that this cell's own logs do not resolve it either.

### Verified

- `bee cells show --id lv-1` → `"status": "capped"`, `"capped_at":
  "2026-08-11T17:32:09.802Z"`, `"tests": "green"` (live, this session).
- `sqlite3 ~/.local/share/opencode/opencode.db` `session` table — the
  concrete parent/child rows and `model` JSON quoted above (live, this
  session; the DB is opencode's own, not bee's).
- `git log -1 --format="%H %s%n%(trailers)"` in this worktree →
  `96bb8f66152a452f9bf1bf3b08c152118285a802 Record a live OpenCode worker
  proof touch` / `cell: lv-1` (live, this session).
- `docs/history/opencode-support/live-worker-proof.md` exists in this
  worktree with the required line, committed under `lv-1`'s own commit —
  a throwaway proof artifact, not part of this feature's own file set.
