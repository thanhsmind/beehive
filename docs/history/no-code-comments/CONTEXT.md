# No Code Comments — Context

**Feature slug:** no-code-comments
**Date:** 2026-09-22
**Shaping session:** complete (two user answers; the rest recorded as recommended readings under gate_bypass full)
**Scope:** Standard
**Domain types:** RUN | CALL

## Feature Boundary

No comment can be added to a code file in this repository. Two layers
enforce it: the write guard hook refuses the write for hooked agents,
and a ratchet test in the declared suite goes red on any file whose
comment-line count rises above its committed baseline. The rule and
the two homes for the "why" land in the doctrine homes. The 47,000
comment lines that exist today stay; their removal is separate,
batched work. Nothing else about hooks, tests, or doctrine changes.

Source of the idea: `docs/REFs/pstack-rules.md` (the pstack talk):
agents used comments to justify workarounds instead of fixing them, so
the Dune framework banned comments outright. The user's reading for
bee: code plus the knowledge layer (`docs/knowledge/`) are the source
of truth; prose in code is a second home that drifts.

## Evidence

Measured in this checkout on 2026-09-22, before any change:

- No rule about comments in code exists in `AGENTS.md`,
  `packages/bee/AGENTS.block.md`, `packages/bee/prompts/worker-cell.md`,
  any skill, or `.bee/expertise/` (rg over all of them).
- Comment lines today: `packages/bee-rs/crates/bee/src` 46,060 (21,854
  of them `///` or `//!` doc comments, 0 block comments);
  `packages/bee-rs/crates/fleet/src` 1,239; `scripts/` 442 shell `#`
  lines (non-shebang). No JS/TS remains under `packages/bee`.
- Comment shapes: 10 lines match workaround/hack/for now/TODO; 1,191
  cite a decision id or provenance. No file carries a license header.
- `unsafe` appears 131 times; 22 `SAFETY:` lines exist; clippy's
  `undocumented_unsafe_blocks` is not configured.
- The write guard (`packages/bee-rs/crates/bee/src/hooks/write_guard/`)
  already reads `tool_input.new_string` for Edit/MultiEdit,
  `tool_input.content` for Write, `tool_input.command` for Bash, and
  `tool_input.input` for codex apply_patch, and refuses with a remedy
  text (`write_guard/tests.rs:131-137, 333-354`).
- A committed baseline precedent exists: `.bee/doc-deferral-baseline.json`,
  read and matched by one normalization function
  (`verbs/drivers/close.rs:1318-1351`).
- A pinned-count precedent exists: `catalog.rs` `PINNED_FLAG_COUNT`,
  bumped on purpose with a dated reason.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never
reinterpreted. Changing one requires the user, a new D-ID or an
explicit supersession note, never a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 (38e323d6) | No comment may be written in code, in any language this repo ships: Rust line and block comments including doc comments (`//`, `///`, `//!`, `/* */`), and `#` lines in shell and python, under `packages/bee-rs/crates/*/src`, `packages/bee/hooks`, `packages/bee/lib`, `scripts`, and `.bee/verify`. Not comments for this rule: a shebang (`#!`), a clippy-required `SAFETY:` line on an unsafe block, and a license header at the top of a file. | User's answer. Doc comments count because they carry "why" in code. |
| D2 (80ec3cd1) | Enforcement is a ratchet in the declared test suite: a committed baseline records the comment-line count per file; a test goes red when any file's count exceeds its baseline or a new code file carries any comment; the baseline only ever goes down, lowered by a dev verb that refuses to raise a count. CI runs it on every push. | User's answer (stop the bleeding first). The static-analysis tier of the pstack ladder. |
| D3 (c2477366) | The write guard hook refuses an Edit, Write, or shell write that adds a comment line to a code file, naming the file and line and the remedy: put the why in `docs/knowledge` or `bee decisions log`; a workaround is fixed or filed with `bee backlog add`. Workers outside the hooked runtimes are caught by the D2 ratchet. | Recommended reading. The architecture tier: impossible at write time for hooked agents. |
| D4 (0ee8248d) | The why a comment used to carry has two homes and no third: a `docs/knowledge` concept (its Pointers section names the code by path) or a `bee decisions log` entry. Code cites nothing inline; the knowledge concept cites the code. When a comment is removed and its why has no home yet, the why moves to one of those homes in the same change. | Recommended reading. One fact, one home. |
| D5 (002a935d) | The rule lands in the same doctrine homes the fix-at rule uses: `packages/bee/AGENTS.block.md` (rendered into `AGENTS.md` and hosts) and `packages/bee/prompts/worker-cell.md`, each stating the ban, the three exceptions, and the two homes for the why. | Recommended reading; touches mistake-fix-at D4 (`5c43da33`). |
| D6 (69326d15) | Removing the existing comments is separate work, batched by module through grooming, after this feature ships: each batch moves every un-homed why into `docs/knowledge` or the decision log, deletes the comments, and lowers the ratchet baseline; no batch deletes a why that has no home yet. | User's answer. |
| D2b (0add770d, touches D2) | The baseline is seeded once, from the per-file maximum over main HEAD and every unmerged `wt/*` branch at seed time, so a sibling feature that merges after this one lands cannot raise a file above its baseline. After those merges the verb's `--write` lowers every entry that sits above the tree. D2's rule stands: the verb never raises a count. | Hat wave finding: five unmerged branches carry about 256 added comment lines under the code roots. |
| D7 (e3bf4a57) | The comment guard is switched by one config key, `no_code_comments` (boolean, default false): this repository's `.bee/config.json` sets it true; a host that onboards bee gets the hook code and the doctrine line but the guard refuses nothing there until the host opts in. The ratchet test and its baseline are this repository's own and never ship to a host. | Recommended reading. `scripts/` exists in most hosts; an always-on hook would refuse comments in a host's own code the moment it installed bee. |

