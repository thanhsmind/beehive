# Plan: piggery-borrow

## Summary

Seven practices from piggery land in bee:

- A result a worker sends to the pi leader is kept when the leader's turn is aborted, and it waits for the user's next turn instead of waking the leader.
- `bee herding cancel` stops the worker's child processes too, not only the pane.
- `.bee/config.json` can cap live herding workers and nesting depth, and bee refuses at the door with a named reason.
- The pi and opencode belts and the bee binary check one contract number and tell the human which side to update.
- The pi belt keeps its state across `/reload`, and a child of the leader is never taken for the leader.
- `bee herding status` shows each worker's context tokens and turns when its transcript is known.
- `bee doctor` prints one advisory line per runtime with the fix.

Mode: `standard` — 3 risk flags: cross-platform, public-contracts, multi-domain
Why this is the least workflow that protects the work: seven small changes; cells that share a file run one after another, the rest run in parallel, every cell is red-first, and the leader adds a live proof for the CLI surfaces.

Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

## Requirements (from CONTEXT.md)

Each decision was revised after the hat wave; the store ids below are the revised ones.

- D1 (`60824879`): requeue and hold on an aborted or errored turn; delete on completed; today's delete when no outcome is seen; no inject while a prompt is open.
- D2 (`7a25e4ed`): cancel reads the tree, TERM, wait 2 s, KILL; seams for tests; unix only, Paseo unchanged.
- D3 (`ffa4e772`): herding.limits refused at herding run and dispatch prepare through a quiet occupancy helper; --explain writes nothing.
- D4 (`d79958fd`): belt contract in the session-init payload; the mismatch reaches the human.
- D5 (`1bf2611c`): one globalThis state slot; PASEO_AGENT_ID kept out of the leader's children only.
- D6 (`7edc0153`): ctx_tokens and turns from the recorded transcript; widget split to backlog.
- D7 (`da84d0f0`): advisory doctor rows per runtime with a fix; the belt contract check rides the pi and opencode rows.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command executed, output kept). Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Settle deletes every in-flight claim without reading the outcome | read | .pi/extensions/bee-guard/events.ts:528 | for (const processing of inFlightClaims) rmSync(processing, { force: true }) |
| 2 | A requeue helper already exists | read | .pi/extensions/bee-guard/result-inbox.ts:159 | export function requeueClaim(processing: string): void { |
| 3 | The belt already listens to agent_before_settle | read | .pi/extensions/bee-guard/events.ts:450 | pi.on("agent_before_settle" |
| 4 | Belt state is a plain module export | read | .pi/extensions/bee-guard/state.ts:3 | export const state: { |
| 5 | Leader detection reads PASEO_AGENT_ID from the environment | read | .pi/extensions/bee-guard/paseo-heartbeat.ts:81 | const agentId = env.PASEO_AGENT_ID |
| 6 | Cancel lives in one function | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:317 | pub(crate) fn cancel_with_backends_and_timeout( |
| 7 | Cancel records the foreground pid before it closes the pane | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:512 | map.insert("cancel_pid".to_string(), Value::Number(pid.into())); |
| 8 | The ledger already has a live worker count | read | packages/bee-rs/crates/bee/src/herding/wave_ledger.rs:278 | pub(crate) fn live_worker_count( |
| 9 | dispatch prepare already refuses typed by role | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2129 | role_not_configured |
| 10 | session-init is one hook entry the belts call | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:186 | "session-init" => session_init::run(&rest, &stdin_str), |
| 11 | The opencode plugin calls session-init | read | .opencode/plugins/bee-guard.ts:421 | runAdvisoryHook(directory, "session-init" |
| 12 | Doctor already has a binary freshness row to reuse | read | packages/bee-rs/crates/bee/src/doctor.rs:260 | if let Some(row) = binary_freshness_row(root) { |
| 13 | pi's agent_before_settle carries the turn outcome | ran | rg -a -o "_lastActivityOutcome = [^;]{0,80}" /home/thanhsmind/.local/share/mise/installs/pi/1.0.0/pi/pi | _lastActivityOutcome = "completed" |
| 14 | The activity hook already reads the transcript path | read | packages/bee-rs/crates/bee/src/hooks/activity.rs:458 | if let Some(path) = field(payload, "transcript_path") { |
| 15 | pi child workers run without a session file | read | packages/bee-rs/crates/bee/src/herding/run.rs:2880 | "--no-session".to_string(), |

## Discovery

Two advisor digests (2026-10-06) compared piggery with bee. The plan-step hat
wave (three seats, 2026-10-07) then found four blockers in the first draft:
an Esc that restarted the leader, a worker losing its guard, an occupancy
verb that prints and writes, and no token source for herdr and tmux workers.
Each decision was revised and re-logged; the synthesis is recorded as the
advisor ref.

## Approach

Each decision is one cell. The doctor work for D4 and D7 sits in one cell
(pbr-7), so pbr-7 owns `doctor.rs`. The pi belt cells chain on shared files:
pbr-1, then pbr-2, then pbr-3 (pbr-3 also waits for pbr-7's shared contract
constant). The belt cells defer the release-manifest regen to the wave
barrier: the leader runs `bee dev regen` once after each wave that touched
`.pi/extensions` or `.opencode`, in its own commit.

## Shape

Wave 1: pbr-1, pbr-4, pbr-5, pbr-6, pbr-7. Wave 2: pbr-2. Wave 3: pbr-3.

## Cells — current slice (preview)

```json
[
  {
    "id": "pbr-1",
    "feature": "piggery-borrow",
    "title": "Keep a worker result when the leader turn is aborted, without waking the leader",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "60824879-eec0-4b05-aba3-cff8954ec841"
    ],
    "files": [
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/result-inbox.ts"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/work/pi-result-mailbox/delivery.md"
    ],
    "action": "Per piggery-borrow D1 (revised). In the agent_before_settle handler (events.ts:450) record event.outcome for the active session BEFORE any early return (pi 1.0.0 sends completed, aborted or error; older pi has no event). In the agent_settled handler (the rmSync loop at events.ts:528): completed -> delete claims as today; aborted or error -> requeueClaim (result-inbox.ts:159) each claim and mark them held so the drain does not inject them until the next turn the user starts (the next input event clears the hold); no outcome seen -> today's delete. Clear the recorded outcome after settle. In the drain tick, skip while promptDepths.get(session) > 0 and skip held claims. Red first in pi_plugin_contracts.rs with the existing stub harness: (a) abort -> marker back under its queued name AND no new turn starts on its own within two drain ticks; (b) after the next user input the held result is delivered; (c) completed turn -> marker gone, including a completed turn that went through a forced continuation; (d) open UI prompt -> no inject; (e) no before_settle event -> today's delete.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "An aborted or errored leader turn requeues its in-flight result claims and does not start a new turn on its own",
        "A held result is delivered on the next turn the user starts",
        "A completed leader turn deletes its in-flight result claims",
        "The drain does not inject while a UI prompt is open",
        "With no before_settle outcome the belt keeps today's delete"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/events.ts",
          "substantive": "settle branches on the recorded outcome"
        }
      ],
      "key_links": [
        "implements piggery-borrow D1 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-2",
    "feature": "piggery-borrow",
    "title": "Keep pi belt state on one globalThis slot and leader identity out of children",
    "lane": "standard",
    "role": "code",
    "deps": [
      "pbr-1"
    ],
    "decisions": [
      "1bf2611c-a21b-4204-9089-167337871cac"
    ],
    "files": [
      ".pi/extensions/bee-guard/state.ts",
      ".pi/extensions/bee-guard/index.ts",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/result-inbox.ts",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      ".pi/extensions/bee-guard/bee-cli.ts",
      ".pi/extensions/bee-guard/hooks.ts",
      ".pi/extensions/bee-guard/tool-dispatch.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      ".pi/extensions/bee-guard/state.ts",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/index.ts"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/work/pi-result-mailbox/delivery.md"
    ],
    "action": "Per piggery-borrow D5 (revised). Put the belt state on globalThis[Symbol.for(\"bee.pi.state\")] with ??=, fill missing default fields on a kept slot, and move inFlightClaims, promptDepths (result-inbox.ts:65,76), forcedContinuationSessions and pbr-1's outcome record and hold set onto it. Decide isPaseoLeader once per process; ONLY for the leader, keep PASEO_AGENT_ID in the slot and delete it from process.env. Move every reader to the slot: events.ts:80-83, :165-170, :355, :846-851; index.ts:141-144; paseo-heartbeat.ts:77-81. Pass the id back on every belt spawn of bee or paseo: bee-cli.ts, hooks.ts:46 and :147, tool-dispatch.ts:53, paseo-heartbeat.ts:439. Workers keep the id (worker Bash guard and hidden leader tools unchanged). Never strip BEE_HERDING_WORKER or BEE_HERDING_JOB_ID. Red first, and the reload test MUST fail before the fix (the harness caches child modules, so load the extension in a way that gives a fresh module scope, e.g. a cache-busting query on every module): reload keeps state and runs session-init once with one drain timer; a leader child sees no PASEO_AGENT_ID while bee and paseo spawns still get it; a worker still hides the leader tools and still runs the worker guard.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test pi_paseo_heartbeat_contracts --test pi_worker_guard_contracts && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "A /reload with a fresh module scope keeps belt state and does not re-run session-init",
        "A child of the leader's shell does not see PASEO_AGENT_ID",
        "The belt's bee and paseo spawns still receive the leader's agent id",
        "A Paseo worker keeps its id, its worker guard and its hidden leader tools"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/state.ts",
          "substantive": "state on a globalThis Symbol.for slot"
        }
      ],
      "key_links": [
        "implements piggery-borrow D5 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-3",
    "feature": "piggery-borrow",
    "title": "Check a contract version between the pi and opencode belts and the binary",
    "lane": "standard",
    "role": "code",
    "deps": [
      "pbr-2",
      "pbr-7"
    ],
    "decisions": [
      "d79958fd-9e3c-44a8-ae99-069baa00d1da"
    ],
    "files": [
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/hooks.ts",
      ".opencode/plugins/bee-guard.ts",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      ".pi/extensions/bee-guard/events.ts",
      ".pi/extensions/bee-guard/hooks.ts",
      ".opencode/plugins/bee-guard.ts",
      "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"
    ],
    "action": "Per piggery-borrow D4 (revised). Add BELT_CONTRACT_VERSION = 1 to the pi belt and the opencode plugin and send belt_contract: {belt: \"pi\"|\"opencode\", version: 1} inside the session-init stdin payload (no argv change to runAdvisoryHook). In hooks/session_init.rs read it, compare with the binary's BELT_CONTRACT (the pub(crate) const pbr-7 defines in doctor.rs), and on mismatch return one line naming the side to update (belt older: bee onboard --apply; binary older: the binary_freshness remedy). Absent field = unknown, silent. The belts show that line to the human: ctx.ui.notify on pi, a toast on opencode, as well as in the preamble. Red first: session_init tests for equal, belt older, binary older and absent; belt tests asserting the payload carries belt_contract and that a mismatch line is notified.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts --test opencode_plugin_contracts && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee hooks::session_init && .bee/bin/bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "Each belt sends its contract version in the session-init payload",
        "A mismatch returns one line naming the side to update and the belt shows it to the human",
        "A payload without a contract is accepted silently"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/session_init.rs",
          "substantive": "compares belt_contract with BELT_CONTRACT"
        }
      ],
      "key_links": [
        "implements piggery-borrow D4 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-4",
    "feature": "piggery-borrow",
    "title": "Make bee herding cancel stop the worker's whole process tree",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "7a25e4ed-c3c4-49cb-8b89-4dd72f2b4174"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md"
    ],
    "action": "Per piggery-borrow D2 (revised). In cancel_with_backends_and_timeout (job_verbs.rs:317; the Alive branch at :501-512), on unix, before closing the pane: read the process tree below the captured foreground pid with one ps -A -o pid=,ppid=,pgid= call, SIGTERM every member, wait up to 2 s, re-read, SIGKILL the members still alive; then close the pane and confirm exit as today. Put BOTH the tree reader and the signal sender behind seams; every existing cancel test (job_verbs.rs:1346-1348 passes the test runner's own pid) must use a no-op sender so nothing signals the test process. Keep the 5 s fail-closed rule and cancel_termination_failed. Windows keeps today's path; Paseo workers keep paseo stop; say both in the cancel help text. Red first: a unix test that starts sh which spawns a detached sleep under setsid, runs the real tree-kill path against that tree, and asserts both pids are gone.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::job_verbs",
    "must_haves": {
      "truths": [
        "Cancel stops detached children of the worker on unix",
        "No existing cancel test signals the test runner",
        "The fail-closed confirm and cancel_termination_failed are unchanged",
        "The cancel help says Windows and Paseo keep today's behavior"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
          "substantive": "tree read, TERM, wait, KILL behind seams"
        }
      ],
      "key_links": [
        "implements piggery-borrow D2 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-5",
    "feature": "piggery-borrow",
    "title": "Report each worker's context tokens and turns in bee herding status",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "7edc0153-b2fa-4ed1-b332-2aeb5882b3f4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/activity.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/devtools/statusline.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/activity.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/devtools/statusline.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/waves-and-occupancy.md"
    ],
    "action": "Per piggery-borrow D6 (revised). Make the activity hook write the worker's transcript_path (activity.rs:458 already reads it from the payload) into the job's mailbox activity record when BEE_HERDING_JOB_ID is set. In bee herding status (herding.rs), read that transcript with the usage parser bee dev statusline already has (statusline.rs:263, :372-385; expose it pub(crate) rather than copy it) and add ctx_tokens (input plus cache tokens of the latest assistant request) and turns (assistant messages) per job to the JSON, and a ctx=12.3k turns=4 suffix to the plain line (format 950, 12.3k, 1.2M). No transcript path or an unreadable file -> both null and no suffix. Do not touch the pi widget. Red first: unit tests on a fixture transcript (fields and suffix), a missing-path case, and the activity record write.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herding:: hooks::activity devtools::statusline",
    "must_haves": {
      "truths": [
        "herding status JSON carries ctx_tokens and turns per job when a transcript is recorded",
        "The plain line shows ctx and turns when known and omits them when not",
        "The activity hook records transcript_path for a herded job"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding.rs",
          "substantive": "status reads usage from the recorded transcript"
        }
      ],
      "key_links": [
        "implements piggery-borrow D6 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-6",
    "feature": "piggery-borrow",
    "title": "Refuse a herding dispatch over the declared limits at the door",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "ffa4e772-2448-4608-befa-e320fac645fa"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs",
      "packages/bee-rs/crates/bee/src/herding/wave_ledger.rs",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs",
      "packages/bee-rs/crates/bee/src/herding/wave_ledger.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/waves-and-occupancy.md"
    ],
    "action": "Per piggery-borrow D3 (revised). Add one read-only occupancy helper that prints nothing and writes nothing: wave_ledger::live_worker_count (wave_ledger.rs:278) over a quiet pane list, with Paseo agents counted live even when no pane list exists (do NOT call the occupancy verb at wave.rs:1093; it prints and marks orphans). Read herding.limits.concurrency and .depth from .bee/config.json. bee herding run refuses before spawn with the exact envelopes in D3 (limits.concurrency with live, limit, retryable:true and the fix text; limits.unverifiable; limits.depth). Read BEE_HERDING_DEPTH (absent = 0); export own depth + 1 next to BEE_HERDING_WORKER at both env build sites (run.rs:2663, :2866) ONLY when a depth limit is set, so pane export lines asserted at run.rs:9067-9183 stay byte-identical. bee dispatch prepare runs the same checks when the role resolves to Resolved::Herding (prepare.rs:228) and refuses with the role_not_configured shape (prepare.rs:2129). Add --explain: print each check (role, claim, limits) with pass or refuse, plain and --json, and return before any claim, reservation or record write. Document herding.limits in docs/config-reference.md. Red first: unit tests for each reason, the no-limits byte-identical case, the Paseo-without-panes count, and --explain writing nothing.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- verbs::drivers::prepare herding::run herding::wave",
    "must_haves": {
      "truths": [
        "A herding run at the concurrency limit is refused with limits.concurrency and retryable true",
        "A fallback occupancy with a concurrency limit is refused with limits.unverifiable",
        "A run at the depth limit is refused with limits.depth and depth is exported only when a depth limit is set",
        "dispatch prepare --explain prints each check and writes nothing",
        "A config without limits produces byte-identical payloads"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
          "substantive": "limits checks and --explain"
        }
      ],
      "key_links": [
        "implements piggery-borrow D3 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "pbr-7",
    "feature": "piggery-borrow",
    "title": "Print one advisory line per runtime with a fix command in bee doctor",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": [
      "da84d0f0-115e-47f4-a2cf-84c85033d716"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs"
    ],
    "read_first": [
      "docs/history/piggery-borrow/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"
    ],
    "action": "Per piggery-borrow D7 (revised). Add one ADVISORY row per runtime (claude, codex, opencode, pi, paseo) to every bee doctor run, in the advisory channel (doctor.rs:1276-1287) so the verdict (doctor.rs:1258) never changes because of them. Each row: tool found on PATH (locate only, never execute; pi on this machine is a wrapper script that changes global config), the repo's bee wiring present and current (reuse the existing wiring checks for claude, codex and pi; opencode wiring is .opencode/plugins/bee-guard.ts; paseo wiring is the herding.paseo config), and on every not-ok row a fix line (bee onboard --apply for stale or missing wiring, an install line naming the tool when only the tool is missing). The pi and opencode rows also compare the belt source's BELT_CONTRACT_VERSION constant (absent = unknown, reported, not a failure) with a binary const BELT_CONTRACT = 1 shared with session-init (define it where pbr-3 can import it, e.g. hooks/mod.rs is NOT in this cell, so define it in doctor.rs as pub(crate) and pbr-3 imports it). Red first: doctor tests for a current runtime, a stale pi belt (fix names bee onboard --apply), a missing tool, an absent runtime that never changes the verdict, and a belt contract mismatch.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee doctor",
    "must_haves": {
      "truths": [
        "bee doctor prints one advisory runtime row per supported runtime",
        "Every not-ok runtime row carries a fix line",
        "The runtime rows never change the doctor verdict",
        "The pi and opencode rows report the belt contract check",
        "No tool is executed to read a version"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/doctor.rs",
          "substantive": "advisory runtime rows with fix lines"
        }
      ],
      "key_links": [
        "implements piggery-borrow D7 as revised after the hat wave (docs/history/piggery-borrow/CONTEXT.md)"
      ],
      "prohibitions": [
        "No new code comments in any language (no_code_comments is on)",
        "No behavior change outside this cell's decision"
      ]
    },
    "behavior_change": true
  }
]
```

## Test matrix

Every cell is red-first in its own test home. After the last cell the leader
runs the full declared suite and a live proof of the CLI surfaces against a
throwaway onboarded repo: `bee doctor` runtime rows, a `herding.limits`
refusal and `--explain`, and `bee herding status` with a recorded transcript.
The pi belt surfaces (Esc, `/reload`) are proven by the stubbed contract
tests only; that narrowing is named in the caps.

## Open Questions

None. The hat wave's questions were answered in the revised decisions.

## Out of scope

Per-role can_dispatch and the pi widget suffix (both filed as backlog rows);
the skipped piggery ideas in CONTEXT.md.
