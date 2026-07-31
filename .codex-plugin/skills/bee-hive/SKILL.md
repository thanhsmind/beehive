---
name: bee-hive
description: >-
  Route the bee workflow: session start, the next skill, gates, and onboarding. Use when starting or resuming any bee session, choosing the next bee skill, running go mode, checking onboarding state, enforcing workflow gates, or setting/checking the gate-bypass level (off/normal/full/total).
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: unavailable
      reason: Onboarding and the vendored .bee/bin helpers run in Node.js 18+.
---

# Hive — the router

## Start

Run `bee orient` — it names the phase, the blockers, and the next skill; follow it.
A pending handoff is presented to the user and never auto-resumed. Orient is for routing,
starting, or resuming work; plain questions are already answered by the session preamble.

## Route

Lifecycle: shape → plan → swarm → capture — `bee orient`'s `next.skill` names the
current stop. Out-of-band requests:

| Skill | When |
|---|---|
| `bee-shaping` | Gray areas or unlocked decisions — lock them (also backlog triage, parking, the implement-plan brief). |
| `bee-planning` | Decisions locked, or scope already clear ("just fix this") — route the lane, shape the work, present the gate. A code-touching route creates the feature's worktree in the same step (`bee worktree new --feature <slug>`) and the work lives there. |
| `bee-swarming` | Merged shape+execution gate approved, cells open — orchestrate workers, or execute one assigned cell, inside the feature's worktree. A `tiny` cell may run inline; `small` and up runs through a dispatched execution worker. |
| `bee-capturing` | Execution done, an area needs documenting, or something just settled — sync specs, record learnings. |
| `bee-reviewing` | Only on an explicit review request — never automatic. Merge/ship with unreviewed candidates: report count + risk, ask ONE question. |
| `bee-researching` | Research a topic, library, or approach — standalone, or from planning discovery. |
| `bee-grooming` | The user asks to clean up, audit, or hunt tech debt. |
| `bee-herding` | The user invokes the cockpit: bootstrap, dispatch, or merge. |
| docs-only change | No pipeline: announce, write, format-check, close with a capture line or "nothing settled". |

## Gates

Never approve a gate yourself, in any mode — headless included. Gates belong to the user:
`bee state gate` records their answer, presented as a plain-language layer plus the fixed
question, report linked, never pasted ("Gate Presentation Contract").

The one recorded exception is gate bypass — `.bee/config.json` `gate_bypass`, a level:
`off` (`false`, default) · `normal` (`true` / `"on"` / `"normal"`; legacy `true` reads as
`normal`) auto-approves Gates 1-2 for non-hard-gate work, while high-risk/hard-gate,
secret reads, and Gate 4 UAT/P1 still stop · `full` (`"full"`) lifts the high-risk/
hard-gate floor; secret reads and a review P1 still stop · `total` (`"total"`) stops for
nothing, secret reads included. To change it: set the config value (preserve every other
field; create it if absent), log a one-line audit decision, and state the level's row —
what auto-approves, what still stops — in the same turn; never silently, and never
`full`/`total` without the user's explicit instruction. Setting the level never approves a
gate; bypass is not headless — headless still stops at every gate ("Gate bypass mode").

## Onboarding

`.bee/onboarding.json` missing or stale → from the bee source root:
`node packages/bee/scripts/onboard_bee.mjs --repo-root <root> --json`. `changes_needed` →
summarize, get approval, re-run with `--apply` — never silently; `blocked_*` → zero
mutations, surface `versions`. Do not continue until it reports `up_to_date`.

## Go mode

`/go` or "run the full pipeline" → `references/go-mode.md`.

## Hard rules

- P1 review findings always block; never build on a red base.
- At ~65% context, write `.bee/HANDOFF.json` and pause.
- Locked decisions are cited, never reinterpreted; no source edits before the execution gate is approved.
- "done/green/fixed" only beside fresh command output in the same message; every close carries a capture line or an explicit "nothing settled".
- The agent runs every bee command ("The agent runs the machinery"); work language only, one tick line per visible step ("Progress ticks", "Communication contract"); a red line is never silenced.
- Form rules bend out loud with a recorded reason; boundary rules never bend ("Judgment contract"). Lanes scale ceremony, never memory ("Re-lane checkpoint", "Capture discipline").

## References

| File | When to load |
|---|---|
| `references/routing-and-contracts.md` | Every quoted heading in this body resolves here. Deep contracts: gates and bypass, lanes and ceremony, delegation, communication, judge tier, onboarding detail |
| `references/go-mode.md` | `/go` runs: gate wording, slice loop, fallbacks, headless + bypass |
