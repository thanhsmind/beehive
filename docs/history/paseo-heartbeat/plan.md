---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: paseo-heartbeat

## Summary

A Pi leader that runs inside Paseo now makes its own Paseo heartbeat when its
session starts, once per agent, every 5 minutes by default. Each time the
heartbeat fires, the bee extension runs the broker tick. If the tick routed
something, the leader gets a short prompt with the news; if not, it answers
one word, so Paseo marks the leader idle again. Workers never make or handle
a heartbeat. Worker results keep their own 2-second drain.

Mode: `high-risk` — 2 risk flags: external systems (Paseo heartbeats, a hard-gate flag), public contracts (two new config keys)
Why this is the least workflow that protects the work: the heartbeat creates Paseo state that bee must not duplicate, and a wrong turn shape stalls the leader (paseo-pi spike 2), so the live proof is the deciding check.

Playbook: `skills/bee-planning/playbooks/feature.md`. Step 1 (data shape) is the marker file and the two config keys below. Step 2 (walking skeleton) is phb-1 plus phb-2 driven live. Step 3 holds: slice 3 (paseo-pi D5) stays out. Step 4 is the live heartbeat proof on phb-2's cap. Step 5 is phb-3.

Deviation (named): the plan-step hat wave is not re-run for this feature. The paseo-pi wave (`docs/history/paseo-pi/reports/hat-wave.md`, "Next slices") already critiqued this slice with five seats, and its four slice-2 findings are requirements below. The advisor ref points at that synthesis.

## Requirements (from CONTEXT.md)

- D1 (paseo-pi D6): each heartbeat runs the code tick and becomes a news prompt or one very short turn; never swallowed.
- D2 (paseo-pi D7): every 5 minutes by default; a config key changes it.
- D3 (paseo-pi D8): the leader turns it on at session start inside Paseo; never a second heartbeat for the same agent.
- From the paseo-pi hat wave: leader-only guard; one tick at a time; the no-news turn is exempt from the continuation nudge; the no-news turn is minimal.

## Load-bearing claims

Labels are `read` or `ran`; evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The extension sees every prompt before the model in one `input` handler | read | .pi/extensions/bee-guard/events.ts:213 | `pi.on("input", (async (event: any, ctx: any) => {` |
| 2 | That handler returns `continue` today, so a `transform` return is a new branch | read | .pi/extensions/bee-guard/events.ts:239 | `return { action: "continue" }` |
| 3 | Session start is where the extension arms per-session work | read | .pi/extensions/bee-guard/events.ts:81 | `startResultDrain(pi, directory, sessionIdOf(ctx), ctx)` |
| 4 | The continuation nudge injects a turn on a block verdict at settle | read | .pi/extensions/bee-guard/events.ts:434 | `await pi.sendUserMessage(parsed.reason)` |
| 5 | Worker results already wake the leader through a 2-second drain | read | .pi/extensions/bee-guard/result-inbox.ts:49 | `export const DRAIN_POLL_MS = 2000` |
| 6 | The broker tick reports what it routed as JSON | read | packages/bee-rs/crates/bee/src/herding/broker.rs:695 | `"heartbeat": outcome.heartbeat,` |
| 7 | Paseo wraps a heartbeat prompt in a system envelope that names the schedule | ran | spike 2: `paseo send <id> "<paseo-system>Schedule \"bee\" fired (id=x, run=1). bee tick</paseo-system>"` and server `schedule/service.js:30` | `Schedule "${schedule.name}" fired (id=${schedule.id}, run=${runId}).` |
| 8 | A swallowed prompt leaves Paseo showing the agent running | ran | spike 2 `paseo inspect f236f8a4-43c2-4411-b2a4-3a6b2607c32e` after a `handled` input | `Status              running` |
| 9 | An extension-started turn settles Paseo to idle | ran | spike 2 `paseo inspect 3317e669-9340-4afe-a7a3-cc9f40c81822` after `sendUserMessage` | `Status              idle` |

## Discovery

Read `events.ts`, `result-inbox.ts`, `broker.rs` and the Paseo 0.10.3 CLI.
Finding: the result drain (claim 5) already starts a turn per worker result,
so the heartbeat tick only needs the broker. `paseo heartbeat create` must run
inside the agent (it reads `PASEO_AGENT_ID`) and prints one schedule row with
an `id`.

