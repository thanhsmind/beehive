---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Pi native stage driver

## Summary

Today every bee worker on Pi is launched into a tmux pane. That pane is the
reason stages feel slow and lossy: the leader waits in the foreground, the
answer comes back as a one-line summary, and a seat that runs long is simply
gone. This work lets bee start a worker as a plain child process instead, with
no pane, and read its full answer back. Panes stay available for the jobs that
still need them.

Two smaller things ride along, because they live in the same file. The session
narrows the model's tool list to what the current stage allows, so it stops
trying calls that would be refused anyway. And when a session ends with work
still claimed, it says so by name instead of ending quietly.

Mode: `high-risk` — 6 risk flags: audit-security, authorization,
external-systems, public-contracts, covered-contract-change, multi-domain.
Why this is the least workflow that protects the work: the belt is bee's only
enforcement point on Pi, so a change there is a change to a trust boundary —
and the contract suite asserts its exact handler and command sets, so any
addition is a deliberate, reviewed edit rather than a silent one.

## Requirements (from CONTEXT.md)

- **D1** Native dispatch is the DEFAULT on Pi; herding is the fallback, not removed. Herding serves a write-capable cell in a worktree, a seat the user wants to watch live, and every role whose configured agent is not a `pi` binary. Both transports stay tested.
- **D2** The native transport is a child `pi` SUBPROCESS (`pi --mode json -p …`), never the in-process SDK and never a `--mode rpc` bridge.
- **D3** A role reaches the native path only when its configured agent is a `pi` binary; any other agent falls back to herding, by config, with no leader choice.
- **D4** Per-stage tool gating is a HARD gate via `pi.setActiveTools`; one slash command re-opens the full set.
- **D5** The close guard is WARN ONLY; `tool_call` stays the ONLY blocking surface in the Pi belt.
- **D6** `.bee/*.json` stays the single store; the pi-workflows engine is not adopted.
- **D7** `.pi/extensions/bee-guard.ts` is the hand-written source of truth; the Rust binary is rebuilt so `doctor`'s byte-compare agrees. `.opencode/plugins/bee-guard.ts` is not edited.
- **D8** One door: `bee dispatch prepare --runtime pi` gains a native arm beside herding; the leader never picks the transport.
- **D9** Claude, Codex and OpenCode behavior does not change.
- **D10** The hat wave contract holds unchanged: 3 seats (5 on high-risk), one wave, 10-minute ceiling, seat-named results, a late seat DROPPED and named.

## Load-bearing claims

