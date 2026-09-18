---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Pi native stage driver

## Summary

Today every bee worker on Pi is launched into a tmux pane. That pane is the
reason stages feel slow and lossy: the leader waits in the foreground, the
answer comes back as a one-line summary, and a seat that runs long is simply
gone. This work lets bee start that same worker as a plain child process, with
no pane. Panes stay available for the jobs that still need them.

The child is started by bee's own Rust code, in the same place that starts a
pane today. That choice is what keeps this small: the job id, the seat name, the
ten-minute cap and the path the answer comes back on already exist there and are
reused untouched. bee's guard belt is not modified for the transport at all.

Two smaller things follow in the last slice, because they live in the belt. The
session narrows the model's tool list to what the current stage allows, and says
so — to the user and to the model. And a session that ends with work still
claimed writes that into the transcript instead of ending quietly.

Mode: `high-risk` — 6 risk flags: audit-security, authorization,
external-systems, public-contracts, covered-contract-change, multi-domain.
Why this is the least workflow that protects the work: spawning a process that
inherits bee's environment and writes to the repo is a trust boundary, and the
plan-step hat wave already found six blockers in the first draft of this shape.

## Requirements (from CONTEXT.md)

- **D1** Native dispatch is the DEFAULT on Pi; herding panes remain the fallback. Both stay tested.
- **D2** The transport is a child `pi` subprocess (`pi --mode json -p …`), never the in-process SDK and never `--mode rpc`. Amended by D11: spawned from Rust.
- **D3** A role reaches the native path only when its configured agent is a `pi` binary; anything else falls back, by config, with no leader choice.
- **D4** Per-stage tool gating is a HARD gate via `pi.setActiveTools`.
- **D5** The close guard is WARN ONLY; `tool_call` stays the ONLY blocking surface in the Pi belt.
- **D6** `.bee/*.json` stays the single store; the pi-workflows engine is not adopted.
- **D7** `.pi/extensions/bee-guard.ts` is hand-written source of truth; the Rust binary is rebuilt so `doctor`'s byte-compare agrees.
- **D8** One door: the leader never picks the transport.
- **D9** Claude, Codex and OpenCode behavior does not change.
- **D10** The hat wave contract holds — 10-minute ceiling, seat-named results, a late seat DROPPED and named. Under D11 this is inherited, not rebuilt.
- **D11** The child is spawned in Rust inside `bee herding run`. The belt gains NO dispatch tool and NO spawner; model-guard stays a named exclusion on it.
- **D12** The hard tool gate names its re-open command, announces the narrowing to the user, and tells the model a tool was removed by stage policy.
- **D13** The close-guard warning lands in the visible transcript, never only a UI toast.

## Load-bearing claims

