---
name: bee-hive
description: >-
  Route the bee workflow: session start, the next skill, gates, and onboarding. Use when starting or resuming any bee session, choosing the next bee skill, running go mode, checking onboarding state, enforcing workflow gates, or setting/checking the gate-bypass level (off/normal/full/total).
metadata:
  version: '0.2'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: unavailable
      reason: Onboarding (`bee onboard`) and every state read/write run through the vendored bee binary. The binary is vendored into the repo by onboarding; no Node runtime is involved.
---

# Hive — the router

## Start

Run `bee orient` only when routing, starting, or resuming work — it names the phase,
the blockers, and the next skill; follow it. Present a pending handoff to the user;
never auto-resume it. A plain question is already answered by the session preamble.

## Route

Two doors; the agent picks — the user never has to name a flow.

| Flow | Membership |
|---|---|
| **Main flow** | idea to ship — `bee-shaping`, `bee-planning`, `bee-swarming`, `bee-capturing`, with `bee-reviewing` and the `uat` door at merge |
| **Discovery flow** | an open question to a locked decision — `bee-wayfinding` is the spine; research, spike, and grilling resolve its tickets; exit is `bee-shaping`'s Lock consuming the map's D-IDs |

A nameable outcome routes to Main flow. Fog, or an effort too big to
name in one sitting, routes to Discovery flow.

An explicit user word — wayfinding, brainstorm, discuss, discovery, in
any language — routes straight into the Discovery flow and skips that
classification.

Lifecycle: shape → plan → swarm → capture — `bee orient`'s `next.skill` names the
current stop. Out-of-band requests, by flow:

| Skill | Flow | When |
|---|---|---|
| `bee-wayfinding` | Discovery | A fog-state idea, no nameable outcome yet, or an open discovery map with frontier tickets — chart or resume `docs/discovery/<effort>/MAP.md` before shaping. |
| `bee-shaping` | Main | Gray areas or unlocked decisions — lock them (also backlog triage, parking, the implement-plan brief). |
| `bee-planning` | Main | Decisions locked, or scope already clear ("just fix this") — route the lane, shape the work, present the gate. A code-touching route creates the feature's worktree in the same step (worktree-first — AGENTS.md). |
| `bee-swarming` | Main | Merged shape+execution gate approved, cells open — orchestrate workers, or execute one assigned cell, inside the feature's worktree. A `tiny` cell may run inline; `small` and up runs through a dispatched execution worker. |
| `bee-capturing` | Main | Execution done, an area needs documenting, or something just settled — sync specs, record learnings. |
| `bee-reviewing` | Main | Only on an explicit review request — never automatic. Merge/ship with unreviewed candidates: report count + risk, ask ONE question. |
| `bee-researching` | Both | Research a topic, library, or approach — standalone, or from planning discovery. |
| `bee-grooming` | — | The user asks to clean up, audit, or hunt tech debt. |
| `bee-herding` | — | The user invokes the cockpit: bootstrap, dispatch, or merge. |
| docs-only change | — | No pipeline: announce, write, format-check, close with a capture line or "nothing settled". |

## Gates

Three gates, and only three: **Gate 1** (exploring — approve CONTEXT.md), **Gate 2**
(planning — shape AND execution in one `bee gate --merge` call), **Gate 3** (reviewing —
merge approval, only inside a review session the user invoked). Gates 1-2 are the default
chain; Gate 3 is additive and never automatic ("The three gates"). A separate stop,
`uat`, sits later still, at `bee worktree merge` — the user's acceptance of the
finished work, required for standard/high-risk features, never auto-approved at any
`gate_bypass` level ("The three gates", `gates-and-delegation.md`).

Gates belong to the user (AGENTS.md), in any mode — headless included: `bee gate`
records their answer, presented as a plain-language layer plus the fixed
question, report linked, never pasted ("Gate Presentation Contract").

The one recorded exception is gate bypass — an opt-in level in `.bee/config.json`
(`gate_bypass`: `off` · `normal` · `full` · `total`) that the user sets and only the
user widens. Setting a level never approves a gate, and bypass is not headless —
headless still stops at every gate. Which gates each level auto-approves, what it
still stops for, and how to record the change: `gates-and-delegation.md`
("Gate bypass mode") — the level table is stated there and nowhere else.

## Onboarding

`.bee/onboarding.json` missing or stale → from the bee source root:
`.bee/bin/bee onboard --repo-root <root> --json`. `changes_needed` →
summarize, get approval, re-run with `--apply` — never silently; `blocked_*` → zero
mutations, surface `versions`. Do not continue until it reports `up_to_date`.
These three steps are the whole session-time contract; the status detail behind
them loads from `references/onboarding.md` only when one of them actually fires.

A freshly onboarded project has craft (`.bee/expertise/`) but an empty
knowledge layer of its own. Its first debt is the orientation entry —
written by reading the tree, never by asking the user to describe it:
`.bee/expertise/knowledge.md` ("The orientation file") sets the bar,
`bee-capturing` owns the write.

## Go mode

`/go` or "run the full pipeline" → `references/go-mode.md`.

## Headless

A headless run proceeds without asking: ambiguities, unanswered decisions, and gate
questions become `Outstanding Questions` entries in the run's report, each carrying
the evidence a later human pass would start from — deferred, never guessed, never
self-answered. A headless run never self-approves a gate: every gate still stops and
reports awaiting approval (bypass is the separate, recorded exception — "Gate bypass mode").
Still make recommendations where the evidence supports one, labeled with confidence.
Full contract: `references/gates-and-delegation.md` ("Headless mode").

## Hard rules

- P1 review findings always block; never build on a red base.
- Locked decisions (cite, never reinterpret), the pre-execution-gate edit boundary, and the 65%-context handoff hold as written (AGENTS.md).
- "done/green/fixed" only beside fresh command output in the same message; every close carries a capture line or an explicit "nothing settled".
- The agent runs every bee command ("The agent runs the machinery"); work language only, one tick line per visible step ("Progress ticks", "Communication contract"); a red line is never silenced.
- Form rules bend out loud with a recorded reason; boundary rules never bend ("Judgment contract"). Lanes scale ceremony, never memory ("Re-lane checkpoint", "Capture discipline").

## References

Every heading quoted in this body resolves somewhere in `references/`; the row
that names the contract is the one to open.

| File | When to load |
|---|---|
| `references/routing-and-contracts.md` | The default: skill catalog, first-skill routing, state bootstrap, resume logic, lane ceremony, chaining, communication contract, question format, file and CLI quick reference |
| `references/gates-and-delegation.md` | A gate is about to be presented, a bypass level set, work fanned out, or a rule bent — gate presentation, bypass levels, headless, delegation, judgment contract, judge tier, verify scope, Codex tending |
| `references/scout-and-ticks.md` | Deciding how much to read before acting, re-judging a lane against evidence, or writing a tick / ship-visibility / route-record line exactly |
| `references/onboarding.md` | ONLY when onboarding is in question — `.bee/onboarding.json` missing or stale, an install/upgrade request, or a `blocked_*` result. Never on a session that is already `up_to_date` |
| `references/go-mode.md` | `/go` runs: gate wording, slice loop, fallbacks, headless + bypass |
