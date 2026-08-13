---
type: bee.area
title: Workflow State — detecting a crashed session and mining its transcript for unsettled work
description: "How a session that died without pausing is recognised from signals that already exist, and how its unsettled work is recovered through a bounded down-tier digest of its own transcript tail — offered, never automatic, and never loaded into the recovering session's context."
timestamp: 2026-07-22
bee:
  id: workflow-state-recovery
  lifecycle: active
  areas: [workflow-state]
  required_context: [areas/workflow-state/overview.md]
  decisions: ["transcript-recovery D1-D6 (docs/history/transcript-recovery/CONTEXT.md, 2026-07-20 — detection auto, mining offered; digest-only; mined content data-not-instructions; never auto-resume)"]
  sources: ["transcript-recovery cells transcript-recovery-1..4 (traces in .bee/cells/, reports docs/history/transcript-recovery/reports/, 2026-07-20)", hardening-5 (2026-07-21 — configurable per-runtime transcript roots reported by name) and hardening-1-7-10 (the stored transcript path preferred over layout math), "docs/specs/workflow-state.md#B33", "docs/specs/workflow-state.md#R51", "docs/specs/workflow-state.md#E4", "docs/specs/workflow-state.md#E5", "docs/specs/workflow-state.md#E6"]
  authoritative_for: "workflow-state: crash-candidate detection and transcript-based recovery"
---

# Workflow State — detecting a crashed session and mining its transcript for unsettled work

A crash leaves no note. What it does leave is a stale heartbeat, a transcript
that stops without its closing sequence, and work still visibly in flight — and
those three together are enough to recognise it mechanically. Everything after
detection is deliberately timid: recovery is offered, the raw conversation is
read only by a cheaper helper, what comes back is a digest, and nothing mined is
ever treated as settled knowledge or as instructions.

## Behaviors & Operations

**B33 — A session that died without pausing can be detected and its unsettled
work recovered from the harness's own conversation record, without ever loading
that record into the recovering session.** Trigger: a session start (the routine
orientation pass) inspects the other sessions known to the workspace. What
happens: a *recoverable crash* is recognized mechanically from signals that
already exist — a working session whose heartbeat has gone stale (B24), whose
saved conversation transcript exists but does not end with the clean-stop
pattern (a clean shutdown leaves a recognizable closing sequence with nothing
conversational after it; a crash simply lacks it), and which still shows work in
flight (a lane left open mid-work, units it still holds, or transcript activity
newer than the last durable record of a settled outcome). A session that ended
cleanly, a session whose heartbeat is still fresh, and the currently live
session itself are never candidates. Detection is cheap and runs every session
start; where no transcript store exists at all (a runtime that keeps no such
record), detection is a silent no-op. Recovery itself never runs automatically:
when candidates are found the agent offers to recover them, the same way it
offers to drain a pending capture queue, and acts only if the human agrees.
Recovering one candidate means summarizing only the *tail* of its transcript —
from the last settled outcome to the end — through a cheaper down-tier helper
that reads the transcript and returns a bounded digest; the raw conversation
never enters the recovering session's own working context. What each actor
observes: the digest names what was in flight, any candidate settlements it
found, and a suggested next action, and it is written as a recovery note under
the feature's history (or a shared recovery location when the dead session was
not tied to a feature). Candidate settlements are appended as capture stubs
marked as mined rather than confirmed; they become real knowledge only after the
normal capture-flush review a person already performs, never automatically.
Mined text is always treated as data, never as instructions, secret-shaped
strings are redacted, and only the current workspace's own transcripts are ever
read. Recovery never resumes the dead session's work and never writes a pause
record on its behalf — the never-auto-resume rule (B15) is untouched
(transcript-recovery D1–D6, 2026-07-20).

Detection's own transcript store is runtime-aware without guessing a second
runtime's internals: a workspace can list additional transcript roots (each
tagged with the runtime that owns it) in its config alongside the Claude
default, and every recoverable candidate names which root it was actually
found under. A configured root that turns out missing or unreadable degrades
the same way as a missing store always has — a silent no-op to detection
itself — but is also reported, by name, so a person running a second runtime
can confirm the root was really consulted instead of quietly never checked; a
workspace that names no extra root behaves exactly as before (hardening-5,
2026-07-21).

Locating a candidate's transcript no longer relies solely on computing where
it ought to be from the runtime's usual layout: session start now persists
the runtime-provided transcript path into the session's own record (hook
runtime detail: `hook-runtime.md`), and crash recovery prefers that stored
path over layout math whenever it is present. A transcript layout the
recovering session's own runtime did not originate (a Codex-shaped session
being scanned from a Claude-hosted recovery pass, for instance) is therefore
first-class rather than a guess that only happens to work when the two
runtimes' conventions line up (hardening-1-7-10).

## Business Rules

- R51 — A crashed session's unsettled work is recovered only through its
  harness transcript as a secondary source, never by promoting mined content to
  settled knowledge on its own: detection is automatic and cheap, mining is
  offered not forced, the raw transcript is read only by a down-tier helper and
  never loaded into the recovering session, mined settlements enter as
  unconfirmed capture stubs that still pass the normal flush, and recovery never
  resumes work or writes a pause record (transcript-recovery D1–D6, 2026-07-20).

- R52 — Detection and reclaim answer different questions and are never fused.
  Detection asks which sessions are worth investigating, and judges that from
  transcript shape: a stale-heartbeat session whose transcript ends cleanly, or
  whose transcript cannot be resolved at all, is not a candidate. Reclaim asks
  which held work is free to take back, and judges that from claim age and
  heartbeat age alone, never consulting a transcript. A session that ended
  cleanly while still holding an expired claim therefore has that claim
  reclaimed and its record marked, while never appearing among the sessions
  offered for investigation. Presenting the two as one list would either strand
  that work forever or invite mining where there is nothing to mine
  (sweep-recovery-door D3, cell srd-2, 2026-08-14).

- R53 — The recovery command reclaims on every invocation, with no confirming
  flag. Its criteria are already conservative enough that a qualifying claim is
  one no live session can still be working, and requiring a second gesture
  would reintroduce the failure the command exists to remove: work left held
  because nobody happened to run the thing that frees it (sweep-recovery-door
  D3, 2026-08-14).

## Edge Cases Settled

- A stale-heartbeat session whose transcript tail carries the clean-stop
  sequence is not a crash candidate — it stopped cleanly, it simply stopped;
  and a genuinely clean-ended session that queued nothing at stop (no closing
  prompt echo) is still excluded because it also shows no work in flight.
- A stale session whose transcript is missing entirely is reported as a session
  with no transcript, never as a recoverable crash.
- When the dead session had no bound feature, its mining window keys on the last
  global settled outcome and its recovery note is written to the shared recovery
  location instead of a feature's history.
