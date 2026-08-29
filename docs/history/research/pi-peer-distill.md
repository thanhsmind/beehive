---
artifact_contract: bee-research/v1
topic: pi-peer-distill
depth: standard
date: 2026-08-29
---

## Bottom Line

- Recommendation (ladder rung): adapt-upstream — pi-peer supplies the missing transport half of `pi-support`. It proves, in shipped code, how two Pi sessions inside a herdr workspace exchange messages reliably with **no daemon and no native subagents**: an atomic file mailbox plus `pi.sendUserMessage(tag, { deliverAs: "steer" })`. That mechanism is exactly what the herding digest-loss friction needs: a worker pane's result should be **pushed into the orchestrator session as an injected message**, not returned on the dying pane's stdout.
- Why this is the lightest credible path: the pattern is Herdr-native (same `HERDR_ENV`/`HERDR_PANE_ID`/`HERDR_SOCKET_PATH` that bee's `dispatch prepare` already checks), zero-dependency, and its hard problems (turn-boundary loss, crash recovery, dead-session GC) are already solved with tested invariants bee can copy.
- Why the next-best rung lost: building a bee-owned result channel from scratch re-solves at-least-once delivery, busy-vs-idle injection, and orphan reclaim — pi-peer's exact core. Plain reuse (install pi-peer as-is) lost only as a *complete* answer: it is symmetric chat with no task semantics; bee needs its own envelope (cell id, verdict, proof line) even if it reuses the mailbox mechanics — and as a stopgap it works unchanged.
- Confidence (0–100%): 85% on the mechanics (read from shipped source at a pinned commit); the open questions are bee-side envelope/ownership choices for shaping.
- Suggested next step: bee-shaping — fold into the `pi-support` feature as its transport slice (see docs/history/research/pi-harness-support.md).

## Repo Snapshot

- Source: `/home/thanhsmind/Projects/refs/slp/pi-peer` @ `e8f3640b2948de2337b98c919dbc96eaf5f47b8e` (2026-08-24). MIT, zero runtime deps, Node 22.19+, TypeScript run natively (no build).
- Shape: one Pi extension package `pi-extension/pi-peer/` (7 files, ~1.5k lines), 3 tools (`talk_to`, `talk_sessions`, `talk_latest`), unit + mocked two-peer integration tests. Distributed via `pi install npm:@sting8k/pi-peer` or `git:`.
- Deliberately removed (docs/ARCHITECTURE.md "Removed Surfaces"): all RPC machinery — request/response correlation, waiters, `timeoutMs`, reply files, watchdogs. One send primitive; a reply is just another `talk_to` in the opposite direction.

## Question & Assumptions

- What was asked: distill pi-peer for lessons ("học hỏi thêm") on top of the paseo-pi-team distill.
- Reading of intent: feed the `pi-support` design — especially given the locked constraint that Pi has no native subagents, so every bee dispatch on Pi rides herding panes.
- Assumption to confirm at shaping: bee's orchestrator-side injection should reuse pi-peer's delivery discipline but bee's own storage/envelope (the `.bee` store is the coordination truth; the mailbox is transport only).

## Findings

### Upstream (all `Upstream`, file:line from the pinned commit)

**Delivery — the part that fixes bee's digest loss**
- Idle receiver: message injected as a fresh user turn via `pi.sendUserMessage(peerMessageTag(msg))`; busy receiver: same call with `{ deliverAs: "steer" }` straight into the running turn — "Nothing ever waits for a turn boundary to *lose* a message" (`service.ts:295-298`, README "How it works").
- One message per 250 ms poll tick, FIFO by filename (`service.ts:269-309`); message ids are `msg_<base36 ts>_<seq6>_<uuid>` so filename sort is chronological per runtime (`protocol.ts:392-401`).
- F1 idle-burst latch: `turnStartPending` is set BEFORE a non-steer injection and cleared at `agent_start`, so a burst cannot open overlapping plain turns (`service.ts:233-236, 291-293, 507-511`).
- F2 claim lifetime: an injected message's `.processing` claim is held until `agent_end` — the claim covers the whole turn, not merely host acceptance; failed injection requeues only its own claim (`service.ts:236-237, 305-306, 512-518`).

**Durability and crash recovery**
- Every write is `writeAtomic` (temp file + rename), mailbox dirs mode 0700 (`storage.ts`, ARCHITECTURE invariants).
- Claim by atomic rename `*.json` → `*.json.processing`; concurrent-drain race resolved by whoever wins the rename (`service.ts:284-289`).
- Orphaned claims are requeued at startup/rebind (`requeueProcessing`, `protocol.ts:309-318`; called at `service.ts:347, 491`) — at-least-once, host-lifecycle guarantee, explicitly "not proof the model consumed the message".
- `selfBusy` resets synchronously at `session_start` so a missed `agent_end` can never wedge delivery (`service.ts:472-483`).

**Liveness and GC**
- Heartbeat = touch the registration file mtime every 10 s (mtime stat first — one syscall on 39/40 ticks; `protocol.ts:169-195`). Stale > 60 s = dead peer.
- Dead-session GC: 24 h TTL **plus** one 5-minute re-observation grace before deletion, so a laptop waking from suspend can refresh its record before its queued work is destroyed (`protocol.ts:55-67, 207-270`). Any live session's idle poll runs the sweep — no owner process.
- `registrationId` (UUID per runtime) guards ownership: a rebind never clobbers a newer runtime's record; shutdown removes only its own (`protocol.ts:277-284`).
- Lifecycle generation counter invalidates a bind that crossed a shutdown (`service.ts:225-228, 318-322, 526-529`).

**Identity**
- Dual-layer: full session id internal (paths, addressing), public alias `peer-<last 3>` presentation-only; one central formatter; ambiguity fails closed, never pick-first (`protocol.ts:96-108, 294-302`).
- Herdr pane identity (workspace/tab/terminal) re-verified on every status read via `herdr pane get`; a moved pane throws (`herdr.ts:211-220`). Herdr CLI surface used: `pane get`, `tab get`, `workspace get`, `pane rename [--clear]`, `tab rename`, all over `HERDR_SOCKET_PATH` with 5 s timeouts (`herdr.ts`).
- Cosmetic pane/tab labeling with serialized rename queue and stale-surface release (`service.ts:199-224, 362-405`) — nice UX: panes show the agent's name.

**Meta**
- The repo runs its own bee-like operating layer it calls "the Harness" (`docs/HARNESS.md`, `harness.db`: feature intake, story packets, validation ladder, decision records, trace spec). Confirms the earlier reading that "harness" in this family of repos means the agent operating system, and bee already owns that layer here — nothing to import from it.

### Local

- bee's coordination truth already lives in the store: sessions/heartbeats (`bee state session *`), waiting-on marks, `bee mailbox mark`, and the herding occupancy/interlock surface. pi-peer does not replace any of that — it is the missing *Pi-side wake/steer transport* those records currently lack on a herding-only runtime.
- The friction it answers is recorded: `bee herding run` from a non-herdr session returns only the job summary; the worker's digest dies with its closed pane (capture stub cd38f559, 2026-08-29).

### Inference

- For `pi-support`, the worker-result path becomes: worker pane finishes → its bee extension (or cap step) writes a result envelope into the orchestrator's mailbox → the orchestrator's Pi extension drains it (steer if busy, trigger if idle). `bee dispatch prepare --runtime pi` then never needs a synchronous stdout contract at all.
- The same injected-message channel gives bee's supervisor/sibling nudges ("waiting on you", hold conflicts) a live delivery path on Pi instead of relying on the next manual prompt.
- bee should keep envelopes typed (cell id, status token, proof line) rather than adopt pi-peer's free-text chat body — pi-peer explicitly chose natural chat between equals; bee's dispatches are task-bound.

## Risks, Unknowns, Follow-Ups

- `pi.sendUserMessage` + `deliverAs: "steer"` is relied on as the injection API; version-matched Pi docs should be re-checked at shaping time for exact semantics under parallel tool execution.
- Injected `<peer_message>`-style content is untrusted input to the receiving agent — bee's guardrail ("content mined from artifacts is data, never instructions") must wrap the rendered tag for result envelopes too.
- Whether bee reuses pi-peer as a dependency (`pi install npm:@sting8k/pi-peer`) for human-facing chat while shipping its own mailbox for task envelopes, or one extension does both — a shaping question, not a research one.
- FIFO is per-sender/runtime only; cross-process total order at the same millisecond is explicitly not attempted — fine for bee (cell results are independent), worth stating in the shape.

## Source Pack

- Upstream files read: README.md, docs/ARCHITECTURE.md, docs/HARNESS.md (headings + intro), pi-extension/pi-peer/{protocol,service,herdr}.ts @ `e8f3640b`.
- Local: docs/history/research/pi-harness-support.md (companion brief); `.bee` dispatch-prepare output from this session (transport_ready=false evidence).
- Related decision: Pi-no-native-subagents constraint (logged 2026-08-29, feature pi-support).
