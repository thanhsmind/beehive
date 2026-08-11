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

### Implemented hook → OpenCode surface table

| bee hook | OpenCode surface | Failure policy | Status |
|---|---|---|---|
| write-guard | `tool.execute.before` on `write`/`edit`/`bash`/`apply_patch` (oc-2/oc-3) **+ new:** `read`/`grep`/`glob`/`question` | BLOCKING — throw on deny, fail closed | Live-proved: read-size deny (below); write/edit/bash/apply_patch already proved in oc-2/oc-3 |
| model-guard | `tool.execute.before` on `task` | BLOCKING — throw on deny, fail closed (same policy, same before-hook, as write-guard) | Live-proved: Task deny (below) |
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
- **On-allow stdout repairs are not read back into `output.args`.**
  write-guard's `AskUserQuestion` header-truncation auto-fix and
  model-guard's dispatch-label/`subagent_type` auto-fix (both emitted as
  `hookSpecificOutput.updatedInput` JSON on a verdict-carrying exit 0,
  main.rs:385-411, model_guard.rs:130-155) are not parsed or merged back
  into OpenCode's mutable `output.args` — the call is let through
  unmodified, exactly as an ordinary allow would be, rather than with the
  repair applied. Doing so would need a reverse field-name mapping (the
  mirror image of `mapToolCall`) for each repaired shape — out of this
  cell's scope, named rather than silently dropped.
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