## Approach

**Recommended path.** A new module `.pi/extensions/bee-guard/paseo-heartbeat.ts`
holds pure helpers; `events.ts` calls them in three places.

- Leader test: `PASEO_AGENT_ID` non-empty and `BEE_HERDING_WORKER` unset (D3).
- Heartbeat test: the prompt starts with `<paseo-system>` and contains
  `Schedule "bee-leader" fired`. Only that schedule name counts.
- Config: `herding.paseo.command` (default `paseo`, the key slice 1 added)
  and a new `herding.paseo.heartbeat_cron` (default `*/5 * * * *`) (D2).
- Session start (D3): if leader, create `.bee/runtime/paseo-heartbeat/<agent id>.json`
  with an exclusive create (`wx`); only the winner runs
  `<paseo> heartbeat create --cron <cron> --name bee-leader --json "bee heartbeat"`
  and writes the returned schedule id into the file. A file that already
  exists means the heartbeat exists: do nothing. A failed create removes the
  file so the next session start can try again. Never throws.
- Input (D1): if leader and heartbeat prompt, run
  `<bee> herding broker tick --json` (timeout 120 s), one tick at a time (a
  second heartbeat while a tick runs gets the no-news text). `claimed` or
  `notices_sent` above 0 is news. Return
  `{action: "transform", text}`: news → `bee heartbeat: the broker routed <claimed> question(s) and sent <notices_sent> notice(s). Read them with bee orient and act.`;
  no news or a failed tick → `bee heartbeat: no news. Reply with the single word ok and do nothing else.`
  Never `handled`.
- Settle: a turn that came from a no-news heartbeat skips the continuation
  nudge injection, so an empty heartbeat never costs a second turn.

**Rejected.**
- Swallowing the heartbeat (`handled`): stalls Paseo (claim 8).
- A timer-only design with no Paseo heartbeat: the user chose the Paseo heartbeat (paseo-pi D6).
- Creating the heartbeat from Rust: `paseo heartbeat create` must run inside the agent's own process tree to read `PASEO_AGENT_ID`.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Duplicate heartbeats (two `session_start` events) | HIGH | phb-1 | test: two concurrent ensure calls run create once |
| A worker makes or handles a heartbeat | MEDIUM | phb-1, phb-2 | test: worker env makes no create and returns continue |
| Heartbeat stalls Paseo | HIGH | phb-2 | live: two heartbeats fire and the leader is idle after each |
| Nudge doubles each empty heartbeat | MEDIUM | phb-2 | test: no-news heartbeat turn skips the nudge |
| Normal prompts change | MEDIUM | phb-2 | existing pi_plugin_contracts tests stay green |
| A hung create leaves an empty marker and no heartbeat (found live) | HIGH | phb-4 | test: an empty marker older than 120 s is retried; a fresh one is not |

