---
artifact_contract: bee-plan/v2
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Pi Worker Surface

## Summary

Two changes to how a Pi session shows and finishes its background work.

First, when you dispatch workers and keep typing, a small list appears under
your input box naming the work still running. It clears itself as each one
finishes, and disappears when none are left. Today that state has one line in
the footer, which is not enough once several workers run at once.

Second, a worker finishes by calling one tool instead of writing its result
file by hand. The host checks the fields before the call runs, so a worker can
no longer hand back a result the orchestrator then rejects as malformed — and
the worker stops paying for an extra model turn just to say it is done.

Nothing about gates, cells, proof, worktrees or the other three runtimes
changes.

Mode: `standard` — 2 risk flags: covered-contract-change, public-contracts
Why this is the least workflow that protects the work: both changes live in one
hand-written extension file that a contract suite already pins byte-for-byte,
so the risk is regression in a shared belt, not novel design — two serial cells
with the existing suite as the net is enough.

## Requirements (from CONTEXT.md)

- **D1**: A1 and A2 ship together here; A3 is a separate backlog row.
- **D2**: The panel uses Pi's own `ui.setWidget(<key>, factory, { placement: "belowEditor" })`. No component library, no vendored panel code.
- **D3**: In-flight workers only. A row is removed when its worker finishes; the widget is not drawn when zero are in flight.
- **D4**: The panel is informational and takes no input.
- **D5**: Row content uses bee's existing tick vocabulary (`▸ ✓ ⚡ ✗`). No new status words.
- **D6** (as amended by `6b7e8f49`): ONE terminating tool, `terminate: true`, **mirroring** the verdict schema bee already owns. It defines no new schema.
- **D7**: A worker that never calls the tool falls back to today's behavior — the leader reads the report file. Transport outcome classification is unchanged.
- **D8**: The tool does not touch model-guard's named exclusion; it spawns nothing.
- **D9**: `.pi/extensions/bee-guard.ts` stays hand-written and authoritative; the binary is rebuilt so `doctor`'s byte-compare agrees.
- **D10**: Claude, Codex and OpenCode behavior does not change.

## Load-bearing claims

