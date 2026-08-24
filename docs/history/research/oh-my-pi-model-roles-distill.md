---
artifact_contract: bee-research/v1
topic: oh-my-pi-model-roles-distill
depth: standard
date: 2026-08-24
---

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream**
- Why this is the lightest credible path: bee already owns every piece
  this needs — a rendered agent set whose names are jobs, a slot map, a
  resolver, a guard. What bee lacks is the **indirection** between them.
  oh-my-pi's design is one indirection plus one fallthrough rule, both
  small, both provably load-bearing in a shipping product.
- Why the next-best rung lost: *built-in* loses because bee's slot list
  is closed and hardcoded in four places, so no configuration reaches a
  job-named role. *Build* loses because the shape is already proven
  upstream and bee's own defects (a dead slot, a dropped `effort`, two
  never-passed kinds) are the exact defects the upstream shape removes.
- Confidence: 85%
- Suggested next step: **bee-wayfinding** — this feeds open tickets 002
  and 003 of `docs/discovery/model-role-split/`, and the owner's answer
  is still required on the role-axis question. Not yet shapeable.

## Repo Snapshot

- Source: `~/Projects/refs/oh-my-pi`, branch `main`, commit
  `2b66ee69f2493d29dfdf0990b14209a7a96c824b` (2026-08-24). Scope
  narrowed to model-role configuration and resolution.
- Source stack: TypeScript, `packages/coding-agent/`. Roles are a config
  record; models live in a separate registry file (`~/.omp/agent/models.yml`).
- Local: bee, Rust (`packages/bee-rs/crates/bee`). Roles are
  `CONFIGURABLE_SLOTS` plus a pseudo-tier, resolved at one dispatch door.

## Question & Assumptions

- What was asked: how does oh-my-pi split model roles, and does its split
  answer the owner's objection that `extraction`/`generation` are cost
  words, not job words — so no one can pick a model that is "good at
  extraction"?
- What success appears to mean: a role vocabulary a user can configure
  per *job* ("this model plans well, that one tests well"), without the
  dead-config disease ticket 001 found.
- Assumptions still needing confirmation: that bee wants an **open** role
  set (user-defined names). Upstream has one; bee's is closed. This is
  the owner's call, still open.

## Findings

### Upstream

**The role list is mixed-axis — they did not solve it by purity.**
`Upstream` `config/model-roles.ts:22-32` and `Docs` `docs/models.md:392-394`:
`default, smol, slow, vision, plan, designer, commit, tiny, task, advisor`.
The framing is intent, not cost — `Docs` `README.md:333`: "Ten roles
route work by intent." Classified from the docs' own wording:

| Class | Roles |
|---|---|
| JOB | `plan` ("Architect"), `designer`, `commit`, `task` ("Subtask"), `advisor` |
| COST / SIZE | `smol` ("Fast"), `tiny` |
| CAPABILITY | `slow` ("Thinking"), `vision` |
| AMBIENT | `default` |

So six of ten name a job. The cost words survive beside them without
harm, because of the two mechanisms below.

**Mechanism 1 — every consumer asks for an ordered LIST of roles.**
`Upstream`, real call sites:

```ts
resolveRoleSelection(["commit", "smol", ...MODEL_ROLE_IDS], settings, available)  // commit/model-selection.ts:46
resolveRoleSelection(["tiny", "commit", "smol"], ...)                              // utils/title-generator.ts:115
resolveRoleSelection(["tiny", "smol"], ...)                                        // auto-thinking/classifier.ts:115
resolveRoleSelection(["smol"], ...)                                                // edit/auto-repair.ts:291
for (const pattern of ["@vision", "@default", activeModelPattern])                 // tools/inspect-image.ts:174
```

An unconfigured role therefore **costs nothing** — it falls through to
the next name. `Docs` `docs/models.md:396`: the `tiny` role, "when
unset, these fall back to `@smol`." `Docs` `docs/tools/inspect_image.md:43`:
"Model selection tries, in order, `@vision`, `@default`, the active
model string from the session, then `availableModels[0]`." Unset roles
inherit structurally — `Upstream` `model-resolver.ts:1010-1027`:
`smol`/`slow`/`designer` inherit the configured `default` first;
`advisor` defaults to the `slow` chain but "never inherits the
primary's model"; `tiny` reuses the `smol` chain.

