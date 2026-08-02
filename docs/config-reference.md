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
- **The models/cli-tier block has to stay valid.** `config set` refused a write that broke it;
  nothing refuses a hand-edit, so re-read [Which model each tier uses](#which-model-each-tier-uses)
  after changing `models`.

Values are ordinary JSON: `false` is a boolean, `12` a number, `"repo"` a string. Nested keys that
the old `--key guards.idle_gate` dot-notation reached are just nested objects in the file.

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

Captured at onboarding (or the first natural moment in exploring), three standard keys — all plain runnable shell commands, never descriptions:

| Key | Meaning | Who runs it, when |
|---|---|---|
| `setup` | install dependencies from scratch | onboarding checks, fresh-clone bootstrap |
| `start` | run the app/dev server | on demand (`/run`-style checks) |
| `test` | **the project's ONE declared test command** | every door, and the same command each time: the green base check before the first claim, `bee finish` at each cap, the orchestrator's wave-close check, `bee close` for the feature, the `bee worktree merge` semantic gate (run against the staged merge), and CI on the host's own cadence |

**`commands.verify` is retired.** It used to sit above `test` as a second, full-suite, CI-owned command. Two repo-wide commands meant every surface had to say which door ran which — and they disagreed: this reference called `verify` "never a local obligation" while the green base check told agents to run it locally before their first claim. One command ends the question. A host that wants a slower full sweep runs it in CI on its own schedule; bee needs no config key to know about it.

Below `commands.test` there is a second, narrower layer that is **not** config: each work cell's own `verify` field, authored per change (one test file / one test function, seconds). Config carries the one repo-wide command; the cell carries the per-change one.

### Projects without tests

A project that deliberately runs no tests declares that in config instead of leaving the key absent: set `commands.test` to the exact sentinel string `"none"` (no-test-repos D1, decision `55b951e1`). Absence keeps its existing meaning — not-captured-yet, the normal onboarding nag — the sentinel means "this repo will never have one." With the sentinel set: the session preamble skips the CI-status-gate paragraph and prints one loud `Test gates disabled by repo declaration` line instead; cells may carry `verify: "none"` (refused everywhere else, exactly like a prose description would be) and cap on that cell records the diff-backed outcome with an auto waiver note rather than a passing verify result; wave-close, session-finish, and worktree-merge all skip with the same loud line, never silently. Nothing here is permanent — re-enable at any time by recording a real `test` command, which restores every gate above on the next session.

### Per-language recipes

`commands.test` runs on every cap, so pick something you are willing to pay for that often. A changed-only mode is ideal where the runner has one; a whole-suite command is fine when the suite is fast.

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
- A command that takes the changed-file list from git itself (jest `--onlyChanged`, testmon) is the best `test` value — it stays correct with zero per-change editing. Where the runner has no such mode (Go, Rust, PHP), record the *narrow invocation shape* and let the session substitute the changed package/crate/class per change — the doctrine cares that the door stays cheap enough to run at every cap, not which selector you use.
- CI should run `commands.test` verbatim (bee's own `ci.yml` does exactly that with `cargo test --release`, and files a deduped `verify-red` issue on red).
- Where the "which tests relate to this file" answer needs a lookup, use the language's native graph (Go: `go list -deps` reversed; Rust: the crate graph; Python: testmon's coverage map). bee's own repo used to ship a derived impact registry for this; it was retired at the R6 Node cutover, because its subject was the `.mjs` suite graph and the Rust suite that replaced it runs whole in ~20s.

## Removed keys

**`commands.verify`** was retired in **2.1.0**. `commands.test` is now the one declared test command and every door runs it. If your `.bee/config.json` still has a `verify`, onboarding warns and it is ignored — delete it. Two migrations matter:

- **You recorded both.** Nothing to do beyond deleting `verify` — `test` already governed the dev loop, and it now governs merge and CI too. If your `verify` was materially broader, decide whether that breadth belongs in `test` (paid at every cap) or in your CI workflow (paid on push).
- **You recorded ONLY `verify`.** You currently have **no test gate at all** — onboarding says so loudly. Move the command to `commands.test`, or set `commands.test` to `"none"` if the repo is deliberately test-free. `"none"` on `verify` no longer declares a no-test repo.

The **top-level** `advisor` key (old "advisor mode") was removed in v0.1.23 (decision fanout-delegation D1). If your `.bee/config.json` still has one, onboarding warns about the stale key and ignores it — delete it. This is **not** the same thing as the `models.<runtime>.advisor` slot above, which is current and valid.

## Other keys

| Key | What it does | Default |
|---|---|---|
| `commands` | the host project's `setup` / `start` / `test` commands — full section above | none — captured at onboarding |
| `gate_bypass` | opt-in autopilot with levels `false` · `"normal"` · `"full"` · `"total"` (legacy `true` = normal); set via `bee-hive`'s "Gates" section (gate-bypass levels) | `false` |
| `hooks` | per-hook kill switch — nine hooks: `session-init`, `prompt-context`, `write-guard`, `model-guard`, `state-sync`, `chain-nudge`, `session-close`, `tools-logger`, `codex-subagent-audit` | all `true` (an absent key also reads `true`) |
| `guards` | `idle_gate` (`false` disables the idle intake gate) · `max_read_lines` (line cap a single inbound file read may pull before the read guard trims it; number > 0) · `memory_root` (one absolute path the write guard will let the agent write — see below) | idle gate on · `800` · no memory root |
| `cells_archive_on_close` | whether a green `bee close` retires the feature's cells into `.bee/cells/archive/<feature>/`, out of the scan path `status`/`orient` parse on every call. Only fires when every one of the feature's cells is capped or dropped; reverse with `bee cells unarchive --feature <f>`. Set `false` for a repo whose own tooling reads `.bee/cells/*.json` by path | `true` |
| `ship_visibility` | how finished work is surfaced — `"off"` or `"draft-pr"`. An unrecognized value normalizes to `"off"` and says so once, by name | `"off"` |
| `worktree_first` | code-touching feature work lives in its own worktree and the write guard refuses feature edits made in the main checkout; the exact string `"off"` disables that refusal — see [specs/worktree-first.md](specs/worktree-first.md) | on |
| `dogfood_repos` | foreign repos whose feedback digest `bee feedback collect`/`rank` (and the [handbook/evolving.md](handbook/evolving.md) loop) fold in — see below | `null` (local digest only) |
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

## Full sample to copy

Clean JSON — paste into `.bee/config.json` and edit values (keep any existing `commands` you already have):

```json
{
  "commands": { "setup": "npm install", "start": "npm run dev", "test": "npx jest --onlyChanged" },
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

The full, copyable version of this file lives at [`.bee/config-sample.json`](../.bee/config-sample.json) — it carries every key bee actually reads, each with a `_doc` note, plus a `dogfood_repos` example. `product_root` is documented there but deliberately left unset: a set-but-missing path warns on every read, so it is the one key you add only when the topology needs it.

A second, ready-to-run demo lives at [`.bee/config-sample-cli-executors.json`](../.bee/config-sample-cli-executors.json): the same file with the `generation` slot dispatched to **agy** (Antigravity, `Gemini 3.5 Flash (High)`) and `review` to **opencode**, both wrapped in `bash -lc '… "$(cat)"'` because neither CLI reads the worker prompt from stdin. Copy it only if those CLIs are installed — otherwise every worker dispatch fails. Presets and the per-flag reasoning: [`docs/model-presets.md`](model-presets.md).

> **ceiling** has no entry — it is always whatever model you run the session on.
