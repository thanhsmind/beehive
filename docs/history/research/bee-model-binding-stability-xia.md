---
artifact_contract: bee-research/v1
topic: bee-model-binding-stability-xia
depth: standard
date: 2026-09-08
mode: xia
---

# Xia: why bee's role→model binding is unstable, and what oh-my-pi does differently

## Bottom Line

- **Recommendation (ladder rung): reuse (rung 1) for the three real defects, plus adapt-upstream (rung 3) for one small idea.** Nothing here needs a new mechanism. bee already adopted omp's role *split* in `model-role-split` (34 cells, 2026-08-25); what is broken is not the design.
- **The owner's report is correct and measurable.** On the `pi` runtime bee names **no model for any role** — four of four slots come back `requested_model: null`, `effective_model_status: "unverified"`. On `claude` only the roles configured with a plain model string are `pinned`. `Local`, `ran`.
- **Three concrete faults, in order of how much they explain:**
  1. **`models.pi` has drifted from its own settled decision.** Decision `a304e8a0` (owner, 2026-08-29) fixes pi's heavy roles on Claude Opus and advisor on Fable. Five of twelve live slots run something else, and `herding.agents` holds **no entry** that runs opus or fable — so the decision cannot even be expressed today. `Local`
  2. **`agy-flash` names no model at all** — its argv is `["agy", "--dangerously-skip-permissions"]`. `agy` picks its own session default (observed live: Gemini 3.8 Flash). Every role bound to that key silently follows whatever agy defaults to. `agy` **does** accept `--model` and `--effort`; the flag is simply absent. `Local`, `ran`
  3. **bee refuses to look at the argv it just built.** `derive_economics` hard-codes `"unverified"` and a null `requested_model` for `channel == "herding-exec"`, even though the payload builder holds the agent name one call earlier. The blindness is a branch, not a limitation. `Local`
- **Why omp's mapping feels "hard" — and it is NOT enforcement.** omp resolves a role to a **typed `Model` object from a registry of models it can actually reach**, in the same process that will run the turn. But omp verifies nothing afterwards, has no guard that refuses a mismatch, and **falls back to the parent session's model when the role's credentials are missing** — logging a warning, not refusing (`task/executor.ts:3186-3191`). What omp actually has over bee is **determinism plus visibility**: it logs requested-vs-resolved and paints the resolved model in the UI. `Upstream`
- **On the owner's "bơm prompt" idea: right instinct, wrong layer — and omp proves both halves.** As a *binding* it is the weakest thing available, and bee already has it: the `[bee-tier: <role>]` marker is prompt-injected role text on a path bee's own economics label `unverified`. But omp **does** inject the model into its system prompt — `<workstation>… - Model: {{model}}` (`prompts/system/project-prompt.md:3-6`), behind a `includeModelInPrompt` setting. It tells the agent what it **is**, never what to become. That is the second belt, and it is worth copying.
- **Confidence: 90%** on the three faults (each measured or read at an anchor); **75%** on the omp comparison (read directly, but omp is a large monorepo and I did not trace every spawn path).
- **Suggested next step: `bee-shaping`** — fixes 1 and 2 are config and need only the owner's confirmation of the values; fix 3 is a small feature and fix 4 is a real decision.

## Source Manifest

| Field | Value |
|---|---|
| Repo | `/home/thanhsmind/Projects/refs/oh-my-pi` |
| Ref | `main` |
| Resolved commit SHA | `3f8347bcd6d17bd985e1d97e4f07d2d765dde006` |
| Narrowed scope | `packages/coding-agent/src/session/role-models.ts`, `src/config/model-resolver.ts`, `src/prompts/agents/*.md` |
| Prior distill | `docs/history/research/oh-my-pi-model-roles-distill.md` (2026-08-24, SHA `2b66ee69`) — its recommendation was **adopted**; this brief does not repeat it |

Mode **`xia`** — port-protocol steps 1–4, no challenge pass, nothing built.

## Question & Assumptions

- **What was asked:** look at omp's hard model-mapping mechanism; bee's feels unstable and does not run correctly on pi; keep bee's workflow but add a prompt-injection mechanism so an agent picks a suitable model.
- **What success appears to mean:** naming a role reliably produces the intended model, on every runtime, and bee can say afterwards which model actually ran.
- **Assumptions still needing confirmation:**
  - That decision `a304e8a0`'s values are still what the owner wants for pi (they may have deliberately moved to the `pi-gpt-*` agents and not logged it).
  - That "bơm prompt" means the binding itself, rather than an extra advisory layer. The recommendation covers both readings.

## Findings

### Local — the measurement

Every runtime × role, straight from bee's own dispatch door (`ran`, this session):

