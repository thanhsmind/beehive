# `.bee/config.json` — configuration reference

Every onboarded repo has a `.bee/config.json`. Any key you leave out uses a built-in default, so the file can be short — but the values below are the ones worth setting per repo. **`.bee/config.json` is strict JSON: no comments, no trailing commas.** The annotated block here is for reading; copy the clean block at the bottom into the real file.

## Setting values — hand-edit the JSON

> **`bee config get|set|unset|validate` are NOT built into the current binary.** They lived only in
> the Node runtime the R6 cutover deleted, and no Rust port replaced them, so each one now refuses by
> name (`bee --help --all` marks them, and the registry carries the reason). **Edit
> `.bee/config.json` directly** — it is plain, strict JSON. `bee status --json` reports the derived
> `gate_bypass_level` and `ship_visibility`, which is what most `config get` calls were really after.

Two rules the deleted CLI used to enforce for you, now yours to keep:

- **`hooks.*` and `guards.*` are local-only namespaces.** Put them in `.bee/config.local.json`
  (gitignored, per-machine), never in the tracked `.bee/config.json` — so one developer muting a
  hook never lands in everyone's config. The overlay wins over the tracked file at read time.
- **The models/cli-executor block has to stay valid.** `config set` refused a write that broke it;
  nothing refuses a hand-edit, so re-read [Which model each role uses](#which-model-each-role-uses)
  after changing `models`.

Values are ordinary JSON: `false` is a boolean, `12` a number, `"repo"` a string. Nested keys that
the old `--key guards.idle_gate` dot-notation reached are just nested objects in the file.

## Which model each role uses

bee picks a worker model by **the job the work is**, never by how expensive the model is. A cell declares a `role`; the dispatch asks for that role; `models.<runtime>` answers with a model. **`role` is a cell's sole model selector** — the old three-value cost enum (`extraction` / `generation` / `ceiling`) is retired as a selector (model-role-split D4, store `97ce5225`).

**The strongest model is still never configured.** It is always the model you run the session on (decision `0015`). Work that needs it is *escalated*, which is a flag and a budget, not a role — see [Escalation](#escalation--the-cost-lever-not-a-role) below.

`models` is keyed by runtime first (Claude Code, Codex, and OpenCode name models differently; Pi names no model at all — it names a herding agent, see [Pi](#pi--modelspi-is-herding-only)), then by role name:

```jsonc
{
  "models": {
    "claude": {
      "code": "sonnet",        // the work WRITES: implementation, wiring, tests
      "read": "haiku",         // the work only READS: retrieval, tracing a call path, mining
      "extraction": "haiku",   // historical tail — every ordered role list ends here
      "generation": "sonnet",  // historical tail — leave both set
      "review": "opus",        // OPTIONAL, not seeded; null → falls through to generation
      "advisor": "opus"        // OPTIONAL, not seeded; null/unset → no advisor
    },
    "codex": {
      "code": null,            // Codex has no per-agent model switch today →
      "read": null,            //   null means "enforce the role via read budget + output cap in the prompt"
      "extraction": null,
      "generation": null
    }
  }
}
```

### What a fresh config seeds

`bee onboard` writes exactly **four** role keys per runtime: **`code` · `read` · `extraction` · `generation`**.

- **`code`** and **`read`** are the names bee's own dispatch sites ask for.
- **`extraction`** and **`generation`** are the **historical tail**: every ordered role list ends in one of them, so upgrading bee moves no existing host's model. Leave them set.
- **`review` and `advisor` are deliberately NOT seeded.** Both already resolve with no config key at all — an unset `review` falls through to `generation`'s model, and an unset `advisor` reads as "no advisor" (decision `4faf1de9`; the advisor has no fall-through of its own). Publishing a default would silently decide that product question for every new host. Copy either key in only when you mean to override that behavior.
- **`ceiling` is not a key and is not a role name.** It never was configurable and it still is not; it is the escalation flag's wire word, nothing a config carries.

To change the worker models, edit `models.claude.code` / `read` (or add your own role key). To change the escalated model, run the session on a different model — there is no config for it.

### Any non-empty role name is legal

bee holds **no fixed list of roles.** Validation checks a role's *presence and shape*, never its membership (D2/D7, store `06e49368` / `4eaf1b71`):

- **`role` is required on a cell**, exactly as `lane` is — `bee cells add` refuses a cell without it.
- **Any non-empty name passes.** Add `"test": "opus"` or `"migrate": "sonnet"` under `models.claude`, declare that role on a cell, and the cell gets that model. A new job role needs no new bee code.
- `code` · `read` · `test` · `docs` · `review` · `design` are the **recommended vocabulary** (D8, store `4eaf1b71`) — authoring guidance, carried on the planning surface and in `bee cells add --help`, and printed in the missing-`role` refusal so an author has somewhere to start. Nothing ever matches against it. It is guidance, never an enum.

### An unconfigured role falls through — it never fails

A dispatch does not ask for one name. It asks for an **ordered list**, headed by the cell's own role and ending in a name every host has configured for years:

| The cell's role | The list the dispatch asks for |
|---|---|
| `read` | `[read, extraction, generation]` |
| anything else | `[<the cell's role>, code, generation]` |

The walk takes the **first name that resolves**; an unset or unresolvable name yields to the next. The last entry always resolves, so a walk can never dead-end. That tail is why a bee upgrade moves no host's model: a config that names only `extraction` and `generation` keeps answering every dispatch exactly as it did before roles existed.

Two rules keep the fall-through honest:

- **A name nothing configures WARNS on stderr**, naming what it fell through to — e.g. `bee: model role "tset" is not configured in models.claude of .bee/config.json — falling through to "code"`. It is never silently accepted, and it never hands back another role's model as if the config had named it.
- **One silent case, and it closes itself — the pre-roles window.** `code` and `read`, the two names bee's own ordered lists ask for, do not warn on a runtime whose `models.<runtime>` configures **NEITHER** of them. That is a host from before roles shipped: falling through to `generation` is the intended no-op, there is no better-fitting model it could have picked, and a warning that fires on every single dispatch is one nobody reads. Configure **either** key — `models.claude.code` is enough — and the window shuts for that runtime: from then on the sibling you left out warns like any other name, so a half-migrated config is loud about what it missed instead of silent about it forever. The window is per runtime, because the table is. An operator-invented name like `test` or `desgin` warns loudly from the start, migrated or not.
- **A present-but-`null` slot is a slot you turned OFF**, not an absent one. It yields without a word, and the built-in default is never consulted for it — answering a cleared slot with a built-in would resurrect the very model you just cleared.

One path deliberately does **not** fall through: `bee dispatch prepare --role <name>` names the slot **outright** — the kind's default slot is not consulted, and neither is a cell's own recorded value. A name that runtime cannot resolve is a typed refusal (`role_not_configured`) whose FIX lists the roles it can, because an operator who typed a flag made a typo, not a policy choice.

### Escalation — the cost lever, not a role

Cost is a separate axis from job. Work that must run on your strongest model — integration, architecture, a security call, an ambiguous spec — is **escalated** (D5, store `97ce5225`):

```bash
bee cells escalate --id <cell-id>                       # run on the session model
bee cells escalate --id <cell-id> --reason "<why>"      # ...past the ration
bee cells escalate --id <cell-id> --off                 # back to the cell's own role model
```

- It sets a boolean flag on the cell. The dispatch reads that flag and runs the cell on the **session model** with no `model` parameter at all.
- **The 40% ration**: escalating when it would put MORE than 40% of the feature's cells on the session model refuses — exactly 40% passes — unless `--reason` names why. The reason is recorded on the cell trace as `escalation_reason`.
- `--off` clears the flag and is never budget-checked.
- `bee status` reports the `role_mix` — which is what `tier_mix` became — with that escalated share. The point is unchanged: keep the strongest model scarce.

### What a slot's value may be

Every role slot — a seeded one, `review`, `advisor`, or a name you invented — takes the same value shapes.

- **The shapes** each slot accepts (decisions 0019/0021; native override D2, codex-native-transport):

  | shape | means |
  |---|---|
  | `"sonnet"` | the runtime's per-agent model switch |
  | `{ "model": "sonnet", "effort": "medium" }` | model + reasoning effort (`low` · `medium` · `high` · `xhigh` · `max`); the effort is applied where the runtime has a per-agent effort switch, recorded and ignored where it does not |
  | `{ "kind": "cli", "command": "codex exec -m … -s read-only -", "promptVia": "stdin" }` | an **external executor** — a separate CLI process dispatched under the same worker contract (effort rides inside the command); `promptVia` declares how the prompt reaches it, never sniffed from the command string |
  | `{ "kind": "native", "model": "gpt-5.5", "effort": "high", "fork_turns": "none", "agent_type": "worker" }` | a **native V2 model override** (codex runtime) — a stronger model applied per-agent on the codex `spawn_agent` metadata, no separate process. `model` is the exact catalog model id. `fork_turns` must be `"none"` (a full-history fork rejects overrides) and defaults to `"none"`; `agent_type` defaults to `"worker"`. `effort` is optional. The route is inert until a capability probe confirms the host build accepts it (D3) |
  | `{ "kind": "herding" }` (optional `"agent": "<name>"`) | routes **cell execution** for this slot through `bee herding run` automatically — no per-cell request, no other fields required. A gather/review/advisor purpose against the same slot is unaffected: it keeps that runtime's own default model for the slot, never `herding`. The agent that runs is the single global `herding.agent_command` by default; the optional `agent` field names an entry in `herding.agents` instead (herd-registry D1/D2), overriding the global default for this slot alone — an unknown name refuses, listing every registry key. Full `herding.*` contract, the `herding.agents` map shape, and all three reference spellings (tier slot, `bee herding run --agent`, string `herding.agent_command`): [bee-herding/references/operational-invariants.md](../skills/bee-herding/references/operational-invariants.md) |
  | `{ "primary": { "kind": "native", "model": "gpt-5.5" }, "fallback": { "kind": "cli", "command": "codex exec … -s read-only -", "promptVia": "stdin" }, "fallback_policy": "explicit-only" }` | a **native primary with an opt-in cli fallback**. The fallback is taken **only** when `fallback_policy` is exactly `"explicit-only"`; without that string the fallback is dropped and never used — silent native→cli fallback is forbidden (D1) |
  | `null` | no per-agent switch: the role is enforced via read budget + output cap in the prompt (for `review`: fall back to generation; for `advisor`: no advisor) |

  Invalid shapes are ignored — the slot's default stands, nothing throws. A native override missing its `model`, a `fork_turns` other than `"none"`, or a composite missing `fallback_policy` is flagged by config validation (`bee status`), never silently trusted.
- **What the short names mean (important).** For Claude Code these are **family aliases**, not exact version strings. The value must be one of exactly `haiku` · `sonnet` · `opus` · `fable` — the Claude Code Agent tool accepts only these four. Each alias is resolved **by Claude Code (not by bee)** to the current model of that family on your account. So `"sonnet"` isn't "some random Sonnet" — it means "the Sonnet tier", and the harness uses the latest. Today they resolve to:

  | alias | resolves to (current) | model id |
  |---|---|---|
  | `haiku` | Haiku 4.5 | `claude-haiku-4-5` |
  | `sonnet` | Sonnet 5 | `claude-sonnet-5` |
  | `opus` | Opus 4.8 | `claude-opus-4-8` |
  | `fable` | Fable 5 | `claude-fable-5` |

  You **cannot pin an exact sub-version** for a Claude Code subagent — the model param is family-alias only, and it tracks the latest of each family as Anthropic ships new ones. (For **Codex**, the `codex` roles take the runtime's real model ids, e.g. `"gpt-5"`, because that runtime addresses models by id.)
- `bee status` prints the active map — every role the runtime configures, bee's own dispatch roles first and the rest in config order, e.g. `Models (claude): generation=… review=… extraction=… test=…` — plus the `role_mix` and its escalated share, and warns when too many cells run escalated — the cost lever erodes when the strongest model touches most dispatches.

### Runtimes: Claude Code, Codex, OpenCode, and Pi — and everything else (agy, …)

`models` accepts **four runtime keys: `claude`, `codex`, `opencode`, and `pi`** — the runtimes bee ships hooks or a guard belt, rendered skills, and a dispatch door for (opencode-support D1; pi-support D5 added `pi`). Any other top-level runtime key (e.g. `"gemini"`) is still **silently ignored**: not an error, just dead config that never resolves.

- **OpenCode** names models as `provider/model` ids (e.g. `"opencode/big-pickle"` on the zero-config `opencode/*` free provider this machine ships out of the box) — a real catalog id, the same way Codex takes its real model ids, never a Claude-style family alias. There is no per-call model override on OpenCode's dispatch (`task`) tool, so `models.opencode.{code,read,extraction,generation,review}` is consumed **structurally**: each `.opencode/agent/bee-{build,gather,extract,review}.md` worker file pins its role's model directly in that file's own `model:` frontmatter (today hand-authored to match this key, not yet rendered by a `bee dev` generator the way `.claude/agents/*.md` is), so a wrong-role dispatch is unrepresentable rather than caught after the fact. Example:

  ```jsonc
  {
    "models": {
      "opencode": {
        "code": "opencode/big-pickle",                 // bee-build
        "read": "opencode/ling-3.0-tiny-free",         // bee-gather
        "extraction": "opencode/ling-3.0-tiny-free",   // bee-extract (historical tail)
        "generation": "opencode/big-pickle",           // historical tail
        "review": "opencode/nemotron-3-ultra-free"     // bee-review
      }
    }
  }
  ```

  Named constraint: the free `opencode/*` provider carries no documented quality tiering, so the mapping above is a size-name heuristic ("tiny" → the read roles, "ultra" → review), not an empirically verified capability ladder — swap in real provider/model ids once a paid provider is configured.

That does *not* mean other CLIs are unusable — they plug in through the **external-executor slot shape** on whichever runtime you actually run the session in. Example, routing the review slot of a Claude Code session through OpenCode:

```json
{
  "models": {
    "claude": {
      "code": { "model": "sonnet", "effort": "medium" },
      "read": "haiku",
      "extraction": "haiku",
      "generation": "sonnet",
      "review": {
        "kind": "cli",
        "command": "bash -lc 'opencode run --model anthropic/claude-opus \"$(cat)\"'",
        "promptVia": "stdin"
      }
    }
  }
}
```

Two rules travel with every cli-shaped slot: it is **gather/review/advisor-only** — cell *execution* against a cli slot is refused (`cli_tier_gather_only`), so implementation work never rides an executor bee cannot supervise — and `promptVia` must state how the prompt reaches the process (`"stdin"`, or the `"$(cat)"` wrapper for CLIs that only take argv), never guessed from the command string. A ready-to-run demo with **agy** (generation) and **opencode** (review) lives at [`.bee/config-sample-cli-executors.json`](../.bee/config-sample-cli-executors.json); per-flag reasoning and more presets: [`docs/model-presets.md`](model-presets.md).

### Pi — `models.pi` is herding-only

> **Results come back — know which path you are on** (`pi-result-mailbox` D1/D2/D6, the feature pi-support D7 split out). The **synchronous** path is the primary contract and it works on every runtime: `bee herding run` blocks, then prints the validated result envelope, which carries **`report_path`** (a path, never the report body) when the worker wrote a report, and `report_note` when a report was expected but is missing or stale. Both are **additive keys** — a result with no report keeps the exact envelope it always had.
>
> On Pi you may **additionally** opt a job nothing is waiting on into async delivery: `bee herding run --inbox-session <session-token>` is the detached fact, it writes a pending marker under `.bee/result-inbox/<token>/` before the pane spawns, and `.pi/extensions/bee-guard.ts` injects that job's finished **header** (`job_id`, `cell_id`, `status`, `summary`, `proof`, `report_path`) into the session — steered when busy, a fresh turn when idle. Its limits are real: delivery is **at-least-once**, so the injected `job_id` is the **dedupe key** and a `job_id` you already handled is a replay, not a second result; the drain only runs while that Pi session is **live** (a job finishing with no session up waits in its marker); and the report body never rides the injection — read `report_path` yourself.

`pi` is a legal runtime at both dispatch doors — `bee dispatch prepare --runtime pi` and `bee dispatch wave --runtime pi` — and resolves `models.pi` in the **same one config home** every other runtime reads (pi-support D5). Pi's guard belt is not configured here at all: it is the checked-in extension `.pi/extensions/bee-guard.ts`, which `bee onboard` copies into the host repo.

**The law: every slot resolves herding, or the door refuses by name.** Pi ships **no Agent/subagent tool surface** (store `7f9c8518`), so an Agent payload, a `spawn_agent` payload, a bare `model` parameter, or a cli command emitted for `pi` would dispatch **nothing** while the envelope read as a successful dispatch. So on `pi` every slot must be `{ "kind": "herding" }` (optionally `"agent": "<herding.agents name>"`), and every other resolution is a typed refusal carrying the one reason word **`pi_requires_herding`**, plus the `slot`, the `resolution` word, and a `fix`:

| what `models.pi.<slot>` resolved | `resolution` | what happens |
|---|---|---|
| `{ "kind": "herding", … }` | — | the herding-exec payload — the one transport Pi can take |
| `"opus"` (plain string) | `model` | refused: set the slot to `{"kind":"herding","agent":"<name>"}` |
| `{ "kind": "native", … }` | `native` | refused, same fix |
| `{ "kind": "cli", … }` | `cli` | refused, same fix |
| `null` / unset-and-nothing-falls-through | `budget` | refused, same fix — a prompt-budget slot needs a subagent to inherit the session model, and there is none |
| an escalated cell, or `--role ceiling` | `escalation` | refused with its **own** remedy: *Pi has no subagent surface — run the escalated cell inline in the session* |

The refusals fire at both doors (`prepare` and `wave`), and **seat roles ride along** under the same law: a `lane-*` or `hat-*` slot must be herding too, and an unconfigured seat falls through to `advisor` — so keep `advisor` herding as well.

**The values (the settled table, pi-support D6).** Herding constrains the **transport**, not the model vendor, so the agents named here are ordinary `herding.agents` entries:

```jsonc
{
  "models": {
    "pi": {
      "code":       { "kind": "herding", "agent": "claude-opus" },   // heavy roles:
      "test":       { "kind": "herding", "agent": "claude-opus" },   //   claude --model opus
      "docs":       { "kind": "herding", "agent": "claude-opus" },
      "review":     { "kind": "herding", "agent": "claude-opus" },
      "advisor":    { "kind": "herding", "agent": "claude-fable" },  // claude --model fable
      "read":       { "kind": "herding", "agent": "agy-flash" },     // cheap roles: agy-flash
      "extraction": { "kind": "herding", "agent": "agy-flash" },
      "generation": { "kind": "herding", "agent": "agy-flash" },
      "supervisor": { "kind": "herding", "agent": "agy-flash" }
    }
  },
  "herding": {
    "agents": {
      "claude-opus":  ["claude", "--model", "opus",  "--permission-mode", "bypassPermissions"],
      "claude-fable": ["claude", "--model", "fable", "--permission-mode", "bypassPermissions"],
      "agy-flash":    { "argv": ["agy", "--dangerously-skip-permissions"] }
    }
  }
}
```

An `agent` name that is not in `herding.agents` refuses and lists every registry key — the same rule every herding slot follows. The full block, annotated, is in [`.bee/config-sample.json`](../.bee/config-sample.json); the `herding.*` contract is [bee-herding/references/operational-invariants.md](../skills/bee-herding/references/operational-invariants.md). Every dispatch this table serves returns its result through `bee herding run`'s own output — that is the path to plan on; add `--inbox-session <token>` only for a job you have detached, and take the async limits with it: at-least-once delivery with `job_id` as the dedupe key, and a drain that needs a live Pi session.

#### Pi guard belt (`.pi/extensions/bee-guard.ts`)

Pi enforces bee rules through the extension [`.pi/extensions/bee-guard.ts`](../.pi/extensions/bee-guard.ts), which `bee onboard` copies into the project. The extension uses two failure policies:
- **Blocking (fail closed):** `write-guard` on `tool_call`. If the check fails, the binary is missing, or the hook returns exit code 2 (`DENY`), `ask`, or invalid JSON, tool execution is blocked.
- **Advisory (fail open):** All other lifecycle events. Errors, crashes, or missing binaries are logged to stderr and swallowed so the session continues.

##### Wired rules

| Rule | Pi Event | Policy | Description |
|---|---|---|---|
| `write-guard` | `tool_call` | Blocking | Validates tool executions (`bash`, `powershell`, `write`, `edit`, `read`, `grep`, `find`, `ls`, and custom tools). |
| `session-init` | `session_start` | Advisory | Runs once per real session boundary — a fresh start, a new session, a resume, or a fork — and caches the session preamble. A `/reload` does not run it again: the session keeps the preamble it already has. It reports a new session as a clear, a resume or a fork as a resume, and anything else as a startup. |
| `prompt-context` | `before_agent_start` | Advisory | Generates the per-turn context delta appended to the system prompt. |
| `activity` | `before_agent_start` (`UserPromptSubmit`), `tool_result` (`PostToolUse` / `PostToolUseFailure`), `agent_settled` (`Stop`), `session_shutdown` (`SessionEnd`) | Advisory | Records session state transitions across prompt submission, tool execution results, turn completion, and session shutdown. |
| `state-sync` | `tool_result` (`PostToolUse`), `agent_settled` (`Stop`) | Advisory | Synchronizes session state after tool execution and on turn completion. |
| `tools-logger` | `tool_result` (`PostToolUse`) | Advisory | Appends one line per tool call to the tools log, carrying the timestamp and the tool name. Tool arguments and results are never logged. The agent-identity fields the rule can carry on other runtimes stay empty on Pi, because Pi's tool result does not carry them. |
| `session-close` | `agent_settled` (`Stop`), `session_before_compact` (`PreCompact`), `session_shutdown` (`SessionEnd`) | Advisory | Manages turn-end marks and continuation nudges on settle; the `PreCompact` arm returns undecidable (fail-open) today so the belt is ready when native `PreCompact` becomes real; and marks the session record closed on shutdown for every reason except `reload`, which keeps the same session running. A shutdown that carries no reason at all also closes the record. |

The extension also runs an advisory result inbox drain on `session_start` to poll for detached `bee herding run` background job results under `.bee/result-inbox/<token>/` and inject them into the session via `pi.sendUserMessage`.

##### Excluded rules

The following rules from the bee catalog are not wired on Pi:

- **`model-guard`:** Pi has no built-in subagent tool surface (`Agent` or `Task` tools). Worker dispatches run via external herding commands (`bee herding run`) through `bash`, which `write-guard` intercepts.
- **`chain-nudge`:** Requires in-process subagent lifecycle events (such as `SubagentStop`) that Pi does not provide.
- **`codex-subagent-audit`:** Codex-specific audit hook for OpenAI subagent lifecycle events (`SubagentStart` / `SubagentStop`); not applicable to Pi.

##### Rules on the Claude belt without a Pi carrier

The Claude hook manifest (`packages/bee/hooks/claude-hooks.json`) fires some rules on lifecycle events that have no equivalent in Pi:

- **`activity` on `PreToolUse`:** Pi's `tool_call` event is strictly the fail-closed blocking path; Pi has no separate advisory pre-tool event.
- **`activity` on `PermissionRequest`:** Pi 0.84.x provides no interactive permission request event.
- **`activity` on `Notification`:** Pi 0.84.x provides no notification event.
- **`state-sync` on `SubagentStop`:** Pi has no `SubagentStop` event (state synchronization runs on `tool_result` and `agent_settled`).


## `retry.fallbackChains` — a chain bee PUBLISHES, never one it runs

A fallback chain is an ordered list of model selectors a dispatch **may** move along after a *transient* provider failure (D10/D11, store `50808d48`).

**Read this part first, because it is the part that is easy to get wrong.** bee never executes a dispatch. `bee dispatch prepare` builds a payload and returns; the orchestrator or the worker runs it. So bee cannot see the quota wall, the 5xx, or the stream stall a chain step answers — and **none of this is a retry loop bee runs** (decision `51341f84`). What bee owns is the **contract**: it parses the config, resolves the chain that applies to this dispatch, and publishes it — with the gate saying when a step is earned — beside the model on the payload. **Advancing a step, and recording the step taken, belong to whoever actually executes the dispatch.**

**Explicit-only.** There is no built-in chain for any role and no role inherits one. With no `retry` key configured, every dispatch payload is byte-identical to a bee that had never heard of chains, and a failure stays exactly as loud as it is today. A chain exists only because you typed it.

```jsonc
{
  "retry": {
    "fallbackChains": {
      "code": ["sonnet", "haiku"],        // keyed by ROLE
      "opus": ["sonnet"],                 // keyed by a concrete MODEL — follows that model into any role
      "anthropic/*": ["local/qwen"]       // keyed by a provider WILDCARD
    }
  }
}
```

- **Which chain applies**, most specific key first: the concrete model selector this dispatch carries → the `provider/*` wildcard it falls under → the role the dispatch travels under. A model-keyed chain outranks a role-keyed one because it is keyed on the thing that actually failed.
- **A `default` key is refused out loud.** No role inherits a chain; keying one by role, by concrete model, or by `provider/*` is the whole of it. (Inheriting a default would change the advisor behavior decision `4faf1de9` settled on live evidence, without the owner asking for it.)
- **Junk drops loudly, never silently.** A chain that is not an array, one that names no usable step, a step repeating the key's own model, or a `default` key each produces a `bee: retry.fallbackChains["<key>"] … is ignored — <why>` line on stderr. A mistyped chain is not a slot somebody turned off; it is a safety net the operator believes is under them.

**The error gate travels with the chain**, on the payload, so no caller re-derives it:

| Field | What it carries |
|---|---|
| `advance_on` | `quota_or_rate_limit` · `provider_auth_or_policy_rejection` · `empty_response` · `malformed_tool_call_replay_safe` · `stream_stall_or_connection_reset` · `server_error_5xx` |
| `never_advance_on` | `tool_error` · `wrong_or_unwanted_result` · `failed_proof` · `red_test` |
| `fallback_when` | the condition in one line |

Every class in `advance_on` is transient or infrastructural: the failure happened **before** the model got to be wrong. Every class in `never_advance_on` is **semantic** — the model was reached, answered, and answered badly. Falling to another model there would hide the defect, which is the one thing bee's loud posture exists to refuse. **No result failure is ever absorbed by a chain.** The negative list is published in full rather than left as "everything not in `advance_on`", because the negative is the half a caller gets wrong.

## `commands` — the host project's lifecycle commands

Captured at onboarding (or the first natural moment in exploring), three standard keys — all plain runnable shell commands, never descriptions:

| Key | Meaning | Who runs it, when |
|---|---|---|
| `setup` | install dependencies from scratch | onboarding checks, fresh-clone bootstrap |
| `start` | run the app/dev server | on demand (`/run`-style checks) |
| `test` | **the project's ONE declared test command** | the same command each time, run at the boundary: the green base check before the first claim, `bee close` for the feature when it has no worktree, the `bee worktree merge` semantic gate when it has one (run against the staged merge), and CI on the host's own cadence. `bee finish` is commit-only proof and records `tests: boundary` — it does not run `test` |

**`commands.verify` is retired.** It used to sit above `test` as a second, full-suite, CI-owned command. Two repo-wide commands meant every surface had to say which door ran which — and they disagreed: this reference called `verify` "never a local obligation" while the green base check told agents to run it locally before their first claim. One command ends the question. A host that wants a slower full sweep runs it in CI on its own schedule; bee needs no config key to know about it.

Below `commands.test` there is a second, narrower layer that is **not** config: each work cell's own `verify` field, authored per change (one test file / one test function, seconds). Config carries the one repo-wide command; the cell carries the per-change one.

### Projects without tests

A project that deliberately runs no tests declares that in config instead of leaving the key absent: set `commands.test` to the exact sentinel string `"none"` (no-test-repos D1, decision `55b951e1`). Absence keeps its existing meaning — not-captured-yet, the normal onboarding nag — the sentinel means "this repo will never have one." With the sentinel set: the session preamble skips the CI-status-gate paragraph and prints one loud `Test gates disabled by repo declaration` line instead; cells may carry `verify: "none"` (refused everywhere else, exactly like a prose description would be) and cap on that cell records the diff-backed outcome with an auto waiver note rather than a passing verify result; wave-close, session-finish, and worktree-merge all skip with the same loud line, never silently. Nothing here is permanent — re-enable at any time by recording a real `test` command, which restores every gate above on the next session.

### Per-language recipes

`commands.test` runs at the boundary (`bee close`/`bee worktree merge`), so pick something you are willing to pay for that often. A changed-only mode is ideal where the runner has one; a whole-suite command is fine when the suite is fast.

| Language | `commands.test` |
|---|---|
| **Node** | `npx jest --onlyChanged` (jest) · `npx vitest related --run <files>` (vitest) |
| **Go** | `go test ./internal/<changed-pkg>/...` — derive the package set from the diff (`go list ./... \| grep …`, or reverse-deps via `go list -deps`) |
| **Rust** | `cargo test -p <changed-crate>` (workspace: one crate) · `cargo test <module>::` (one module path) |
| **Python** | `pytest tests/test_<area>.py` (by path) · `pytest -k <expr>` (by name) · `pytest --testmon` (coverage-map impacted, needs pytest-testmon) |
| **PHP** | `vendor/bin/phpunit --filter <TestClass>` · `vendor/bin/phpunit tests/<Area>/` (by dir) · Laravel: `php artisan test --filter <name>` |
| **No tests** | `"none"` (sentinel — declares the repo deliberately test-free) |

bee's own repo is a Rust project since the port (plans/rust-port.md): the key is
`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`.
The `PATH` prefix is deliberate and portable — rustup installs to `$CARGO_HOME`/`~/.cargo/bin`, and an
agent session started before rustup (or before a PATH change) otherwise fails the cap door with
`cargo: command not found` rather than a real red — a failure mode worth knowing, because it reads
as tooling noise and is in fact the door never having run at all.

Notes:
- A command that takes the changed-file list from git itself (jest `--onlyChanged`, testmon) is the best `test` value — it stays correct with zero per-change editing. Where the runner has no such mode (Go, Rust, PHP), record the *narrow invocation shape* and let the session substitute the changed package/crate/class per change — the doctrine cares that the door stays cheap enough to run at the boundary, not which selector you use.
- CI should run `commands.test` verbatim (bee's own `ci.yml` does exactly that with `cargo test --release`, and files a deduped `verify-red` issue on red).
- Where the "which tests relate to this file" answer needs a lookup, use the language's native graph (Go: `go list -deps` reversed; Rust: the crate graph; Python: testmon's coverage map). bee's own repo used to ship a derived impact registry for this; it was retired at the R6 Node cutover, because its subject was the `.mjs` suite graph and the Rust suite that replaced it runs whole in ~20s.

## Removed keys

**`commands.verify`** was retired in **2.1.0**. `commands.test` is now the one declared test command and every door runs it. If your `.bee/config.json` still has a `verify`, onboarding warns and it is ignored — delete it. Two migrations matter:

- **You recorded both.** Nothing to do beyond deleting `verify` — `test` already governed the dev loop, and it now governs merge and CI too. If your `verify` was materially broader, decide whether that breadth belongs in `test` (paid at the boundary) or in your CI workflow (paid on push).
- **You recorded ONLY `verify`.** You currently have **no test gate at all** — onboarding says so loudly. Move the command to `commands.test`, or set `commands.test` to `"none"` if the repo is deliberately test-free. `"none"` on `verify` no longer declares a no-test repo.

The **top-level** `advisor` key (old "advisor mode") was removed in v0.1.23 (decision fanout-delegation D1). If your `.bee/config.json` still has one, onboarding warns about the stale key and ignores it — delete it. This is **not** the same thing as the `models.<runtime>.advisor` slot above, which is current and valid.

## Other keys

| Key | What it does | Default |
|---|---|---|
| `models` | runtime-keyed role→model map — the job a piece of work is picks the model that runs it; full section above | `code` · `read` · `extraction` · `generation` seeded per runtime at onboarding |
| `retry.fallbackChains` | the ordered model chain bee **publishes** on a dispatch for its executor to follow after a *transient* provider failure — explicit-only, no built-in chain, never a retry loop bee runs; full section above | unset — no chain, every payload unchanged |
| `commands` | the host project's `setup` / `start` / `test` commands — full section above | none — captured at onboarding |
| `gate_bypass` | opt-in autopilot with levels `false` · `"normal"` · `"full"` · `"total"` (legacy `true` = normal); set via `bee-hive`'s "Gates" section (gate-bypass levels) | `false` |
| `hooks` | per-hook kill switch — nine hooks: `session-init`, `prompt-context`, `write-guard`, `model-guard`, `state-sync`, `chain-nudge`, `session-close`, `tools-logger`, `codex-subagent-audit` | all `true` (an absent key also reads `true`) |
| `guards` | `idle_gate` (`false` disables the idle intake gate) · `max_read_lines` (line cap a single inbound file read may pull before the read guard trims it; number > 0) · `memory_root` (one absolute path the write guard will let the agent write — see below) | idle gate on · `800` · no memory root |
| `cells_archive_on_close` | whether a green `bee close` retires the feature's cells into `.bee/cells/archive/<feature>/`, out of the scan path `status`/`orient` parse on every call. Only fires when every one of the feature's cells is capped or dropped; reverse with `bee cells unarchive --feature <f>`. Set `false` for a repo whose own tooling reads `.bee/cells/*.json` by path | `true` |
| `ship_visibility` | how finished work is surfaced — `"off"` or `"draft-pr"`. An unrecognized value normalizes to `"off"` and says so once, by name | `"off"` |
| `worktree_first` | code-touching feature work lives in its own worktree and the write guard refuses feature edits made in the main checkout; the exact string `"off"` disables that refusal — see [specs/worktree-first.md](specs/worktree-first.md) | on |
| `worktree_cleanup_on_merge` | `bee worktree merge` removes the worktree directory, deletes its branch, and drops its grant by default on a merge that merged something (per-merge opt-out: `--no-cleanup`). Set `false` to opt the whole repo out; a non-boolean value is refused rather than read as either outcome | `true` |
| `uat_stop` | where the `uat` acceptance stop sits — `"merge"` blocks `bee worktree merge` until the gate is approved; `"close"` (the default, i.e. absent) lets the merge land so the product is testable on main, then blocks `bee close` instead. Under `"close"`, exactly what changes: the merge lands the code so the product can be reloaded and tested, the worktree is HELD while `uat` is pending (`--cleanup` and `worktree_cleanup_on_merge: true` are both ignored — the merge result reports `WORKTREE_MERGE_CLEANUP_SUPPRESSED_UAT_PENDING` instead of tearing it down, because a failed uat is fixed in that same worktree and merged again), and `bee close` blocks until the user approves `uat`. `"off"` removes the stop everywhere. Any other value refuses rather than guessing. The legacy `uat_before_merge` boolean is read only as this key's back-compat alias (`true`→`"merge"`, `false`→`"off"`) when `uat_stop` itself is absent — but with BOTH keys absent, the default is `"close"`, not this alias's own true/false shape | absent means `"close"` |
| `staging_before_merge` | whether the repo uses the staging mixing ground; explicit `true` turns it on repo-wide so `bee staging add`/`bee staging rebuild` work. Leaving the key absent or setting it `false` makes both refuse `STAGING_DISABLED` — staging is opt-in — so the repo runs feature worktree -> `uat` gate -> main with no staging step. A non-boolean value refuses `STAGING_CONFIG_INVALID`. This key does not touch the `uat` gate itself — the `uat` gate stays exactly as `uat_stop` configures it | absent means OFF |
| `dogfood_repos` | foreign repos whose feedback digest `bee feedback collect`/`rank` (and the [handbook/evolving.md](handbook/evolving.md) loop) fold in — see below | `null` (local digest only) |
| `product_root` | where the project's PRODUCT docs live (`docs/backlog.md`, `docs/specs/`, the product README) when they are NOT beside `.bee/` — a path relative to the bee root, or absolute. For the "workshop + nested product repo" (repo-divorce) topology where `.bee/` sits one level above the product's own git repo. Unset ⇒ the bee root (every ordinary single-root repo is unaffected). A set-but-missing path warns loudly to stderr rather than silently reading nothing. `.bee/*` runtime state and `docs/history/` (bee's own workshop trail) are never affected — only the product's own docs. | unset ⇒ bee root |
| `doc_viewer` | opt-in URL prefix for a local doc viewer (e.g. mdview) — `base_url` + `project` join as `<base_url>/p/<project>/<repo-relative-path>`, so the agent gives a clickable URL instead of a bare path — see below | unset ⇒ bare paths |
| `herding` | `agent_command` / `control_command` — the runtime adapter for `bee herding run`/`--continue` (one external agent as a cell-execution worker), `bee herding wave`, and `bee herding control-loop`; both keys are optional and independent. Canonical doc, not duplicated here: [skills/bee-herding/references/operational-invariants.md](../skills/bee-herding/references/operational-invariants.md) | absent ⇒ today's `claude` spawn, unchanged |

### `guards.memory_root` (GH #71) — letting the agent keep its own memory

The write guard contains every write to the worktree, so the agent's persistent memory at
`~/.claude/projects/<slug>/memory/` is unreachable and durable learnings are lost. Declaring a
memory root is the one escape hatch — and **it takes two steps, on purpose**, because an agent can
edit `.bee/config.local.json` by itself but cannot create the marker file:

```bash
mkdir -p ~/.claude/projects/<slug>/memory
touch ~/.claude/projects/<slug>/memory/.bee-write-root
```

```jsonc
// .bee/config.local.json  (gitignored — never the tracked .bee/config.json)
{ "guards": { "memory_root": "~/.claude/projects/<slug>/memory" } }
```

**Understand what you are granting.** A declared root is a place bee will let the agent write **at
any phase, with no gate, no reservation and no hold** — writes there skip those checks entirely,
which is the point: a learning must be recordable even at phase `idle`, when the intake gate is
shut. Declare a directory that holds nothing but memory.

The root is honored only while the `.bee-write-root` file is there — delete it and the grant is off
immediately, with no config edit. Everything else stays denied exactly as before: traversal out of
the root, a symlink inside it that resolves outside it, and `~/…`/`$HOME/…` *target* spellings (only
absolute target paths are honored; the leading `~` is expanded in the **config value** alone). A
root is refused outright — after resolving symlinks — if it is the filesystem root, a bare home
directory, a directory that contains this worktree, anything inside or containing a `.git`/`.bee`
directory, or not an existing directory. Refused, malformed, or unset means today's behavior,
unchanged. The `apply_patch` tool path never honors a memory root.

### `dogfood_repos` (P18, evolving loop)

Other repos running bee whose collected friction should feed into ranking here. Accepts a bare path
array or `{path,label}` objects (both normalize to objects); each entry is `realpath`-contained and
must have its own `.bee/feedback-digest.json` already written (`.bee/bin/bee feedback
digest` in that repo). A configured repo that is missing, unreadable, or dead is **skipped with a
warning**, never thrown:

```jsonc
{
  "dogfood_repos": [
    { "path": "../anphabe-gogl", "label": "anphabe-gogl" }
  ]
}
```

Every field pulled from a listed repo's digest is **revalidated and datamark-wrapped** by
`mergeDigests` before it is used (decision D2b) — this repo never trusts a foreign digest's bytes as
written. `null` (the default) means `collect`/`rank` return the local digest only, and
`corroboration` is 1 for every cluster (see `docs/07-contracts.md`'s evolving contract).

### `doc_viewer` (decision `4205835b`) — clickable doc links in agent prose

Two fields, no template to get wrong:

```jsonc
{
  "doc_viewer": {
    "base_url": "http://10.255.255.254:7700",
    "project": "beedashboard"
  }
}
```

bee joins them as `<base_url>/p/<project>/<repo-relative-path>` — mdview's own URL layout — and
reads the result into the session preamble and the compaction capsule, so every doc reference the
agent writes for the rest of the session is that URL instead of a bare path (e.g.
`docs/history/<feature>/plan.md`). This is opt-in and scoped to agent prose only: `bee orient`,
`bee status`, and every other CLI surface keep printing bare paths regardless of the key.

**Unset** is today's behavior, unchanged and silent — every doc reference stays a bare path.

**Half-set is loud, not silent.** `base_url` without `project` (or either one empty, or `doc_viewer`
set to something that is not an object) produces no URL, plus one stderr line naming the key. A key
that looks configured but quietly does nothing is worse than one left unset, so bee warns instead of
staying quiet.

**The one limit.** bee joins the URL; it does not encode it. A repo-relative path that contains a
space (or another character a URL cannot carry as-is) has to be percent-escaped by whoever writes the
link — bee will not do it for you.

## Full sample to copy

Clean JSON — paste into `.bee/config.json` and edit values (keep any existing `commands` you already have):

```json
{
  "commands": { "setup": "npm install", "start": "npm run dev", "test": "npx jest --onlyChanged" },
  "gate_bypass": false,
  "guards": { "idle_gate": true, "max_read_lines": 800 },
  "models": {
    "claude": {
      "code": { "model": "sonnet", "effort": "medium" },
      "read": "haiku",
      "extraction": "haiku",
      "generation": "sonnet",
      "review": "opus",
      "advisor": "opus"
    },
    "codex": { "code": null, "read": null, "extraction": null, "generation": null }
  }
}
```

The full, copyable version of this file lives at [`.bee/config-sample.json`](../.bee/config-sample.json) — it carries every key bee actually reads, each with a `_doc` note, plus a `dogfood_repos` example. `product_root` is documented there but deliberately left unset: a set-but-missing path warns on every read, so it is the one key you add only when the topology needs it.

A second, ready-to-run demo lives at [`.bee/config-sample-cli-executors.json`](../.bee/config-sample-cli-executors.json): the same file with the `generation` slot dispatched to **agy** (Antigravity, `Gemini 3.5 Flash (High)`) and `review` to **opencode**, both wrapped in `bash -lc '… "$(cat)"'` because neither CLI reads the worker prompt from stdin. Copy it only if those CLIs are installed — otherwise every worker dispatch fails. Presets and the per-flag reasoning: [`docs/model-presets.md`](model-presets.md).

> **`ceiling` has no entry** — it is not a role name and never was configurable. The strongest model is always the one you run the session on, and a cell reaches it through `bee cells escalate`, not through this file.
>
> `review` and `advisor` appear in the sample above to show their shapes. A fresh `bee onboard` writes neither on purpose — both already resolve with no key at all.
>
> **`models.pi` is not in this copy sample on purpose** — the pi runtime is herding-only, so every slot needs a `herding.agents` entry standing behind it (pi-support D5/D6). Copy its block out of [`.bee/config-sample.json`](../.bee/config-sample.json) when you mean to run Pi, and read the delivery paths with it — sync `bee herding run` output is the contract to plan on, and the opt-in async drain is at-least-once with `job_id` as the dedupe key: [Pi](#pi--modelspi-is-herding-only).
