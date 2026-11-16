---
name: bee-hive
description: >-
  Route the bee workflow: session start, the next skill, gates, onboarding. Use when starting or resuming a bee session, asking where work stands or which step comes next, checking onboarding state, running go mode, or setting the gate-bypass level (off/normal/full/total).
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
A context mark never stops work; a handoff happens only on the user's word or at feature close (rule: `agents-context-handoff-65`).

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
| `bee-shaping` | Main | Gray areas or unlocked decisions — lock them (also backlog triage, `Qualify` on request, or `Brief` to render `implement-plan.md` / post-Gate-3 `walkthrough.md`). |
| `bee-planning` | Main | Decisions locked, or scope already clear ("just fix this") — for any file-writing change, `bee-shaping` (Lock) writes a short `CONTEXT.md` brief first, then `bee-planning` routes the lane, shapes the work, and presents the gate. A code-touching route creates the feature's worktree in the same step (rule: `agents-worktree-first`). |
| `bee-swarming` | Main | Merged gate approved, cells open — orchestrate or execute one cell in the feature worktree. |
| `bee-capturing` | Main | Execution done, an area needs documenting, or something just settled — sync specs, record learnings. |
| `bee-reviewing` | Main | Only on an explicit review request — never automatic (rule: `agents-review-user-invoked`). Merge/ship with unreviewed candidates: report count + risk, ask ONE question. |
| `bee-researching` | Both | Research a topic, library, or approach — standalone, or from planning discovery. |
| `bee-grooming` | — | The user asks to clean up, audit, or hunt tech debt. |
| `bee-herding` | — | The user invokes the cockpit: bootstrap, dispatch, or merge. |
| `bee-hive` (Gates) | — | Turn `gate_bypass` on/off, widen it, or check it. |
| `Resume session` | — | Check `.bee/HANDOFF.json` first (`references/routing-and-contracts.md`, "Resume Logic"). |
| docs-only change | — | File-writing docs change: no pipeline — short brief + Gate 1, announce, write through `bee-technical-writing`, format-check, close with a capture line or "nothing settled" (rule: `agents-capture-line-at-close`). |
| pure question | — | Writes no file: answer directly — no lane, no brief, no record. |

**Surface-scope-earlier check** (before routing to `bee-shaping`): concrete acceptance criteria plus references to existing patterns → offer jumping straight to planning with a one-paragraph scoping synthesis whose decisions still carry D-IDs.

## Gates

Three gates: **Gate 1** (exploring — approve `CONTEXT.md`), **Gate 2** (planning — `shape` + `execution` in one `bee gate --merge` call), **Gate 3** (reviewing — merge approval, user-invoked only; rule: `agents-review-user-invoked`).
A separate stop, `uat`, is required for `standard`/`high-risk` features and never auto-approved at any `gate_bypass` level (`references/gates-and-delegation.md`, "The three gates").
Gate bypass (`gate_bypass`: `off` · `normal` · `full` · `total` in `.bee/config.json`) is user-set only; setting a level never approves a gate, and bypass is not headless (`references/gates-and-delegation.md`, "Gate bypass mode").

## Onboarding

`.bee/onboarding.json` missing or stale → from the bee source root:
`.bee/bin/bee onboard --repo-root <root> --json`. `changes_needed` →
summarize, get approval, re-run with `--apply` — never silently; `blocked_*` → zero
mutations, surface `versions`. Do not continue until it reports `up_to_date`.
These three steps are the whole session-time contract; the status detail behind
them loads from `references/onboarding.md` only when one of them actually fires.

A freshly onboarded project's first debt is its orientation entry — written by reading the tree, never by asking the user (`.bee/expertise/knowledge.md` "The orientation file"; `bee-capturing` owns the write).
On a `workspace protocol:` notice (`workspace_protocol.action_needed: true`), run `references/onboarding.md` ("Workspace protocol") before any feature work.

## Go mode

`/go` or "run the full pipeline" → `references/go-mode.md`.

## Headless

Headless: never ask, never self-approve; defer ambiguities to `Outstanding Questions` → `references/gates-and-delegation.md` ("Headless mode").

## References

Every heading quoted in this body resolves somewhere in `references/`; the row
that names the contract is the one to open.

| File | When to load |
|---|---|
| `references/routing-and-contracts.md` | Resuming a handoff (`HANDOFF.json`), checking lane ceremony or concurrency rules, running the leader completeness check, formatting a turn/question/catch-me-up/pushback, or checking chaining or file paths |
| `references/gates-and-delegation.md` | A gate is about to be presented, a bypass level set, work fanned out, or a rule bent — gate presentation, bypass levels, headless, delegation, judgment contract, judge tier, verify scope, Codex tending |
| `references/scout-and-ticks.md` | Deciding how much to read before acting, re-judging a lane against evidence, or writing a tick / ship-visibility / route-record line exactly |
| `references/onboarding.md` | ONLY when onboarding is in question — `.bee/onboarding.json` missing or stale, an install/upgrade request, or a `blocked_*` result. Never on a session that is already `up_to_date`. Also when the user asks to set up lint, type check, tests or verify commands (section Set up the project's checks) |
| `references/go-mode.md` | `/go` runs: gate wording, slice loop, fallbacks, headless + bypass |
| `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md` ("Principle homes") | Where the craft principle set lives — one row per principle, its spoken line, and the classes it routes to |