Labels: `ran` = a command was executed this session and its bytes are quoted.
`read` = the file was opened at the cited line and its bytes are quoted.
No `guessed` row survives the gate. Rows marked **(corrected)** were found wrong
or weak by the plan-step hat wave and re-verified; `hat-synthesis.md` records
what each one said before.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A child `pi -p --no-session` LOADS `.pi/extensions/bee-guard.ts`. It is not yet shown to ENFORCE — see Open Questions. **(corrected)** | ran | `docs/history/pi-native-stage-driver/evidence.md` § A1-A2 | `bee: hook prompt-context could not decide this payload — allowing the operation (fail-open).          The guard did NOT run on it.` |
| 2 | bee's session preamble reaches that child, so a worker starts with bee context, not blank. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A3 | `"text":"Yes.\n\nPhase: \`idle\` \| Mode: \`none\`  \nFeature: \`pi-native-stage-driver\`  \nGates: \`none pending (no active work)\`"` |
| 3 | The child's stdout is JSONL preceded by non-JSON noise, so the parser must skip unparseable lines. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A4 | `mise ~/.config/mise/config.toml tools: pi@0.85.1` then `{"type":"session","version":3,"id":"01a0b519-…","cwd":"…/beehive--wt--pi-native-stage-driver"}` |
| 4 | The child settles and reports cost, so a per-worker budget is enforceable from its own output. | ran | `docs/history/pi-native-stage-driver/evidence.md` § A5 | `"usage":{"input":13924,"output":114,…"cost":{…"total":0.001253952112}},"stopReason":"stop"` then `{"type":"agent_settled"}` |
| 5 | Pi ships a working precedent for the exact spawn argv D2 requires. | read | `docs/history/pi-native-stage-driver/evidence.md` § B1 | `const args: string[] = ["--mode", "json", "-p", "--no-session"];` … `const proc = spawn(invocation.command, invocation.args, { cwd: cwd ?? defaultCwd, shell: false, stdio: ["ignore", "pipe", "pipe"], });` |
| 6 | `setActiveTools` narrows the model's list, so D4's hard gate is reachable. | read | `docs/history/pi-native-stage-driver/evidence.md` § B2 | `pi.setActiveTools(["read", "bash"]); // Switch to read-only` |
| 7 | The additive-only rule binds a loader tool's own execution, so narrowing from an event handler is legal. | read | `docs/history/pi-native-stage-driver/evidence.md` § B3 | `3. During loader execution, call \`pi.setActiveTools([...currentTools, ...matchingTools])\`. The change must be additive: do not remove currently active tools in the same call.` |
| 8 | pi 0.85.1 has no `session_stop`, so D5/D13 must hang on `agent_settled` — which the belt already registers at `bee-guard.ts:2305`. | ran | `docs/history/pi-native-stage-driver/evidence.md` § B4 | count `0`, beside `Use \`agent_settled\` for status integrations that need to know Pi will not continue running automatically.` |
| 9a | The Pi refusal the door changes sits at one site. **(corrected: a second call site exists.)** | read | `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2237-2243` | `if runtime == "pi" && !matches!(resolved, Resolved::Herding { .. }) { return Ok(Prepared::Value(pi_requires_herding_refusal(marker_role, &resolved, is_escalated))); }` |
| 9b | A second `pi_requires_herding_refusal` call site exists and must be handled too. **(corrected)** | ran | `rg -n 'runtime == "pi"' packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` | five sites: `2184`, `2237`, `2464`, `2480`, `2524` |
| 9c | `--seat`, the 600 s hat clamp and `detached_delivery` all live INSIDE the herding payload branch — which is why D11 keeps the work there. **(new)** | read | `prepare.rs:2464`, `:2480`, `:2524` | `if runtime == "pi" && kind != "cell" && role.is_some()` (seat); `configured if runtime == "pi" && hat_seat.is_some()` (clamp); `if runtime == "pi"` (detached) |
| 10a | `doctor` embeds the belt at compile time and byte-compares the on-disk file. **(corrected anchor)** | read | `packages/bee-rs/crates/bee/src/doctor.rs:47` and `:305` | `include_str!` at `:47`; the compare is `let same = on_disk.as_slice() == PI_EXTENSION_SOURCE.as_bytes();` at `:305` |
| 10b | No generator writes the belt, so the checked-in copy is the source of truth. | read | `packages/bee-rs/crates/bee/src/devtools/mod.rs:547` | `"pi" => return None,` |
| 11a | The contract suite parses the belt source and derives its `pi.on` and `registerCommand` sets. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:107`, `:288` | `include_str!` at `:107`; `const MARKER: &str = "pi.on(\"";` at `:288` |
| 11b | That suite has NO `registerTool` derivation, so it could never have proved a belt-hosted tool. This is why D11 moves the work out of the belt. **(new)** | ran | `rg -n 'registerTool' packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` | one hit, line 35, inside a comment: `//      \`PI_BUILTIN_TOOLS\` export — a sibling extension's \`pi.registerTool\`` |
| 12 | On runtime `pi`, an execution role is configured to a non-`pi` agent, so D1/D3's fallback is forced by config. **(corrected: the old row's "verbatim" string was hand-composed.)** | ran | `.bee/bin/bee team show --runtime pi --json` | role `code`: `"model": "gemini-3.8-flash-high"`, `"transport": "herding: agy-flash"`, `"slot": {"kind": "herding", "agent": "agy-flash", …}`. Role `advisor`: `"transport": "herding: pi-gpt-6-astra"` |
| 13a | A brand-new feature inherits the previous feature's role plan, so the door refuses every non-cell dispatch. | ran | `docs/history/pi-native-stage-driver/evidence.md` § C1 | `"reason":"stage_required"`, then with `--stage read-only-gather`, `"reason":"stage_not_applicable"` |
| 13b | The cause is that the stored packet keeps its own, different feature name. | ran | `docs/history/pi-native-stage-driver/evidence.md` § C2 | `packet.feature : release-2-41-2` while `state.feature : pi-native-stage-driver` |
| 13c | The fix has TWO homes, not one: the state branch compares the active feature, and the lane branch compares nothing at all. **(corrected)** | read | `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:633-639` and `:641-649` | the lane branch returns `approved_cell_packet` with no feature comparison; the state branch guards only on `m.get("feature") == Some(feature)` |
| 14 | A herding worker runs with a muted hook posture that a belt-spawned child would not have had — the reason D11 keeps the spawn on this path. **(new)** | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2499` and `packages/bee-rs/crates/bee/src/hooks/mod.rs:91` | `pane_env.insert("BEE_HERDING_WORKER".to_string(), "1".to_string());` and `std::env::var("BEE_HERDING_WORKER").is_ok_and(\|v\| !v.is_empty())`, with `:145-146` muting every hook but `activity` |
| 15 | `herding.agents` carries TWO config shapes, so "is this a `pi` binary" cannot be `agents[name][0]`. **(new)** | ran | `.bee/config.json` `herding.agents`, read with `python3` | `pi-gpt-5.6-luna  ARRAY  argv0='pi'` versus `agy-flash  DICT  argv0='agy'` (argv nested under an `argv` key) |

## Discovery

The first draft of this plan put the spawner in bee's Pi extension. The
plan-step hat wave returned five seats and found six blockers in it; the full
record is `docs/history/pi-native-stage-driver/hat-synthesis.md`. Five of the six
disappear if the spawn happens where a pane is started today, because the seat
name, the ceiling, the result drain, the worker hook posture and the per-agent
environment already exist there. The owner chose that shape on 2026-09-19
(decision `31fb9e15`), and CONTEXT.md gained D11 to D13.

The wave also corrected five rows of this table, including one where the
"verbatim" evidence had been hand-composed rather than copied. Those are fixed
above and the mistake is recorded.

## Approach

**Recommended path.** Four slices, walking-skeleton first.

Slice 1 fixes the stale-packet bug so the dispatch door works at all (claims
13a-13c). Slice 2 is the walking skeleton: a no-pane runner in `bee herding run`,
the door arm that selects it, and one real end-to-end drive. Slice 3 proves the
hat wave runs with no panes at all, including drop-and-name. Slice 4 adds the two
belt behaviors — the hard tool gate with D12's obligations, and the close guard
with D13's.

Rejected alternatives, one line each:
- A registered spawner tool in the belt — rejected by D11; six blockers, and it makes the belt's own recorded premise false.
- In-process SDK (`createAgentSession`) — rejected by D2; least stable, no types on this host to pin against.
- `pi --mode rpc` bridge — rejected by D2; stable protocol, no shipped example, a bridge to maintain.
- Replacing panes on Pi entirely — rejected by D1; `team.pi` routes execution roles to a non-`pi` agent (claim 12).

**SMALLER PATH check.** Is there a cheaper shape that still honors every locked
decision? The wave supplied one, and it was taken — that is what D11 is. Asked
again of the shape as it now stands: could slice 2 alone ship? It would honor D1,
D2, D3, D8, D9 and D11, but D10, D12 and D13 are locked decisions, so stopping
there would quietly deliver less than was agreed. FAIL on scope integrity. Kept
at four slices. Slice 3 is much smaller than it was, because D11 inherits the
seat, the ceiling and the drain rather than rebuilding them.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Stale approved packet, two homes | MEDIUM | `pnsd-1` | red-first test per branch; a lane-file packet and a state packet both refused when the feature differs |
| Child spawn in the runner | HIGH | `pnsd-2` | the child inherits the worker posture and per-agent env, not the leader's; a denied write inside the child is blocked, `green:live` |
| Agent-shape detection | HIGH | `pnsd-3` | both config shapes classified correctly; a non-`pi` agent still returns a pane payload byte-identical to main |
| Door payload regression | HIGH | `pnsd-3` | claude and codex payloads byte-identical to main (D9) |
| Binary/byte drift | MEDIUM | `pnsd-4`, `pnsd-6`, `pnsd-7` | `doctor --runtime pi` reports `ready` after every belt edit and rebuild |
| Tool gate legibility | MEDIUM | `pnsd-6` | the user sees the re-open command; the model is told why a tool went away |
| Close-guard visibility | LOW | `pnsd-7` | the warning is in the transcript, and is present with `ctx.hasUI` false |

Waves: `pnsd-1` runs alone. Then `pnsd-2` and `pnsd-3` run in parallel — different
files, and the payload contract between them is fixed in this plan below, not left
to either cell. `pnsd-4` is serial after both.

**The payload contract, fixed here so two parallel cells cannot disagree.**
The door's native payload is the SAME `bee herding run` command it returns today,
with one added flag naming the no-pane runner. No new JSON shape, no new keys, no
new delivery carrier: the result comes back exactly as a pane worker's does, through
the existing mailbox report and result drain. `pnsd-3` adds the flag; `pnsd-2`
implements it. Neither cell may invent a second carrier.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "planning", "classification": "required", "role": "plan", "reason": "High-risk lane: the shape and its slices need plan-altitude reasoning."},
    {"stage": "implementation", "classification": "required", "role": "code", "reason": "Every slice writes Rust in the runner and the door, or TypeScript in the belt."},
    {"stage": "test-and-live-proof", "classification": "required", "role": "test", "reason": "A spawn that inherits env and writes to the repo is only proven by a live drive."},
    {"stage": "documentation-and-capture", "classification": "required", "role": "docs", "reason": "Two mapped verify features (pi-runtime, pi-hat-wave) change and must be re-synced."},
    {"stage": "read-only-gather", "classification": "required", "role": "read", "reason": "Multi-file hunts across the runner, the door, the hooks and the contract suite."},
    {"stage": "fact-extraction", "classification": "conditional", "role": "extraction", "condition": "a single already-located fact is needed during execution", "reason": "Cheap tier for narrow lookups only."},
    {"stage": "generation-fallback", "classification": "conditional", "role": "generation", "condition": "a role with no configured slot is requested", "reason": "Fallback only; never selected directly."},
    {"stage": "independent-review", "classification": "conditional", "role": "review", "condition": "the user invokes a review", "reason": "Review is user-invoked, never automatic."},
    {"stage": "generic-advisor", "classification": "required", "role": "advisor", "reason": "High-risk work owes an advisor consult; the hat-wave synthesis serves it."},
    {"stage": "supervision", "classification": "not-applicable", "role": "supervisor", "reason": "Single-leader feature; no supervised multi-session run."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No convergence lane: the shape is settled, not contested."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No convergence lane."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No convergence lane."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "High-risk: 5 seats. This seat audited the claims table and found the fabricated row."},
    {"stage": "hat-risks", "classification": "required", "role": "hat-risks", "reason": "The spawn is a trust boundary; this seat found the worker-posture blocker."},
    {"stage": "hat-value", "classification": "required", "role": "hat-value", "reason": "Five Pi features already shipped without fixing this."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "This seat produced the shape now locked as D11."},
    {"stage": "hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "D4's hard gate changes what the user can do in every turn."},
    {"stage": "deployment", "classification": "not-applicable", "role": "deploy", "reason": "No release is cut by this feature; the release script owns that."}
  ]
}
```

