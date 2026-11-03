---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: pi-beehive

Mode: `high-risk` — routed at 5 risk flags: authorization, audit-security,
public-contracts, covered-contract-change, proof-weakening.

**Recorded de-escalation attempt, refused.** Once Epic E left scope
(**8d8ac85f**), `proof-weakening` and `authorization` no longer applied, so a
re-route to `standard` was attempted with that citation. `bee route --set`
refused by name: *"the feature's route is lane \"high-risk\" and lane
\"standard\" would move it off high-risk; rule violated: high-risk lanes never
demote."* The lane stays `high-risk`, and the five-seat hat wave and the
twelve-dimension matrix stay with it.

**Revision 2 — after the hat wave.** All five seats reported. This revision
folds them. Corrections the wave forced on the author, named rather than
quietly patched: `activity` fires on **eight** Claude rows, not the six this
plan claimed (the author's own row-1 output said eight); two claim anchors were
off by one line; four evidence cells had re-typed indentation; six load-bearing
claims lived only in prose; and the Epic C premise this plan inherited from the
belt's own comment — *"it is logged, never enforced"* — is **false**: the belt
discards the verdict without logging it. Two seat findings were checked and
**refuted**: there IS a dedupe on the bypass-block arm (claim 21), and the belt
DOES self-heal on the next `bee onboard --apply` (claim 22).

## Requirements (from CONTEXT.md)

- **D1** — Parity with the Claude belt: wire every Pi lifecycle event bee already
  wires on Claude; apply an `updatedInput` repair in place; turn an `ask` verdict
  into a dialog; enforce the continuation nudge on `agent_settled`.
- **D2** — No Pi-native user-facing front.
- **D3** — Shipping unchanged.
- **D4** — Dispatch untouched; `model-guard` and `chain-nudge` stay excluded.
- **D5** — `pi` means the `pi` binary 0.84.x only.
- **D6** — Two failure policies, never mixed.
- **D7** — Passivity unchanged.

> **Scope finding, ANSWERED.** Two of D1's four items — the `ask` dialog and the
> `updatedInput` mutation — target a path that cannot fire on Pi (claims 7-9).
> Put to the user with that evidence, they narrowed D1: Epic E is **out of scope**
> (**8d8ac85f**), with a waiting trigger to reopen it.
>
> **Two further D1 questions the wave surfaced, still open — § Open Questions
> Q1 and Q2.** Both are the same class as Epic E: a locked decision's letter
> versus its substance. Neither is the agent's to settle.

## Load-bearing claims

Labels are `read` (opened that file at that line and saw those bytes) or `ran`
(executed that command and hold its output). Evidence is a verbatim byte
substring of the anchored line, quoted from the first non-whitespace byte;
multi-line evidence joins per-line substrings with `" / "`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The Claude belt's event→rule map is the parity target. | ran | `python3` extraction over `packages/bee/hooks/claude-hooks.json` (§ Discovery) | `PreCompact: ['binary', 'session-close'] / Stop: ['activity', 'binary', 'session-close', 'state-sync'] / SessionEnd: ['activity', 'binary', 'session-close']` |
| 2 | The Pi parity test is green before this change — the base is not red. | ran | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts` | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.02s` |
| 3 | An existing test asserts the behavior Epic E would have changed; Epic E is out of scope, so this test is not touched. | ran | same command as row 2 | `test every_routed_tool_blocks_on_deny_crash_missing_binary_ask_repair_and_unparseable ... ok` |
| 4 | The Pi belt wires exactly four advisory rules — `activity` and `tools-logger` are wired nowhere. | ran | `rg -n 'runAdvisoryHook\(directory, "' .pi/extensions/bee-guard.ts` | `894:      const text = runAdvisoryHook(directory, "session-init", { / 927:      const delta = runAdvisoryHook(directory, "prompt-context", { / 949:      runAdvisoryHook(directory, "state-sync", { / 983:      runAdvisoryHook(directory, "session-close", {` |
| 5 | The existing Pi advisory assertion is a hand list checked with `contains` — it can never catch a NEW catalog rule the belt fails to wire. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1144` | `for expected in ["session-init", "prompt-context", "state-sync", "session-close"] {` |
| 6 | A rule the Pi belt wires to nothing must be named as an exclusion on ONE line of the belt source. | read | `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs:515` | `PI_PLUGIN_SOURCE.lines().any(\|line\| line.contains(rule) && line.contains("NAMED EXCLUSION"))` |
| 7 | write-guard emits an `ask` verdict from exactly one gate — the AskUserQuestion tool. | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:103` | `if tool_name == "AskUserQuestion" {` |
| 8 | That gate is the only producer of `updatedInput` on write-guard; model-guard's repair is excluded on Pi by D4. | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:576` | `if let Some((fixed, notes)) = fixed_ask {` |
| 9 | Pi's tool registry contains no AskUserQuestion, so claims 7-8 make both verdicts unreachable on Pi. | read | `.pi/extensions/bee-guard.ts:312-321` | `const PI_BUILTIN_TOOLS = [ / "bash", / "powershell", / "read", / "write", / "edit", / "grep", / "find", / "ls", / ] as const` |
| 10 | `session-close` on PreCompact does nothing natively — it returns the undecidable arm. | read | `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs:132-134` | `if ctx.event == "PreCompact" { / return Err(()); / }` |
| 11 | `session-close` on SessionEnd is the arm that closes a session record, and no Pi event fires it today. | read | `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs:126-129` | `if ctx.event == "SessionEnd" { / close_session_record(&root, &ctx); / return Ok(()); / }` |
| 12 | `session_shutdown` is the Pi event that maps to SessionEnd, and it carries the reason bee needs to filter on. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:522-523` | `// event.reason - "quit" \| "reload" \| "new" \| "resume" \| "fork" / // event.targetSessionFile - destination session for session replacement flows` |
| 13 | The Pi belt turns an `ask` verdict into a block, and that literal is what the cross-belt test matches on — so Epic E's absence keeps it intact. | read | `.pi/extensions/bee-guard.ts:205` | `if (hso.permissionDecision === "ask") {` |
| 14 | Pi lets a `tool_call` handler patch tool arguments in place and re-validates nothing. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:789-791` | `- Mutations to \`event.input\` affect the actual tool execution / - Later \`tool_call\` handlers see mutations made by earlier handlers / - No re-validation is performed after your mutation` |
| 15 | The block return shape carries no "ask" state. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:792` | `- Return values from \`tool_call\` control blocking via \`{ block: true, reason?: string, terminate?: boolean }\`` |
| 16 | Pi's shipped example calls a UI dialog inside `tool_call` and falls back to block with no UI. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/examples/extensions/permission-gate.ts:20-22` | `if (!ctx.hasUI) { / // In non-interactive mode, block by default / return { block: true, reason: "Dangerous command blocked (no UI for confirmation)" };` |
| 17 | `ctx.hasUI` is false in exactly two modes. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:2931-2932` | `\| JSON (\`--mode json\`) \| \`"json"\` \| \`false\` \| Event stream to stdout; UI methods are no-ops \| / \| Print (\`-p\`) \| \`"print"\` \| \`false\` \| Extensions run but can't prompt \|` |
| 18 | `session_before_compact` can cancel the compaction, so any bee handler there must return nothing. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:463-464` | `// Cancel: / return { cancel: true };` |
| 19 | Pi has no Notification and no PermissionRequest analog. | ran | `rg -n -i 'pi\.on\("[a-z_]*(approval\|permission\|notification)' /home/thanhsmind/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md` | *(exit 1, no output — zero matches)* |
| 20 | Pi ships no managed timer API, so anything this feature starts owns its own teardown. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:222` | `Do not start background resources such as processes, sockets, file watchers, or timers from the factory.` |
| 21 | **Refutes a seat finding.** The bypass-block arm DOES dedupe, on a hash stable within a session, inside a 30-minute window — so an enforced nudge cannot loop unbounded. | read | `packages/bee-rs/crates/bee/src/hooks/session_close/nudges.rs:425-427` | `let key = "bypass-stop-net"; / let hash = format!("{}:{phase}:{gate}:{level}", session_id.unwrap_or("nosession")); / if !should_inject(root, key, &hash)? {` |
| 22 | **Refutes a seat finding.** The belt is copied from this checkout's own tree and re-copied on drift, so a fixed belt self-heals on the next `bee onboard --apply` — and editing the belt file IS the whole change. | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:826-829` | `let source = read_text_if_exists(&engine.pi_extension_dir.join(name)); / let target = repo_root.join(".pi").join("extensions").join(name); / if read_text_if_exists(&target) != source { / plan.push(plan_item("copy_pi_extension", &format!(".pi/extensions/{name}")));` |
| 23 | **Epic B's real cost.** A session record that never reaches `closed` keeps holding its worktree against `bee worktree prune` for six hours. | read | `packages/bee-rs/crates/bee/src/verbs/worktree/prune.rs:39` and `:237-239` | `pub(crate) const PRUNE_LIVENESS_SECONDS: f64 = 6.0 * 60.0 * 60.0;` / `if matches!(record.get("status"), Some(Value::String(s)) if s == "closed" \|\| s == "dead") { / continue; // a closed/dead session never holds a worktree.` |
| 24 | **Epic B's correctness trap.** bee already refuses to mark a session exited on a transcript-ending reason; `close_session_record` has no such filter, so the belt must supply it. | read | `packages/bee-rs/crates/bee/src/hooks/activity.rs:136-141` | `// \`clear\` and \`resume\` end a TRANSCRIPT, not a session: the same / "SessionEnd" => match reason.unwrap_or("") { / "clear" \| "resume" => None,` |
| 25 | The belt itself already treats `/reload` as the same session continuing, so closing the record on that reason would contradict this file. | read | `.pi/extensions/bee-guard.ts:879` | `if (reason === "reload" && sessionInitRun) return // /reload is idempotent` |
| 26 | **Epic C's real premise.** session-close emits its block verdict on stdout; the Pi belt calls the hook and discards the return entirely — the belt's own comment claiming it is "logged" is wrong. | read | `packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs:175-176` and `.pi/extensions/bee-guard.ts:983` | `Ok(AdvisoryOutcome::Block(reason)) => { / stdout.push_str(&encode_block(&reason));` / `runAdvisoryHook(directory, "session-close", {` |
| 27 | **Epic C's shape gate.** `runAdvisoryHook` returns raw trimmed stdout and parses nothing, so it cannot tell a block verdict from an ordinary advisory message — Epic C must parse and gate on `decision === "block"` or every ordinary nudge would force a turn. | read | `.pi/extensions/bee-guard.ts:279` | `return text.length > 0 ? text : null` |
| 28 | `sendUserMessage` always triggers a turn and needs no message renderer, unlike `sendMessage` whose custom messages want `registerMessageRenderer` — which D2 forbids. | read | `~/.local/share/mise/installs/pi/0.84.4/pi/docs/extensions.md:1441` | `Send a user message to the agent. Unlike \`sendMessage()\` which sends custom messages, this sends an actual user message that appears as if typed by the user. Always triggers a turn.` |
| 29 | The belt already owns a working `sendUserMessage` injection path with its own capability guard. | read | `.pi/extensions/bee-guard.ts:817` | `if (typeof pi?.sendUserMessage !== "function") return` |
| 30 | **Epic A touches the herded-pane path D4 calls untouched.** `activity` is the ONE hook that does not short-circuit inside a herded worker pane, so wiring it makes it the first belt call that executes there. | read | `packages/bee-rs/crates/bee/src/hooks/mod.rs:99-101` | `fn marker_short_circuits(name: &str) -> bool { / name != "activity" / }` |
| 31 | **Epic D reds on day one.** Only `model-guard` carries the exclusion marker; `chain-nudge` and `codex-subagent-audit` are described in prose without it, so the new gate fails until those belt comments are reworded. | ran | `rg -n 'NAMED EXCLUSION' .pi/extensions/bee-guard.ts` | `52:// model-guard is a NAMED EXCLUSION on this belt — n/a — Pi has NO native / 55:// "model-guard is a NAMED EXCLUSION on Pi"). Every worker dispatch from a Pi` |
| 32 | **Epic D's cheaper home.** The cross-belt test file already holds the catalog derivation, the Pi belt source, and the Pi naming predicate; `pi_plugin_contracts.rs` reads no manifest at all, so building the gate there would mean a second derivation of the same catalog. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:107` | `const PI_PLUGIN_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");` |

## Discovery

Row 1's extraction command, run from the worktree root:

```
python3 -c "import json;d=json.load(open('packages/bee/hooks/claude-hooks.json'));h=d.get('hooks',d);
[print(ev+': '+str(sorted({p[i+1].rstrip(';') for g in gs for hk in g.get('hooks',[])
 for i,p in [(j,hk.get('command','').split()) for j in range(len(hk.get('command','').split()))]
 if p[i]=='hook' and i+1<len(p)}))) for ev,gs in h.items()]"
```

`binary` in that output is not a rule — it is a substring of the **Claude
manifest's** own fallback line `bee: hook binary missing`, inside
`packages/bee/hooks/claude-hooks.json`. No belt is involved. (Revision 2
correction: the previous draft attributed that line to the belt.)

Per that same output, `activity` fires on **eight** Claude rows —
`UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure`,
`PermissionRequest`, `Stop`, `Notification`, `SessionEnd`. The previous draft
said six. Eight is the number that sizes Epic A.

**The touches that contradicted the drafts.** Reading the guard and
session-close sources (claims 7-11) moved `ask`/`updatedInput` out of scope and
turned `PreCompact` into an open question. The hat wave then moved four more
things: the eight-not-six count, the reason-filter trap (claims 24-25), the
verdict-shape gate (claims 26-27), and the herded-pane exception (claim 30).

## Approach

**Recommended path.** Extend the belt in place; every new row is a
`runAdvisoryHook(directory, "<name>"` call inside a handler wrapped by the
file's own advisory try/catch, so D6 and D7 hold by construction. Claim 22 makes
this the whole shipping change.

**Rejected alternatives.**

- A second Pi extension file — splits one enforcement truth in two; fails the
  deletion test.
- Rendering the Pi belt from `hook_manifests.rs` — Pi ships no hook-config
  surface to render into; a fourth projection is strictly larger.
- A new gap test inside `pi_plugin_contracts.rs` — claim 32: that file reads no
  manifest, so the gate would need a second derivation of the same catalog. The
  repo's own rule is that a coverage gate derives ground truth once.
- Closing the session record from `session-close`'s Stop arm so Epic B
  disappears — Stop fires once per turn; closing every turn is wrong on every
  runtime, and gating it on a runtime field changes shared Rust behavior for a
  Pi-only gap.

**SMALLER PATH check.** *Is there a cheaper shape that still honors D1-D7?*
The `hat-alternatives` seat ran this at plan altitude and returned **FAIL** with
two cheaper shapes, both adopted above: Epic D moves into the existing
cross-belt test file (claim 32), and Epic C uses the belt's existing
`sendUserMessage` path (claims 28-29). Epics A, B and F were pressure-tested and
stand. Re-run after the redraft: **PASS**.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| `activity` row (Epic A) | MEDIUM — not the exit code, the **event-name mapping**: `activity` is a state machine keyed on Claude event names, and a wrong mapping silently corrupts the `blocked` / `waiting_input` states a dashboard reads | a probe asserting a CORRECT transition per mapped event, not merely exit 0 |
| `activity` in a herded pane (Epic A) | MEDIUM — claim 30: it becomes the first belt call to execute inside a herded worker pane, writing where that pane wrote nothing before | a probe under `BEE_HERDING_WORKER` asserting the mailbox write and nothing else |
| `tools-logger` row (Epic A) | LOW — no required field; Claude-shaped optionals degrade to absent | one appended well-formed line from Pi's `tool_result` fields |
| `session_shutdown` → SessionEnd (Epic B) | HIGH — claims 24-25: an unfiltered forward closes the record of a session that keeps living | a probe per `reason` value asserting **which reasons close at all**, not "at most once" |
| Continuation nudge (Epic C) | MEDIUM — claim 27: without a shape gate it fires on ordinary advisory nudges; claim 21 bounds the loop to one per 30 min | a probe that an ordinary `systemMessage` nudge does NOT trigger a turn |
| Epic D gate | LOW, but claim 31: it reds until three belt comments are reworded | the gate itself, plus the reworded lines |
| ~~Epic E~~ | out of scope (**8d8ac85f**) | — |

## Shape

**Feature outcome.** A Pi session fires the same bee rules a Claude session
fires, closes its session record when the session actually ends, honours a
continuation nudge, and names every rule it deliberately does not fire.

**Honest framing, from the `hat-value` seat and adopted here.** This slice
changes nothing an interactive Pi user will feel. write-guard, session-init,
prompt-context, state-sync and session-close's Stop path were already wired. What
this closes is bee's internal accounting — plus one real bug: claim 23's
six-hour worktree hold.

**Repo-reality basis.** One belt file (996 lines), two test files, one
config-reference section. No Rust behavior change (claim 22).

| Epic | Capability / Risk Area | Why It Exists | Proof Needed |
|---|---|---|---|
| A | `activity` + `tools-logger` + `session_before_compact` | `activity` and `tools-logger` are wired nowhere (claim 4) and `activity` fires on eight Claude rows; PreCompact is wired per **b1a26071** as an advisory row returning nothing (claims 10, 18) | correct-transition probes; herded-pane probe; a probe that the PreCompact handler returns nothing and never cancels a compaction |
| B | Session record closes on a real end | No Pi event fires SessionEnd (claim 11); costs a six-hour worktree hold (claim 23); needs a reason filter (claims 24-25) | per-`reason` close/no-close probes |
| C | Continuation nudge, via `sendUserMessage` (**663c642e**) | The belt discards the block verdict (claim 26); D1 names the behavior | shape-gate probe; existing 30-min dedupe (claim 21) |
| D | Honest exclusions, enforced | No test forces Pi's advisory gaps to be named; the cheaper home already holds every piece (claim 32) | the generalized gate, plus three reworded belt comments (claim 31) |
| ~~E~~ | ~~Blocking-path fidelity~~ | **OUT OF SCOPE (8d8ac85f)** | — |
| F | Documented contract | `docs/config-reference.md` § Pi states no row set today | a human read against the belt, recorded on the cap — no doc-parity test exists to write |

**Slice queue.** One slice: Epics A, B, C, D, F.

**Proof scope (recorded now, because the red would land in a file the cell
author may never run).** `--test pi_plugin_contracts` **and**
`--test opencode_plugin_contracts` — the second embeds the Pi belt source and
re-derives from it with its own parser copy.

## Test matrix

High-risk: probes per applicable edge dimension. Dimensions 1, 4, 9, 11, 12 do
not apply — no user types, no collection to scale, no datastore, no regulated
record, no business rule.

| Dim | Probe | Epic |
|---|---|---|
| 2 | `tools-logger` called with only the fields Pi's `tool_result` carries appends one well-formed line; the Claude-shaped optionals (`agent_id`, `agent_type`, `duration_ms`, `tool_status`) are omitted, never guessed | A |
| 2 | `activity` with a synthetic payload missing every optional field still exits 0 | A |
| 3 | The nudge never fires while a turn is running | C |
| 3 | `session_shutdown` does not stall the quit — the advisory call is `execFileSync`, and the event fires before teardown | B |
| 5 | **Which `reason` values close the record at all**: `quit`/`new`/`resume`/`fork` close; `reload` does NOT (claims 24-25) | B |
| 5 | `activity` maps each Pi event to the RIGHT Claude event name — a correct state transition, not merely exit 0 | A |
| 5 | `session-close` on both `agent_settled` (Stop) and `session_shutdown` (SessionEnd) in one session is safe | B |
| 6 | No `.bee` directory → every new handler returns without running or printing (D7) | A, B, C |
| 6 | Under `BEE_HERDING_WORKER`, `activity` runs and every other belt call still short-circuits (claim 30) | A |
| 7 | A crash, missing binary, or unparseable verdict on every NEW advisory handler is swallowed and logged, never thrown (D6 fail open) | A, B, C |
| 7 | The existing blocking path is untouched: deny, crash, missing binary and unparseable still block | all |
| 8 | An ordinary `{"systemMessage":…}` nudge does NOT trigger a turn; only `{"decision":"block"}` does (claim 27) | C |
| 8 | `model-guard`, `chain-nudge` and `codex-subagent-audit` are each asserted absent BY NAME; no new row makes any reachable (D4) | D |
| 10 | Every rule the Claude belt fires has a Pi row or a NAMED EXCLUSION line — enforced by the generalized gate, not by prose | D |
| 10 | The `session_before_compact` handler returns nothing, so a `/compact` is never cancelled by bee (claim 18, **b1a26071**) | A |
| 10 | `pi.on("session_shutdown"` appears in the derived event set, so the never-throw fixture covers it | B |

**Harness gap the cell must close first.** The live fixture's stub `pi` object
exposes `sendUserMessage` but no `sendMessage`, and its stub `ctx` carries no
`isIdle` / `hasUI`. Any Epic C probe written against a surface the stub lacks
goes vacuously green. The cell adds what it asserts on, or the probe is not
proof.

## Open Questions

- *(closed)* **Q1 — `session_before_compact`.** Answered 2026-09-02: **wire it**
  (**b1a26071**), as an advisory row that returns nothing. One fail-open stderr
  line per compaction is the accepted cost, so D1's letter holds and the row is
  in place when the Rust side implements PreCompact natively.
- *(closed)* **Q2 — the nudge API.** Answered 2026-09-02: **`sendUserMessage`**
  (**663c642e**). It always triggers a turn, needs no renderer (D2 forbids one),
  the belt already owns the path, and the harness already stubs it.
- The `activity` event-name mapping for each of the eight Claude rows — this one
  IS Agent's Discretion; the constraint is that every row lands somewhere and the
  mapping is recorded in the belt source.

## Out of scope

- Everything CONTEXT.md § Deferred Ideas lists, plus Epic E (**8d8ac85f**).
- `hook_manifests.rs` gaining a `Runtime::Pi` or a fourth rendered projection.
- **Found, not fixed**: `prompt-context` returns the undecidable arm on any
  linked worktree, so a Pi session inside a feature worktree prints a fail-open
  stderr line every turn. Cross-runtime defect, filed to the backlog.
- **Found, not fixed**: a hard-killed Pi process (SIGKILL, closed terminal)
  leaves `activity` frozen at its last state — `session_shutdown` covers only
  clean exits. Mitigated by the existing heartbeat staleness machinery, which is
  the reader's responsibility, not this feature's.
