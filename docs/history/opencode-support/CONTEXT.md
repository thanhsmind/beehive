# OpenCode Support — Context

**Feature slug:** opencode-support
**Date:** 2026-08-11
**Shaping session:** complete
**Scope:** Deep
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Bee treats OpenCode as a third first-class runtime — hooks enforcement, skill
rendering, model tiers, and onboarding — proven by a working OpenCode session on
this repository; host-project onboarding ships with it but the beehive repo is
the first consumer and test bed.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | OpenCode becomes the third first-class runtime, at parity with Codex: hooks, rendered skills, `models.opencode` config key, and onboarding. The two-runtime doctrine in `docs/06-runtime-integration.md` changes to three. | The degradation ladder (skills → PLAYBOOK → helper CLI) was offered and declined; user wants the full workflow in OpenCode. |
| D2 | Enforcement is real blocking, delivered as an OpenCode TypeScript plugin mapping bee's guard hooks onto `tool.execute.before`/`tool.execute.after` (abort-capable). OpenCode's experimental config hooks (`file_edited`, `session_completed`) cannot block tools and are not an acceptable enforcement vehicle. | Write-gate and secret guards must stop the action, not advise. |
| D3 | The beehive repo itself is the first consumer: render and install OpenCode support locally, verify with an OpenCode session on this repo. Host-project onboarding follows, not leads. | Picks the test bed and work order. |
| D4 | Swarming must work under OpenCode — worker dispatch equivalent to bee-build/bee-gather. The concrete mechanism (OpenCode's subagent/agent API) is a planning research question, not a shaping decision. | Single-session-only was offered and declined. |

### Agent's Discretion

Naming and layout of the OpenCode artifact tree (`.opencode-plugin/` vs other),
marker spelling for runtime-only prose, and the internal plugin code structure —
constrained by the hook-runtime area rule R1: every per-runtime difference is a
named export from the one catalog of record, never ad hoc branching.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| First-class runtime | A runtime bee renders projections for and enforces guards in: hooks + skills + models.<runtime> + onboarding target. Contrast: external CLI executor (`kind:"cli"`), which OpenCode is today. |
| Enforcement | A guard that aborts the tool call, not one that reports after the fact. |

## Specific Ideas And References

- User previously explored OpenCode's two hook surfaces: experimental config hooks
  in `opencode.json` and TypeScript plugins in `.opencode/plugins/` /
  `~/.config/opencode/plugins/` with `tool.execute.before/after`,
  `session.created/compacted/completed`, `chat.message`, `permission.ask`. D2 picks
  the plugin surface.

## Existing Code Context

From the quick scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs:44` — `Runtime { Claude, Codex }` enum plus `Target { Plugin, Repo }`; one catalog renders per-runtime hook manifests. OpenCode joins this enum.
- `packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:34` — `RENDER_RUNTIMES: ["claude", "codex"]`, target dir picked by string match at `:42-45`.
- `packages/bee-rs/crates/bee/src/devtools/plugin_distribution.rs:1057` — `parse_runtime()` accepts only `claude|codex|both`.
- `packages/bee-rs/crates/bee/src/devtools/install_support.rs:40-67` — `merge-plugin-state --claude <f> --codex <f>` hard-wired flag pair.
- `packages/bee-rs/crates/bee/src/onboard/skills.rs` — `compute_skill_items(source, target, runtime)` takes `"claude"`/`"codex"` literals.
- `.bee/config-sample-cli-executors.json:14,48` and `docs/model-presets.md:112-171` — OpenCode-as-CLI-executor wiring (`opencode run --model … "$(cat)"`, `--agent plan|build`, no-stdin quirk); reusable knowledge of the OpenCode CLI surface, flagged not yet smoke-tested.

### Established Patterns

- Catalog-of-record with named per-runtime exports — `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md` R1: "every difference between projections is a named export rather than drift". A third runtime adds a named difference set, never ad hoc branches. (That doc's implementation pointers still cite the deleted `packages/bee/hooks/catalog.mjs` — stale since the R6 Rust port; fix in passing.)
- Runtime-only prose markers `<!-- bee:only claude -->` / `<!-- bee:only codex -->` — closed 2-value grammar in `docs/06-runtime-integration.md:74-98`; needs an `opencode` value.
- Codex sidecar pattern `skills/<name>/agents/openai.yaml` → copied into rendered trees — precedent for per-runtime skill metadata.

### Integration Points

- `packages/bee-rs/crates/bee/src/devtools/mod.rs:71-104,351-358` — `bee dev` verb dispatch and `render_projection_text_for(runtime)`.
- `docs/config-reference.md:78-100` — `models.<runtime>` schema currently rejects (silently ignores) an `opencode` key; becomes a real key.
- `docs/06-runtime-integration.md` — doctrine doc; two-belt design and degradation ladder text changes under D1.
- `docs/knowledge/areas/onboarding/` — installer entry points, distribution sources, repo-local guardrails; the Codex-vs-Claude asymmetry documented there gets a third column.

## Canonical References

- `docs/06-runtime-integration.md` — runtime integration doctrine (changes under D1).
- `docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md` — catalog/projection rule R1.
- `docs/knowledge/areas/onboarding/installer-entrypoints-and-source-staging.md` — install flow.
- https://opencode.ai/docs — OpenCode plugin and config surface (verify against installed version during planning).

## Outstanding Questions

### Resolve Before Planning

- [ ] None — shaping decisions cover scope; the rest is technical investigation.

### Deferred To Planning

- [ ] OpenCode plugin API exact contract — package name, hook signatures, abort semantics of `tool.execute.before`, plugin discovery order (project vs global) — verify against a real installed OpenCode version, not blog posts.
- [ ] Whether OpenCode has a skill/plugin distribution mechanism equivalent to `.claude-plugin`/`.codex-plugin`, or skills ride AGENTS.md + rendered tree only — decides the `.opencode-plugin/` layout.
- [ ] Worker dispatch mechanism for D4 — OpenCode subagent/agent API, or fallback to bee's cli-shaped gather/build dispatch — and what "parity with bee-build/bee-gather" means there.
- [ ] `models.opencode.{extraction,generation,review}` mapping — how OpenCode names models/providers (`-m provider/model`) and whether tier switching is per-call or per-session.
- [ ] Which of bee's guard hooks are expressible in OpenCode's event set, and the named fallback for each one that is not.
- [ ] Whether `install_support.rs` flag pair (`--claude/--codex`) generalizes to `--runtime <name>` or gains a third flag — API-shape choice.

## Deferred Ideas

- Host-project onboarding polish (docs, `bee onboard` UX for OpenCode-only projects) — lands after the beehive-repo proof per D3.
- Smoke-testing the existing `opencode-review` CLI-executor preset (`docs/model-presets.md:112`) — adjacent debt, not this feature's scope.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