## Shape

**Slice 1 — unblock the door.** `pnsd-1`.
**Slice 2 — walking skeleton.** `pnsd-2` and `pnsd-3` in parallel, then `pnsd-4`.
**Slice 3 — the wave with no panes.** `pnsd-5`.
**Slice 4 — the belt's session surface.** `pnsd-6`, `pnsd-7`.

Only slices 1 and 2 are previewed as cells. Later slices keep headlines.

Slice 3 headline: drive a 5-seat hat wave with every seat a child process, prove
the 10-minute ceiling and that a late seat is dropped AND named (D10) — inherited
machinery, so this slice is proof, not construction.
Slice 4 headlines: hard per-stage tool gate with a named re-open command, a user
notice and a model notice (D4, D12); close-guard warning in the transcript (D5, D13).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `pnsd-1` | Scope the approved plan packet to its own feature | `plan_packets.rs`, `state_group/tests.rs` | — | Starting a new feature right after another no longer refuses every advisor, gather and hat dispatch | red-first test per branch; scoped `cargo test` green |
| `pnsd-2` | Add a no-pane runner to `bee herding run` | `herding/run.rs`, `herding/tests.rs` | `pnsd-1` | A bee worker on Pi runs as a child process with no tmux pane, and its answer comes back the same way a pane worker's does | scoped `cargo test` green, plus a denied write attempted inside the child and blocked |
| `pnsd-3` | Select the no-pane runner from the dispatch door | `prepare.rs` | `pnsd-1` | `bee dispatch prepare --runtime pi` picks the no-pane runner for a `pi`-agent role and still returns a pane payload for every other role | scoped `cargo test` green, including byte-equality of the claude and codex payloads |
| `pnsd-4` | Drive one native worker end to end and record the evidence | `.bee/verify/verify-app/features/pi-runtime.md` | `pnsd-2`, `pnsd-3` | `doctor --runtime pi` still reports `ready`, and the verify recipe drives a real no-pane dispatch | `green:live` — the recipe run against a launched sandbox, evidence attached |

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
    "action": "Make an approved plan packet belong to the feature that produced it. Read evidence.md section C for the reproduction and the exact refusal strings. `get_approved_preview_packet` has TWO return sites — find them by searching for `approved_cell_packet`, and carry the enclosing function name rather than a line number. They differ today and both are wrong in different ways: the lane-file branch compares NO feature at all, and the state branch compares only the ACTIVE feature name, never the packet's own `feature` field. At BOTH sites, if the stored packet carries a non-empty `feature` that differs from the requested feature, treat the packet as absent and return None. A packet with no `feature` field keeps today's behavior so older stores do not break. Write one red test per branch first — a lane-file packet and a state packet, each carrying a foreign feature name — watch both fail for that reason, then fix. Do not change how a matching packet is returned, and do not touch the gate or dispatch code.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee plan_packets",
    "must_haves": {
      "truths": [
        "A lane-file packet whose own feature differs from the active feature is not returned",
        "A state packet whose own feature differs from the active feature is not returned",
        "A packet with no feature field is still returned, so existing stores keep working",
        "Starting a new feature after another one no longer refuses a non-cell dispatch"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs", "substantive": "both return sites compare the packet's own feature field; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/state_group/tests.rs", "substantive": "one test per branch, each failing before the fix for the mismatched-feature reason"}
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
    "title": "Add a no-pane runner to bee herding run",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["pnsd-1"],
    "decisions": ["D2", "D10", "D11"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/tests.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/hooks/mod.rs",
      "docs/history/pi-native-stage-driver/evidence.md",
      "docs/history/pi-native-stage-driver/hat-synthesis.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Give `bee herding run` a second way to start a worker: spawn it as a child process instead of splitting a tmux pane (per D11). Everything around the worker stays exactly as it is — same job id, same `--seat`, same ceiling handling, same mailbox report path, same result-inbox marker and drain. Only the launch changes. Build the child argv the way Pi's own example does (evidence.md section B1): `--mode json -p --no-session`, then `--model`, then `--tools`, then the task, spawned with shell false and an explicit cwd, stdout and stderr piped. Parse stdout as JSONL and SKIP every line that does not parse — evidence.md section A4 shows a non-JSON banner arrives first. Take the assistant text and the usage/cost from the child's own events (evidence.md section A5) and write the same report file a pane worker writes, so the leader reads it by the path it already reads. Two things the pane path already gets right and this path MUST get right the same way, both found by the hat wave: build the child environment EXPLICITLY the way the pane env is built near the `BEE_HERDING_WORKER` insertion — never let the child inherit the leader's session identity — and set `BEE_HERDING_WORKER=1` in it, because `hooks/mod.rs` reads that marker to mute every leader-only hook; without it the child registers its own acting session and can adopt the leader's handoff. Find the cwd the same way the pane path finds it, from the prepared `--cwd`, never from the process cwd. Do not add any new result carrier and do not change the report format. Do not touch `.pi/extensions/bee-guard.ts` (per D11).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee herding",
    "must_haves": {
      "truths": [
        "A worker starts as a child process with no tmux pane and its answer lands in the same report file a pane worker writes",
        "The child environment carries BEE_HERDING_WORKER=1 and does not carry the leader's session identity",
        "Unparseable stdout lines are skipped and the answer still returns",
        "A non-zero child exit surfaces the child's stderr rather than reporting success",
        "The job id, seat and ceiling behave exactly as on the pane path"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "a no-pane launch path beside the pane launch; explicit child env; JSONL parse that tolerates noise. No TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/src/herding/tests.rs", "substantive": "tests for env construction, noise-tolerant parsing, and non-zero exit surfacing"}
      ],
      "key_links": [
        "the no-pane path writes the same mailbox report the drain already reads",
        "the child env is built by the same explicit construction the pane env uses"
      ],
      "prohibitions": [
        "No edit to .pi/extensions/bee-guard.ts",
        "No new result carrier and no change to the report format",
        "The child must not inherit BEE_SESSION_ID, CLAUDE_CODE_SESSION_ID or PI_SESSION_ID",
        "No change to the pane launch path's behavior"
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
    "title": "Select the no-pane runner from the dispatch door",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": ["pnsd-1"],
    "decisions": ["D1", "D3", "D8", "D9", "D11"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "docs/history/pi-native-stage-driver/CONTEXT.md",
      "docs/history/pi-native-stage-driver/hat-synthesis.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Make the door choose the no-pane runner for a Pi role whose configured agent is a `pi` binary, and keep the pane payload for everything else (per D1, D3, D11). The payload contract is fixed in plan.md and is deliberately small: return the SAME `bee herding run` command returned today, with one added flag naming the no-pane runner. No new JSON shape, no new keys, no new delivery carrier. Do NOT add a new `Resolved` variant or a second refusal path; the Pi arm stays inside the herding branch so that `--seat`, the 600 s hat clamp and `detached_delivery` keep applying — the hat wave found all three live there (claims 9c), and a new arm outside that branch would silently lose them. Deciding 'this agent is a pi binary' must handle BOTH config shapes in `herding.agents`: a bare argv array, and an object carrying an `argv` key (claim 15). Read the first argv element after normalizing the shape; `agents[name][0]` is correct only by luck. Never decide from the role name, and never from a leader-supplied flag — the leader must not pick the transport (per D8). A role that resolves to neither shape still refuses, with the existing reason string unchanged. Add a test asserting the claude and codex payloads are byte-identical to main (per D9) rather than trusting review. There are five `runtime == \"pi\"` sites in this file; search for them and say in the cap which ones you touched.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee prepare",
    "must_haves": {
      "truths": [
        "A pi-agent role on runtime pi returns a payload naming the no-pane runner",
        "Both herding.agents config shapes are classified correctly",
        "A non-pi-agent role returns today's pane payload, unchanged",
        "Claude and codex payloads are byte-identical to main",
        "--seat, the 600 s hat clamp and detached_delivery still apply on the native path"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "substantive": "shape-normalizing agent classification and the added flag, inside the existing herding branch. No TODO stubs"}
      ],
      "key_links": ["the native selection reads the configured agent argv, not the role name"],
      "prohibitions": [
        "No new CLI flag that lets a caller select a transport",
        "No new Resolved variant and no second refusal path",
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
    "decisions": ["D7", "D11"],
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
    "action": "Prove the no-pane path works for a real user, not only in unit tests — five Pi features already shipped unit-green without fixing this. Rebuild the binary first: `doctor` embeds the belt with `include_str!` and byte-compares the on-disk file, so a stale binary reports drift and the run is worthless (per D7). Launch a sandbox with control-bee, run `doctor --runtime pi`, and require `overall_status` ready with `wiring_matches_binary` ok. Add one sub-feature to pi-runtime.md for the no-pane dispatch path, following the file's existing four-H2 contract. Drive it: one no-pane dispatch, the full answer returned through the normal report path, evidence captured as the --json payload plus a control-bee snapshot. Then close the one trust-boundary question the plan could not close on paper: inside the child, attempt a write the guard should deny, and record what happened. The earlier probe never tested this — it ran with `--tools read` and an explicit instruction not to use a tool — so this is the first real test of enforcement inside a child. If the write is NOT blocked, stop and report it as a P1 rather than capping. Record the proof line as green:live with the command and the scope reason. Do not modify pi-hat-wave.md in this cell; the wave is the next slice.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "doctor --runtime pi reports ready after a rebuild",
        "One no-pane dispatch is driven end to end against a launched sandbox",
        "The full worker answer is returned through the existing report path",
        "A write denied by the guard is attempted inside the child and the outcome is recorded"
      ],
      "artifacts": [
        {"path": ".bee/verify/verify-app/features/pi-runtime.md", "substantive": "a new sub-feature for the no-pane dispatch path with its driving recipe and gotchas"}
      ],
      "key_links": ["the recipe drives the binary rebuilt from this branch, not a vendored stale copy"],
      "prohibitions": [
        "No edit to pi-hat-wave.md in this cell",
        "No claim of green without the fresh command output beside it",
        "Do not cap if the in-child write is not blocked — report it"
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

High-risk: probes per applicable dimension. Each writer judges existing coverage
first and authors only the gap.

| # | Dimension | Scenario | Pass when |
|---|---|---|---|
| 1 | Happy path | New feature started after a tiny-lane feature; `dispatch prepare --kind advisor` | payload returned, no `stage_required` refusal |
| 2 | Boundary | A lane-file packet carrying a foreign feature name | not returned |
| 3 | Boundary | A packet carrying no `feature` field at all | still returned, old stores unbroken |
| 4 | Happy path | `dispatch prepare --runtime pi` for a role whose agent argv starts with `pi` | payload names the no-pane runner |
| 5 | Boundary | Same, for an agent configured in the OBJECT shape with `argv` | classified by argv, not by luck |
| 6 | Boundary | A role whose agent is `agy-flash` | pane payload, byte-identical to today |
| 7 | Regression | `dispatch prepare --runtime claude` and `--runtime codex`, every kind | payload bytes identical to main (D9) |
| 8 | Regression | A Pi hat dispatch on the native path | `--seat`, the 600 s clamp and `detached_delivery` all still present |
| 9 | Error path | Child exits non-zero | the child's stderr surfaces; never a silent success |
| 10 | Error path | Child stdout carries non-JSON lines | parser skips them and still returns the answer (claim 3) |
| 11 | Trust boundary | A write the guard denies, attempted INSIDE a native child | blocked, proven `green:live` — the open question claim 1 could not close |
| 12 | Trust boundary | Child environment at spawn | carries `BEE_HERDING_WORKER=1`; carries no `BEE_SESSION_ID`, `CLAUDE_CODE_SESSION_ID` or `PI_SESSION_ID` (claim 14) |
| 13 | Concurrency | 5 hat seats dispatched at once, no panes | all 5 results arrive seat-named; the wave stays inside 10 minutes |
| 14 | Timeout | A seat exceeds the ceiling | DROPPED and NAMED in the result set (D10) |
| 15 | Idempotence | `doctor --runtime pi` after each belt edit and rebuild | `overall_status: "ready"`, `wiring_matches_binary: "ok"` |
| 16 | Behavior change | The `pi-hat-wave` verify recipe, on main and on head | both pass; head additionally passes with no pane |
| 17 | User-facing | A stage narrows the tool list | the user sees the re-open command named; the model is told a tool was removed by policy (D12) |
| 18 | User-facing | A session settles with a claimed uncapped cell, `ctx.hasUI` false | the warning is in the transcript, not only a toast (D13) |

## Open Questions

- Does the write guard ENFORCE inside a child, or only load? Claim 1 proves loading
  only. `pnsd-4` closes this with a live deny attempt, and is instructed to refuse to
  cap if the write is not blocked. Until then it is a known unknown, not an assumption.
- Does the child need an explicit recursion fence? Under D11 the child has no dispatch
  tool, so the belt-hosted hazard is gone. `pnsd-2` should confirm the `--tools`
  allowlist closes it rather than assume it does.

## Out of scope

- The pi-workflows engine, durable park/resume, and a typed human-decision gate (D6).
- The five design rules from `pi-workflows-xia.md` § "Five rules worth taking".
- The dead Antigravity usage-limit code in the belt (filed P3).
- The hat-wave ceiling drift between doctrine and the `runtime == "pi"` clamp (filed P2).
- Removing panes from Pi (D1).
- Giving execution roles a `pi` agent so they reach the native path — a config change the
  owner may want later; this feature does not make it.
