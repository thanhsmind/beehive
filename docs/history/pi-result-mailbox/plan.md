---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Pi Result Mailbox

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change, multi-domain, external-systems
Why this is the least workflow that protects the work: the result contract is what every herding proof line rests on; a silent report drop or an unhandled redelivery corrupts orchestrator judgment.

Review wave: reviewer dispatch a56966b0 (FAIL — 4 P1, 9 P2) and advisor consult f7229ea9 (proceed-with-conditions, 4). This rev 2 folds in every named fix. Headline reversals from rev 1: the delivery guarantee is **at-least-once with job-id dedupe**, never exactly-once; injection is **path-only** (no inline report — fence-escape and truncation both die); `report_path` is **additive-only** (present only when a report exists, per the envelope's no-new-key law); "detached" becomes a **first-class flag** on `herding run` because no detach fact exists today.

## Requirements (from CONTEXT.md)

- D1: `report-{round}.md` beside `result-N.json`, atomic; result without report stays legal.
- D2 with a NAMED DEVIATION (reviewer P2 truncation + advisor condition 1): the run output carries `report_path` when a report exists — **no inline `report` at any size**. D2's letter permits inline ≤ cap; one `println!` line read through the orchestrator's Bash tool truncates, and a truncated envelope is unparseable — strictly worse than today's loss. Path-only preserves D2's intent (the orchestrator gets the report) at the cost of one file read. Decision logged.
- D3: the brief's contract template instructs report-first/result-last; worker stays bee-ignorant.
- D4 (store `f979d4c5`) with a RECORDED TERM REFRAME (reviewer P2): the "result inbox" holds **pending markers written at dispatch**, not finished envelopes — the drain stats each marker's job mailbox per tick. A never-finishing job leaves a visible marker (bounded, listable; named limit, not a hazard).
- D5: injected content is data — fenced header (job id, cell id, summary, proof, `report_path`) only; the report body NEVER rides the injection.
- D6: one delivery PATH per job, structural: only `--inbox-session` dispatches write a marker; the sync path writes none. WITHIN the async path the guarantee is **at-least-once** (pi-peer's own law) — the injected header's job id is the dedupe key, and docs/tests say so.
- D7: caveat lift text names the real limits: at-least-once async delivery, live-session drain, sync path as the primary contract.

## Discovery

Rev-1 anchors stand (mailbox.rs:320-345 template, :378 MailboxResult, :533-571 tolerant parse; run.rs:2161 brief write, :2471 continue rounds, :1907 poll). Rev-2 additions, all reviewer-verified:

- **No detach fact exists**: `run.rs:266-333` full flag list — no detach, no session id; both exec paths block in `wait_for_round` (`:2213`, `:2521`). "Background" today means the orchestrator's shell backgrounds the blocking verb (swarming-reference.md:445-449). Wave never writes this mailbox (`wave.rs:792` comment only).
- **Session id does not reach the child**: `resolve_session_id_no_flag` (`state_group/store.rs:398-418`) is env-based and answers `None` under >1 live session; the Pi belt passes its id only on hook stdin (`bee-guard.ts:497-504`).
- **Envelope key-set law**: `run.rs:5133-5156` asserts the exact sorted key set; `the-run-verb-and-worker-outcomes.md:211-212` records the no-new-key law; `options`/`leaning`/`dissent` are the additive precedent (`run.rs:2706-2734`).
- **Round is discarded before the envelope**: `read_result` (`run.rs:1810-1829`) drops the round; `result_envelope` (`run.rs:2686`) is a pure builder — the report resolves at the parse site (where fs access exists), rides the result struct, and the builder stays pure.
- **Same-round resume** (`run.rs:2366-2377`) can leave a stale `report-N.md`; **malformed result** (`run.rs:1827`, `:2747-2749`) currently drops everything.
- **TS harness**: stub `pi` implements only `on` (`pi_plugin_contracts.rs:307-312`); `run_harness` has no timeout (`:405`) — a load-time timer hangs every fixture; never-throw fixture hand-lists events (`:1202-1210`). Belt wires `agent_settled`, not `agent_end` (`bee-guard.ts:610-616`) — F1/F2 map onto `before_agent_start`/`agent_settled`.
- **prm-4 real targets**: caveat at `.bee/config-sample.json:28` (compiled via `onboard/templates.rs:253` `include_str!` — touches the template test), `docs/config-reference.md:182, 226, 448`, `catalog-projections-and-activation.md:250`, `model-roles-and-escalation.md:202`; envelope doc `skills/bee-swarming/references/swarming-reference.md:450-456` (skills path — regen obligation); knowledge homes `bee-herding/the-run-verb-and-worker-outcomes.md` + `handing-a-foreign-agent-its-brief.md`.

## Approach

1. **Report rides the mailbox** (prm-1): template instructs report-first/result-last, and instructs a RESUMED attempt to rewrite the report before its result (stale-report guard). `MailboxResult` gains `report_path: Option<String>` + the resolved round; `read_result` resolves the report at parse time: worker-declared path wins; else the convention probe accepts `report-{round}.md` only when its mtime ≥ the round's brief/ack delivery time — older is reported as `report_note: "stale report from an earlier attempt"` (advisor condition 3's explicit expected-but-missing fact). The Malformed arm surfaces an existing report path beside the error (a good report never dies with a broken result JSON). Envelope: `report_path` (and `report_note`) added ONLY when present — the exact-key test at `run.rs:5133` is EXTENDED with new additive rows, never loosened.
2. **The detach fact becomes a flag** (prm-1): `bee herding run --inbox-session <token>` — presence IS the detached fact (D6 structural); the marker `.bee/result-inbox/<token>/<job-id>.json` (atomic, written BEFORE the pane spawns — advisor condition 3) holds job id, mailbox path, cell id. No flag → no marker → sync-only delivery. The token is the orchestrator session id the Pi preamble already shows the model; `dispatch prepare --runtime pi`'s herding payload documents passing it. Unresolvable/absent token = no marker + a stderr note (matrix row).
3. **The Pi drain** (prm-2): registered only from `session_start` (never load-time), timer `.unref()`'d; per tick: list own inbox, for each marker whose mailbox holds `result-N.json` → atomic rename to `.processing` → inject the FENCED HEADER ONLY (job id, cell id, summary, proof, report_path; body read is the orchestrator's own next move) — steer when busy, plain turn when idle with the F1 latch on `before_agent_start`; claims consumed at `agent_settled` (F2), requeued on failed injection, orphans reclaimed at `session_start`. At-least-once; the job id in the header is the dedupe key. Fence content is header-only and fixed-shape, so fence-escape has no carrier (advisor condition 1 satisfied by construction).

