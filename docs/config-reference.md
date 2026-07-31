# `.bee/config.json` — configuration reference

Every onboarded repo has a `.bee/config.json`. Any key you leave out uses a built-in default, so the file can be short — but the values below are the ones worth setting per repo. **`.bee/config.json` is strict JSON: no comments, no trailing commas.** The annotated block here is for reading; copy the clean block at the bottom into the real file.

## Setting values — use the CLI, not a hand-edit

You do not have to hand-edit the JSON. Set/read/remove any key through the CLI (validated on write, dot-notation for nested keys):

```bash
node .bee/bin/bee.mjs config get   --key product_root
node .bee/bin/bee.mjs config set   --key product_root --value repo
node .bee/bin/bee.mjs config set   --key guards.idle_gate --value false   # nested key
node .bee/bin/bee.mjs config unset --key guards.idle_gate                 # remove (prunes the empty parent)
node .bee/bin/bee.mjs config validate                                     # check models/cli-tier config
```

The value is parsed as JSON when it parses (`false` → boolean, `12` → number, `{...}` → object), otherwise kept as a string (`repo` → `"repo"`); pass `--string` to force a string. `set`/`unset` refuse to write if the change would make the models/cli-tier config invalid, or if the existing file is unparseable (it is never silently clobbered).

One routing rule to know: `hooks.*` and `guards.*` are **local-only namespaces** — `config set`/`unset` always write them to `.bee/config.local.json` (gitignored, per-machine), even without `--local`, so one developer muting a hook never lands in the tracked config. `config get` reads overlay-over-tracked and warns if such a value still sits in the tracked file.

## Which model each tier uses

There are three tiers, but **you only configure the two cheaper ones.** The **ceiling** (strongest) tier is **never configured — it is always the model you are running the session on** (decision 0015). So if you run the session on Fable, ceiling work runs on Fable; run it on Opus, ceiling is Opus. bee doesn't pick it; it inherits your session model.

You configure only `generation` and `extraction`, under **`models`**, keyed by runtime (Claude Code vs Codex name models differently). Beside the two tiers sit two configurable **roles**, `review` and `advisor`:

```jsonc
{
  "models": {
    "claude": {
      "extraction": "haiku",    // cheapest — retrieval, mechanical edits
      "generation": "sonnet",   // the mid worker that runs the loops (most cells)
      // no "ceiling" — it's whatever model runs your session
      "review": "opus",         // reviews what generation implemented; null → falls back to generation
      "advisor": "opus"         // consulted by a worker whose verify keeps failing; null/unset → no advisor
    },
    "codex": {
      "extraction": null,       // Codex has no per-agent model switch today →
      "generation": null,       //   null means "enforce the tier via read budget + output cap in the prompt"
      "review": null,
      "advisor": null
    }
  }
}
```

