---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: gather-reads-the-read-slot

Mode: `standard` — 2 risk flags: covered-contract-change (tests pin
`slot_for_kind("gather") == "generation"`), public-contracts (the
`dispatch prepare` JSON's `logical_tier` and marker bytes move for hosts
that configure `read`).
Why this is the least workflow that protects the work: the change is one
slot lookup plus the tables that restate it, but it moves which model every
host's gathers run on, and a wrong-model dispatch completes clean — so it
needs a red test first, a claims table, and the hat wave, not a tiny cell.

## Requirements (from CONTEXT.md)
- D1: `--kind gather` with no `--role` resolves `[read, generation]` — never through `extraction`; the winning slot's shape is obeyed (pane or subagent); a host with no `read` key is byte-identical to today. `cell_role_list("read")` stays as it is, with a comment naming why the two read lists differ.
- D2: marker and `economics.logical_tier` carry the WINNER on the cell-role path (already) and the default-gather path (new); every other path keeps today's marker bytes.
- D3: the agent comes from the ASKED name and the kind — `cell` → `bee-build`; `gather` with no `--role` → `bee-gather`; else `pinned_agent_type(<asked>)` — so `--role extraction|read` keeps `bee-extract` (B11), `--kind reviewer` keeps `bee-review` on a review-less host.
- D8: the herding-fallback contract keys on the winner on both sides (prepare's `payload.fallback`, the guard's `configured_model_set`); a `read` winner has no built-in default and publishes none; guard FIX text that names a kind appends `--role <role>`.
- D4: `AGENT_ROLES_BY_NAME["bee-gather"] = ["read", "generation"]`; template body names the read role; `ROLE_AGENTS` untouched.
- D5: every agent template `description:` opens by naming `bee dispatch prepare` as the only door.
- D7: `bare_dispatch_denied`'s FIX leads with `bee dispatch prepare` and derives its agent list from `ROLE_AGENTS` (no retyped `bee-gather = generation`).
- D6: Claude agent files stay unconditional (agent-model-unpin D1/D2 cited, not reopened).