Labels: `ran` = a command was executed this session and its bytes are quoted.
`read` = the file was opened at the cited line and its bytes are quoted.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A child `pi -p --no-session` loads `.pi/extensions/bee-guard.ts`, so the belt is active inside a native worker. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A1-A2 | `bee: hook prompt-context could not decide this payload — allowing the operation (fail-open).          The guard did NOT run on it.` |
| 2 | bee's session preamble reaches that child, so a native worker starts with bee context, not blank. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A3 | `"text":"Yes.\n\nPhase: \`idle\` \| Mode: \`none\`  \nFeature: \`pi-native-stage-driver\`  \nGates: \`none pending (no active work)\`"` |
| 3 | The child's stdout is JSONL preceded by non-JSON noise, so a parser must skip unparseable lines. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A4 | `mise ~/.config/mise/config.toml tools: pi@0.85.1` then `{"type":"session","version":3,"id":"01a0b519-…","cwd":"…/beehive--wt--pi-native-stage-driver"}` |
| 4 | The child settles normally and reports cost, so a budget can be enforced per worker. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A5 | `"usage":{"input":13924,"output":114,…"cost":{…"total":0.001253952112}},"stopReason":"stop"` then `{"type":"agent_settled"}` |
| 5 | Pi ships a working precedent for the exact spawn shape D2 requires. | read | `docs/history/pi-native-stage-driver/evidence.md` § B1 | `const args: string[] = ["--mode", "json", "-p", "--no-session"];` … `const proc = spawn(invocation.command, invocation.args, { cwd: cwd ?? defaultCwd, shell: false, stdio: ["ignore", "pipe", "pipe"], });` |
| 6 | `setActiveTools` narrows the model's list, so D4's hard gate is reachable. | read | `docs/history/pi-native-stage-driver/evidence.md` § B2 | `pi.setActiveTools(["read", "bash"]); // Switch to read-only` |
| 7 | The additive-only rule binds a loader tool's own execution, not an event handler, so narrowing from an event handler is legal. | read | `docs/history/pi-native-stage-driver/evidence.md` § B3 | `3. During loader execution, call \`pi.setActiveTools([...currentTools, ...matchingTools])\`. The change must be additive: do not remove currently active tools in the same call.` |
| 8 | pi 0.85.1 has no `session_stop`, so D5's warn-only close guard must hang on `agent_settled`. | ran | `docs/history/pi-native-stage-driver/evidence.md` § B4 | count `0`, beside `Use \`agent_settled\` for status integrations that need to know Pi will not continue running automatically.` |
| 9 | The door refuses every non-herding resolution for Pi today; that refusal is the one place D8 changes. | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2237-2243` | `if runtime == "pi" && !matches!(resolved, Resolved::Herding { .. }) { return Ok(Prepared::Value(pi_requires_herding_refusal(marker_role, &resolved, is_escalated))); }` |
| 10a | `doctor` embeds the belt at compile time and byte-compares the on-disk file, so D7's rebuild step is mandatory. | read | `packages/bee-rs/crates/bee/src/doctor.rs:47` | the file is pulled in with `include_str!` and compared at `:310-312` |
| 10b | No generator writes the belt, so the checked-in copy is the source of truth. | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:529-547` | the runtime match returns `None` for `"pi"`, so `bee dev regen` renders nothing for it |
| 11 | The contract suite parses the belt source and asserts its handler and command sets, so every added `pi.on` / `registerCommand` needs a matching fixture row. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:107` | the suite `include_str!`s the belt, then derives the `pi.on("<event>"` set at `:285-313` and the `pi.registerCommand("<name>"` set at `:315`+ |
| 12 | `team.pi` routes most roles to a non-`pi` agent, so D1/D3's fallback is forced by config, not preference. | ran | `.bee/bin/bee team show --runtime pi --json` | `code/read/test/docs/extraction/generation/supervisor/lane-3 → gemini-3.8-flash-high (herding: agy-flash)`; only `plan/review/advisor/hat-*/lane-1/lane-2` run a `pi` agent |
| 13a | A brand-new feature inherits the previous feature's role plan, so the dispatch door refuses every non-cell dispatch. | ran | `docs/history/pi-native-stage-driver/evidence.md` § C1 | `"reason":"stage_required"`, then with `--stage read-only-gather`, `"reason":"stage_not_applicable"` |
| 13b | The cause is that the stored packet keeps its own, different feature name. | ran | `docs/history/pi-native-stage-driver/evidence.md` § C2 | `packet.feature : release-2-41-2` while `state.feature : pi-native-stage-driver` |
| 13c | The lookup never compares that field, so the fix has one home. | read | `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:641-649` | the state branch matches only `m.get("feature") == Some(feature)` before returning `approved_cell_packet` |

## Discovery

The one unknown that could have killed the shape — whether bee's guard survives
inside a native child — was settled by running a real child from this worktree
(claims 1–4). The belt loads, the preamble arrives, the process settles and
reports cost. Pi's own shipped subagent example gives the exact spawn shape
(claim 5), so D2 copies a working precedent rather than inventing one.

One finding changed the slice order. Starting this feature left the previous
feature's approved packet in `.bee/state.json`, so the dispatch door refuses
every non-cell dispatch for a brand-new feature (claim 13). That is a red base
under this feature's own execution, so it becomes cell 1 — fix first, then build.

## Approach

**Recommended path.** Four slices, walking-skeleton first.

Slice 1 fixes the stale-packet bug (claim 13) so the door works at all. Slice 2
is the walking skeleton: one registered tool that spawns one child, returns one
full answer, end to end, real behavior, no stubs — plus the door arm (D8) and
the config slot (D1, D3). Slice 3 makes it a fan-out: parallel seats, the
10-minute ceiling, seat-named results, drop-and-name (D10). Slice 4 adds the two
session-surface behaviors that share the file: the hard tool gate (D4) and the
warn-only close guard (D5).

Rejected alternatives, one line each:
- In-process SDK (`createAgentSession`) — rejected by D2; no `dist/*.d.ts` on this host to pin against.
- `pi --mode rpc` bridge — rejected by D2; stable protocol but no shipped example and a bridge to maintain.
- Replacing herding on Pi — rejected by D1; `team.pi` routes most roles to a non-`pi` agent (claim 12).
- Doing the tool gate and close guard first — rejected; they are session polish, and the transport is what the user actually reported.

**SMALLER PATH check.** Is there a cheaper shape that still honors every locked
decision? Considered: ship slice 2 alone and stop. It would honor D2, D6, D7,
D8 and D9 — but D10 names the hat wave budget and drop-and-name explicitly, and
D4/D5 are locked decisions, so stopping early would quietly shrink the agreed
scope. FAIL on scope integrity, not on cost. Kept at four slices, with slice 1
justified by a reproduced red base rather than by preference.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Stale approved packet | MEDIUM | `pnsd-1` | a red-first test that a new feature does not inherit a prior packet |
| Belt gains a registered tool | HIGH | `pnsd-2` | the new tool has an explicit `mapToolCall` row and the fail-safe default test stays green |
| Door gains a native arm | HIGH | `pnsd-3` | Claude/Codex/OpenCode payloads byte-unchanged; pi native arm covered |
| Parallel seats + ceiling | MEDIUM | `pnsd-5` | a late seat is dropped AND named, never silently lost |
| Hard tool gate | MEDIUM | `pnsd-6` | off-stage tool absent from the active set; the re-open command restores it |
| Close guard | LOW | `pnsd-7` | warning names the cell; the session still ends |
| Binary/byte drift | MEDIUM | every belt cell | `doctor --runtime pi` reports `ready` after a rebuild |

Waves: `pnsd-1` runs alone (it unblocks dispatch). Then `pnsd-2` and `pnsd-3`
run in parallel — different files, no overlap. `pnsd-4` is serial after both
(it is the end-to-end proof that needs them). `pnsd-5`, `pnsd-6` and `pnsd-7`
all touch the belt, so they run serially after `pnsd-4`, in that order.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "planning", "classification": "required", "role": "plan", "reason": "High-risk lane: the shape and its slices need plan-altitude reasoning."},
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "Every slice writes TypeScript in the belt or Rust in the door."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "The contract suite asserts the belt's handler and command sets; each belt change needs its fixture row red-first."},
    {"stage": "documentation-and-capture", "classification": "required", "role": "docs", "reason": "Two mapped verify features (pi-runtime, pi-hat-wave) change and must be re-synced."},
    {"stage": "read-only-gather", "classification": "required", "role": "read", "reason": "Multi-file hunts across the belt, the door and the contract suite."},
    {"stage": "fact-extraction", "classification": "conditional", "role": "extraction", "condition": "a single already-located fact is needed during execution", "reason": "Cheap tier for narrow lookups only."},
    {"stage": "generation-fallback", "classification": "conditional", "role": "generation", "condition": "a role with no configured slot is requested", "reason": "Fallback only; never selected directly."},
    {"stage": "independent-review", "classification": "conditional", "role": "review", "condition": "the user invokes a review", "reason": "Review is user-invoked, never automatic."},
    {"stage": "generic-advisor", "classification": "required", "role": "advisor", "reason": "High-risk work owes an advisor consult; the hat-wave synthesis serves it."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Single-leader feature; no supervised multi-session run."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No convergence lane: the shape is settled, not contested."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No convergence lane."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No convergence lane."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "High-risk: 5 seats. This seat checks the claims table against the code."},
    {"stage": "hat-risks", "classification": "required", "role": "hat-risks", "reason": "The belt is a trust boundary; the risk seat is the one that must not be skipped."},
    {"stage": "hat-value", "classification": "required", "role": "hat-value", "reason": "Five Pi features already shipped without fixing this; the value seat tests whether the sixth is different."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Carries the SMALLER PATH question against the four-slice shape."},
    {"stage": "hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "D4's hard tool gate changes what the user can do in every turn."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release is cut by this feature; the release script owns that."}
  ]
}
```

## Shape

**Slice 1 — unblock the door.** `pnsd-1`.
**Slice 2 — walking skeleton.** `pnsd-2`, `pnsd-3`, then `pnsd-4` end to end.
**Slice 3 — fan-out.** `pnsd-5`.
**Slice 4 — session surface.** `pnsd-6`, `pnsd-7`.

Only slice 1 and slice 2 are previewed as cells below. Later slices keep
one-line headlines and become cells when their slice starts.

Slice 3 headline: parallel seats, 10-minute ceiling, seat-named results,
drop-and-name a late seat (D10).
Slice 4 headlines: hard per-stage tool gate with a re-open command (D4);
warn-only close guard naming the uncapped cell (D5).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `pnsd-1` | Scope the approved plan packet to its own feature | `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs`, `packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs` | — | Starting a new feature right after another one no longer refuses every advisor, gather and hat dispatch with `stage_not_applicable` | red-first test: a packet whose own `feature` differs from the active feature is not returned; `cargo test -p bee state_group` green |
| `pnsd-2` | Register the native dispatch tool in the Pi belt | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` | `pnsd-1` | A Pi session can start one bee worker with no tmux pane and get its full answer back | `cargo test -p bee --test pi_plugin_contracts` green, including the unchanged fail-safe `default:` assertion and a new explicit `mapToolCall` row for the tool |
| `pnsd-3` | Give the dispatch door a native arm for Pi | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, `packages/bee-rs/crates/bee/src/config.rs` | `pnsd-1` | `bee dispatch prepare --runtime pi` returns a native payload for a `pi`-agent role and still returns herding for every other role | `cargo test -p bee` green; a byte-equality test that the claude and codex payloads are unchanged (D9) |
| `pnsd-4` | Drive one native worker end to end and record the evidence | `.bee/verify/verify-app/features/pi-runtime.md` | `pnsd-2`, `pnsd-3` | `doctor --runtime pi` still reports `ready`, and the verify recipe drives a real native dispatch | `green:live` — the verify recipe run against a launched sandbox, evidence attached |

```json
[
  {
    "id": "pnsd-1",
    "feature": "pi-native-stage-driver",
    "title": "Scope the approved plan packet to its own feature",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["D8"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs",
      "docs/history/pi-native-stage-driver/evidence.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Make an approved plan packet belong to the feature that produced it. Today get_approved_preview_packet returns whatever packet sits in the store as long as the ACTIVE feature name matches the requested one; it never reads the packet's own `feature` field, so a new feature inherits the previous feature's role plan and every non-cell dispatch then refuses. Read evidence.md section C for the reproduction and the exact refusal strings. Find the two return sites by searching for `approved_cell_packet` in plan_packets.rs — one reads the lane file, one reads state.json; carry the enclosing function name, not a line number. At each site, if the stored packet carries a non-empty `feature` field that differs from the requested feature, treat it as absent and return None. A packet with no `feature` field at all keeps today's behavior, so older stores do not break. Write the test red first: assert that a packet whose own feature is \"other-feature\" is NOT returned when the active feature is \"this-feature\", watch it fail for that reason, then fix. Do not change how a matching packet is returned, and do not touch the gate or dispatch code (per D8: the door keeps one shape).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee plan_packets",
    "must_haves": {
      "truths": [
        "A packet whose own feature field differs from the active feature is not returned",
        "A packet with no feature field is still returned, so existing stores keep working",
        "Starting a new feature after another one no longer refuses a non-cell dispatch with stage_not_applicable"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "both packet return sites compare the packet's own feature field; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs", "substantive": "a test that fails before the fix for the mismatched-feature reason"}
      ],
      "key_links": ["get_approved_role_plan still reads through get_approved_preview_packet, unchanged"],
      "prohibitions": [
        "No change to prepare.rs or any gate verb",
        "No change to the shape of a packet that does match"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "pnsd-2",
    "feature": "pi-native-stage-driver",
    "title": "Register the native dispatch tool in the Pi belt",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["pnsd-1"],
    "decisions": ["D2", "D5", "D7", "D9"],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "docs/history/pi-native-stage-driver/evidence.md",
      "docs/history/pi-native-stage-driver/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Give the Pi belt one registered tool that runs a bee worker as a child pi process, with no tmux pane. Copy the spawn shape from evidence.md section B1 verbatim in spirit (per D2): argv starts [\"--mode\",\"json\",\"-p\",\"--no-session\"], then --model, optionally --thinking, then --tools, then the task as the final positional; spawn with shell:false and stdio [\"ignore\",\"pipe\",\"pipe\"] and an explicit cwd. Parse stdout as JSONL and SKIP any line that does not parse — evidence.md section A4 shows a non-JSON mise banner arrives first. Return the child's full assistant text, never a one-line summary (per D4 of pi-stage-dispatch, and the reason this feature exists). Surface a non-zero exit with the child's stderr attached; never report a silent success. Add the new tool name as an EXPLICIT case in mapToolCall rather than letting it fall to the default arm, and keep `hook: \"write-guard\"` for it — the contract suite asserts the default arm has no null return and at least two write-guard literals, and it derives the pi.on and registerCommand name sets from this source, so add the matching fixture rows in pi_plugin_contracts.rs in the same cell. The belt keeps exactly two failure policies (per D5): tool_call stays the only blocking surface, everything this cell adds is advisory and swallows its own errors. Do not add a pi.on handler in this cell. Do not edit .opencode/plugins/bee-guard.ts (per D7). Write the contract test rows first and watch them fail before the belt edit.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "A Pi session can run one bee worker with no tmux pane and receive its full answer",
        "Unparseable stdout lines are skipped and the answer still returns",
        "A non-zero child exit surfaces the child's stderr to the leader",
        "The new tool has an explicit mapToolCall row routed to write-guard"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "one registered tool that spawns and parses a child pi process; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "fixture rows for the new tool name and its mapToolCall route"}
      ],
      "key_links": ["mapToolCall routes the new tool name explicitly, not through its default arm"],
      "prohibitions": [
        "No new blocking surface: tool_call stays the only one",
        "No edit to .opencode/plugins/bee-guard.ts",
        "No change to the existing five pi.on handlers",
        "No second returner added to tool_result"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "pnsd-3",
    "feature": "pi-native-stage-driver",
    "title": "Give the dispatch door a native arm for Pi",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["pnsd-1"],
    "decisions": ["D1", "D3", "D8", "D9"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "docs/history/pi-native-stage-driver/CONTEXT.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Turn the Pi herding-only refusal into a per-slot arm (per D8). Find it by searching prepare.rs for `pi_requires_herding_refusal`; the guard reads `if runtime == \"pi\" && !matches!(resolved, Resolved::Herding { .. })` and refuses every other resolution. Replace the blanket refusal with this rule: when the role's configured agent is a pi binary, return the native payload the belt's tool from pnsd-2 consumes; otherwise keep returning today's herding payload byte-for-byte (per D1 and D3). Decide 'is a pi binary' from the configured agent command in herding.agents — the first argv element being `pi` — never from the role name, and never from a leader-supplied flag: the leader must not be able to pick the transport (per D8). A role that resolves to neither still refuses, with the existing reason string unchanged. Claude, Codex and OpenCode payloads must come out identical to main (per D9); add a test that asserts that byte-equality rather than trusting review. Keep the pi-only detached_delivery note on the herding arm only.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee prepare",
    "must_haves": {
      "truths": [
        "A pi-agent role returns a native payload on runtime pi",
        "A non-pi-agent role returns today's herding payload, unchanged",
        "Claude and Codex payloads are byte-identical to main",
        "The leader cannot select the transport by flag"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "per-slot native arm beside the herding arm; no TODO stubs"}
      ],
      "key_links": ["the native arm reads the configured agent command, not the role name"],
      "prohibitions": [
        "No new CLI flag that selects a transport",
        "No change to the claude or codex payloads",
        "No change to the existing refusal reason string for an unresolvable role"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "pnsd-4",
    "feature": "pi-native-stage-driver",
    "title": "Drive one native worker end to end and record the evidence",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": ["pnsd-2", "pnsd-3"],
    "decisions": ["D7", "D10"],
    "files": [
      ".bee/verify/verify-app/features/pi-runtime.md"
    ],
    "read_first": [
      ".bee/verify/verify-app/features/pi-runtime.md",
      ".bee/verify/verify-app/features/README.md",
      "docs/history/pi-native-stage-driver/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [".bee/verify/verify-app/features/pi-runtime.md"],
    "action": "Prove the native path works for a real user, not only in unit tests. Rebuild the binary first, because doctor embeds the belt with include_str! and byte-compares the on-disk file — a stale binary reports drift and the run is worthless (per D7). Then launch a sandbox with control-bee, run doctor --runtime pi and require overall_status ready with wiring_matches_binary ok. Add one sub-feature to pi-runtime.md, named for the native dispatch path, following the file's existing four-H2 contract: what it is, how a user reaches it, how to drive it, its gotchas. Drive it: one native dispatch, one full answer returned, evidence captured as the --json payload plus a control-bee snapshot. Record the proof line as green:live with the command and the scope reason. Do not modify pi-hat-wave.md in this cell — the wave arrives in the next slice (per D10).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "doctor --runtime pi reports ready after the belt edit and a rebuild",
        "One native dispatch is driven end to end against a launched sandbox",
        "The full worker answer is returned, not a one-line summary"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/features/pi-runtime.md", "substantive": "a new sub-feature for native dispatch with its driving recipe and gotchas"}
      ],
      "key_links": ["the recipe drives the binary rebuilt from this branch, not a vendored stale copy"],
      "prohibitions": [
        "No edit to pi-hat-wave.md in this cell",
        "No claim of green without the fresh command output beside it"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": true
    }
  }
]
```

## Test matrix

High-risk: probes per applicable dimension of `references/edge-dimensions.md`.
Each writer judges existing coverage first and authors only the gap.

| # | Dimension | Scenario | Pass when |
|---|---|---|---|
| 1 | Happy path | New feature started after a tiny-lane feature; `dispatch prepare --kind advisor` | payload returned, no `stage_required` refusal |
| 2 | Happy path | `dispatch prepare --runtime pi --kind gather` for a role whose agent is a `pi` binary | payload names the native transport |
| 3 | Boundary | Same, for a role whose agent is `agy-flash` | payload is the herding command, unchanged from today (D3) |
| 4 | Regression | `dispatch prepare --runtime claude` and `--runtime codex`, every kind | payload bytes identical to main (D9) |
| 5 | Error path | Native child exits non-zero | leader sees the child's stderr, never a silent success |
| 6 | Error path | Native child writes unparseable stdout lines | parser skips them and still returns the answer (claim 3) |
| 7 | Timeout | A seat exceeds the 600 s ceiling | the seat is DROPPED and NAMED in the result set (D10) |
| 8 | Concurrency | 5 hat seats dispatched at once | all 5 results arrive seat-named; the wave stays inside 10 minutes |
| 9 | Trust boundary | A native child attempts a write the guard denies | the write is blocked inside the child, same verdict as the parent |
| 10 | Trust boundary | The new tool passed to `mapToolCall` | routed by its explicit row; the `default:` fail-safe test stays green |
| 11 | Idempotence | `doctor --runtime pi` after the belt edit and a rebuild | `overall_status: "ready"`, `wiring_matches_binary: "ok"` (D7) |
| 12 | Behavior change | The `pi-hat-wave` verify recipe, on main and on head | both pass; head additionally passes with no pane |

## Open Questions

- Which ONE delivery path returns a worker's result to the leader — the tool's own return value, `sendMessage(…, { deliverAs: "followUp" })`, or the existing result-inbox drain? Claims 1–4 prove the child works; they do not pick the carrier. `pnsd-2` must pick one and record it, per pi-result-mailbox D6.
- Whether a native child needs a reservation identity. `BEE_AGENT_NAME` has no native carrier today; it matters only once a native *execution* cell exists, which is beyond slice 2.

## Out of scope

- The pi-workflows engine, durable park/resume, and a typed human-decision gate (D6).
- The five design rules from `pi-workflows-xia.md` § "Five rules worth taking" — deferred in CONTEXT.md, still unshaped.
- The dead Antigravity usage-limit code in the belt (filed P3 in `.bee/backlog.jsonl`).
- Removing herding from Pi (D1).
