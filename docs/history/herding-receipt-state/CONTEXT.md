# Herding Receipt State — Context

**Feature slug:** herding-receipt-state
**Date:** 2026-08-20
**Shaping session:** complete (fix-first brief, tiny lane)
**Scope:** Quick

## Feature Boundary

`bee herding run`'s delivery receipt reads pane text for the brief-file needle, and the pane ECHOES the keystrokes the send itself types — during agent boot (live smoke smoke-agy-delivery-1/-2, 2026-08-20: deliver_pointer returned Ok ~6s after spawn, before the agy TUI was up; the echoed pointer satisfied the needle, the TUI then discarded the buffered input, the pane's input box stayed empty, and the run sat in wait_for_round until idle-timeout). A text receipt cannot distinguish "the agent accepted the prompt" from "the terminal echoed my own typing". One file.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | The delivery receipt is a STATE transition, not text: after each pointer send, poll `agent_status` (plus round-1 result presence) for up to a short per-attempt window; `working`/`done` (or result-1.json) is the receipt, `idle` past the window drives a resend. The pane-text needle check is dropped as a receipt. Bounded attempts stay (~30, ~1s apart); the pointer stays idempotent | Live smoke: a real accepted prompt flips agy to Generating (working) immediately; the echo-based receipt returned a false positive during boot and lost the brief |

## Evidence

- smoke-agy-delivery-1: job.json had pane recorded (post-receipt write) while the pane input box was empty for 60s+; agent idle forever; manual `herdr agent prompt <job-id> "ping by name"` landed instantly and flipped the pane to Generating.
- smoke-agy-delivery-2: identical; bee process alive 100s+ stuck in wait_for_round.
- waggledance-2a session: 4/4 agy runs needed a manual prompt; 3/3 manual prompts over the "Verifying your account" banner worked — the banner is harmless; the boot-echo race is the defect.
