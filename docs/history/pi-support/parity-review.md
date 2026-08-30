# Pi vs Claude Code — harness parity review

Date: 2026-08-30. Three independent read-only reviewers, disjoint scopes,
synthesised here. Question asked: can the bee harness run as stably on the
Pi runtime as it does on Claude Code today?

**Verdict: no — not the same way.** Enforcement is close to parity. Execution
works but is not unattended-safe. The install/context layer is not at parity
at all.

## Layer verdicts

| Layer | Verdict |
|---|---|
| Hook / enforcement belt | PARITY WITH NAMED GAPS |
| Work execution (dispatch, workers, results) | PARITY WITH NAMED GAPS |
| Install / context surface | NOT AT PARITY |

## Hook belt

Pi wires write-guard (`tool_call`), session-init (`session_start`),
prompt-context (`before_agent_start`), state-sync (`tool_result`) and
session-close (`agent_settled`). The blocking path genuinely fails closed six
ways (`.pi/extensions/bee-guard.ts:153-244`) — stricter than Claude's own
belt, which fails open on a missing binary (`.claude/settings.json:45`).

Gaps:

1. `tools-logger` is silently unwired. No exclusion names it, and the
   advisory-coverage list is hand-authored (`pi_plugin_contracts.rs:1142-1150`),
   so it dropped off with every test green.
2. The passivity pre-check can fail open on the blocking surface:
   `beeStorePresent` returns false on any stat error and `directoryOf` falls
   back to `process.cwd()` (`bee-guard.ts:99-105, 515-518, 855`).
3. Advisory parity is a hand list; only blocking rules ride the derived
   four-belt test. The next advisory rule can drop off unseen.
4. No PreCompact and no SessionEnd analog.
5. Turn end runs session-close only; Claude's Stop also runs state-sync.
6. Module-scope singletons assume one session per process — a second session
   poisons the preamble cache and kills the first's result drain
   (`bee-guard.ts:483-485, 599-613, 807`).
7. `CONTEXT.md:30` (D2) over-promises: `activity` is named but never wired.

## Execution

Claude runs a worker in-process through the Agent tool. Pi has no subagent
surface, so every cell resolves `{"kind":"herding"}` and runs as a terminal
pane (`prepare.rs:1506-1512, 1663-1680`). That adds herdr or tmux, a live
multiplexer pane, the worker CLI binaries, foreign trust-store files and a
screen-width floor as hard dependencies.

Risks to unattended operation:

1. A worker pane sitting on any human prompt hard-stops the run until someone
   touches a terminal; a prompt herdr misreads as idle burns the full 900s
   timeout first (`run.rs:1626-1636, 98-115`).
2. Sync-vs-detached delivery discipline is enforced by an instruction string
   the orchestrator model must obey, not by code (`prepare.rs:88, 1721-1731`).
3. A failed inbox-marker write is a stderr note and the run proceeds — the one
   case where an async result is genuinely lost, silently (`run.rs:1993-2018`).
4. No automatic re-dispatch: `fallback:"default"` resolves Null for every
   non-claude runtime (`models.rs:107-119`).
5. `transport_ready:false` is data, never a refusal, so a headless Pi session
   gets a payload guaranteed to SpawnFail (`prepare.rs:1700-1706`).
6. An orchestrator killed mid-run orphans a live pane whose result reaches
   nobody without a marker.
7. Pending inbox markers are never removed by `run.rs` on any outcome.

Escalated cells are refused by name on pi with a typed remedy — run inline.
The cell stays claimed and recoverable, so this is bounded, not a hang.

## Install / context

The whole `.pi/` tree in a real host project is ONE file:
`.pi/extensions/bee-guard.ts`. Claude gets 4 agent files, 13 skill trees,
settings and a statusline.

1. There is no `.pi/skills`. `REPO_SKILL_TARGETS` has three entries and no pi
   (`templates.rs:317-321`), so a Pi session has zero auto-loadable bee craft
   and must hand-read another runtime's rendered projection — content already
   stripped for `claude`.
2. `pi` is not a legal skill marker label (`skill_trees.rs:49, 127-132`), so
   pi-scoped skill content cannot even be authored today. Gap 1 cannot be
   closed incrementally until this moves.
3. No worker/subagent definition files and no `agents_sync` record for pi.
   Codex's absence carries a written reason (`agents.rs:332`); pi's carries
   nothing.
4. `bee doctor --runtime pi` refuses — pi's health is unobservable
   (`doctor.rs:49-52, 61-67`).
5. The pi belt sits outside the managed ledger (`plan.rs:227-237`), so nothing
   flags host-side drift between onboard runs.
6. No downgrade guard on the pi copy: an older bee silently rolls the guard
   backwards, unlike the skill sync's `blocked_downgrade` preflight
   (`plan.rs:594-599` vs `skills.rs:459-478`).
7. No stale-file removal for `.pi/extensions` — an extension dropped from the
   source stays live on every host forever.
8. `pi_extension` is excluded from `PACKAGE_ROLES`
   (`plugin_distribution.rs:50-58, 210-216`), so the installed-package proof
   skips the pi belt even though the release manifest records it.
9. The managed AGENTS.md block carries no pi-facing text, so nothing tells a
   Pi agent where the skills it cannot auto-load actually live.

## Shipping is real

`.pi/extensions/bee-guard.ts` is in the release manifest under role
`pi_extension`, and its sha256 matches byte-for-byte in the source checkout
and in a real host project. Onboarding copies it "when missing or drifted".
The belt genuinely arrives.

## The unproven part

`docs/history/pi-support/plan.md:51` lists the whole risk map as unit proof —
stub-binary fixtures, driver unit tests, contract tests. No live Pi session
appears anywhere in the record. Every finding above is code reading; nobody
has watched bee actually work on pi.

Machine readiness is not the blocker: `pi 0.84.3`, `herdr`, `tmux`, `claude`
and `agy` are all on PATH here.