**Mechanism 2 — the role set is open.** `Upstream`
`model-roles.ts:77-91` (`getKnownRoleIds`) appends any role found in
`cycleOrder`, `modelRoles`, or `modelTags`. `Docs` `docs/settings.md:361`
— `modelTags`: "Custom role/tag metadata; can introduce additional
roles." A worked custom role `review` appears at
`docs/task-agent-discovery.md:70-74`.

**The entry is a bare string with a small grammar — not an object.**
`Upstream` `settings-schema.ts:663`, `:6268`:
`modelRoles: Record<string, string>`. The string may carry:
`provider/id` or a fuzzy id; a comma-separated ordered candidate list
(`model-resolver.ts:971-975`); a trailing thinking suffix
`:low|:high|:xhigh|:max` (`:121-130`); a trailing `@upstream` routing
hint (`:355-364`); or another role alias, resolved recursively with
cycle detection (`:1090`). `Docs` config example, `docs/settings.md:335-355`:

```yaml
modelRoles:
  default: anthropic/claude-sonnet-4-5
  smol: openai/gpt-4.1-mini
  slow: anthropic/claude-opus-4-5:high
  plan: anthropic/claude-opus-4-5
  advisor: anthropic/claude-sonnet-4-5:medium
```

Cost, context window, `maxTokens`, temperature and compat live on the
**model registry entry**, never on the role (`Docs` `docs/models.md:58-86`).

**Agents point at roles by name; dispatch names only the agent.**
`Docs` `docs/task-agent-discovery.md:53`: "Give the agent a role alias in
frontmatter, then dispatch it by name. For model routing, task dispatch
sets only `agent`; it does not set a worker model." Bundled agents,
`Upstream`: `scout.md:5` `model: "@smol"`, `librarian.md:5` `"@smol"`,
`reviewer.md:6` `"@slow"`, `designer.md:4` `"@designer"`. The frontmatter
`model` is itself an ordered list — `Docs`
`docs/task-agent-discovery.md:42`: "`model` accepts one selector, CSV, or
an array. Entries are tried in order after role aliases are expanded."
Precedence, `Docs` `:204-209`: `task.agentModelOverrides[agent]` → the
agent's own list → the parent's active model → its default.

The docs recommend the indirection explicitly — `Docs` `:97-107`: "Route
these tiers through roles by keeping aliases in
`task.agentModelOverrides` and concrete selectors only in `modelRoles`."

**Fallback is a separate layer from resolution, and it is error-class
gated.** `Docs` `docs/settings.md:439-463` — `retry.fallbackChains` is a
`Record<string, string[]>` keyed by role, by exact model selector, or by
`provider/*` wildcard; "Any role without an explicit chain inherits the
`default` chain", and a model-selector key "applies whenever that model
is active, no matter which role it is assigned to, and survives role
reassignment." Revert policy is configurable — `Docs` `:476-478`:
`fallbackRevertPolicy` defaults to `cooldown-expiry`.

Which failures advance the chain — `Upstream` `turn-recovery.ts:2049-2061`,
`:1101-1106`, and `Docs` `docs/non-compaction-retry-policy.md:49-50,72-73`:

| Advances the chain | Does NOT advance it |
|---|---|
| `UsageLimit` (429, quota) | tool errors |
| `AccountPolicy` (credential rotated first) | bad or unwanted output |
| `MalformedFunctionCall` (replay-safe only) | `ThinkingLoop` — explicitly excluded |
| `EmptyResponse` | anything not `AIError.retriable` |
| stream stall / HTTP2 reset / premature close | |
| 500, 502, 503, 504 | |

Candidates are also filtered *during* the walk — thinking-signature
binding, effort-ceiling support, context window fit, missing API key
(`Upstream` `turn-recovery.ts:1664-1685`) — and a proactive switch
exists before failure when usage health reads `depleted`/`reserve`
(`:1396`, `:1483`).

**Capability is a named role plus a post-hoc filter, not a requirement
the resolver matches.** `Upstream` `image-vision-fallback.ts:103-118`
resolves the chain, then filters `model?.input.includes("image")`;
`tools/inspect-image.ts:187-190` errors after the fact. There is no
declarative "this work needs vision" input to resolution.

### Local

bee's picture, from this session's verified reads (all `Local`):

