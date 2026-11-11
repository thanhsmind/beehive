# A supervising project reads the bee state of the projects it manages — Context

**Feature slug:** supervisor-read-scope
**Date:** 2026-11-11
**Shaping session:** complete (owner spec-drop from waggledance@d35b1d9; owner decision 033faaf6 recorded in waggledance)
**Scope:** High-risk (a security guard opens a read path; a new config key; an external CLI seam)
**Domain types:** RUN

## Feature Boundary

The cross-project guard refuses every read of another project's tree. A
waggledance session must measure bee across every project waggledance
manages, and its own daemon already reads those stores. After this
feature, a repo that sets `guards.supervises: "waggledance"` may read the
`.bee/` store and `docs/knowledge/` of every project in the waggledance
registry, read-only. Every write, source read, git operation and mutating
bee verb in a managed project stays refused, and secret-shaped files stay
refused. `bee knowledge report` and `bee report outcomes` take `--root` so
the supervisor measures another store without a `cd`.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Opt-in key `guards.supervises` (string, value `waggledance`) in `.bee/config.json` or its local overlay. Governed like `guards.cross_project_allow`: `bee config set` and agent edits that add or change it are refused; other `guards.*` keys stay settable. (902d1477) | An agent that widens its own reach has no mechanism. `guards.worker_outward` and `guards.memory_root` are user toggles, so the whole subtree is not governed. |
| D2 | Managed roots come from `waggledance list --json` (`projects[].root_path`), run only when the key is set and a candidate already resolved to another project. Binary from the hook PATH; never one that resolves inside this project. Killed after 2 s. Spawn failure, timeout, non-zero exit or bad output grants nothing and never panics. No cache file. The registry is the owner's authority. (5c498adf, supersedes f242c7c8) | A hang or panic in a hook reads as allow on a fail-open host. |
| D3 | A refused path belongs to a managed project when its canonical project root equals the canonical form of a registry `root_path`, or shares that root's git common dir. Symlinked stale roots and `<repo>--wt--<slug>` worktrees both count. (318dee0d) | The guard already canonicalises and owns worktrees by git common dir. |
| D4 | The readable surface is `<root>/.bee/**` and `<root>/docs/knowledge/**` of the resolved other root, minus any path `is_secret_path` matches and any file named `config.local.json`. A Grep tool call whose path is a surface directory holding such a file at any depth is refused; the walk refuses past 50000 entries. (2c92af66, supersedes 0b2d16cd) | `is_secret_path` checks a basename; a directory-wide Grep would print secret content. |
| D5 | Read and Grep: every other-project candidate in the surface; Glob also needs no `..` in its pattern. Bash/exec: no newline or CR in the raw command; tokenizer not truncated; no `$` or backtick in any token; no glob or brace sign in a candidate path token; no redirect sign except the exact tokens `2>/dev/null` and `2>&1`; no `cd`/`pushd`; hook cwd inside this project; every segment a head from cat, head, tail, wc, ls, jq, or `bee knowledge report` / `bee report outcomes` run as bare `bee` or this store's own `.bee/bin/bee`. Every other-project candidate in the surface, except a bee report segment's `--root` value, which may equal a managed root or its `.bee`. sqlite3, rg, grep, sort, cut, stat, file are not granted in the shell. (8452f29c, supersedes 6214ac34 and e02a99ee) | A newline hides a second command; shell expansion defeats a path check; sqlite3 and sort run programs; a `/x/bee` head runs any binary. The Grep tool covers rg; bee stores hold no sqlite file. |
| D6 | `bee knowledge report` and `bee report outcomes` take `--root <path>` (a store root or a path inside one); the store root resolves from that path. No byte of the target store changes: timing and log rows go to the cwd store, or are skipped. (a8d8ad76, supersedes 6be395b9) | `report_outcomes` records timing on the resolved root today. |
| D7 | With the key set, a managed-project refusal keeps today's text and adds one sentence naming the rule that tripped and the readable surface; a failed registry lookup adds that the registry was not readable; an unmanaged project adds that it is not in the registry. Without the key every refusal is byte-identical to today. (06fe4cb5, supersedes 27171f3b) | The owner must tell registry-down from not-managed; an agent must learn why. |
| C1 | Contract: `cross_project_denial` keeps its signature; a sibling entry point takes the managed-root list from an injected provider so unit tests need no waggledance binary. Proving tests: (a) managed `.bee/state.json` Read allowed, (b) write-shaped Bash to it refused, (c) managed `src/` Read refused, (d) unregistered project refused, (e) symlinked registry root resolves; plus the leak cases of D4 and D5. (68db537b) | |

### Agent's Discretion

- How the read-only segment check walks the token list (the tokenizer already emits separators).
- How the `--root` flag is registered in the command registry, provided `bee dev regen` stays clean.

### Known costs the owner accepted

- A supervising session cannot write into a managed project; writes still go through `waggledance_dispatch`.
- A shell command that mixes a granted head with anything else (for example `cat x | python3`) is refused as a whole.
- In the grant, jq filters cannot use `$`, `>` or `<`; Read, Grep and the bee report verbs cover what that loses.
- Spec item 1 named rg and `sqlite3 -readonly` as examples; the Grep tool replaces rg, and sqlite3 is not granted (bee stores hold no sqlite file).
- An agent that registers a project in waggledance widens what the supervisor reads; the registry is the owner's authority.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md | read | § Cross-project refusal is the rule this feature extends. |

### Code

- docs/history/cross-project-guard/CONTEXT.md (D1-D4 of the guard being extended)

- packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs (`Scope::load`, `other_root`, `cross_project_denial`, `refusal`)
- packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs (`is_secret_path`, `tokenize_deep`, `is_separator`, `config_governed_view`, `diff_governed_views`)
- packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs (`cross_project_denial` call site)
- packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs (`run_report`)
- packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs (`run`)