- **To change the worker models**, edit `models.claude.generation` / `extraction` (e.g. `"opus"` for a stronger worker tier). To change the **ceiling**, just run the session on a different model — there is no config for it.
- **`review`** (decision 0021) is the model that reviews what `generation` implemented — an independent reviewer beats self-review, so a review slot stronger than generation is the point. `null` → the generation tier reviews.
- **`advisor`** (advisor v1) is a *worker-level, on-failure consult*: a worker that has failed its verify calls the advisor once or twice before blocking, and takes advice only — it never gets authority. Unlike `review` it has **no fallback**: `null`, unset, or an advisor no stronger than the worker's own model simply means "no consult happens".
- **The six value shapes** each slot accepts (decisions 0019/0021; native override D2, codex-native-transport):

  | shape | means |
  |---|---|
  | `"sonnet"` | the runtime's per-agent model switch |
  | `{ "model": "sonnet", "effort": "medium" }` | model + reasoning effort (`low` · `medium` · `high` · `xhigh` · `max`); the effort is applied where the runtime has a per-agent effort switch, recorded and ignored where it does not |
  | `{ "kind": "cli", "command": "codex exec -m … -s read-only -", "promptVia": "stdin" }` | an **external executor** — a separate CLI process dispatched under the same worker contract (effort rides inside the command); `promptVia` declares how the prompt reaches it, never sniffed from the command string |
  | `{ "kind": "native", "model": "gpt-5.5", "effort": "high", "fork_turns": "none", "agent_type": "worker" }` | a **native V2 model override** (codex runtime) — a stronger model applied per-agent on the codex `spawn_agent` metadata, no separate process. `model` is the exact catalog model id. `fork_turns` must be `"none"` (a full-history fork rejects overrides) and defaults to `"none"`; `agent_type` defaults to `"worker"`. `effort` is optional. The route is inert until a capability probe confirms the host build accepts it (D3) |
  | `{ "primary": { "kind": "native", "model": "gpt-5.5" }, "fallback": { "kind": "cli", "command": "codex exec … -s read-only -", "promptVia": "stdin" }, "fallback_policy": "explicit-only" }` | a **native primary with an opt-in cli fallback**. The fallback is taken **only** when `fallback_policy` is exactly `"explicit-only"`; without that string the fallback is dropped and never used — silent native→cli fallback is forbidden (D1) |
  | `null` | no per-agent switch: the tier is enforced via read budget + output cap in the prompt (for `review`: fall back to generation; for `advisor`: no advisor) |

  Invalid shapes are ignored — the slot's default stands, nothing throws. A native override missing its `model`, a `fork_turns` other than `"none"`, or a composite missing `fallback_policy` is flagged by config validation (`bee status`), never silently trusted.