| Runtime | Role | `requested_model` | `effective_model_status` | Payload shape |
|---|---|---|---|---|
| claude | review / advisor / plan | opus / fable / opus | **pinned** | Agent + `model` param |
| claude | code | `null` | unverified | Bash → herding pane |
| **pi** | code / review / advisor / plan | `null` (all four) | **unverified** (all four) | Bash → herding pane |
| codex | code / advisor / plan | `null` | *no economics record at all* | Agent |
| codex | review | `null` | unverified | Bash → cli-exec |

The binding is enforceable in exactly one place: a claude Agent dispatch carrying a real `model` parameter.

### Local — fault 1, the drift

Decision `a304e8a0` (2026-08-29, source `user`), verbatim: *"models.pi role table values are settled: heavy roles (code, test, docs, review) stay on Claude Opus and advisor stays on Fable — on the pi runtime these resolve as kind:herding slots whose herding.agents entries run the claude CLI (claude --model opus / claude --model fable, claude-sonnet-entry shape); cheap roles (read, extraction, generation, supervisor) default to the agy-flash herding agent."*

Live `.bee/config.json`:

| Slot | Decision says | Config runs | |
|---|---|---|---|
| pi.code | claude opus | `agy-flash` | ✗ |
| pi.test | claude opus | `agy-flash` | ✗ |
| pi.docs | claude opus | `agy-flash` | ✗ |
| pi.review | claude opus | `pi-gpt-5.6-luna` | ✗ |
| pi.advisor | claude fable | `pi-gpt-6-astra` | ✗ |
| pi.read / extraction / generation / supervisor | `agy-flash` | `agy-flash` | ✓ |

And `herding.agents` holds five entries, none of which runs opus or fable — the closest is `claude-sonnet` (`claude --model sonnet`). The decision's own shape ("claude-sonnet-entry shape") is available; the entries were never added.

Decision `4a6e38be` (same day) adds a second requirement the live config also misses: on pi every slot must point at *"a herding.agents entry **whose argv carries the pi model and thinking level**"*. `agy-flash`'s argv carries neither.

### Local — fault 2, the agent that names no model

```
"agy-flash": { "argv": ["agy", "--dangerously-skip-permissions"], ... }
```

`agy --help` lists `--model` ("Model for the current CLI session") and `--effort` (`low|medium|high`), and `agy models` returns concrete ids — `gemini-3.8-flash-high`, `gemini-3.1-pro-high`, `claude-opus-4-6-thinking`, and eleven more (`ran`). None is named in the argv, so the process chooses. A worker pane read this session showed **Gemini 3.8 Flash · high** — a value bee never asked for and cannot report.

This key backs `code`, `read`, `test`, `docs`, `extraction`, `generation`, `supervisor` and `lane-3` on **both** claude and pi. It is the single largest source of the instability, and it is one missing flag.

### Local — fault 3, the branch that refuses to look

`packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:374-391`:

```rust
} else if channel == "cli-exec" || channel == "herding-exec" {
    "unverified"
```
```rust
let requested_model = if channel == "cli-exec" || channel == "herding-exec" || channel == "session-model" {
```

Both branches are unconditional. Yet `prepare.rs:1805` appends `--agent "<name>"` to the very command being built, and `wave.rs:402-417` already resolves that name to its full argv. bee holds the argv, and often the argv contains `--model X`. Nothing reads it. `unverified` here does not mean *unverifiable* — it means *not looked at*.

### Upstream — what omp actually does

- **The agent file declares an intent, not a model.** `packages/coding-agent/src/prompts/agents/reviewer.md` frontmatter carries `model: "@slow"`; `scout.md` carries `model: "@smol"` plus `thinking-level: medium`. The `@name` is an alias.
- **The alias resolves through a role table, with loop protection.** `src/config/model-resolver.ts:1176-1195` resolves a cross-role alias (`modelRoles.default = "@slow"`) into concrete model patterns and refuses to recurse into a self-alias.
- **Resolution returns a typed model from a live registry.** `resolveRoleModelFull` (`src/session/role-models.ts:67`) resolves against `availableModels` — the models the process can actually reach — and hands back a `Model` object, consumed in-process at `session-maintenance.ts:2246` and `model-controls.ts:187`.
- **No prompt injection in the BINDING path.** The model is a value passed to the turn, never a sentence a model is asked to honour.
- **But omp does inject the model into the system prompt, as identity.** `prompts/system/project-prompt.md:3-6` renders a `<workstation>` block carrying `- Model: {{model}}`, populated from `getActiveModelString()` and gated by the `includeModelInPrompt` setting (`system-prompt.ts:1007`, `sdk.ts:3211-3212`). The agent is told what it *is*. Nothing asks it to become something else.
- **omp enforces nothing.** No verification that the model that answered is the model requested; no guard refusing a mismatch. When a role's model has no working credentials, omp **silently falls back to the parent session's model** and logs `"Subagent model has no working credentials; falling back to parent session model"` with requested vs resolved (`task/executor.ts:3179-3206`). The resolved model is also painted into the progress display (`:3221-3227`).