### Agent's Discretion

The baseline file's name and shape (a tracked JSON beside
`.bee/doc-deferral-baseline.json` is the precedent; per-file counts,
sorted, one normalization for seeding and matching); the dev verb's
spelling (`bee dev comment-baseline --write|--check` or a step inside
`bee dev regen`); how the ratchet test finds code files (the D1 path
list is the authority — declared once, read by the test and the hook);
how the hook decides "adds a comment line" for an Edit (the new_string's
comment lines minus the old_string's, so a pure move or delete passes)
and for a Bash write (heredoc or redirect into a code path carrying a
comment line); the exact refusal wording; whether a `SAFETY:` line
must sit directly above `unsafe`. Constraints: the hook refuses only
under a code path, never a `.md`, `.json`, or `.toml`; the ratchet never
counts lines inside string literals as comments where the parser can
tell, and where it cannot, a false red is fixed by rewriting the string,
never by an allowlist; no baseline entry may ever be raised; a
refusal names the remedy; hosts that onboard bee get the doctrine line
and the hook, never the baseline (the baseline is this repo's).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| comment line | A line whose first non-blank characters open a comment in that file's language: `//`, `///`, `//!`, `/*`, or `*` inside a block comment for Rust; `#` for shell and python. |
| code file | A file under one of the D1 paths whose extension or shebang names Rust, shell, or python. |
| exception | A shebang line, a `SAFETY:` line on an unsafe block, or a license header at file top — never counted, never refused. |
| baseline | The committed per-file comment-line counts the ratchet compares against; it only goes down. |
| ratchet | The declared-suite test: red when any file's count exceeds its baseline, or a code file absent from the baseline carries a comment. |
| why | The rationale a comment used to carry; its homes are a `docs/knowledge` concept or a decision log entry. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/hooks/write_guard/` — `mod.rs`, `checks.rs`, `guards.rs`, `detectors.rs`, `paths.rs`, `tests.rs`: the tool-call parser (Edit/MultiEdit/Write/Bash/apply_patch) and the refusal path D3 extends.
- `packages/bee-rs/crates/bee/tests/hook_contracts.rs` — `a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr` and siblings: the end-to-end shape a new refusal test copies.
- `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:1318-1351` — the doc-deferral baseline: one normalization function, a tracked JSON, seed-and-match.
- `packages/bee-rs/crates/bee/src/devtools/mod.rs:95-103` — the dev verb dispatch (`render-prompt`, `release-manifest`, `regen`); a new verb lands here and in the hand-edited registry payload (decision 3358743e).
- `packages/bee-rs/crates/bee/src/catalog.rs` `PINNED_FLAG_COUNT` — a flag added for the new verb bumps it with a reason.
- `packages/bee/AGENTS.block.md` bullet "Write a mistake down the MOMENT…" and `packages/bee/prompts/worker-cell.md` Result-form paragraph — the D5 homes; `bee dev regen` renders `AGENTS.md` and `.bee/bin/prompts/worker-cell.md` from them (mistake-fix-at plan claims 40–42).

### Established Patterns

- A refusal names its remedy in the same message (every write-guard deny).
- A committed baseline is seeded and matched by ONE function (doc-deferral D1).
- A dev verb that writes has a `--check` twin that only reads (`release-manifest --write` / `--check`).
- Doctrine edits go through the template and regen, never a hand edit of `AGENTS.md`.

### Integration Points

- `.bee/verify/verify-app/features/` — a new feature file for the comment guard (how a user meets the refusal and the ratchet), plus the README index.
- `docs/knowledge/areas/` — the concept that owns the write guard and the one that owns the test suite / dev verbs must state the rule and the baseline.
- The vendored binary: a hook change is inert until `.bee/bin/bee` is reinstalled after merge (pattern 20260805).

## Canonical References

- `docs/REFs/pstack-rules.md` — the talk; the comment ban and its reason.
- `docs/history/mistake-fix-at/CONTEXT.md` — D4 (`5c43da33`): the doctrine homes this feature reuses.
- `docs/knowledge/areas/workflow-state/` — the write guard's owning concept (planning locates the exact file).

## Outstanding Questions

<!-- bee:not-a-deferral: these are planning questions answered by the plan, not promises -->
### Deferred To Planning

- [ ] Whether the ratchet test lives in the bee crate's unit tests or as an integration target under `tests/` — planning picks the one the declared suite already runs.
- [ ] Whether `scripts/release.sh` and `.bee/verify/verify-app/control-bee` (no extension, bash shebang) are found by extension or by shebang — the D1 path list plus shebang detection decides.
<!-- /bee:not-a-deferral -->

## Deferred Ideas

<!-- bee:not-a-deferral: out-of-scope work already recorded as decisions; no promise beyond D6 -->
- The batched removal of the existing 47,000 comment lines (D6) — grooming work after this ships; each batch lowers the baseline.
<!-- /bee:not-a-deferral -->