- Slots are cost-shaped and closed: `CONFIGURABLE_SLOTS =
  ["extraction","generation","review"]` (`models.rs:37`), `advisor` added
  by `MODEL_NORMALIZE_SLOTS` (`:40`), `ceiling` a non-configurable
  pseudo-tier (`:324-326`).
- **bee already has job names — at the agent layer.** `bee-build`,
  `bee-gather`, `bee-extract`, `bee-review` are rendered agents
  (`onboard/templates.rs:222-230`), each **welded 1:1 to a cost slot** by
  `pinned_agent_type` (`verbs/drivers/guard.rs:32-39`). `bee-build` and
  `bee-gather` both sit on `generation`, so "my coder model" and "my
  reader model" cannot differ.
- Selection is by door kind or cell tier, and the door is narrow:
  `DISPATCH_KINDS` has four entries, of which only `cell` and `gather`
  are ever literally passed anywhere in the repo; `reviewer` and
  `advisor` appear only inside placeholder text.
- An unconfigured or unreachable slot does **not** fall through — it is
  dead config (ticket 001) or a typed refusal.
- `effort` exists on the entry (`models.rs:167-181`) and is displayed
  (`model_guard.rs:338-341`) but dropped at the door for every
  `Resolved::Model` (`prepare.rs:800`, `:1050`, `:1063`); only the codex
  `native` branch emits it (`:898-899`).
- Fallback is single-step and loud by design — decisions `3ceba8f5` D2
  (explicit-only composite) and `267192c1` (herding `fallback: "default"`).

### Dependency matrix

One row per component, source mapped to local.

| Component | oh-my-pi | bee | Verdict |
|---|---|---|---|
| Role vocabulary | 10 built-in, **open** to custom (`model-roles.ts:77-91`) `Upstream` | 3+1, **closed**, hardcoded in 4 places (`models.rs:37`, `model_guard.rs:192-193`) `Local` | **CONFLICT** |
| Role axis | mixed; 6 of 10 name a job (`README.md:333`) `Docs` | cost only `Local` | **CONFLICT** |
| Job names | roles (`plan`, `designer`, `commit`) `Upstream` | agents (`bee-build`, `bee-gather`) welded to cost slots `Local` | **EXISTS**, coupled |
| Consumer selection | ordered role **list** per call site `Upstream` | one `--kind`, or the cell's `tier` `Local` | **NEW** |
| Unconfigured role | falls through to the next name `Upstream` | dead config or typed refusal `Local` | **NEW** — the cure for tickets 001 and 005 |
| Agent → model | frontmatter `model: "@role"`, itself a list (`task-agent-discovery.md:42`) `Docs` | pinned by tier, 1:1, not configurable `Local` | **CONFLICT** |
| Entry shape | bare string + grammar (`list`, `:effort`, `@alias`) `Upstream` | JSON object, 5 variants `Local` | **CONFLICT** (both rich, different) |
| Per-role effort | `:high` suffix, delivered `Upstream` | `{model, effort}` parsed, shown, **dropped** `Local` | **EXISTS**, broken |
| Fallback chain | ordered, per role **or** per model **or** `provider/*` `Docs` | single-step, explicit-only, loud `Local` | **CONFLICT** — ticket 003 |
| Fallback trigger | error-class gated; transient only `Upstream` | n/a (no chain) `Local` | **NEW** — ticket 003's open question, answered upstream |
| Capability match | named role + post-hoc filter `Upstream` | none `Local` | **NEW**, low value |

### Cross-cutting sweep

Wiring outside the role map that a bee adaptation would have to touch —
hunted explicitly, `Local`:

- `hooks/model_guard.rs` — `CLAUDE_TIERS` / `CODEX_TIERS` (`:192-193`),
  a second `resolve_tier` (`:442-467`), `PINNED_AGENT_TYPE` (`:605-629`),
  `dispatch_kind_for_tier` (`:660-666`), and the marker parser
  (`:195-224`). An open role set breaks every one of these: they are
  closed lists.
- `verbs/drivers/prepare.rs:34-40` — `slot_for_kind` ends in a
  catch-all `_ => "advisor"`. A new kind without its own arm silently
  resolves the advisor slot.
- Four private copies of `MODEL_TIERS` (`verbs/cells/validate.rs:29`,
  `verbs/state_group/mod.rs:166`, `verbs/status_full/mod.rs:60`,
  `hooks/session_preamble/mod.rs:106`) — no shared constant.