Waves: wave 1 runs phb-1 and phb-3 in parallel (disjoint files); phb-2 runs after phb-1 because it calls phb-1's helpers and shares its test file.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"phb-1 and phb-2 add the heartbeat module and wire it into the extension."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each code cell writes its own tests red-first; the leader drives the live heartbeat."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"phb-3 adds the heartbeat section to the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Discovery is done."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"required","role":"review","reason":"The slice judge for the behavior cell phb-2 dispatches the review role."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The paseo-pi hat wave is the advisor consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Covered by the paseo-pi wave (named deviation)."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Covered by the paseo-pi wave (named deviation)."}
  ]
}
```

## Shape

Phase plan, one slice.

| Phase | What Changes | Why Now | Demo | Unlocks |
|---|---|---|---|---|
| 1 | Leader heartbeat | paseo-pi D6-D8 | a Pi leader in Paseo gets a `bee-leader` heartbeat; each firing ends with the leader idle | slice 3 answers can rely on a woken leader |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| phb-1 | Add the Paseo heartbeat helpers | .pi/extensions/bee-guard/paseo-heartbeat.ts (new); packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs (new) | — | nothing yet; unblocks phb-2 | helper tests green |
| phb-2 | Make the Pi leader create and answer its Paseo heartbeat | .pi/extensions/bee-guard/events.ts; packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs | phb-1 | a Pi leader in Paseo gets one `bee-leader` heartbeat, and each firing becomes a news prompt or one "ok" turn | tests green, then a live run |
| phb-3 | Document the leader heartbeat | docs/knowledge/areas/bee-herding/the-paseo-channel.md | — | the concept explains the heartbeat, its keys and its turns | knowledge check green |
| phb-4 | Retry a heartbeat whose create never finished | .pi/extensions/bee-guard/paseo-heartbeat.ts; packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs | phb-2 | a leader whose first heartbeat create hung gets its heartbeat at the next session start instead of never | heartbeat tests green |

```json
[
  {
    "id": "phb-1",
    "feature": "paseo-heartbeat",
    "lane": "high-risk",
    "role": "code",
    "change_class": "api",
    "title": "Add the Paseo heartbeat helpers",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-heartbeat/CONTEXT.md",
      "docs/history/paseo-heartbeat/plan.md",
      ".pi/extensions/bee-guard/result-inbox.ts",
      ".pi/extensions/bee-guard/bee-cli.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"
    ],
    "action": "Red first. Create .pi/extensions/bee-guard/paseo-heartbeat.ts, plain TypeScript in the style of result-inbox.ts (node: imports, exported functions, no default export, no code comments). Export: (1) isPaseoLeader(env = process.env): true when PASEO_AGENT_ID is a non-empty string and BEE_HERDING_WORKER is unset or empty. (2) isHeartbeatPrompt(text): true when text, trimmed at the start, begins with <paseo-system> and contains Schedule \"bee-leader\" fired. (3) HEARTBEAT_NAME = \"bee-leader\", DEFAULT_CRON = \"*/5 * * * *\". (4) readPaseoSettings(mainRoot): reads <mainRoot>/.bee/config.json and returns {command, cron}: command from herding.paseo.command else \"paseo\", cron from herding.paseo.heartbeat_cron else DEFAULT_CRON; a missing or bad file gives the defaults. (5) heartbeatMarkerPath(mainRoot, agentId) = <mainRoot>/.bee/runtime/paseo-heartbeat/<agentId>.json. (6) ensureHeartbeat(mainRoot, agentId, settings, run): creates the marker directory, then writeFileSync(marker, '{}', {flag: 'wx'}); on EEXIST returns {created: false, reason: 'exists'}; otherwise calls run(settings.command, ['heartbeat','create','--cron', settings.cron, '--name', HEARTBEAT_NAME, '--json', 'bee heartbeat']) which returns a Promise of stdout; parses the first JSON object in stdout and reads its id from key id, Id or ID; writes {agent_id, schedule_id, cron, created_at} into the marker and returns {created: true, id}; on any error removes the marker and returns {created: false, reason: <message>}; never throws. (7) parseTick(stdout): reads claimed and notices_sent numbers from the first JSON object (missing means 0); returns {claimed, notices_sent, news: claimed > 0 || notices_sent > 0}, or null for an unparseable body. (8) heartbeatText(tick): news gives `bee heartbeat: the broker routed ${claimed} question(s) and sent ${notices_sent} notice(s). Read them with bee orient and act.`; null or no news gives `bee heartbeat: no news. Reply with the single word ok and do nothing else.`. Create packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs modeled on pi_plugin_contracts.rs: it runs node (named skip when node is missing or too old, same rule as that file) on a small script that imports paseo-heartbeat.ts and prints JSON results; cover every export above, including two concurrent ensureHeartbeat calls on one temp mainRoot with a stub run that counts calls (exactly one call, one created true, one exists), a failed run that removes the marker, and the leader test with a worker env.",
    "must_haves": {
      "truths": [
        "isPaseoLeader is true with PASEO_AGENT_ID set and false when BEE_HERDING_WORKER is set or PASEO_AGENT_ID is empty",
        "isHeartbeatPrompt matches only a paseo-system envelope naming the bee-leader schedule",
        "readPaseoSettings returns paseo and */5 * * * * by default and the config values when set",
        "two concurrent ensureHeartbeat calls for one agent run paseo heartbeat create exactly once",
        "a failed heartbeat create removes the marker and returns created false without throwing",
        "parseTick reports news only when claimed or notices_sent is above 0 and returns null for a bad body",
        "heartbeatText gives the news text or the one-word ok text"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/paseo-heartbeat.ts",
          "substantive": "the heartbeat helpers"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
          "substantive": "node-run contract tests for the helpers"
        }
      ],
      "key_links": [
        "ensureHeartbeat uses an exclusive wx create of the marker before any paseo call"
      ],
      "prohibitions": [
        "No code comments",
        "No change to events.ts in this cell",
        "Never return or suggest action handled"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "phb-2",
    "feature": "paseo-heartbeat",
    "lane": "high-risk",
    "role": "code",
    "change_class": "behavior",
    "title": "Make the Pi leader create and answer its Paseo heartbeat",
    "deps": [
      "phb-1"
    ],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      ".pi/extensions/bee-guard/events.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-heartbeat/plan.md",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      ".pi/extensions/bee-guard/events.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "action": "Red first. In events.ts import the phb-1 helpers. session_start (events.ts:64): after the result drain starts, when isPaseoLeader(), fire and forget ensureHeartbeat(mainCheckoutRoot(directory), process.env.PASEO_AGENT_ID, readPaseoSettings(mainRoot), run) where run spawns the command with execFile (timeout 30 s, stdin closed) and resolves stdout; errors only console.error. input (events.ts:213): keep the existing steer/followUp branch unchanged; before returning, when isPaseoLeader() and isHeartbeatPrompt(text): if a tick is already running, use the no-news text; else run `<bee binary> herding broker tick --json` (resolveBeeBinary, cwd the directory, timeout 120 s), parse with parseTick, and build heartbeatText; set a module flag quietHeartbeatTurn = !news; return {action: 'transform', text}. Never return handled. agent_settled (events.ts:358): when quietHeartbeatTurn is set, clear it and skip the continuation nudge injection for this settle (the same effect hadForcedContinuation has); everything else in agent_settled runs unchanged. Add tests to pi_paseo_heartbeat_contracts.rs with the stub-pi harness: a leader session_start calls the stub paseo once with heartbeat create; a worker env (BEE_HERDING_WORKER=1) makes no call; a heartbeat input with a stub bee printing {\"claimed\":0,\"notices_sent\":0} returns transform with the ok text; with claimed 1 returns the news text; a normal input returns continue; a quiet heartbeat settle with a block verdict injects no nudge. The existing pi_plugin_contracts tests must stay green. No code comments.",
    "must_haves": {
      "truths": [
        "a leader session start creates the bee-leader heartbeat through ensureHeartbeat",
        "a herded worker session start creates no heartbeat",
        "a bee-leader heartbeat input returns transform with the news text or the ok text, never handled",
        "a normal user input still returns continue",
        "a no-news heartbeat turn skips the continuation nudge at settle",
        "the existing pi_plugin_contracts tests stay green"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/events.ts",
          "substantive": "session_start, input and agent_settled heartbeat wiring"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
          "substantive": "event-level heartbeat tests"
        }
      ],
      "key_links": [
        "events.ts input returns transform for a bee-leader heartbeat"
      ],
      "prohibitions": [
        "No code comments",
        "No action handled anywhere",
        "The steer/followUp branch of input stays byte-identical"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts --test pi_plugin_contracts",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "phb-3",
    "feature": "paseo-heartbeat",
    "lane": "high-risk",
    "role": "docs",
    "title": "Document the leader heartbeat",
    "deps": [],
    "decisions": [
      "D1",
      "D2",
      "D3"
    ],
    "files": [
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "read_first": [
      "docs/history/paseo-heartbeat/CONTEXT.md",
      "docs/history/paseo-heartbeat/plan.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "action": "In docs/knowledge/areas/bee-herding/the-paseo-channel.md replace the one planned-work line about slice 2 with a section 'The leader heartbeat' in the same ASD-STE100 style: who is a leader (PASEO_AGENT_ID set, BEE_HERDING_WORKER unset); the bee-leader heartbeat made once per agent at session start, its marker file .bee/runtime/paseo-heartbeat/<agent id>.json, and the keys herding.paseo.heartbeat_cron (default */5 * * * *) and herding.paseo.command; what each firing does (broker tick, news text or the one-word ok text, never a swallowed prompt, and why: a swallowed prompt stalls Paseo); that worker results do not wait for it (the 2-second result drain); that a no-news turn skips the continuation nudge; how to stop it (paseo heartbeat delete <id> from the marker, then delete the marker). Add 'paseo-pi D6, D7, D8' to the frontmatter decisions list. Change nothing else in the file.",
    "must_haves": {
      "truths": [
        "the concept names the bee-leader heartbeat, its marker file and both config keys with defaults",
        "the concept states news text, the ok text, and that a heartbeat is never swallowed",
        "the concept states how to stop the heartbeat",
        "the frontmatter cites paseo-pi D6, D7, D8"
      ],
      "artifacts": [
        {
          "path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md",
          "substantive": "the leader heartbeat section"
        }
      ],
      "key_links": [
        "the section links nothing new; it extends the existing concept"
      ],
      "prohibitions": [
        "No change to any other file"
      ]
    },
    "verify": ".bee/bin/bee knowledge check --json",
    "affects_skills": [],
    "affects_specs": []
  },
  {
    "id": "phb-4",
    "feature": "paseo-heartbeat",
    "lane": "high-risk",
    "role": "code",
    "change_class": "bugfix",
    "title": "Retry a heartbeat whose create never finished",
    "deps": [
      "phb-2"
    ],
    "decisions": [
      "D3"
    ],
    "files": [
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "read_first": [
      "docs/history/paseo-heartbeat/plan.md",
      ".pi/extensions/bee-guard/paseo-heartbeat.ts",
      "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs"
    ],
    "action": "Red first. Live run 2026-10-06: `paseo heartbeat create` through the AppImage CLI hung, its children were reparented, the run promise never settled, and the marker stayed `{}`, so every later session start saw EEXIST and the leader never got a heartbeat. In ensureHeartbeat (paseo-heartbeat.ts:116), on EEXIST read the marker: if it parses with a non-empty schedule_id, or its mtime is younger than STALE_MARKER_MS = 120000, return {created: false, reason: 'exists'} as today; otherwise it is stale: unlinkSync it and try the wx create once more (a loser of that second race returns exists), then continue with the create as usual. Add an exported STALE_MARKER_MS. When the create fails (catch branch), make the reason end with ` — FIX: set herding.paseo.command to the npm @getpaseo/cli paseo binary`. Tests in pi_paseo_heartbeat_contracts.rs: an empty marker with mtime 200 s ago is retried and the stub run is called once; an empty marker 10 s old returns exists with no call; a marker with a schedule_id 200 s old returns exists with no call; a failed create's reason contains herding.paseo.command. No code comments; no change to events.ts.",
    "must_haves": {
      "truths": [
        "an empty marker older than 120 seconds is removed and the create runs once",
        "an empty marker younger than 120 seconds returns exists with no create",
        "a marker with a schedule_id returns exists with no create, whatever its age",
        "a failed create reason names herding.paseo.command"
      ],
      "artifacts": [
        {
          "path": ".pi/extensions/bee-guard/paseo-heartbeat.ts",
          "substantive": "the stale-marker retry"
        },
        {
          "path": "packages/bee-rs/crates/bee/tests/pi_paseo_heartbeat_contracts.rs",
          "substantive": "stale-marker tests"
        }
      ],
      "key_links": [
        "ensureHeartbeat reads the marker on EEXIST before returning exists"
      ],
      "prohibitions": [
        "No code comments",
        "No change to events.ts"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_paseo_heartbeat_contracts",
    "affects_skills": [],
    "affects_specs": []
  }
]
```

## Test matrix

| Dimension | Probe | Cell | Pass when |
|---|---|---|---|
| Concurrency | two session_start events race | phb-1 | one `heartbeat create` call |
| Identity | worker env | phb-1, phb-2 | no create; input returns continue |
| Input extremes | tick prints a bad body | phb-1 | parseTick null; ok text |
| External failure | `paseo heartbeat create` fails | phb-1 | marker removed; created false; no throw |
| State | no-news heartbeat with a block verdict at settle | phb-2 | no nudge injected |
| Regression | normal input | phb-2 | `continue`; pi_plugin_contracts green |
| User-visible path | live Pi leader in Paseo, cron `* * * * *` | phb-2 cap | marker holds one id; two firings each end with `paseo inspect` Status idle; the timeline shows the ok text |

## Open Questions

- Slice 3 (paseo-pi D5) is a separate feature.

## Out of scope

- Deleting the heartbeat when the leader session ends; the concept tells the user how to stop it.
- Any change to the result drain.