Rejected alternatives (rev-1 set plus):
- Inline report in the run output or the injection — truncation unparseability + fence escape (reviewer P2 ×2, advisor condition 1).
- Exactly-once via delivered-set persistence — heavier than the at-least-once + dedupe-key contract pi-peer already proved; D6's path-split still kills the sync/async double.
- Deriving "detached" from shell backgrounding — invisible to bee; the flag is the fact.

Risk map: result contract + probe freshness / HIGH / roundtrip, legacy, stale-report, malformed-with-report fixtures · envelope additive keys / MEDIUM / extend `run.rs:5133` family · marker lifecycle / MEDIUM / flag-presence rows incl. no-token · TS drain / HIGH / extended harness (stub `sendUserMessage`, harness timeout, never-throw rows for new events) · docs / LOW / pointer checks + template test.

## Shape

One slice, 4 cells:

1. `prm-1` (role: code, Rust) — mailbox.rs template (report instruction + resume-rewrite rule), `MailboxResult` report fields + round, `read_result` resolution (declared > fresh convention probe > stale note; Malformed surfaces report), envelope additive keys, `--inbox-session` flag + pre-spawn marker write. Tests: all Q3/Q4 rows + marker rows. Cites D1, D2 (deviation), D3, D6.
2. `prm-2` (role: code, TS; deps: prm-1) — the drain in `.pi/extensions/bee-guard.ts` per Approach 3; session_start-registered, `.unref()` timer, header-only fenced injection, F1 on `before_agent_start`, F2 on `agent_settled`. Cites D4 (reframe), D5, D6.
3. `prm-3` (role: test; deps: prm-1, prm-2) — extend `tests/pi_plugin_contracts.rs`: harness gains a hard timeout and a `sendUserMessage`-capable stub; rows: busy→steer, idle→trigger, failed-injection requeue, orphan reclaim at session_start, replay-after-restart (at-least-once with same job id — the DOCUMENTED behavior), marker-without-result not injected, no-marker (sync) never injected, header-only fence (no report body), never-throw rows for every new `pi.on` event added to the hand list. Cites D4, D5, D6; store `f979d4c5`.
4. `prm-4` (role: docs; deps: prm-1, prm-2) — the six caveat/envelope sites named in Discovery (swarming-reference is skills/** → this cell acks wave-barrier regen), the two bee-herding knowledge homes, replacement text naming the at-least-once limits (advisor condition 4). Cites D7, D1.

Dependencies: prm-1 → prm-2 → (prm-3 ∥ prm-4).

## Test matrix

- **Contract/compat**: legacy result (no field, no file) → envelope byte-identical to today (exact-key test green unchanged); report present → `report_path` key added; declared path wins over probe.
- **Freshness/order**: resumed same-round attempt with stale report → `report_note`, never silent attach; report-first instruction present in rendered brief; malformed result + good report → error AND report_path both surfaced.
- **Delivery**: sync (no flag) → no marker, no injection; `--inbox-session` → marker before spawn; marker+no-result → no injection; replay after restart → second injection with same job id (documented at-least-once), dedupe key present in header; no-token dispatch → no marker + stderr note.
- **Injection safety**: fence contains header fields only — a report body containing a fence never reaches the injection; steer vs trigger by busy state; F1 burst latch; drain never throws (missing dirs, malformed marker, malformed result).
- **Harness**: all existing Pi fixtures still terminate (timeout guard proves no hang); new events in the never-throw list.
- **Docs**: caveat text gone from all six sites; onboard template test green with the edited sample.
- Not applicable, named: data migration; auth; model-consumption proof (out of scope, distill:41 — named in docs).

## Out of scope

- pi-peer changes; dispatch-door law; wave orchestration; Claude/Codex/OpenCode async notify.
- Exactly-once delivery machinery; result-inbox GC beyond consumed markers (stale markers are visible and listable).