## Load-bearing claims
Labels: `read` (opened the file at that line), `ran` (executed the command). Evidence is a verbatim byte substring of the anchored line(s).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A gather resolves the `generation` slot today | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:57 | `"cell" \| "gather" => Some("generation"),` |
| 2 | The tier-shaped walk has one list rule today (review only), so `read` walks `[read]` alone | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:655 | `if slot == "review" {` |
| 3 | The read job's fall-through list already exists on the cell path | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:362 | `if role == "read" { &["extraction", "generation"] }` |
| 4 | The default gather's agent is chosen by `pinned_agent_type(marker_role)`, so a `read`/`extraction` winner would pick `bee-extract` unless the gather kind pins its own | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1533 | `if kind == "cell" { "bee-build" } else { pinned_agent_type(marker_role) };` |
| 5 | The non-cell path walks `tier_role_list(tier_token)` and only the cell path records the winner | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1452-1454 | `if from_role { cell_role_list(tier_token) } else { tier_role_list(tier_token) };` / `if from_role {` |
| 6 | The seeded config describes `read` as the gather slot | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:232 | `"read": { "model": "haiku", "description": "multi-file gathers and scans, read-only" },` |
| 7 | `bee-gather` declares only `generation` (opencode pin + drift check walk this) | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:386 | `("bee-gather", &["generation"]),` |
| 8 | The template body still says the tier-era role | read | packages/bee/agents/bee-gather.md.tmpl:7 | `You run at the **generation** tier` |
| 9 | `read` is an alias of `extraction` for the agent lookup only | read | packages/bee-rs/crates/bee/src/verbs/drivers/guard.rs:240 | `[("read", "extraction"), ("code", "generation")];` |
| 10 | Claude agent files render unconditionally by a locked decision (D6) | read | packages/bee-rs/crates/bee/src/onboard/agents.rs:230 | `// agent-model-unpin D1/D2: a known agent renders UNCONDITIONALLY —` |
| 11 | The guard's FIX derives a kind from `slot_for_kind` by equality, so `generation` stops mapping to `gather` once the slot moves unless the lookup walks the list | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:782 | `.find(\|kind\| crate::verbs::drivers::slot_for_kind(kind) == Some(role))` |
| 12 | On this host a default gather is a pane today, travelling as `generation` | ran | `.bee/bin/bee dispatch prepare --runtime claude --kind gather --json` | `"tool": "Bash",` / `"logical_tier": "generation",` |
| 13 | An explicit model-shaped role on a gather returns the `bee-gather` subagent today | ran | `.bee/bin/bee dispatch prepare --runtime claude --kind gather --role code --json` | `"subagent_type": "bee-gather",` / `"model": "opus"` |
| 14 | A test pins the old slot and must turn red first | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2923 | `assert_eq!(slot_for_kind("gather"), Some("generation"));` |
| 15 | A guard test pins the derived kind for `generation` | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:2684 | `assert_eq!(dispatch_kind_for_role("generation"), Some("gather"));` |
| 16 | Doctrine already binds `--role extraction` on a gather to `bee-extract` (B11, kept by D3) | read | docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md:143 | `` `--kind gather --role extraction`, which resolves to `bee-extract` on the `` |
| 17 | The legacy-host fixture configures `extraction` + `generation` and no `read`, and a test pins its gather on `generation`'s model — so the walk must not pass through `extraction` | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2997 | `r#"{"models":{"claude":{"extraction":"haiku","generation":"sonnet","review":"opus"}}}"#;` |
| 18 | The bare-dispatch FIX retypes the agent→role pairs and teaches hand-naming first (D7) | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1120 | `"FIX: name one of bee's rendered agents in subagent_type (bee-gather = generation, \` |
| 19 | A test pins that old wording | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:1794 | `assert!(stderr.contains("bee-gather = generation")` |
| 20 | `read` has no built-in default, so a marker saying `read` on a host with no `read` key names nothing the guard knows (why D2 records the winner) | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:110-112 | `m.insert("extraction".into(), Value::String("haiku".into()));` / `m.insert("generation".into(), Value::String("sonnet".into()));` / `m.insert("review".into(), Value::String("opus".into()));` |
| 21 | Prepare gates the herding fallback on a closed slot list that lacks `read` (D8) | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1741-1743 | `if let Some(model) = CONFIGURABLE_SLOTS` / `.contains(&marker_role)` / `.then(\|\| default_models(runtime).get(marker_role).cloned())` |
| 22 | That closed list is `extraction, generation, review` | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:59 | `pub(crate) const CONFIGURABLE_SLOTS: [&str; 3] = ["extraction", "generation", "review"];` |
| 23 | The guard's fallback mirror keys the default on the kind's HEAD slot, which becomes `read` and would stop admitting `generation`'s default (D8) | read | packages/bee-rs/crates/bee/src/hooks/model_guard.rs:609 | `let Some(slot) = crate::verbs::drivers::slot_for_kind(kind) else { continue };` |

## Discovery
Ran `dispatch prepare` for `gather`, `gather --role code`, `gather --role read`, `reviewer`, `advisor` on this host: the door already returns Bash for a herding slot and Agent for a model slot (claims 12, 13). Read the resolver chain end to end (claims 1–5) and the agent-surface tables (6–10). The pane gather's digest is at `.bee/mailbox/job-1788320911110/report-1.md`. Finding: the transport IS dynamic; the one static binding is `gather = generation`, a tier-era leftover the role split never moved.

## Approach
Recommended (D1–D5): move the gather kind's default slot to the read job and let the existing fall-through walk carry every host that never configured it; pin the default gather's agent by kind; make the agent tables and template say the same job; put the door in every agent description.
Rejected: (a) render Claude agent files by reachability — reopens agent-model-unpin D2 and on this host removes only `bee-extract`; (b) make the model-guard rewrite an `Agent` call into a `Bash` pane — a PreToolUse hook can change tool input, never the tool; (c) delete the agent files and inline contracts into prepare's prompt — loses the read-only tool permission the files carry (D6).
Risk map: `slot_for_kind`/`tier_role_list` — MEDIUM (wrong-model gathers on hosts that configure `read`; proof: red-then-green unit tests for the walk and for a host with no `read` key) · agent pin by kind — LOW (existing prepare tests for `bee-gather`/`bee-extract`) · guard FIX kind lookup — LOW (test at model_guard.rs:2684 extended) · templates/regen — LOW (`bee dev regen` leaves no diff, `cargo test` agents_block_render_parity).

## Shape
Playbook: `bugfix` (`references/planning-reference.md` "Class playbooks" › bugfix) — red first: a test that a `read`-configured host still routes its default gather to `generation` fails for that reason, then the fix.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 — resolver | `slot_for_kind("gather")` → `read`; `tier_role_list("read")` → `[read, generation]`; the default-gather path records the winner; a `--kind gather` with no `--role` pins `bee-gather`; `dispatch_kind_for_role` walks the kind's list and its FIX appends `--role <role>`; fallback keyed on the winner on both sides (D8); `bare_dispatch_denied` FIX leads with prepare (D7) | the one static binding | `dispatch prepare --kind gather` on a `read: sonnet` host → Agent bee-gather sonnet; on a `read: {kind:herding}` host → Bash pane; on a host with no `read` → identical bytes to today | 2 |
| 2 — surface | `AGENT_ROLES_BY_NAME["bee-gather"]`; template body + all four `description:` lines (D5); `bee dev regen`; docs parity (workers.md table + quoted FIX at line 14, dispatch.md kinds line, doctrine B14) | tables must say what the resolver does | `bee dev regen` clean; `bee status` shows no agent-file drift | close |

Current slice: both phases — three cells; the resolver cell (src) and the docs cell (docs) are file-disjoint and run concurrently; the surface cell waits on the resolver cell for the list.

## Test matrix
- Happy: `read: sonnet` host, `--kind gather` → `tool: Agent`, `subagent_type: bee-gather`, `model: sonnet`, marker `[bee-tier: read]`, `logical_tier: read`.
- Happy (pane): `read: {kind:herding, agent:x}` → `tool: Bash`, command carries `--agent "x"`, `logical_tier: read`.
- Edge: no `read` key (`extraction: haiku, generation: sonnet, review: opus`) → resolves `generation`, marker `[bee-tier: generation]`, `subagent_type: bee-gather`, `model: sonnet` — byte-identical to today's envelope (the existing `absent_role_every_dispatch_is_byte_identical` test stays untouched and green).
- Edge: `read: null` (explicitly off), `generation: sonnet` → `generation` wins, `bee-gather` still the agent.
- Edge (D3): no `review` key → `--kind reviewer` is byte-identical to today (`[bee-tier: review]`, `subagent_type: bee-review`, model sonnet via the walk) — the winner is NOT recorded on that path.
- Edge (D8, prepare): `read: {kind:"herding", agent:"x", fallback:"default"}` → Bash pane, NO `payload.fallback` (read has no built-in default), `logical_tier: read`.
- Edge (D8, guard): no `read` key, `generation: {kind:"herding", fallback:"default"}` → `configured_model_set` still contains `sonnet`; an Agent dispatch with `model: "sonnet"` and no marker is allowed.
- Edge (D8, FIX): a `[bee-tier: generation]` herding refusal names `--kind gather --role generation`; a `[bee-tier: read]` one names `--kind gather --role read`; `extraction` still names no kind.
- Edge (order): `dispatch_kind_for_role("generation")` answers `gather`, not `reviewer` — `DISPATCH_KINDS` order is load-bearing and pinned by that test.
- Surface (opencode): `models.opencode.read = "opencode/x"` pins `x` into `.opencode/agent/bee-gather.md`; no `read` key → generation's model as today; `read: {kind:"cli"}` removes the file exactly as a cli `generation` does today (same class, different slot).
- Surface (records): the sync record's `rendered_from` key and the drift finding's `slot` for `bee-gather` move `generation` → `read` (head of the declared list); `bee dev regen` rewrites `.bee/onboarding.json` accordingly.
- Error (D7): a bare `Agent` dispatch's FIX opens with `bee dispatch prepare` and contains no `bee-gather = generation`.
- Live (verify-app, at cap of the resolver cell): on a scratch host with `read: haiku, generation: sonnet`, the real binary's `dispatch prepare --kind gather --json` returns `bee-gather`/`haiku`/`[bee-tier: read]`, and `--kind reviewer --json` returns `bee-review`; on the same host with `read: {kind:"herding", agent:"x"}` it returns the Bash pane command.
- Edge: `--kind gather --role extraction` → `bee-extract` (unchanged, B11).
- Error: `dispatch_kind_for_role("generation")` still `Some("gather")`; `("read")` → `Some("gather")`; `("review")` → `Some("reviewer")`.
- Surface: `render_opencode_agent_template("bee-gather")` pins the `read` slot's model when set; `validate_agent_files_drift` on opencode follows; `agents_block_render_parity` green; `bee dev regen` no diff.

## Open Questions
(none — the hat wave's blockers G1–G3 and warnings G4–G9 are each answered by a decision or a matrix row above; record: docs/history/gather-reads-the-read-slot/reports/hat-wave.md)

## Out of scope
- Teaching `role_for_agent`/`ROLE_AGENTS` a role list (the bare-name guard gap named in CONTEXT.md).
- Reachability-based rendering of Claude agent files (agent-model-unpin D2 stands).
- workers.md line 38's stale `model:` prose.
