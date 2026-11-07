# Workspace protocol, dispatch receipts, brief discipline — Context

**Feature slug:** workspace-protocol
**Date:** 2026-11-07
**Shaping session:** complete (gate bypass full; owner ask "Làm 1,2,3" after a read of the slp workspace protocols)
**Scope:** Standard
**Domain types:** CALL | READ

## Feature Boundary

Three ideas from the slp projects (`/mnt/data/Projects/refs/slp/`: paseo-pi-team,
paseo-room, paseo-doctrine-downstream) come into bee:

1. A host project keeps one root file, `WORKSPACE_PROTOCOL.md`, with the
   human decision boundaries, the merge owner and the deploy owner as
   key-value lines. bee validates it and reports its state. It never gates.
2. Every `bee dispatch prepare` record carries a digest of the prompt it
   built and of the protocol file at that moment. A leader can check later,
   for example after a resume, whether either one changed.
3. Brief discipline: a worker never stops to offer a menu of options, and a
   reviewer or advisor brief never carries the leader's leaning. The
   leaning check on `--purpose` warns and never refuses.

Out of scope: a gate on a missing protocol (paseo's graduated admission),
a session preamble line, the paseo context budget rule.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The protocol lives at `<repo root>/WORKSPACE_PROTOCOL.md`. bee parses every line `^([A-Z][A-Z0-9_]*):[ \t]*(.*)$`, inside or outside a fenced block. A key with an empty value takes the following `- <item>` lines (leading spaces allowed) as its list; the list ends at the next key line or the first blank line. Values are trimmed; a trailing ` # comment` stays part of the value. Unknown keys are legal and kept, so a prose line such as `NOTE: x` becomes a harmless key. A key seen twice: the last one wins and the state gets the issue `duplicate key <KEY>`. | Same file name and key format as paseo-pi-team's template, so one file serves bee and Paseo. |
| D2 | The state is one of `missing` (no file), `unreadable` (I/O error or not UTF-8), `invalid` (issues found), `valid`. Required for `valid`: `MERGE_OWNER` non-empty, `DEPLOY_OWNER` non-empty, `HUMAN_DECISION_BOUNDARIES` with at least one item. Each failure is one issue string naming the key. A scalar `HUMAN_DECISION_BOUNDARIES: x` is an issue that says to use a `- item` list. A value or list item that still holds a template placeholder (it contains `<` and `>`, or ` | ` between choices) is an issue `<KEY> still holds a template placeholder`. | The three facts the leader must not guess. |
| D3 | Test commands have one home: `.bee/config.json` `commands.test`. When the protocol carries `FULL_TEST` with a non-empty value that differs from `commands.test` (a string; an array compares as its items joined with ` && `), that is an issue: `FULL_TEST drifts from .bee/config.json commands.test — delete FULL_TEST or make it equal`. Equal or empty is fine. | Single source of truth; the owner's "gather test commands" idea is narrowed on purpose. |
| D4 | New verb group `bee protocol`: `bee protocol check [--json]` prints the state, the issues, the parsed keys and the file sha256; exit code 0 for every state (it reports, never gates). `bee protocol init` writes the bee template to the root only when no file exists; an existing file is a typed refusal naming it, never an overwrite. The template ships filled so `init` then `check` gives `valid`: `WORKSPACE_PROTOCOL_VERSION: 1`, `PROJECT_CRITICALITY: medium`, `MERGE_OWNER: human`, `DEPLOY_OWNER: human`, `HUMAN_DECISION_BOUNDARIES` (product behavior changes; public API break; irreversible schema migration; secret or infrastructure mutation; deployment; cost above budget), `PROTECTED_AREAS` (one item `none`). The template holds no placeholder. `init` ends with one next-step line: review and edit `WORKSPACE_PROTOCOL.md` — the defaults are bee's guess, not the owner's choice. | paseo-pi-team reports and does not gate; the same here. |
| D5 | `bee doctor` always prints one row `workspace-protocol`: `missing` and `valid` have status `ok` (the missing detail says the file is optional and `bee protocol init` writes one), `invalid` and `unreadable` have status `advisory` with the issues and the fix `bee protocol check`. The row never makes `overall_status` blocked. | Hosts without a protocol must not go red. |
| D6 | Leader doctrine (the `.bee/LEADER.md` source and its vendored copies): when `WORKSPACE_PROTOCOL.md` exists the leader reads it in full before routing work, copies the boundaries a cell touches into that cell's brief, and names in the Gate 2 message (or, under bypass, in the auto-approval reason and the closing report) each listed boundary the plan touches. A boundary adds no new stop: whether that gate stops is the `gate_bypass` level's call, as for every gate. Workers never read the file. | slp readership rule: Lead reads, Peer gets an excerpt. The bypass table stays the one home for what stops (hat-user-impact F1). |
| D7 | Every record `bee dispatch prepare` appends to `.bee/logs/dispatch.jsonl` (every kind) gains `prompt_sha256` (sha256 hex of the exact text field the returned payload carries to the worker: `prompt` for an Agent or spawn payload, `stdin` for a Bash herding or cli payload, the codex message field where that is the carrier) and `protocol_sha256` (sha256 hex of the protocol file bytes at prepare time, `null` when missing or unreadable). Both prepare and verify read the protocol from the control root (the main checkout that holds the bee store), never from a worktree copy. The prepare JSON output carries both beside `dispatch_id`. Existing keys and the advisor `brief_sha256` keep their meaning. | The receipt is written at dispatch time by bee, not by the agent. |
| D8 | New verb `bee dispatch verify --dispatch-id <id> [--prompt-file <path>] [--json]`. It finds the newest prepare record with that id and reports `protocol`: `same` / `changed` / `appeared` / `removed` / `absent` (recorded vs current file digest) and `prompt`: `match` / `mismatch` / `not_checked` (`--prompt-file` bytes vs `prompt_sha256`; a record without `prompt_sha256` gives `unrecorded`). Verify reads the dispatch log and the protocol from the control root, from main or from a worktree alike; a `dispatch_not_found` refusal's fix names the control root it searched. A `--prompt-file` that is not UTF-8 is `mismatch` (the one `&str` hasher stays the only hasher). The file a leader passes is that carried field saved byte-exact, e.g. `jq -j '.payload.prompt // .payload.stdin'` over the prepare output; a test round-trips it. Unknown id is a typed refusal `{ok:false, reason:"dispatch_not_found"}`. Exit 0 whenever the record is found. | Lets a leader prove after a resume that the brief and the protocol are still the same. |
| D9 | `packages/bee/prompts/worker-cell.md` gains one contract line: never stop to offer the leader a menu of implementation options; choose inside the cell's scope and record the choice as a departure; a decision outside the scope is one short yes/no question through `[PARKED]` or `[BLOCKED]`. | paseo-room "Brief discipline". |
| D10 | `packages/bee/prompts/advisor.md` gains the line the reviewer prompt already has: the brief never carries the orchestrator's own verdict, leaning or expected findings. | Same rule for both read-only consult kinds. |
| D11 | For `--kind reviewer` and `--kind advisor`, except a `--role` starting with `hat-` (a hat purpose carries its perspective on purpose — gates-and-delegation "Hat wave"), prepare runs ONLY the leaning-stem arm of the existing rule (`first_leaning_stem_outside_tagged` in `brief_lint.rs`, exposed `pub(crate)` or through a thin wrapper — no copy of the stem list) over `--purpose`. The four-section shape arm of `lint_brief` never runs on a purpose. A hit never refuses: the output gains `purpose_leaning` (the lint's reason and fix), text mode prints it as one warning line, the dispatch record gets `purpose_leaning: true`, and the payload is built as before. The `--brief-file` refusal stays exactly as it is. | prepare.rs states why the guard must not refuse on `--purpose`: a false fire would refuse the Gate-3 advisor consult and deadlock high-risk work. Warn keeps that fence. |
| D12 | The paseo-room rule "do not narrow the review dimensions" is not taken. | bee's reviewer is lens-scoped by design (`reviewer.md` line 2, one lens per specialist). |

### Environment facts

- `prepare.rs` `append_prepare_record` (line ~1364) writes `.bee/logs/dispatch.jsonl`; current records carry `dispatch_id`, `kind`, `cell`, tier fields, no prompt digest.
- `brief_sha256` already exists for advisor `--brief-file` blind lanes; `lint_brief` lives in `verbs/drivers/brief_lint.rs:309`.
- `reviewer.md` line 4 already says the brief never carries the orchestrator's verdict, leaning or expected findings.
- paseo-pi-team `cli/lib/workspace-protocol.mjs` uses the same four states and treats missing keys as advisory; bee makes three keys required (D2).