Every row is load-bearing: if the claim is false, the shape changes. Labels are
`read` (opened the file at the line), `ran` (executed and kept the output), or
`guessed` (not permitted past the gate).

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Pi 0.85.1 ships the below-editor widget call, so D2 needs no component library. | read | `docs/history/pi-worker-surface/CONTEXT.md:134` naming host `extensions.md:2616` | `// Widget below editor` / `ctx.ui.setWidget("my-widget", ["Line 1", "Line 2"], { placement: "belowEditor" });` — and the next line shows the factory form this feature uses: `ctx.ui.setWidget("my-widget", (tui, theme) => new Text(...))` |
| 2 | Pi 0.85.1 documents `terminate: true`, so D6's tool can end the worker's turn. | read | `docs/history/pi-worker-surface/CONTEXT.md:30` | D2's row cites `extensions.md:2613-2619`; at the host path `:2019`: "Return `terminate: true` from `execute()` to hint that the automatic follow-up LLM call should be skipped after the current tool batch." |
| 3 | bee already owns the verdict schema, so D6's tool mirrors rather than defines. | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:490` | "pub(crate) struct MailboxResult {" — required `status`, `summary`, `files_changed`, `proof` parsed at `:655-690` |
| 4 | The worker — not bee — is the party that writes that schema, so D7's fallback path is the current path and regresses nothing. | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:20` | "`.bee/mailbox/<job-id>/result-N.json` <- round-numbered result, written by the WORKER, tmp-then-rename; its appearance at the final name IS the done signal" — confirmed live in this session: a gather worker's `result-1.json` carried exactly `status`/`summary`/`files_changed`/`proof`/`report_path` |
| 5 | An unmapped tool is DENIED by the belt, so the verdict tool needs its own `mapToolCall` row — without it every call is blocked. | ran | `.bee/bin/bee hook write-guard` fed `{"tool_name":"Write","tool_input":{"verdict":"DONE","file_path":""}}` | "bee write guard denied this target: it could not be canonically contained inside the physical worktree." — EXIT=2 |
| 6 | The default arm produces exactly that denied shape for a tool with no command/path/url argument. | read | `.pi/extensions/bee-guard.ts:522-526` | `return { hook: "write-guard", tool_name: "Write", tool_input: { ...args, file_path: "" }, passthrough: false }` |
| 7 | A pending marker exists before the worker is spawned, so the widget needs no new timer and no new writer. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2260-2262` | "write this job's PENDING marker for the session named by `--inbox-session`, BEFORE the worker is spawned — the drain must never be able to find a finished mailbox with no marker pointing at it." |
| 8 | The marker carries what a row needs to render. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2277-2288` | inserts `job_id`, `seat`, `mailbox`, `cell_id`, `created_at` |
| 9 | The parity test forbids only a non-`write-guard` blocking destination, so D8 holds. | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:1879-1884` | "the Pi belt's only BLOCKING destination is write-guard (model-guard is a NAMED EXCLUSION — Pi has no subagent surface, store 7f9c8518)" |
| 10 | `doctor` byte-compares the belt against the embedded copy, so D9's rebuild is mandatory, not optional. | read | `packages/bee-rs/crates/bee/src/doctor.rs:47` | the belt is embedded with `include_str!` and compared at `:310-312` |
| 11 | A single footer status line is the belt's only current surface for worker state — the gap the widget fills. | read | `.pi/extensions/bee-guard.ts:2028` | `function refreshModelUsageStatus(ctx: any): void {` … `ctx.ui.setStatus(MODEL_USAGE_STATUS_KEY, initialText)` — one keyed status line, refreshed at `session_start`, `turn_end` and `session_tree` |
| 12 | The two cells overlap on the belt file, which is why the plan names a serial edge rather than a parallel wave. | read | `.pi/extensions/bee-guard.ts:2074` | `export default function (pi: ExtensionAPI) {` — the single registration body both cells extend, holding `pi.on(...)` handlers and all six `pi.registerCommand` calls |
| 13 | A finished worker's outcome already reaches the conversation on its own, which is what lets D3 drop the row instead of showing a result in it. | read | `.pi/extensions/bee-guard.ts:916` | `await pi.sendUserMessage(renderResultInjection(marker, result, latest.round), steer ? { deliverAs: "steer" } : undefined)` |
| 14 | Pi validates a tool's parameters BEFORE `execute()` runs, which is the whole of D6's second gain. | read | `docs/history/pi-worker-surface/CONTEXT.md:134` naming host `extensions.md:2033` | "`prepareArguments(args)` is optional. If defined, it runs before schema validation and before `execute()`." — so schema validation precedes `execute()` on every call |
| 15 | The belt already runs a polling drain on its own unref'd timer, so the widget adds no second timer. | read | `.pi/extensions/bee-guard.ts:965` | `const timer = setInterval(() => {` … `}, DRAIN_POLL_MS)` with `(timer as any).unref()` at `:981` and the slot assignment at `:983` |
| 16 | A marker is written ONLY for a detached run, which is exactly why the panel shows detached workers and no others. | read | `packages/bee-rs/crates/bee/src/herding/run.rs:2271` | `let Some(token) = opts.inbox_session.as_deref() else { return };` — no `--inbox-session`, no marker, no row |

## Discovery

Four reality touches were taken before this plan was drafted; one changed the
feature. The research brief claimed bee never machine-parses a worker verdict.
Reading `herding/mailbox.rs:490` and then opening a real `result-1.json` on disk
disproved it — the schema and its validation already exist. D6 was amended
(`6b7e8f49`) rather than replaced: the tool now mirrors that schema, and the
payoff is stated honestly as `terminate: true` plus host-side validation.

The second touch was a run, not a read: feeding the belt's own default-arm
output to `bee hook write-guard` returned exit 2. That converted "the fail-safe
arm probably covers it" into claim 5, and made the `mapToolCall` row a required
part of cell `pws-1` instead of an optional detail.

The third located the widget's data source — `write_inbox_marker` already writes
a pending marker before spawn — so no timer, no new writer, and one honest
limit: only detached workers appear.

## Approach

**Recommended path.** Extend the existing belt in place, twice, serially.
`pws-1` adds the verdict tool and its routing row plus the contract fixtures
that pin them (D6, D8, claims 2/3/5/6/9). `pws-2` adds the widget reading the
result-inbox markers the drain already polls (D2–D5, claims 1/7/8). Both cells
rebuild the binary so `doctor` agrees (D9, claim 10).

**Rejected alternatives.**
- *Vendor the source's panel* — 1677 lines carrying run-manager state this
  feature has no use for; D2 already rejected it.
- *Define a new verdict schema in the belt* — a second home for a fact
  `mailbox.rs` already owns; killed by claim 3 and the D6 amendment.
- *One combined cell* — the two changes share a file but nothing else; a single
  cell would make the verdict tool's failure and the widget's failure
  indistinguishable at cap.
- *Run the two cells in parallel* — the two cells share **all three** of their
  files: `.pi/extensions/bee-guard.ts`,
  `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs` and
  `.bee/verify/verify-app/features/pi-runtime.md`. A named serial edge, per the
  concurrency law's file-overlap reason.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| `mapToolCall` routing row | MEDIUM — a wrong row either blocks every verdict call or opens a hole in a shared belt | `pws-1` | `pi_plugin_contracts` green, plus the claim-5 command re-run against the new row |
| Belt byte-compare drift | MEDIUM — a source edit without a rebuild makes `doctor` report drift for every user | `pws-1`, `pws-2` | `bee doctor --runtime pi --json` reports `wiring_matches_binary: ok` |
| Widget render cost | LOW — a redraw on every drain tick | `pws-2` | the widget draws nothing when zero markers exist (D3) |
| Cross-belt regression | LOW — D10; the three other belts share `bee hook` but no code here | `pws-1` | full `cargo test --release` on the bin |

Waves: `pws-1` then `pws-2`. One serial edge, reason: both cells write
`.pi/extensions/bee-guard.ts` and `.bee/verify/verify-app/features/pi-runtime.md`.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "pi",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "execute-cell", "classification": "required", "role": "code", "reason": "Both cells write TypeScript in the belt and Rust in the contract suite."},
    {"stage": "author-tests", "classification": "required", "role": "test", "reason": "Contract fixtures for the routing row and the widget's marker read."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "a cell needs a lookup outside the files its brief names", "reason": "Known-location reads only; the plan already carries the anchors."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "a single fact is needed from a known file", "reason": "Cheapest tier for a narrow lookup."},
    {"stage": "plan-check", "classification": "required", "role": "advisor", "reason": "The plan-step hat wave is this plan's check."},
    {"stage": "hat-facts-gaps", "classification": "required", "role": "hat-facts-gaps", "reason": "Default plan-step seat: audits the claims table against its anchors."},
    {"stage": "hat-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "Default plan-step seat: the SMALLER PATH check at plan altitude."},
    {"stage": "hat-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "Default plan-step seat: what the user sees in the planned behavior."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Five-seat wave is high-risk only; this lane is standard."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "Five-seat wave is high-risk only; this lane is standard."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "the user invokes an independent review", "reason": "Review is user-invoked, never an automatic stage."},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "Each cell updates its sub-feature rows in the verification map."},
    {"stage": "plan", "classification": "required", "role": "plan", "reason": "This plan."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "No generated content in this feature."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Two serial cells need no supervisor."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "No deployment; release is a separate flow."},
    {"stage": "lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No blind-lane convergence in this feature."},
    {"stage": "lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "No blind-lane convergence in this feature."},
    {"stage": "lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "No blind-lane convergence in this feature."}
  ]
}
```

## Shape

Two cells, serial, sharing two files. Slice 1 is the whole feature — each cell
lands a real user-visible surface end to end, no stubs.

`pws-1` is the walking skeleton for the verdict path. Two files are in play and
they are not interchangeable: `result-N.json` is the machine-read verdict
(`MailboxResult`), and `report-N.md` is the worker's prose report. The tool
writes `result-N.json` in exactly the shape bee reads today, so the new path and
the current path are the same contract. D7's fallback is about the OTHER file:
when no verdict is written, the leader still has `report-N.md` to read, which is
what it does today for a worker that produces no parsable result.

`pws-2` is the walking skeleton for the panel: a real dispatched worker appears
as a row and the row clears when it finishes.

## Plan check — hat wave findings

One wave, three seats (`hat-facts-gaps`, `hat-alternatives`, `hat-user-impact`),
600 s ceiling, dispatched through the door at the `advisor` tier.

**`hat-user-impact` — returned.** Drew the four-moment SEE mock and found that
`✓` and `✗` from D5's vocabulary can never appear in the widget, because D3
removes a row the instant its worker finishes. A failed worker vanishes with no
visible failure in the panel, and the row label was left undefined.

*Leader's synthesis (synthesis is decide-altitude and stays here):*

- **Not a decision conflict.** The user chose "vanishes on completion" over
  "stays briefly, then fades" with that exact tradeoff stated, and the
  result-inbox drain already injects each finished worker's outcome — including
  a failure — into the conversation. D3 stands as written. D5 governs the
  vocabulary a row may use, not a requirement that all four glyphs appear: in
  this widget only `▸` is reachable, and that is correct.
- **The row label gap is real and is resolved here, under the Agent's
  Discretion clause in CONTEXT.md.** A row renders its `seat` (for example
  `hat-facts-gaps`, `extraction`, `code`), never the raw `job_id` —
  `job-1789892858126-4153108-1` is not a name a human can use. When a marker
  carries `cell_id`, the row renders `<seat> · <cell_id>`. This is what lets the
  user connect the result the drain later delivers back to the row that was
  there.
- `pws-2`'s brief carries this label rule and the "zero markers draws nothing"
  case; the test matrix already pins the second.

**`hat-alternatives` — returned.** One cheaper shape proposed: merge `pws-1`
and `pws-2` into a single cell sharing the drain's marker scan. It also
confirmed the live Pi drive earns its cost, because `pi_plugin_contracts.rs`
exercises the belt through a Node stub and cannot show a real widget.

*Leader's synthesis:* **declined, on the seat's own cost line** — "One larger
cap and less isolated failure." The two changes share a file but nothing else;
merged, a red cap cannot say whether the verdict tool or the widget broke, and
the serial edge already gives the ordering the merge was reaching for. The live
proof finding is **accepted** and is why `pws-2` carries `green:live` rather
than a contract test alone.

**`hat-facts-gaps` — returned. 11 blockers, 3 warnings, and the most valuable
seat of the three.** All three seats resolved; none was dropped.

*Accepted and fixed:*

- **Row 1's quote was wrong.** `extensions.md:2613` holds
  `ctx.ui.setWorkingIndicator()`, not the prose quoted. The real call is at
  `:2616`. Row 1 now carries the actual bytes — better evidence than the
  original, since it shows the exact signature.
- **Three load-bearing claims lived only in prose**: host-side validation before
  `execute()`, the drain's existing timer, and the detached-only limit. Now rows
  14, 15 and 16. The seat even handed over the anchor for the third —
  `run.rs:2271` returns early when `inbox_session` is absent.
- **`result-N.json` and `report-N.md` were used interchangeably** in Shape while
  the test matrix treats them as different files. They ARE different files; the
  Shape section now says which one each path uses and why.
- **Neither proof ran the `doctor` byte-compare** that the risk map itself
  demands. Both cell proofs now carry it, plus explicit commands in place of
  "green" and "`green:live`".
- **No test pinned D3's core behavior** — a row disappearing when its worker
  finishes. Added, along with rows for D4, D5, a sparse marker and the
  detached-only limit.
- The file-overlap count said two shared files; it is three.

*Rejected, with evidence:*

- **"Row 5 returned EXIT=0, not exit 2."** Re-run twice — once with `cwd` set to
  this worktree and once with `cwd` set to the main checkout. Both returned
  `EXIT=2` with "bee write guard denied this target: it could not be canonically
  contained inside the physical worktree." The seat's own payload is not shown
  in its report, so the likeliest cause is that it probed a well-formed call
  rather than the empty-`file_path` shape the default arm actually produces.
  Claim 5 stands, and is now stronger for having been challenged.
- **Row 4's "`...`" elision** — the row was rewritten before this seat reported,
  and now anchors `mailbox.rs:20` with the contract text rather than an elided
  JSON body.
- **The `seat`/`cell_id` conditional-field warning** is accepted as a real gap
  but resolved by decision, not by a claim: the sparse-marker test row pins the
  fallback to the job id's short suffix.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pws-1 | Register the terminating verdict tool and route it explicitly | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`, `.bee/verify/verify-app/features/pi-runtime.md` | — | A Pi worker ends its run by calling one tool; the host rejects a bad field before the call runs, and the orchestrator reads the same result file as before | (1) `node --check .pi/extensions/bee-guard.ts`; (2) `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` green; (3) rebuild, then `bee doctor --runtime pi --json` reports `wiring_matches_binary: "ok"`; (4) the claim-5 probe re-run against the NEW row — `printf '{"hook_event_name":"PreToolUse","session_id":"p","cwd":"<repo>","tool_name":"<verdict-tool>","tool_input":{"status":"done","summary":"s","files_changed":[],"proof":"p"}}' \| .bee/bin/bee hook write-guard` exits 0, where the same payload with no row exits 2 |
| pws-2 | Draw in-flight workers in a widget below the editor | `.pi/extensions/bee-guard.ts`, `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`, `.bee/verify/verify-app/features/pi-runtime.md` | pws-1 | While workers run in the background, a list under the input names them; each row clears as its worker finishes, and the list disappears when none are left | (1) `node --check .pi/extensions/bee-guard.ts`; (2) `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` green; (3) rebuild, then `bee doctor --runtime pi --json` reports `wiring_matches_binary: "ok"`; (4) `green:live` — in a real Pi session, dispatch one worker with `bee herding run … --inbox-session "$PI_SESSION_ID"`, capture the widget showing a `<seat>` row while `.bee/result-inbox/<token>/<job>.json` exists, and capture it gone once the drain clears that marker |

```json
[
  {
    "id": "pws-1",
    "feature": "pi-worker-surface",
    "title": "Register the terminating verdict tool and route it explicitly",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["6b7e8f49", "dae51a75", "e29aa9cd"],
    "files": [".pi/extensions/bee-guard.ts", "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", ".bee/verify/verify-app/features/pi-runtime.md"],
    "read_first": [".pi/extensions/bee-guard.ts", "packages/bee-rs/crates/bee/src/herding/mailbox.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Register ONE terminating tool on the Pi belt whose parameter schema MIRRORS MailboxResult (herding/mailbox.rs:490 — status done|blocked, summary, files_changed, proof; optional options, leaning, report_path). It writes .bee/mailbox/<job-id>/result-N.json tmp-then-rename and returns terminate: true. Define no new schema (D6 as amended by 6b7e8f49). Add an explicit mapToolCall row for the tool name routing to write-guard — without it the fail-safe default arm produces a Write with an empty file_path and every call is denied with exit 2 (claim 5, claim 6). Leave model-guard a named exclusion: this tool spawns nothing (D8). Add contract fixtures pinning the route and the terminate flag, and add a pi-runtime.md sub-feature row. Rebuild the binary so doctor's byte-compare agrees (D9). Do not touch the Claude, Codex or OpenCode belts (D10).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "A worker calling the verdict tool ends its run without a follow-up assistant turn",
        "A verdict missing a required field is rejected by the host before execute() runs",
        "A worker that never calls the tool still resolves through its report file, unchanged"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "one registered terminating tool plus its explicit mapToolCall row; no TODO stubs"},
        {"path": "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", "substantive": "fixtures pinning the tool's write-guard route and the model-guard exclusion"}
      ],
      "key_links": ["the tool's schema fields match MailboxResult's parsed fields in herding/mailbox.rs"],
      "prohibitions": [
        "No new verdict schema defined in the belt",
        "No dispatch tool and no process spawner added to the belt",
        "No change to transport outcome classification in herding/wave.rs",
        "No edit to the Claude, Codex or OpenCode belts"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  },
  {
    "id": "pws-2",
    "feature": "pi-worker-surface",
    "title": "Draw in-flight workers in a widget below the editor",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": ["pws-1"],
    "decisions": ["7dfd593d"],
    "files": [".pi/extensions/bee-guard.ts", "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs", ".bee/verify/verify-app/features/pi-runtime.md"],
    "read_first": [".pi/extensions/bee-guard.ts", "packages/bee-rs/crates/bee/src/herding/run.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Draw a widget with ui.setWidget(<key>, factory, { placement: \"belowEditor\" }) listing in-flight workers, sourced from the pending markers already in .bee/result-inbox/<token>/ (D2). Re-render on the drain's existing setInterval tick at bee-guard.ts:965 — add no second timer (claim 15). One row per marker, rendering its seat, or '<seat> · <cell_id>' when the marker carries cell_id, never the raw job_id; fall back to the job id's short suffix when neither field is present. Rows carry only the tick glyph for in-flight work (D5). Remove a row the moment its marker clears, and do not draw the widget at all when no markers exist (D3). Take no input (D4). Advisory posture: an absent or unreadable inbox draws nothing and never throws. Add contract fixtures and a pi-runtime.md sub-feature row. Rebuild the binary so doctor's byte-compare agrees (D9).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts",
    "must_haves": {
      "truths": [
        "A dispatched detached worker appears as a row naming its seat",
        "The row disappears once that worker's marker clears",
        "No widget is drawn when no workers are in flight"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "a setWidget factory reading the result-inbox markers; no TODO stubs, no second timer"}
      ],
      "key_links": ["the widget reads the same inbox directory the drain polls"],
      "prohibitions": [
        "No second timer or polling loop",
        "No widget input handling",
        "No row kept after its worker finishes",
        "No raw job_id shown as a row's label"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

Triad, at its smallest demonstrating size. Each writer judges existing coverage
in `pi_plugin_contracts.rs` first and authors only the gap.

| Case | Cell | Scenario | Pass when |
|---|---|---|---|
| Happy — verdict routes | pws-1 | The verdict tool name is fed through `mapToolCall` | The routed hook is `write-guard` and the payload is not the empty-path Write shape |
| Happy — verdict accepted | pws-1 | A well-formed verdict call runs | The hook returns exit 0 and the tool's result carries `terminate: true` |
| Edge — missing field | pws-1 | A verdict call omits `proof` | Pi rejects before `execute()` runs; no `result-N.json` is written |
| Error — fallback intact | pws-1 | A worker ends with no verdict call | The orchestrator still reads `report-N.md`; transport classification is unchanged (D7) |
| Behavior change probe | pws-1 | The full `pi_plugin_contracts` suite on `main` and on head | Both green; the only new failures on head are the new cases |
| Happy — row appears | pws-2 | One detached worker dispatched with `--inbox-session` | A row naming its seat appears under the editor |
| Edge — zero workers | pws-2 | No pending markers exist | The widget is not drawn at all (D3) |
| Error — unreadable inbox | pws-2 | The result-inbox directory is absent or unreadable | The widget draws nothing and the session is not interrupted (advisory posture) |
| Parity — model-guard | pws-1 | The belt-parity test derives the belt's routes | model-guard remains excluded by name; only `write-guard` is a blocking destination (D8) |
| Happy — row clears | pws-2 | A worker's marker is removed from the result-inbox after its row was drawn | The row disappears on the next drain tick and the widget is cleared when it was the last one (D3's core behavior) |
| Edge — no input | pws-2 | The widget is focused or a key is pressed while it is drawn | Nothing is consumed; the editor keeps every keystroke (D4) |
| Edge — vocabulary | pws-2 | A row is rendered for an in-flight worker | The row carries `▸` and no other tick glyph, since `✓`/`✗` are unreachable while only in-flight rows exist (D5) |
| Edge — sparse marker | pws-2 | A marker carries no `seat` and no `cell_id` | The row still renders, falling back to the job id's short suffix rather than dropping the worker from the list |
| Edge — detached only | pws-2 | A worker is run in the foreground, with no `--inbox-session` | No marker is written and no row appears — the recorded limit, not a defect (claim 16) |

## Open Questions

(none) — the three questions CONTEXT.md deferred to planning were answered by
reality touches and are recorded there with their evidence.

## Out of scope

- A4 (hook `model_select` / `thinking_level_select`) and A5 (guard
  `registerCommand` with `pi.getCommands()`) — backlog polish, not this feature.
- A3, the prose guidance baseline — separate feature, filed as a P2 `proposal`
  under `prose-guidance-baseline` (D1).
- Foreground workers in the panel: no pending marker is written without
  `--inbox-session`, and a foreground run blocks the leader anyway.
- Any change to the Claude, Codex or OpenCode belts (D10).
