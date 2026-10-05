# Paseo channel for bee on Pi — Context

**Feature slug:** paseo-pi
**Date:** 2026-10-05
**Shaping session:** complete
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

A Pi leader that runs inside Paseo can dispatch bee workers as Paseo agents,
answer their questions through Paseo messaging, and wake on a Paseo heartbeat.
The feature ends at the Paseo channel: herdr panes, native Agent workers and the
bee workflow itself do not change.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Paseo is a new herding channel beside herdr, chosen per team config. Herdr panes keep working unchanged. | Lower risk and an easy way back; bee must not depend on a pre-release Paseo. |
| D2 | On the Paseo channel, an answer to a worker question goes into the running worker through Paseo messaging (steer into the active turn). The stop-and-next-round rule of mailbox-broker D3 (store `caf5f716`) stays in force for herdr panes. | D3 banned typing into a terminal pane. Paseo messaging is a daemon API, not keystroke injection, so the D3 reason does not apply to this channel. |
| D3 | A Paseo heartbeat wakes the Pi leader's model only when there is news. The Pi extension catches the heartbeat prompt, runs the bee code tick (result drain, broker), and starts a model turn only when that tick returns work. | A model turn on every empty heartbeat costs money for nothing. |
| D4 | Each worker on the Paseo channel runs the provider and model that bee team config binds to its role (claude, codex, pi and others). Paseo only carries the worker. | The dispatch door stays the one place that picks the model. |

Store ids: logged 2026-10-05 with `bee decisions log --feature paseo-pi` (tags
`paseo-pi`).

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Paseo channel | The herding transport that starts a worker as a Paseo agent instead of a herdr pane. |
| Heartbeat | A Paseo cron prompt that Paseo sends to the leader agent. |
| Code tick | One run of bee's existing code-only work (result drain, mailbox broker) with no model call. |

## Specific Ideas And References

- Seatworks (`/home/thanhsmind/Projects/refs/seatworks`, commit `2e11099f`) is the
  reference for how a Paseo plugin seats agents, sends mail and steers
  (`plugin/server/core/paseo-adapter.ts`, `plugin/server/runtime/outbox.ts`,
  `plugin/server/runtime/patrol.ts`).
- Prior evaluation: `docs/history/research/seatworks-slp-pi-small-models.md`,
  `docs/history/research/seatworks-xia.md`.

## Existing Code Context

### Reusable Assets

- `.pi/extensions/bee-guard/` — the Pi extension that already holds bee dispatch,
  verdict, result inbox and steer tools.
- `packages/bee-rs/crates/bee/src/herding/` — herding run, job mailbox, broker and
  control loop.

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs` — the dispatch door
  that picks the channel.
- `packages/bee-rs/crates/bee/src/herding/broker.rs` — where an answer reaches a
  worker.

## Canonical References

- `docs/knowledge/areas/bee-herding/the-mailbox-broker.md` — the broker and the
  question outcome.
- `docs/history/mailbox-broker/CONTEXT.md` — mailbox-broker D1–D6.

## Outstanding Questions

### Resolve Before Planning

None.

### For Planning

- [ ] Can the installed Paseo (0.6.1) steer into an active turn, or does D2 need a
  newer Paseo? Research digest answers it.
- [ ] Which Pi extension event lets the extension handle a heartbeat prompt
  without a model turn (D3)? Research digest answers it.
- [ ] How does a worker inside a Paseo agent hand back its result — the existing
  file mailbox, or a Paseo finish callback?