- `onboard/agents.rs:129-147` renders `{{TIER_MODEL}}` into each agent
  file from the slot map; an agent naming a *role* instead would change
  what onboarding writes.
- The session preamble publishes the resolved slots
  (decision `46827304` D2) — an open role set changes that block.
- `bee cells tier` and `bee state worker add --tier` both carry the
  closed 3-value enum (`handlers_close.rs:1146-1152`,
  `state_group/workers.rs:89`).

### Inference

- The reason oh-my-pi tolerates a mixed-axis role list is the
  fallthrough, not the naming. Purity of axis is not what makes roles
  configurable; **cheap failure of an unset role** is. This inverts the
  premise of discovery ticket 002, which assumed a role must earn its
  place by having a dispatch site. Upstream, a role earns its place by
  being *nameable*; a role nobody configures costs one array entry.
- bee's dead-slot defect (ticket 001), its unenforced reachability
  (ticket 005), and its never-passed kinds are all one defect: bee
  resolves a role to exactly one answer and refuses when that answer is
  absent. A fallthrough list removes the whole class.
- The owner's objection is correct about bee and *half* correct about
  the general case: `extraction`/`generation` are indeed cost words, and
  a user cannot judge "which model is good at generation". But the fix
  upstream is not to delete the cost words — it is to let job words
  exist beside them and let a consumer name both, in order.

## Risks, Unknowns, Follow-Ups

- **Open role set vs bee's guard.** bee's model-guard validates a
  `[bee-tier: …]` marker against a closed list. An open role set needs
  that check to become "is this role configured?" rather than "is this
  one of four words". This is the largest single piece of work and the
  main risk. `Local` `model_guard.rs:190-224`.
- **Fallthrough vs bee's loud-failure stance.** Decisions `3ceba8f5` D2,
  `267192c1`, and `4faf1de9` (an advisor consult recorded NOT OBTAINED
  rather than substituted) make loud failure a settled posture. A
  fallthrough on *resolution* (role unset) is compatible with it; a
  fallthrough on *runtime error* is ticket 003 and reopens that stance.
  Upstream separates exactly these two layers — that separation is the
  transferable idea, and it is what lets ticket 003 be answered without
  reversing the posture.
- **Effort delivery on the claude runtime.** Upstream delivers effort as
  a suffix because it owns its own model client. bee dispatches through
  the Agent tool, which takes no effort parameter. So a per-role effort
  in bee is deliverable on codex-native and on cli/herding transports,
  and **not** on claude Agent dispatch today. Unverified whether that is
  a harness limit or a bee gap — open question.
- Not researched: how oh-my-pi's `cycleOrder` interacts with roles, and
  the `modelProviderOrder` layer. Out of the narrowed scope.

## Source Pack

- Upstream repo: `~/Projects/refs/oh-my-pi` @ `2b66ee69`. Code read:
  `packages/coding-agent/src/config/model-roles.ts`,
  `config/model-resolver.ts`, `config/settings.ts`,
  `config/settings-schema.ts`, `session/retry-fallback-chains.ts`,
  `session/turn-recovery.ts`, `priority.json`, `commit/model-selection.ts`,
  `utils/title-generator.ts`, `auto-thinking/classifier.ts`,
  `session/unexpected-stop-classifier.ts`, `edit/auto-repair.ts`,
  `tools/inspect-image.ts`, `utils/image-vision-fallback.ts`,
  `task/structured-subagent.ts`, `modes/interactive-mode.ts`,
  `prompts/agents/{scout,librarian,reviewer,designer}.md`.
- Upstream docs read: `docs/models.md`, `docs/settings.md`,
  `docs/task-agent-discovery.md`, `docs/agent-hub.md`,
  `docs/local-models.md`, `docs/system-prompt-customization.md`,
  `docs/vibe-mode.md`, `docs/non-compaction-retry-policy.md`,
  `docs/tools/inspect_image.md`, `docs/cli-reference.md`,
  `docs/advisor-watchdog.md`, `README.md`, `AGENTS.md` (silent on roles).
- Local files: as cited under Findings → Local and Cross-cutting sweep.
- Local decisions cited: `8dad7c2e`, `a2f85972`, `de967733`, `3ff7cd72`,
  `3ceba8f5`, `267192c1`, `4faf1de9`, `46827304`, `0015`.