### Inference — the real difference

omp's binding is *tight* because **the thing that resolves the model is the thing that runs the turn** — and where it can still drift (auth fallback), it says so out loud. bee's herding transport puts a foreign process in that gap: bee writes a command line and hopes. That gap is a deliberate bee capability — it is what lets one role run `claude`, another `pi`, another `agy` — and it is not a defect to remove. But it means bee's binding can only ever be as strong as *what bee writes into the argv and then checks*. Today it writes an agent name and checks nothing.

This also explains why the owner's instinct reaches for prompt injection: on pi there is no model parameter to hold, and no rendered agent file either (pi has none — `docs/history/pi-support/parity-review.md:85-87`), so the prompt looks like the only surface left. It is not: **the argv is the surface**, and it is already there.

## Risks, Unknowns, Follow-Ups

- **Fix 1 contradicts nothing, but it needs the owner's word on which side is right** — the logged decision, or the live config that has run for ten days. One of the two is stale, and the decision log says superseding is the owner's move.
- **Fault 3's fix has a named limit.** Reading `--model` out of an argv proves what bee *asked for*, not what ran. It would earn a new status word (`declared`), never `pinned`. Claiming otherwise would repeat the false-confidence failure bee's own review doctrine warns about.
- **B14 is a fence with a reason.** `agent-model-unpin` deliberately removed `model:` from rendered claude agent files, making a present model line the drift. Any move toward "put the model in the agent file, like omp" contradicts it, and would need superseding — not quiet reversal.
- **I did not trace every omp spawn path.** The three anchors above are read; a fourth path could exist.

## Recommendation — ranked

1. ~~**Name a model in every `herding.agents` entry.**~~ **DONE, 2026-09-08.** `agy-flash` is now `agy --model gemini-3.8-flash-high --dangerously-skip-permissions`, pinned to the value a live worker pane was observed running — behaviour-preserving, and the silent-drift path is closed. The four other registry entries already named their models.
2. ~~**Reconcile `models.pi` with decision `a304e8a0`.**~~ **ANSWERED by the owner, 2026-09-08: keep the running config.** The logged opus/fable table is superseded; heavy pi roles stay on `agy-flash` and the `pi-gpt-*` agents, and the owner accepts flash- and gpt-class models on them. No conflict with `4a6e38be` — its parenthetical `pi -a --model …` is a shape example, and `a304e8a0` itself already licensed `agy-flash` on pi slots (B16: "herding constrains the transport, never the model vendor").
3. **Let bee read the model out of the argv it built.** Pass the resolved agent argv into `derive_economics`, scan for the model token, and report it as `requested_model` with a new honest status — `declared` — instead of `null`/`unverified`. Small, contained, and it makes `bee models show` and the session preamble tell the truth on pi for the first time.
4. **Then the prompt layer — as a second belt, never the binding. omp does exactly this, so it is adapt-upstream, not invention.** omp renders `- Model: {{model}}` into its system prompt's `<workstation>` block behind an `includeModelInPrompt` setting: the agent is told what it **is**, never what to become. Two honest homes in bee:
   - **Into the worker's brief:** one line naming the model and effort the role asked for, so a harness that *can* switch in-session has the instruction, and a human reading the pane can see a mismatch immediately.
   - **Into the orchestrator's preamble:** the role→model table, so the leader picks the right *role*. This already half-exists in the dispatch-door line.

   Neither can enforce anything, and the brief should say so where it is written. A prompt that says "you are Opus" run on Flash produces a Flash answer that believes it is Opus — which is worse than no line at all.

**Not recommended:** replacing the argv binding with prompt injection; putting `model:` back into rendered agent files without superseding `agent-model-unpin`; adding a second model table beside `models.<runtime>`.

## Source Pack

- **Local files read:** `.bee/config.json` (`models`, `herding.agents`); `packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:374-391`; `.../prepare.rs:1793-1805`; `.../herding/wave.rs:402-417`; `docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md` (B14, B16); `docs/history/research/oh-my-pi-model-roles-distill.md`.
- **Live command output (`ran`):** `bee dispatch prepare` across `{claude,pi,codex} × {code,review,advisor,plan}`; `agy --help`; `agy models`; `bee decisions search`.
- **Upstream read:** `oh-my-pi@3f8347bc` — `src/session/role-models.ts`, `src/config/model-resolver.ts:1176-1195`, `src/prompts/agents/reviewer.md`, `src/prompts/agents/scout.md`.
- **Decisions cited:** `a304e8a0`, `4a6e38be`, `6974e1e7`, `a6512134`, `agent-model-unpin` (B14), `pi-support D5/D6` (B16).