- **What the short names mean (important).** For Claude Code these are **family aliases**, not exact version strings. The value must be one of exactly `haiku` · `sonnet` · `opus` · `fable` — the Claude Code Agent tool accepts only these four. Each alias is resolved **by Claude Code (not by bee)** to the current model of that family on your account. So `"sonnet"` isn't "some random Sonnet" — it means "the Sonnet tier", and the harness uses the latest. Today they resolve to:

  | alias | resolves to (current) | model id |
  |---|---|---|
  | `haiku` | Haiku 4.5 | `claude-haiku-4-5` |
  | `sonnet` | Sonnet 5 | `claude-sonnet-5` |
  | `opus` | Opus 4.8 | `claude-opus-4-8` |
  | `fable` | Fable 5 | `claude-fable-5` |

  You **cannot pin an exact sub-version** for a Claude Code subagent — the model param is family-alias only, and it tracks the latest of each family as Anthropic ships new ones. (For **Codex**, the `codex` tiers take the runtime's real model ids, e.g. `"gpt-5"`, because that runtime addresses models by id.)
- `bee_status` prints the active map (`Models (claude): generation=… extraction=… · ceiling = the session model`), and warns if too many cells sit on the ceiling tier — the point is to keep the strong (session) model scarce.

### Runtimes: Claude Code and Codex — and everything else (OpenCode, agy, …)

`models` accepts exactly **two runtime keys: `claude` and `codex`** — the two harnesses bee ships hooks and dispatch transports for. Any other top-level runtime key (e.g. `"opencode"`, `"gemini"`) is **silently ignored**: not an error, just dead config that never resolves.

That does *not* mean other CLIs are unusable — they plug in through the **external-executor slot shape** on whichever runtime you actually run the session in. Example, routing the review tier of a Claude Code session through OpenCode:

```json
{
  "models": {
    "claude": {
      "extraction": "haiku",
      "generation": { "model": "sonnet", "effort": "medium" },
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

## `commands` — the host project's lifecycle commands

Captured at onboarding (or the first natural moment in exploring), four standard keys — all plain runnable shell commands, never descriptions:

| Key | Meaning | Who runs it, when |
|---|---|---|
| `setup` | install dependencies from scratch | onboarding checks, fresh-clone bootstrap |
| `start` | run the app/dev server | on demand (`/run`-style checks) |
| `test` | **the SCOPED dev-loop test command** — only the tests related to the current change | your machine, often: the orchestrator's wave-close check after cells cap, and the `bee worktree merge` semantic gate (run against the staged merge) |
| `verify` | **the FULL test suite** | **CI-owned, on the project's own CI cadence** — push, nightly, or scheduled, the host workflow decides (plus the release gate). Never a local per-cell, per-cap, or session-start obligation — a red CI run auto-files a `verify-red` issue instead |

The split is the point (ci-owned-verify D1/D5): `test` must be *narrower* than `verify`, or every dev-loop iteration pays the full-suite price as the suite grows. Every consumer that wants `test` falls back to `verify` when `test` is missing, so a repo that only recorded `verify` keeps working — just slower.

Below `commands.test` there is a third, narrower layer that is **not** config: each work cell's own `verify` field, authored per change (one test file / one test function, seconds). Config carries the two repo-wide commands; the cell carries the per-change one.

### Projects without tests

A project that deliberately runs no tests declares that in config instead of leaving the keys absent: set `commands.verify` (and/or `commands.test`) to the exact sentinel string `"none"` (no-test-repos D1, decision `55b951e1`). Absence keeps its existing meaning — not-captured-yet, the normal onboarding nag — the sentinel means "this repo will never have one." With the sentinel set: the session preamble skips the CI-status-gate paragraph and prints one loud `Test gates disabled by repo declaration` line instead; cells may carry `verify: "none"` (refused everywhere else, exactly like a prose description would be) and cap on that cell records the diff-backed outcome with an auto waiver note rather than a passing verify result; wave-close, session-finish, and worktree-merge all skip with the same loud line, never silently. Nothing here is permanent — re-enable at any time by recording a real `test`/`verify` command, which restores every gate above on the next session.

### Per-language recipes

Pick your runner's changed-only/related mode for `test`; `verify` is whatever runs everything.

| Language | `commands.test` (scoped) | `commands.verify` (full) |
|---|---|---|
| **Node** | `npx jest --onlyChanged` (jest) · `npx vitest related --run <files>` (vitest) · in bee's own repo: `node scripts/run_verify.mjs --impacted-from-git` | `npm test` / `npm run build && npm test` |
| **Go** | `go test ./internal/<changed-pkg>/...` — derive the package set from the diff (`go list ./... \| grep …`, or reverse-deps via `go list -deps`) | `go test ./...` |
| **Rust** | `cargo test -p <changed-crate>` (workspace: one crate) · `cargo test <module>::` (one module path) | `cargo test --workspace` |
| **Python** | `pytest tests/test_<area>.py` (by path) · `pytest -k <expr>` (by name) · `pytest --testmon` (coverage-map impacted, needs pytest-testmon) | `pytest` |
| **PHP** | `vendor/bin/phpunit --filter <TestClass>` · `vendor/bin/phpunit tests/<Area>/` (by dir) · Laravel: `php artisan test --filter <name>` | `vendor/bin/phpunit` (hoặc `composer test`) |
| **No tests** | `"none"` (sentinel — declares the repo deliberately test-free) | `"none"` (either key alone is sufficient; setting both is fine) |

Notes:
- A command that takes the changed-file list from git itself (jest `--onlyChanged`, testmon, bee's `--impacted-from-git`) is the best `test` value — it stays correct with zero per-change editing. Where the runner has no such mode (Go, Rust, PHP), record the *narrow invocation shape* and let the session substitute the changed package/crate/class per change — the doctrine cares that the dev loop never runs the full suite, not which selector you use.
- CI should run `commands.verify` verbatim (bee's own `ci.yml` does exactly that via `scripts/verify_all.mjs`, and files a deduped `verify-red` issue on red).
- Where the "which tests relate to this file" answer needs a lookup: bee's own repo ships a derived impact registry (`node scripts/impact_registry.mjs --query <file>`); other languages use their native graph (Go: `go list -deps` reversed; Rust: the crate graph; Python: testmon's coverage map).

## Removed keys

The **top-level** `advisor` key (old "advisor mode") was removed in v0.1.23 (decision fanout-delegation D1). If your `.bee/config.json` still has one, onboarding warns about the stale key and ignores it — delete it. This is **not** the same thing as the `models.<runtime>.advisor` slot above, which is current and valid.

## Other keys

| Key | What it does | Default |
|---|---|---|
| `commands` | the host project's `setup` / `start` / `test` (scoped, dev loop) / `verify` (full, CI-owned) commands — full section above | none — captured at onboarding |
| `gate_bypass` | opt-in autopilot with levels `false` · `"normal"` · `"full"` · `"total"` (legacy `true` = normal); set via `bee-hive`'s "Gates" section (gate-bypass levels) | `false` |
| `hooks` | per-hook kill switch — nine hooks: `session-init`, `prompt-context`, `write-guard`, `model-guard`, `state-sync`, `chain-nudge`, `session-close`, `tools-logger`, `codex-subagent-audit` | all `true` (an absent key also reads `true`) |
| `guards` | `idle_gate` (`false` disables the idle intake gate) · `max_read_lines` (line cap a single inbound file read may pull before the read guard trims it; number > 0) · `memory_root` (one absolute path the write guard will let the agent write — see below) | idle gate on · `800` · no memory root |
| `lanes`, `capabilities` | advanced per-repo overrides | `{}` |
| `dogfood_repos` | foreign repos whose feedback digest `bee.mjs feedback collect`/`rank` (and the [handbook/evolving.md](handbook/evolving.md) loop) fold in — see below | `null` (local digest only) |
| `product_root` | where the project's PRODUCT docs live (`docs/backlog.md`, `docs/specs/`, the product README) when they are NOT beside `.bee/` — a path relative to the bee root, or absolute. For the "workshop + nested product repo" (repo-divorce) topology where `.bee/` sits one level above the product's own git repo. Unset ⇒ the bee root (every ordinary single-root repo is unaffected). A set-but-missing path warns loudly to stderr rather than silently reading nothing. `.bee/*` runtime state and `docs/history/` (bee's own workshop trail) are never affected — only the product's own docs. | unset ⇒ bee root |

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
must have its own `.bee/feedback-digest.json` already written (`node .bee/bin/bee.mjs feedback
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

## Full sample to copy

Clean JSON — paste into `.bee/config.json` and edit values (keep any existing `commands` you already have):

```json
{
  "commands": { "setup": "npm install", "start": "npm run dev", "test": "npx jest --onlyChanged", "verify": "npm run build && npm test" },
  "gate_bypass": false,
  "guards": { "idle_gate": true, "max_read_lines": 800 },
  "models": {
    "claude": {
      "extraction": "haiku",
      "generation": { "model": "sonnet", "effort": "medium" },
      "review": "opus",
      "advisor": "opus"
    },
    "codex": { "extraction": null, "generation": null, "review": null, "advisor": null }
  }
}
```

The full, copyable version of this file lives at [`.bee/config-sample.json`](../.bee/config-sample.json) — it carries every key, including `hooks`, `lanes`/`capabilities`, and a `dogfood_repos` example.

A second, ready-to-run demo lives at [`.bee/config-sample-cli-executors.json`](../.bee/config-sample-cli-executors.json): the same file with the `generation` slot dispatched to **agy** (Antigravity, `Gemini 3.5 Flash (High)`) and `review` to **opencode**, both wrapped in `bash -lc '… "$(cat)"'` because neither CLI reads the worker prompt from stdin. Copy it only if those CLIs are installed — otherwise every worker dispatch fails. Presets and the per-flag reasoning: [`docs/model-presets.md`](model-presets.md).

> **ceiling** has no entry — it is always whatever model you run the session on.
